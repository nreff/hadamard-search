#!/usr/bin/env python3
"""Exact H17-invariant cyclic SDS attack for a Hadamard matrix of order 2060.

The four Boolean supports encoded here live in the additive group Z_515 and
are unions of orbits of the (unique) order-17 subgroup of Z_515^*.  A model
therefore gives four circulant sequences whose Goethals--Seidel array is a
Hadamard matrix of order 4 * 515.  This is deliberately only a restricted
construction family: UNSAT excludes these H17-invariant cyclic seeds, not a
Hadamard matrix of order 2060 in general.
"""

from __future__ import annotations

import argparse
from collections import Counter
from functools import lru_cache
from itertools import combinations
import json
from math import gcd, isqrt
from pathlib import Path
import re
from typing import Dict, Iterable, List, Mapping, Optional, Sequence, Tuple


MODULUS = 515
HADAMARD_ORDER = 4 * MODULUS
MULTIPLIER_ORDER = 17
SEQUENCE_COUNT = 4
TARGET_ROW_SUMS = (1, 3, 31, 33)
TARGET_SUPPORT_SIZES = (257, 256, 242, 241)
TARGET_FIXED_COUNTS = (2, 1, 4, 3)
TARGET_NONFIXED_COUNTS = (15, 15, 14, 14)
TARGET_TOTAL_DISAGREEMENTS = 2 * MODULUS


def multiplier_subgroup(
    modulus: int = MODULUS, order: int = MULTIPLIER_ORDER
) -> Tuple[int, Tuple[int, ...]]:
    """Derive the subgroup using the least generator of exact given order."""
    if modulus <= 1 or order <= 1:
        raise ValueError("modulus and multiplier order must exceed one")
    for generator in range(2, modulus):
        if gcd(generator, modulus) != 1:
            continue
        subgroup = tuple(sorted({pow(generator, exponent, modulus) for exponent in range(order)}))
        if len(subgroup) != order:
            continue
        if pow(generator, order, modulus) != 1:
            continue
        # Exactness matters if this helper is used with a composite order.
        if any(order % divisor == 0 and pow(generator, divisor, modulus) == 1
               for divisor in range(1, order)):
            continue
        return generator, subgroup
    raise ValueError("no multiplier subgroup of the requested order exists")


def multiplicative_orbits(
    residues: Iterable[int], multipliers: Sequence[int], modulus: int = MODULUS
) -> Tuple[Tuple[int, ...], ...]:
    """Return canonically ordered multiplier orbits on the requested residues."""
    remaining = {residue % modulus for residue in residues}
    orbits: List[Tuple[int, ...]] = []
    while remaining:
        representative = min(remaining)
        orbit = tuple(sorted({representative * multiplier % modulus for multiplier in multipliers}))
        if not set(orbit) <= remaining:
            raise ValueError("multipliers do not act on the requested residue set")
        orbits.append(orbit)
        remaining.difference_update(orbit)
    return tuple(orbits)


@lru_cache(maxsize=1)
def h_orbits() -> Tuple[Tuple[int, ...], ...]:
    _, subgroup = multiplier_subgroup()
    return multiplicative_orbits(range(MODULUS), subgroup)


@lru_cache(maxsize=1)
def signed_shift_orbits() -> Tuple[Tuple[int, ...], ...]:
    """Orbits of nonzero shifts under H and negation."""
    _, subgroup = multiplier_subgroup()
    signed_subgroup = tuple(sorted(set(subgroup) | {(-value) % MODULUS for value in subgroup}))
    return multiplicative_orbits(range(1, MODULUS), signed_subgroup)


def row_sum_quadruples(order: int = HADAMARD_ORDER) -> Tuple[Tuple[int, int, int, int], ...]:
    """Enumerate sorted nonnegative odd quadruples with square sum ``order``."""
    if order <= 0:
        raise ValueError("order must be positive")
    cases: List[Tuple[int, int, int, int]] = []
    limit = isqrt(order)
    for first in range(1, limit + 1, 2):
        for second in range(first, isqrt(order - first * first) + 1, 2):
            remainder_two = order - first * first - second * second
            if remainder_two < 0:
                continue
            for third in range(second, isqrt(remainder_two) + 1, 2):
                fourth_square = remainder_two - third * third
                fourth = isqrt(fourth_square)
                if fourth >= third and fourth % 2 == 1 and fourth * fourth == fourth_square:
                    cases.append((first, second, third, fourth))
    return tuple(cases)


