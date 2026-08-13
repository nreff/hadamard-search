#!/usr/bin/env python3
"""Seeded local search for the H17-invariant cyclic SDS(515) shard.

This is a heuristic companion to :mod:`search_sds_515`, not a replacement for
its exact OPB model.  Every state has the exact fixed/nonfixed orbit counts
required by that model.  A move swaps a selected and an unselected orbit of
the same size in one sequence, so those counts remain exact throughout.

The 17 correlation equations are weighted cuts on the 35 H-orbits.  Per-vertex
flip fields make the residual delta of a swap cheap to evaluate, while an
accepted move updates all fields in O(17 * 35) time.  Objective zero is exactly
the required 17-equation witness and is then checked on all 514 shifts by the
independent verifier in ``search_sds_515``.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
from pathlib import Path
import random
import time
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

from search_sds_515 import (
    MODULUS,
    MULTIPLIER_ORDER,
    SEQUENCE_COUNT,
    TARGET_FIXED_COUNTS,
    TARGET_NONFIXED_COUNTS,
    TARGET_TOTAL_DISAGREEMENTS,
    h_orbits,
    shift_pair_multiplicities,
    variable_layout,
    verify_orbit_assignment,
    verify_signed_model,
)


ShiftVector = Tuple[int, ...]
OrbitRows = List[List[bool]]


def cut_weights() -> Tuple[Tuple[Tuple[int, ...], ...], ...]:
    """Return ``weights[shift][left][right]`` for the 17 exact cuts."""
    orbit_count = len(h_orbits())
    result: List[Tuple[Tuple[int, ...], ...]] = []
    for _, terms in shift_pair_multiplicities():
        matrix = [[0] * orbit_count for _ in range(orbit_count)]
        for left, right, weight in terms:
            matrix[left][right] = weight
            matrix[right][left] = weight
        result.append(tuple(tuple(row) for row in matrix))
    return tuple(result)


CUT_WEIGHTS = cut_weights()
SHIFT_COUNT = len(CUT_WEIGHTS)
FIXED_ORBITS = tuple(index for index, orbit in enumerate(h_orbits()) if len(orbit) == 1)
NONFIXED_ORBITS = tuple(
    index for index, orbit in enumerate(h_orbits()) if len(orbit) == MULTIPLIER_ORDER
)


def exact_residuals(rows: Sequence[Sequence[bool]]) -> ShiftVector:
    """Recompute the 17 disagreement residuals without incremental state."""
    residuals = []
    for weights in CUT_WEIGHTS:
        disagreements = 0
        for row in rows:
            disagreements += sum(
                weights[left][right]
                for left in range(len(row))
                for right in range(left + 1, len(row))
                if bool(row[left]) != bool(row[right])
            )
        residuals.append(disagreements - TARGET_TOTAL_DISAGREEMENTS)
    return tuple(residuals)


def squared_objective(residuals: Sequence[int]) -> int:
    """The exact sum of squared disagreement residuals."""
    return sum(residual * residual for residual in residuals)


def random_rows(rng: random.Random) -> OrbitRows:
    """Draw an assignment with all eight orbit-count constraints exact."""
    rows: OrbitRows = []
    for fixed_count, nonfixed_count in zip(TARGET_FIXED_COUNTS, TARGET_NONFIXED_COUNTS):
        selected = set(rng.sample(FIXED_ORBITS, fixed_count))
        selected.update(rng.sample(NONFIXED_ORBITS, nonfixed_count))
        rows.append([orbit in selected for orbit in range(len(h_orbits()))])
    return rows


def _flip_fields(rows: Sequence[Sequence[bool]]) -> List[List[List[int]]]:
    """Compute the objective-cut delta of flipping each vertex by itself."""
    fields: List[List[List[int]]] = []
    for row in rows:
        row_fields: List[List[int]] = []
        for weights in CUT_WEIGHTS:
            row_fields.append([
                sum(
                    weights[vertex][other]
                    * (1 if bool(row[vertex]) == bool(row[other]) else -1)
                    for other in range(len(row))
                    if other != vertex
                )
                for vertex in range(len(row))
            ])
        fields.append(row_fields)
    return fields


@dataclass
class SearchState:
    """An exact-cardinality assignment with incrementally maintained cuts."""

    rows: OrbitRows
    residuals: List[int]
    fields: List[List[List[int]]]
    objective: int

    @classmethod
    def from_rows(cls, rows: Sequence[Sequence[bool]]) -> "SearchState":
        copied = [[bool(value) for value in row] for row in rows]
        residuals = list(exact_residuals(copied))
        return cls(copied, residuals, _flip_fields(copied), squared_objective(residuals))

    def swap_delta(self, sequence: int, selected: int, unselected: int) -> ShiftVector:
        """Return the exact 17-cut delta for a legal selected/unselected swap."""
        row = self.rows[sequence]
        if not row[selected] or row[unselected]:
            raise ValueError("swap requires a selected then an unselected orbit")
        return tuple(
            self.fields[sequence][shift][selected]
            + self.fields[sequence][shift][unselected]
            + 2 * CUT_WEIGHTS[shift][selected][unselected]
            for shift in range(SHIFT_COUNT)
        )

    def objective_after(self, delta: Sequence[int]) -> int:
        return self.objective + sum(
            2 * residual * change + change * change
            for residual, change in zip(self.residuals, delta)
        )

    def apply_swap(
        self, sequence: int, selected: int, unselected: int, delta: Sequence[int]
    ) -> None:
        """Apply a legal swap and update residuals/fields without recomputation."""
        row = self.rows[sequence]
        if not row[selected] or row[unselected]:
            raise ValueError("swap requires a selected then an unselected orbit")
        if len(delta) != SHIFT_COUNT:
            raise ValueError("wrong delta length")

        orbit_count = len(row)
        for shift, weights in enumerate(CUT_WEIGHTS):
            field = self.fields[sequence][shift]
            selected_field = field[selected]
            unselected_field = field[unselected]
            for vertex in range(orbit_count):
                if vertex == selected or vertex == unselected:
                    continue
                selected_sign = 1 if row[vertex] else -1
                unselected_sign = -selected_sign
                field[vertex] -= 2 * (
                    weights[vertex][selected] * selected_sign
                    + weights[vertex][unselected] * unselected_sign
                )
            edge_weight = weights[selected][unselected]
            field[selected] = -selected_field - 2 * edge_weight
            field[unselected] = -unselected_field - 2 * edge_weight

        row[selected] = False
        row[unselected] = True
        self.residuals = [
            residual + change for residual, change in zip(self.residuals, delta)
        ]
        self.objective = squared_objective(self.residuals)

    def assert_exact(self) -> None:
        """Expensive invariant check, intended for tests and debug runs."""
        for sequence, row in enumerate(self.rows):
            fixed = sum(row[index] for index in FIXED_ORBITS)
            nonfixed = sum(row[index] for index in NONFIXED_ORBITS)
            if fixed != TARGET_FIXED_COUNTS[sequence]:
                raise AssertionError("fixed-orbit count drift")
            if nonfixed != TARGET_NONFIXED_COUNTS[sequence]:
                raise AssertionError("nonfixed-orbit count drift")
        exact = exact_residuals(self.rows)
        if tuple(self.residuals) != exact:
            raise AssertionError("incremental residual drift")
        if self.objective != squared_objective(exact):
            raise AssertionError("incremental objective drift")
        if self.fields != _flip_fields(self.rows):
            raise AssertionError("incremental flip-field drift")


Move = Tuple[int, int, int, ShiftVector, int]


def _legal_pairs(row: Sequence[bool], orbit_class: Sequence[int]) -> Iterable[Tuple[int, int]]:
    selected = [orbit for orbit in orbit_class if row[orbit]]
    unselected = [orbit for orbit in orbit_class if not row[orbit]]
    for left in selected:
        for right in unselected:
            yield left, right


def _choose_tabu_move(
    state: SearchState,
    tabu_until: Sequence[Sequence[int]],
    iteration: int,
    aspiration: int,
    rng: random.Random,
) -> Move:
    """Choose a best admissible move, breaking exact ties from the seed."""
    best: Optional[Move] = None
    tie_count = 0
    for sequence, row in enumerate(state.rows):
        for orbit_class in (FIXED_ORBITS, NONFIXED_ORBITS):
            for selected, unselected in _legal_pairs(row, orbit_class):
                delta = state.swap_delta(sequence, selected, unselected)
                objective = state.objective_after(delta)
                tabu = (
                    tabu_until[sequence][selected] > iteration
                    or tabu_until[sequence][unselected] > iteration
                )
                if tabu and objective >= aspiration:
                    continue
                move = (sequence, selected, unselected, delta, objective)
                if best is None or objective < best[4]:
                    best = move
                    tie_count = 1
                elif objective == best[4]:
                    tie_count += 1
                    if rng.randrange(tie_count) == 0:
                        best = move
    if best is None:
        raise AssertionError("tabu policy excluded every legal move")
    return best


@dataclass(frozen=True)
class SearchResult:
    seed: int
    best_rows: Tuple[Tuple[bool, ...], ...]
    best_residuals: ShiftVector
    best_objective: int
    best_restart: int
    best_iteration: int
    iterations: int
    restarts_completed: int
    elapsed_seconds: float

    @property
    def witness_found(self) -> bool:
        return self.best_objective == 0

    def summary(self) -> Dict[str, object]:
        return {
            "seed": self.seed,
            "witness_found": self.witness_found,
            "best_objective": self.best_objective,
            "best_residuals": list(self.best_residuals),
            "best_disagreements": [
                TARGET_TOTAL_DISAGREEMENTS + residual for residual in self.best_residuals
            ],
            "best_restart": self.best_restart,
            "best_iteration": self.best_iteration,
            "iterations": self.iterations,
            "restarts_completed": self.restarts_completed,
            "elapsed_seconds": round(self.elapsed_seconds, 6),
        }


def tabu_search(
    seed: int,
    restarts: int,
    steps_per_restart: int,
    tabu_tenure: int = 9,
    tenure_jitter: int = 5,
    stagnation_steps: int = 1500,
    debug_check_interval: int = 0,
    deadline: Optional[float] = None,
) -> SearchResult:
    """Run deterministic-seed steepest tabu search over legal swaps."""
    if restarts <= 0 or steps_per_restart <= 0:
        raise ValueError("restarts and steps_per_restart must be positive")
    if tabu_tenure < 1 or tenure_jitter < 0 or stagnation_steps < 1:
        raise ValueError("invalid tabu-search controls")

    rng = random.Random(seed)
    started = time.monotonic()
    best_rows: Optional[Tuple[Tuple[bool, ...], ...]] = None
    best_residuals: ShiftVector = ()
    best_objective: Optional[int] = None
    best_restart = -1
    best_iteration = -1
    iterations = 0
    restarts_completed = 0

    for restart in range(restarts):
        state = SearchState.from_rows(random_rows(rng))
        tabu_until = [[0] * len(h_orbits()) for _ in range(SEQUENCE_COUNT)]
        last_improvement = 0
        for local_iteration in range(steps_per_restart):
            if deadline is not None and time.monotonic() >= deadline:
                elapsed = time.monotonic() - started
                if best_rows is None or best_objective is None:
                    raise AssertionError("deadline expired before initial state evaluation")
                return SearchResult(
                    seed, best_rows, best_residuals, best_objective, best_restart,
                    best_iteration, iterations, restarts_completed, elapsed,
                )

            if best_objective is None or state.objective < best_objective:
                best_objective = state.objective
                best_residuals = tuple(state.residuals)
                best_rows = tuple(tuple(row) for row in state.rows)
                best_restart = restart
                best_iteration = local_iteration
                last_improvement = local_iteration
                if best_objective == 0:
                    state.assert_exact()
                    elapsed = time.monotonic() - started
                    return SearchResult(
                        seed, best_rows, best_residuals, best_objective, best_restart,
                        best_iteration, iterations, restarts_completed, elapsed,
                    )

            # Restart early when the current trajectory has ceased improving
            # the run-global incumbent for a sustained period.
            if local_iteration - last_improvement >= stagnation_steps:
                break

            move = _choose_tabu_move(
                state,
                tabu_until,
                local_iteration,
                best_objective,
                rng,
            )
            sequence, selected, unselected, delta, _ = move
            state.apply_swap(sequence, selected, unselected, delta)
            tenure = tabu_tenure + (rng.randrange(tenure_jitter + 1) if tenure_jitter else 0)
            tabu_until[sequence][selected] = local_iteration + tenure
            tabu_until[sequence][unselected] = local_iteration + tenure
            iterations += 1
            if debug_check_interval and iterations % debug_check_interval == 0:
                state.assert_exact()
        restarts_completed += 1

    elapsed = time.monotonic() - started
    if best_rows is None or best_objective is None:
        raise AssertionError("search did not evaluate an initial state")
    return SearchResult(
        seed, best_rows, best_residuals, best_objective, best_restart,
        best_iteration, iterations, restarts_completed, elapsed,
    )


def _full_model_assignments(rows: Sequence[Sequence[bool]]) -> Dict[int, bool]:
    base, xor_maps, variable_count = variable_layout()
    assignments: Dict[int, bool] = {}
    for sequence, variables in enumerate(base):
        assignments.update(
            (variable, bool(rows[sequence][orbit]))
            for orbit, variable in enumerate(variables)
        )
    for sequence, mapping in enumerate(xor_maps):
        assignments.update(
            (variable, bool(rows[sequence][left]) != bool(rows[sequence][right]))
            for (left, right), variable in mapping.items()
        )
    if set(assignments) != set(range(1, variable_count + 1)):
        raise AssertionError("failed to construct a complete exact-model assignment")
    return assignments


def _expanded_signs(rows: Sequence[Sequence[bool]]) -> List[List[int]]:
    signs: List[List[int]] = []
    for row in rows:
        sequence = [1] * MODULUS
        for selected, orbit in zip(row, h_orbits()):
            if selected:
                for residue in orbit:
                    sequence[residue] = -1
        signs.append(sequence)
    return signs


def serialize_witness(result: SearchResult, prefix: Path) -> Dict[str, object]:
    """Write exact OPB model and signs, then independently verify both."""
    if not result.witness_found:
        raise ValueError("only an objective-zero result can be serialized as a witness")
    prefix.parent.mkdir(parents=True, exist_ok=True)
    model_path = prefix.with_suffix(".model")
    signs_path = prefix.with_suffix(".signs.json")

    assignments = _full_model_assignments(result.best_rows)
    literals = [
        ("x{}" if assignments[variable] else "-x{}").format(variable)
        for variable in sorted(assignments)
    ]
    model_lines = ["s SATISFIABLE"]
    for start in range(0, len(literals), 80):
        suffix = " 0" if start + 80 >= len(literals) else ""
        model_lines.append("v " + " ".join(literals[start:start + 80]) + suffix)
    model_path.write_text("\n".join(model_lines) + "\n", encoding="utf-8")

    signs_payload = {
        "format": "h17-invariant-cyclic-sds-515-witness-v1",
        "modulus": MODULUS,
        "seed": result.seed,
        "objective": result.best_objective,
        "residuals": list(result.best_residuals),
        "orbit_selection": [[int(value) for value in row] for row in result.best_rows],
        "signs": _expanded_signs(result.best_rows),
    }
    signs_path.write_text(json.dumps(signs_payload, indent=2) + "\n", encoding="utf-8")

    orbit_verification = verify_orbit_assignment(result.best_rows)
    model_verification = verify_signed_model(model_path)
    return {
        "model_path": str(model_path),
        "signs_path": str(signs_path),
        "orbit_verification": orbit_verification,
        "model_verification": model_verification,
    }


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Seeded swap-tabu search for the H17-invariant Z_515 SDS shard."
    )
    parser.add_argument("--seeds", default="0", help="comma-separated integer RNG seeds")
    parser.add_argument("--restarts", type=int, default=8)
    parser.add_argument("--steps", type=int, default=3000, help="maximum steps per restart")
    parser.add_argument("--tabu-tenure", type=int, default=9)
    parser.add_argument("--tenure-jitter", type=int, default=5)
    parser.add_argument("--stagnation-steps", type=int, default=1500)
    parser.add_argument(
        "--time-limit",
        type=float,
        help="wall-clock seconds shared by all seeds (steps remain the reproducible limit)",
    )
    parser.add_argument("--debug-check-interval", type=int, default=0)
    parser.add_argument(
        "--witness-prefix",
        type=Path,
        help="write PREFIX-seed-N.model/signs.json if an exact witness is found",
    )
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = parse_args(argv)
    seeds = [int(value.strip()) for value in args.seeds.split(",") if value.strip()]
    if not seeds:
        raise SystemExit("--seeds must contain at least one integer")
    deadline = time.monotonic() + args.time_limit if args.time_limit is not None else None
    results: List[SearchResult] = []
    witness: Optional[Dict[str, object]] = None
    for seed in seeds:
        if deadline is not None and time.monotonic() >= deadline:
            break
        result = tabu_search(
            seed=seed,
            restarts=args.restarts,
            steps_per_restart=args.steps,
            tabu_tenure=args.tabu_tenure,
            tenure_jitter=args.tenure_jitter,
            stagnation_steps=args.stagnation_steps,
            debug_check_interval=args.debug_check_interval,
            deadline=deadline,
        )
        results.append(result)
        if result.witness_found:
            verify_orbit_assignment(result.best_rows)
            if args.witness_prefix is not None:
                prefix = args.witness_prefix.parent / (
                    args.witness_prefix.name + "-seed-{}".format(seed)
                )
                witness = serialize_witness(result, prefix)
            break

    best = min(results, key=lambda result: result.best_objective) if results else None
    output: Dict[str, object] = {
        "format": "h17-invariant-cyclic-sds-515-heuristic-v1",
        "algorithm": "steepest-swap-tabu",
        "objective": "sum_of_17_squared_exact_disagreement_residuals",
        "target_disagreements_per_shift": TARGET_TOTAL_DISAGREEMENTS,
        "runs": [result.summary() for result in results],
        "best": best.summary() if best is not None else None,
        "witness": witness,
        "scope_caveat": (
            "failure to find objective zero proves nothing, even within the H17 shard"
        ),
    }
    print(json.dumps(output, indent=2, sort_keys=True))
    return 0 if best is not None and best.witness_found else 1


if __name__ == "__main__":
    raise SystemExit(main())