def orbit_compatible_parameters(
    row_sums: Sequence[int],
) -> Optional[Tuple[Tuple[int, ...], Tuple[int, ...], Tuple[int, ...]]]:
    """Return support, fixed-orbit, and nonfixed-orbit counts, or ``None``.

    There are five singleton H-orbits and thirty orbits of size 17.  Thus an
    H-invariant support of size k exists exactly when k = f + 17q with
    0 <= f <= 5 and 0 <= q <= 30.
    """
    fixed_orbit_count = sum(len(orbit) == 1 for orbit in h_orbits())
    nonfixed_orbit_count = sum(len(orbit) == MULTIPLIER_ORDER for orbit in h_orbits())
    supports: List[int] = []
    fixed_counts: List[int] = []
    nonfixed_counts: List[int] = []
    for row_sum in row_sums:
        if row_sum < 0 or (MODULUS - row_sum) % 2:
            return None
        support = (MODULUS - row_sum) // 2
        fixed = support % MULTIPLIER_ORDER
        nonfixed = (support - fixed) // MULTIPLIER_ORDER
        if not (0 <= fixed <= fixed_orbit_count and 0 <= nonfixed <= nonfixed_orbit_count):
            return None
        supports.append(support)
        fixed_counts.append(fixed)
        nonfixed_counts.append(nonfixed)
    return tuple(supports), tuple(fixed_counts), tuple(nonfixed_counts)


def compatible_row_sum_cases() -> Tuple[Tuple[Tuple[int, ...], Tuple[Tuple[int, ...], ...]], ...]:
    cases = []
    for row_sums in row_sum_quadruples():
        parameters = orbit_compatible_parameters(row_sums)
        if parameters is not None:
            cases.append((row_sums, parameters))
    return tuple(cases)


@lru_cache(maxsize=1)
def shift_pair_multiplicities() -> Tuple[Tuple[int, Tuple[Tuple[int, int, int], ...]], ...]:
    """Count orbit-pair disagreements for one representative of each ±H shift orbit.

    Each triple is ``(left_orbit, right_orbit, multiplicity)``.  Pairs whose
    endpoints lie in the same H-orbit are omitted because their XOR is zero.
    """
    orbits = h_orbits()
    orbit_of = {residue: index for index, orbit in enumerate(orbits) for residue in orbit}
    result = []
    for shift_orbit in signed_shift_orbits():
        shift = shift_orbit[0]
        counts: Counter[Tuple[int, int]] = Counter()
        for residue in range(MODULUS):
            left = orbit_of[residue]
            right = orbit_of[(residue + shift) % MODULUS]
            if left != right:
                counts[tuple(sorted((left, right)))] += 1
        result.append(
            (shift, tuple((left, right, counts[(left, right)]) for left, right in sorted(counts)))
        )
    return tuple(result)


def orbit_pairs() -> Tuple[Tuple[int, int], ...]:
    pairs = {
        (left, right)
        for _, multiplicities in shift_pair_multiplicities()
        for left, right, _ in multiplicities
    }
    return tuple(sorted(pairs))


def _opb_constraint(terms: Iterable[Tuple[int, int]], relation: str, target: int) -> str:
    rendered = " ".join(
        "{:+d} x{}".format(coefficient, variable)
        for coefficient, variable in terms
        if coefficient
    )
    return "{} {} {} ;".format(rendered, relation, target)


def _opb_xor_constraints(edge: int, left: int, right: int) -> Tuple[str, str, str, str]:
    """Encode edge = left XOR right over Boolean variables."""
    return (
        _opb_constraint(((1, edge), (-1, left), (1, right)), ">=", 0),
        _opb_constraint(((1, edge), (1, left), (-1, right)), ">=", 0),
        _opb_constraint(((-1, edge), (1, left), (1, right)), ">=", 0),
        _opb_constraint(((-1, edge), (-1, left), (-1, right)), ">=", -2),
    )


def variable_layout() -> Tuple[
    Tuple[Tuple[int, ...], ...],
    Tuple[Mapping[Tuple[int, int], int], ...],
    int,
]:
    orbit_count = len(h_orbits())
    base = tuple(
        tuple(sequence * orbit_count + orbit + 1 for orbit in range(orbit_count))
        for sequence in range(SEQUENCE_COUNT)
    )
    next_variable = SEQUENCE_COUNT * orbit_count + 1
    xor_maps: List[Mapping[Tuple[int, int], int]] = []
    for _ in range(SEQUENCE_COUNT):
        mapping: Dict[Tuple[int, int], int] = {}
        for pair in orbit_pairs():
            mapping[pair] = next_variable
            next_variable += 1
        xor_maps.append(mapping)
    return base, tuple(xor_maps), next_variable - 1


def build_opb() -> Tuple[str, Dict[str, object]]:
    """Build the exact pseudo-Boolean model and deterministic audit metadata."""
    compatible = compatible_row_sum_cases()
    if compatible != ((TARGET_ROW_SUMS, (TARGET_SUPPORT_SIZES, TARGET_FIXED_COUNTS,
                                          TARGET_NONFIXED_COUNTS)),):
        raise AssertionError("the H17 parameter sieve no longer has the expected unique case")

    orbits = h_orbits()
    base, xor_maps, variable_count = variable_layout()
    constraints: List[str] = []

    # One exact negative-support cardinality for each circulant sequence.
    for variables, support_size in zip(base, TARGET_SUPPORT_SIZES):
        constraints.append(
            _opb_constraint(
                ((len(orbit), variable) for orbit, variable in zip(orbits, variables)),
                "=",
                support_size,
            )
        )

    # A single XOR variable per sequence/orbit-pair, shared by all 17 shifts.
    for sequence in range(SEQUENCE_COUNT):
        for (left, right), edge in xor_maps[sequence].items():
            constraints.extend(_opb_xor_constraints(edge, base[sequence][left], base[sequence][right]))

    # PAF_A(t)+...+PAF_D(t)=0 iff the four total disagreements equal 2v.
    for _, multiplicities in shift_pair_multiplicities():
        terms = []
        for sequence in range(SEQUENCE_COUNT):
            for left, right, multiplicity in multiplicities:
                terms.append((multiplicity, xor_maps[sequence][(left, right)]))
        constraints.append(_opb_constraint(terms, "=", TARGET_TOTAL_DISAGREEMENTS))

    base_variable_count = SEQUENCE_COUNT * len(orbits)
    xor_variable_count = SEQUENCE_COUNT * len(orbit_pairs())
    xor_link_constraint_count = 4 * xor_variable_count
    support_equation_count = SEQUENCE_COUNT
    shift_equation_count = len(signed_shift_orbits())
    equality_constraint_count = support_equation_count + shift_equation_count
    lines = [
        "* #variable= {} #constraint= {} #equal= {} intsize= 16".format(
            variable_count, len(constraints), equality_constraint_count
        ),
        "* H17-invariant cyclic Goethals--Seidel/SDS search over Z_515",
        "* x1..x140 select negative H-orbits in four length-515 sequences",
        "* remaining variables are shared exact XORs for unordered H-orbit pairs",
        "* SAT constructs H(2060); UNSAT only excludes this restricted subfamily",
    ]
    lines.extend(constraints)

    generator, subgroup = multiplier_subgroup()
    shift_orbits = signed_shift_orbits()
    metadata: Dict[str, object] = {
        "format": "h17-invariant-cyclic-sds-515-opb-v1",
        "modulus": MODULUS,
        "hadamard_order": HADAMARD_ORDER,
        "multiplier_generator": generator,
        "multiplier_subgroup": list(subgroup),
        "h_orbit_count": len(orbits),
        "h_orbit_size_histogram": {
            str(size): sum(len(orbit) == size for orbit in orbits)
            for size in sorted({len(orbit) for orbit in orbits})
        },
        "signed_shift_orbit_count": len(shift_orbits),
        "signed_shift_orbit_representatives": [orbit[0] for orbit in shift_orbits],
        "signed_shift_orbit_size_histogram": {
            str(size): sum(len(orbit) == size for orbit in shift_orbits)
            for size in sorted({len(orbit) for orbit in shift_orbits})
        },
        "row_sum_candidate_count": len(row_sum_quadruples()),
        "compatible_row_sum_case_count": len(compatible),
        "row_sums": list(TARGET_ROW_SUMS),
        "negative_support_sizes": list(TARGET_SUPPORT_SIZES),
        "fixed_point_negative_counts": list(TARGET_FIXED_COUNTS),
        "nonfixed_orbit_counts": list(TARGET_NONFIXED_COUNTS),
        "orbit_pair_count": len(orbit_pairs()),
        "base_variable_count": base_variable_count,
        "xor_variable_count": xor_variable_count,
        "variable_count": variable_count,
        "support_equation_count": support_equation_count,
        "shift_equation_count": shift_equation_count,
        "xor_link_constraint_count": xor_link_constraint_count,
        "equality_constraint_count": equality_constraint_count,
        "constraint_count": len(constraints),
        "target_total_disagreements": TARGET_TOTAL_DISAGREEMENTS,
        "scope_caveat": (
            "UNSAT excludes H17-invariant cyclic Goethals--Seidel/SDS seeds over Z_515 only"
        ),
    }
    return "\n".join(lines) + "\n", metadata


def export_opb(path: Path) -> Dict[str, object]:
    text, metadata = build_opb()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return metadata


def verify_orbit_assignment(values: Sequence[Sequence[bool]]) -> Dict[str, object]:
    """Independently expand and verify four orbit assignments on all 514 shifts."""
    orbits = h_orbits()
    if len(values) != SEQUENCE_COUNT or any(len(row) != len(orbits) for row in values):
        raise ValueError("expected a 4 by 35 Boolean orbit assignment")
    sequences: List[List[int]] = []
    support_sizes = []
    fixed_counts = []
    nonfixed_counts = []
    for sequence_index, row in enumerate(values):
        signs = [1] * MODULUS
        selected = [bool(value) for value in row]
        for selected_value, orbit in zip(selected, orbits):
            if selected_value:
                for residue in orbit:
                    signs[residue] = -1
        support = sum(sign < 0 for sign in signs)
        fixed = sum(selected_value and len(orbit) == 1 for selected_value, orbit in zip(selected, orbits))
        nonfixed = sum(selected_value and len(orbit) == MULTIPLIER_ORDER
                       for selected_value, orbit in zip(selected, orbits))
        if support != TARGET_SUPPORT_SIZES[sequence_index]:
            raise ValueError("sequence {} has support {}, expected {}".format(
                sequence_index, support, TARGET_SUPPORT_SIZES[sequence_index]))
        sequences.append(signs)
        support_sizes.append(support)
        fixed_counts.append(fixed)
        nonfixed_counts.append(nonfixed)

    for shift in range(1, MODULUS):
        disagreements = sum(
            signs[index] != signs[(index + shift) % MODULUS]
            for signs in sequences
            for index in range(MODULUS)
        )
        if disagreements != TARGET_TOTAL_DISAGREEMENTS:
            raise ValueError("shift {} has {} total disagreements, expected {}".format(
                shift, disagreements, TARGET_TOTAL_DISAGREEMENTS))
    return {
        "verified": True,
        "row_sums": [sum(signs) for signs in sequences],
        "negative_support_sizes": support_sizes,
        "fixed_point_negative_counts": fixed_counts,
        "nonfixed_orbit_counts": nonfixed_counts,
        "checked_nonzero_shifts": MODULUS - 1,
    }


_MODEL_LITERAL = re.compile(r"(?P<negative>[-~]?)(?:x)?(?P<variable>[1-9][0-9]*)$")


def parse_signed_model_text(text: str) -> Dict[int, bool]:
    """Parse standard OPB solver lines such as ``v x1 -x2 ... 0``."""
    assignments: Dict[int, bool] = {}
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith(("c", "s", "*")):
            continue
        tokens = stripped.split()
        if tokens and tokens[0] == "v":
            tokens = tokens[1:]
        for token in tokens:
            if token == "0":
                continue
            match = _MODEL_LITERAL.fullmatch(token)
            if match is None:
                continue
            variable = int(match.group("variable"))
            value = match.group("negative") not in ("-", "~")
            previous = assignments.get(variable)
            if previous is not None and previous != value:
                raise ValueError("model assigns x{} both truth values".format(variable))
            assignments[variable] = value
    return assignments


def verify_signed_model(path: Path) -> Dict[str, object]:
    """Verify a complete solver model without evaluating the OPB text itself."""
    assignments = parse_signed_model_text(path.read_text(encoding="utf-8"))
    base, xor_maps, variable_count = variable_layout()
    missing = [variable for variable in range(1, variable_count + 1) if variable not in assignments]
    extras = sorted(variable for variable in assignments if variable > variable_count)
    if missing or extras:
        raise ValueError("model must assign exactly x1..x{} ({} missing, {} extra)".format(
            variable_count, len(missing), len(extras)))

    rows = [[assignments[variable] for variable in variables] for variables in base]
    for sequence in range(SEQUENCE_COUNT):
        for (left, right), edge in xor_maps[sequence].items():
            expected = rows[sequence][left] != rows[sequence][right]
            if assignments[edge] != expected:
                raise ValueError("x{} is not the required XOR auxiliary".format(edge))
    result = verify_orbit_assignment(rows)
    result["assigned_variable_count"] = len(assignments)
    return result


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Audit/export the H17-invariant cyclic SDS attack on order 2060."
    )
    parser.add_argument("--export-opb", type=Path, help="write the exact OPB instance")
    parser.add_argument("--verify-model", type=Path, help="independently verify a signed OPB model")
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = parse_args(argv)
    if args.export_opb is not None and args.verify_model is not None:
        raise SystemExit("choose at most one of --export-opb and --verify-model")
    if args.verify_model is not None:
        output = verify_signed_model(args.verify_model)
    elif args.export_opb is not None:
        output = export_opb(args.export_opb)
        output = dict(output)
        output["exported_opb"] = str(args.export_opb)
    else:
        _, output = build_opb()
    print(json.dumps(output, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
