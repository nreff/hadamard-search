#!/usr/bin/env python3

import argparse
import hashlib
import json
from pathlib import Path
import sys
import time
from typing import Dict, List, Optional, Sequence, Tuple

import z3


ORDER = 167
HALF = 84
U_LENGTH = 83
V_LENGTH = 81
PUBLISHED_Q_RUNS = (83, 2, 81, 1)
PUBLISHED_S_RUNS = (
    (4,) * 5
    + (2, 1, 1) * 5
    + (1, 5)
    + (4,) * 4
    + (2, 1, 1) * 6
    + (4,) * 4
    + (3,)
    + (1, 2, 1) * 5
    + (3,)
    + (4,) * 4
    + (3,)
    + (1, 2, 1) * 5
)
PUBLISHED_DEFECTS = {
    4: -512,
    8: 384,
    12: -256,
    16: 128,
    26: -64,
    30: 128,
    34: -192,
    38: 256,
    42: -320,
    46: 256,
    50: -192,
    54: 128,
    58: -64,
}
def expand_runs(runs: Sequence[int]) -> List[int]:
    values = []
    sign = 1
    for run in runs:
        if run <= 0:
            raise ValueError("run lengths must be positive")
        values.extend([sign] * run)
        sign = -sign
    return values


def aperiodic_autocorrelation(values: Sequence[int], shift: int) -> int:
    if shift < 0 or shift >= len(values):
        raise ValueError("aperiodic shift falls outside the sequence")
    return sum(values[index] * values[index + shift] for index in range(len(values) - shift))


def special_quadruple(s: Sequence[int], q: Sequence[int]) -> Tuple[List[int], ...]:
    if len(s) != ORDER or len(q) != ORDER:
        raise ValueError("special quadruple requires length-167 sequences")
    involution = [1] * HALF + [-1] * (ORDER - HALF)
    s_prime = [value * flip for value, flip in zip(s, involution)]
    sq = [left * right for left, right in zip(s, q)]
    sq_prime = [value * flip for value, flip in zip(sq, involution)]
    return list(s), s_prime, sq, sq_prime


def aggregate_defects(quadruple: Sequence[Sequence[int]]) -> Dict[int, int]:
    if len(quadruple) != 4 or any(len(values) != ORDER for values in quadruple):
        raise ValueError("expected four length-167 sequences")
    defects = {}
    for shift in range(1, ORDER):
        total = sum(aperiodic_autocorrelation(values, shift) for values in quadruple)
        if total != 0:
            defects[shift] = total
    return defects


def published_sequences() -> Tuple[List[int], List[int]]:
    q = expand_runs(PUBLISHED_Q_RUNS)
    s = expand_runs(PUBLISHED_S_RUNS)
    if len(q) != ORDER or len(s) != ORDER:
        raise AssertionError("published run-length encodings must have length 167")
    return q, s


def fixed_q_components(s: Sequence[int]) -> Tuple[List[int], List[int]]:
    if len(s) != ORDER:
        raise ValueError("fixed-q reduction requires a length-167 sequence")
    return list(s[:U_LENGTH]), list(s[85:166])


def reduced_correlations(u: Sequence[int], v: Sequence[int]) -> Dict[int, int]:
    if len(u) != U_LENGTH or len(v) != V_LENGTH:
        raise ValueError("expected U length 83 and V length 81")
    return {
        shift: aperiodic_autocorrelation(u, shift)
        + (aperiodic_autocorrelation(v, shift) if shift < len(v) else 0)
        for shift in range(1, 82)
    }


def _bounded_sum_vectors(lengths: Sequence[int], target: int) -> List[Tuple[int, ...]]:
    vectors: List[Tuple[int, ...]] = []

    def visit(index: int, partial_sum: int, partial: List[int]) -> None:
        if index == len(lengths):
            if partial_sum == target:
                vectors.append(tuple(partial))
            return
        remaining_bound = sum(lengths[index + 1 :])
        length = lengths[index]
        for value in range(-length, length + 1, 2):
            next_sum = partial_sum + value
            if next_sum - remaining_bound <= target <= next_sum + remaining_bound:
                partial.append(value)
                visit(index + 1, next_sum, partial)
                partial.pop()

    visit(0, 0, [])
    return vectors


def compressed_residue_vectors(
    length: int, modulus: int, even_sum: int, odd_sum: int
) -> List[Tuple[int, ...]]:
    if modulus <= 0 or modulus % 2 != 0:
        raise ValueError("residue compression currently requires a positive even modulus")
    residue_lengths = [
        1 + (length - 1 - residue) // modulus if residue < length else 0
        for residue in range(modulus)
    ]
    even_residues = list(range(0, modulus, 2))
    odd_residues = list(range(1, modulus, 2))
    even_vectors = _bounded_sum_vectors(
        [residue_lengths[index] for index in even_residues], even_sum
    )
    odd_vectors = _bounded_sum_vectors(
        [residue_lengths[index] for index in odd_residues], odd_sum
    )
    vectors = []
    for even_values in even_vectors:
        for odd_values in odd_vectors:
            values = [0] * modulus
            for index, value in zip(even_residues, even_values):
                values[index] = value
            for index, value in zip(odd_residues, odd_values):
                values[index] = value
            vectors.append(tuple(values))
    return vectors


def periodic_autocorrelation(values: Sequence[int], shift: int) -> int:
    if not values:
        raise ValueError("periodic autocorrelation requires a nonempty sequence")
    return sum(
        values[index] * values[(index + shift) % len(values)]
        for index in range(len(values))
    )


def fixed_q_quotient_target(modulus: int) -> Tuple[int, ...]:
    target = [0] * modulus
    target[0] = U_LENGTH + V_LENGTH
    target[82 % modulus] -= 1
    target[(-82) % modulus] -= 1
    return tuple(target)


def fixed_q_quotient_cases(
    modulus: int = 4,
) -> List[Tuple[Tuple[int, ...], Tuple[int, ...]]]:
    u_vectors = compressed_residue_vectors(U_LENGTH, modulus, 0, 9)
    v_vectors = compressed_residue_vectors(V_LENGTH, modulus, 9, 0)
    target = fixed_q_quotient_target(modulus)
    v_buckets: Dict[Tuple[int, ...], List[Tuple[int, ...]]] = {}
    for values in v_vectors:
        signature = tuple(
            periodic_autocorrelation(values, shift) for shift in range(modulus)
        )
        v_buckets.setdefault(signature, []).append(values)

    cases = []
    for u_values in u_vectors:
        u_signature = tuple(
            periodic_autocorrelation(u_values, shift) for shift in range(modulus)
        )
        complement = tuple(
            target[index] - u_signature[index] for index in range(modulus)
        )
        cases.extend((u_values, v_values) for v_values in v_buckets.get(complement, []))
    return sorted(cases)


def _residue_plus_counts(length: int, sums: Sequence[int]) -> Tuple[int, ...]:
    modulus = len(sums)
    lengths = tuple(len(range(residue, length, modulus)) for residue in range(modulus))
    if any(
        (residue_length + residue_sum) % 2
        for residue_length, residue_sum in zip(lengths, sums)
    ):
        raise ValueError("residue sums have the wrong parity for their class lengths")
    plus_counts = tuple(
        (residue_length + residue_sum) // 2
        for residue_length, residue_sum in zip(lengths, sums)
    )
    if any(
        plus_count < 0 or plus_count > residue_length
        for plus_count, residue_length in zip(plus_counts, lengths)
    ):
        raise ValueError("residue sum falls outside its class bounds")
    return plus_counts


def fixed_q_parity_case_is_feasible(
    case: Tuple[Tuple[int, ...], Tuple[int, ...]], modulus: int = 4
) -> bool:
    """Apply exact lag-parity consequences to a cyclic quotient case.

    Reducing every disagreement cardinality ``D_k = 82-k`` modulo two gives a
    linear system in the sign bits.  XORing the rows with
    ``k mod 8 in {3,4,5,6}`` shows that the U residue-3 plus count is even;
    the U odd marginal then makes the residue-1 plus count odd.  Likewise the
    rows with ``k mod 8 in {4,5,6,7}`` make the V residue-2 plus count even,
    and the V even marginal makes the residue-0 plus count odd.
    """
    if modulus != 4:
        raise ValueError("the fixed-q lag-parity sieve is derived for modulus 4")
    u_sums, v_sums = case
    if len(u_sums) != modulus or len(v_sums) != modulus:
        raise ValueError("quotient case width does not match its modulus")
    u_plus = _residue_plus_counts(U_LENGTH, u_sums)
    v_plus = _residue_plus_counts(V_LENGTH, v_sums)
    return u_plus[1] % 2 == 1 and v_plus[0] % 2 == 1


def fixed_q_parity_feasible_case_indices(modulus: int = 4) -> List[int]:
    return [
        case_index
        for case_index, case in enumerate(fixed_q_quotient_cases(modulus))
        if fixed_q_parity_case_is_feasible(case, modulus)
    ]


def fixed_q_quotient_orbits(modulus: int = 4) -> List[Tuple[int, ...]]:
    """Partition quotient cases under reversal of the length-81 V sequence."""
    cases = fixed_q_quotient_cases(modulus)
    case_indices = {case: index for index, case in enumerate(cases)}
    unseen = set(range(len(cases)))
    orbits = []
    while unseen:
        case_index = min(unseen)
        u_sums, v_sums = cases[case_index]
        reversed_v_sums = tuple(
            v_sums[(V_LENGTH - 1 - residue) % modulus]
            for residue in range(modulus)
        )
        reversed_index = case_indices[(u_sums, reversed_v_sums)]
        orbit = tuple(sorted({case_index, reversed_index}))
        unseen.difference_update(orbit)
        orbits.append(orbit)
    return orbits


def fixed_q_parity_quotient_orbits(modulus: int = 4) -> List[Tuple[int, ...]]:
    """Return V-reversal orbits that survive the exact lag-parity sieve."""
    feasible = set(fixed_q_parity_feasible_case_indices(modulus))
    orbits = fixed_q_quotient_orbits(modulus)
    if any(
        bool(feasible.intersection(orbit)) != set(orbit).issubset(feasible)
        for orbit in orbits
    ):
        raise AssertionError("lag-parity feasibility is not closed under V reversal")
    return [orbit for orbit in orbits if orbit[0] in feasible]


def fixed_q_opb_v_reversal_variable_map() -> Tuple[int, ...]:
    """Return the one-based OPB variable permutation induced by V reversal."""
    variable_map = list(range(1 + U_LENGTH + V_LENGTH))
    for index in range(V_LENGTH):
        variable_map[U_LENGTH + 1 + index] = U_LENGTH + V_LENGTH - index

    next_variable = U_LENGTH + V_LENGTH + 1
    for shift in range(1, 82):
        u_edge_count = U_LENGTH - shift
        variable_map.extend(range(next_variable, next_variable + u_edge_count))
        next_variable += u_edge_count
        v_edge_count = max(0, V_LENGTH - shift)
        first_v_edge = next_variable
        variable_map.extend(range(next_variable, next_variable + v_edge_count))
        for edge_index in range(v_edge_count):
            variable_map[first_v_edge + edge_index] = (
                first_v_edge + v_edge_count - 1 - edge_index
            )
        next_variable += v_edge_count
    if len(variable_map) != next_variable or next_variable - 1 != 6806:
        raise AssertionError("fixed-q OPB reversal map has inconsistent numbering")
    return tuple(variable_map)


def _add_residue_sum_constraints(
    solver, variables: Sequence[z3.BoolRef], modulus: int, sums: Sequence[int]
) -> None:
    for residue, residue_sum in enumerate(sums):
        indices = list(range(residue, len(variables), modulus))
        plus_count = (len(indices) + residue_sum) // 2
        solver.add(z3.PbEq([(variables[index], 1) for index in indices], plus_count))


def add_fixed_q_quotient_constraints(
    solver,
    u: Sequence[z3.BoolRef],
    v: Sequence[z3.BoolRef],
    modulus: int = 4,
    case_index: Optional[int] = None,
) -> Tuple[int, int]:
    cases = fixed_q_quotient_cases(modulus)
    feasible_indices = (
        fixed_q_parity_feasible_case_indices(modulus)
        if modulus == 4
        else list(range(len(cases)))
    )
    if case_index is not None:
        if case_index < 0 or case_index >= len(cases):
            raise ValueError(
                "quotient case index {} falls outside 0..{}".format(
                    case_index, len(cases) - 1
                )
            )
        if case_index not in feasible_indices:
            raise ValueError(
                "quotient case {} is excluded by the exact lag-parity lemma".format(
                    case_index
                )
            )
        u_sums, v_sums = cases[case_index]
        _add_residue_sum_constraints(solver, u, modulus, u_sums)
        _add_residue_sum_constraints(solver, v, modulus, v_sums)
        return len(cases), 0

    selectors = [
        z3.Bool("quotient_{}_case_{}".format(modulus, index))
        for index in feasible_indices
    ]
    solver.add(z3.PbEq([(selector, 1) for selector in selectors], 1))
    for selector, case_index in zip(selectors, feasible_indices):
        u_sums, v_sums = cases[case_index]
        for variables, sums in ((u, u_sums), (v, v_sums)):
            for residue, residue_sum in enumerate(sums):
                indices = list(range(residue, len(variables), modulus))
                plus_count = (len(indices) + residue_sum) // 2
                solver.add(
                    z3.Implies(
                        selector,
                        z3.PbEq(
                            [(variables[index], 1) for index in indices], plus_count
                        ),
                    )
                )
    return len(cases), len(selectors)


def _opb_constraint(terms: Sequence[Tuple[int, int]], relation: str, target: int) -> str:
    rendered = " ".join(
        "{:+d} x{}".format(coefficient, variable)
        for coefficient, variable in terms
        if coefficient
    )
    return "{} {} {} ;".format(rendered, relation, target)


def _opb_xor_constraints(edge: int, left: int, right: int) -> List[str]:
    return [
        _opb_constraint(((1, edge), (-1, left), (1, right)), ">=", 0),
        _opb_constraint(((1, edge), (1, left), (-1, right)), ">=", 0),
        _opb_constraint(((-1, edge), (1, left), (1, right)), ">=", 0),
        _opb_constraint(((-1, edge), (-1, left), (-1, right)), ">=", -2),
    ]


def build_fixed_q_opb(
    quotient_modulus: int = 0, quotient_case_index: Optional[int] = None
) -> Tuple[str, Dict[str, object]]:
    if quotient_modulus not in (0, 4):
        raise ValueError("OPB export supports quotient modulus 0 or 4")
    if quotient_modulus == 0 and quotient_case_index is not None:
        raise ValueError("an OPB quotient case requires quotient modulus 4")

    u_variables = list(range(1, U_LENGTH + 1))
    v_variables = list(range(U_LENGTH + 1, U_LENGTH + V_LENGTH + 1))
    constraints = []
    constraints.append(
        _opb_constraint([(1, u_variables[index]) for index in range(0, U_LENGTH, 2)], "=", 21)
    )
    constraints.append(
        _opb_constraint([(1, u_variables[index]) for index in range(1, U_LENGTH, 2)], "=", 25)
    )
    constraints.append(
        _opb_constraint([(1, v_variables[index]) for index in range(0, V_LENGTH, 2)], "=", 25)
    )
    constraints.append(
        _opb_constraint([(1, v_variables[index]) for index in range(1, V_LENGTH, 2)], "=", 20)
    )
    constraints.append(_opb_constraint(((1, u_variables[0]),), "=", 1))
    constraints.append(_opb_constraint(((1, u_variables[-1]),), "=", 0))

    next_variable = U_LENGTH + V_LENGTH + 1
    correlation_constraints = []
    for shift in range(1, 82):
        edges = []
        for variables in (u_variables, v_variables):
            for index in range(max(0, len(variables) - shift)):
                edge = next_variable
                next_variable += 1
                constraints.extend(
                    _opb_xor_constraints(edge, variables[index], variables[index + shift])
                )
                edges.append(edge)
        correlation_constraints.append(
            _opb_constraint([(1, edge) for edge in edges], "=", 82 - shift)
        )
    constraints.extend(correlation_constraints)

    quotient_case_count = 0
    quotient_case = None
    if quotient_modulus:
        cases = fixed_q_quotient_cases(quotient_modulus)
        quotient_case_count = len(cases)
        if quotient_case_index is None:
            raise ValueError("sharded OPB export requires a quotient case index")
        if quotient_case_index < 0 or quotient_case_index >= quotient_case_count:
            raise ValueError(
                "quotient case index {} falls outside 0..{}".format(
                    quotient_case_index, quotient_case_count - 1
                )
            )
        quotient_case = cases[quotient_case_index]
        for variables, sums in zip((u_variables, v_variables), quotient_case):
            for residue, residue_sum in enumerate(sums):
                selected = variables[residue::quotient_modulus]
                plus_count = (len(selected) + residue_sum) // 2
                constraints.append(
                    _opb_constraint([(1, variable) for variable in selected], "=", plus_count)
                )

    variable_count = next_variable - 1
    edge_variable_count = variable_count - U_LENGTH - V_LENGTH
    equality_count = 4 + 2 + 81 + (2 * quotient_modulus if quotient_modulus else 0)
    integer_size = 8
    lines = [
        "* #variable= {} #constraint= {} #equal= {} intsize= {}".format(
            variable_count, len(constraints), equality_count, integer_size
        ),
        "* fixed-q length-167 modular Golay completion",
        "* x1..x83 are U signs; x84..x164 are V signs; 1 means +1",
        "* remaining variables are exact XOR edge indicators",
    ]
    if quotient_case_index is not None:
        lines.append(
            "* quotient_modulus={} quotient_case_index={}".format(
                quotient_modulus, quotient_case_index
            )
        )
    lines.extend(constraints)
    metadata: Dict[str, object] = {
        "variable_count": variable_count,
        "base_sign_variable_count": U_LENGTH + V_LENGTH,
        "edge_variable_count": edge_variable_count,
        "constraint_count": len(constraints),
        "equality_constraint_count": equality_count,
        "integer_size": integer_size,
        "xor_link_constraint_count": 4 * edge_variable_count,
        "correlation_equation_count": 81,
        "quotient_modulus": quotient_modulus,
        "quotient_case_count": quotient_case_count,
        "quotient_case_index": quotient_case_index,
    }
    if quotient_case is not None:
        metadata["u_residue_sums"] = list(quotient_case[0])
        metadata["v_residue_sums"] = list(quotient_case[1])
    return "\n".join(lines) + "\n", metadata


def _fixed_q_t_fixed_literals() -> Tuple[
    Tuple[Tuple[Optional[int], bool], ...],
    Tuple[Tuple[Optional[int], bool], ...],
]:
    """Express a T-fixed full assignment through 82 independent OPB bits.

    A literal is ``(variable, complemented)``.  A missing variable denotes the
    Boolean constant stored in ``complemented``.
    """
    u: List[Tuple[Optional[int], bool]] = [(None, False)] * U_LENGTH
    u[0] = (None, True)
    u[-1] = (None, False)
    for index in range(1, 41):
        u[index] = (index, False)
        u[82 - index] = (index, index % 2 == 0)
    u[41] = (41, False)

    v: List[Tuple[Optional[int], bool]] = [(None, False)] * V_LENGTH
    for index in range(40):
        variable = 42 + index
        v[index] = (variable, False)
        v[80 - index] = (variable, index % 2 == 1)
    v[40] = (82, False)
    if any(variable is None for variable, _ in u[1:-1]) or any(
        variable is None for variable, _ in v
    ):
        raise AssertionError("T-fixed literal parameterization is incomplete")
    return tuple(u), tuple(v)


def _add_opb_affine_literal(
    terms: List[Tuple[int, int]],
    target: int,
    literal: Tuple[Optional[int], bool],
    coefficient: int = 1,
) -> int:
    variable, complemented = literal
    if variable is None:
        return target - coefficient * int(complemented)
    if complemented:
        terms.append((-coefficient, variable))
        return target - coefficient
    terms.append((coefficient, variable))
    return target


def _opb_combined_constraint(
    terms: Sequence[Tuple[int, int]], relation: str, target: int
) -> str:
    combined: Dict[int, int] = {}
    for coefficient, variable in terms:
        combined[variable] = combined.get(variable, 0) + coefficient
    return _opb_constraint(
        [(coefficient, variable) for variable, coefficient in sorted(combined.items())],
        relation,
        target,
    )


def build_fixed_q_t_fixed_opb(
    quotient_case_index: int,
) -> Tuple[str, Dict[str, object]]:
    """Build the exact OPB restriction fixed by the normalized involution T.

    In sign notation, ``T(U)_i = -(-1)^i U_(82-i)`` and
    ``T(V)_i = (-1)^i V_(80-i)``.  This is a construction subfamily: SAT gives
    a full fixed-q solution, while UNSAT only excludes T-fixed solutions.
    """
    cases = fixed_q_quotient_cases(4)
    if quotient_case_index < 0 or quotient_case_index >= len(cases):
        raise ValueError(
            "quotient case index {} falls outside 0..{}".format(
                quotient_case_index, len(cases) - 1
            )
        )
    if quotient_case_index not in fixed_q_parity_feasible_case_indices(4):
        raise ValueError(
            "quotient case {} is excluded by the exact lag-parity lemma".format(
                quotient_case_index
            )
        )

    u_sums, v_sums = cases[quotient_case_index]
    u_literals, v_literals = _fixed_q_t_fixed_literals()
    next_variable = 83
    xor_constraints: List[str] = []
    correlation_constraints: List[str] = []
    edge_variable_count = 0

    # Odd correlations vanish separately under T.  At every even lag, all
    # noncentral edges occur in equal-disagreement reflection pairs.
    for shift in range(2, 82, 2):
        terms: List[Tuple[int, int]] = []
        target = 82 - shift
        central_disagreements = 0
        for literals in (u_literals, v_literals):
            length = len(literals)
            for index in range(length - shift):
                reflected_index = length - 1 - index - shift
                if index > reflected_index:
                    continue
                left = literals[index]
                right = literals[index + shift]
                if index == reflected_index:
                    left_variable, left_complemented = left
                    right_variable, right_complemented = right
                    if left_variable != right_variable:
                        raise AssertionError("central T-fixed edge did not collapse")
                    central_disagreements += int(
                        left_complemented != right_complemented
                    )
                    continue

                left_variable, left_complemented = left
                right_variable, right_complemented = right
                if left_variable is None or right_variable is None:
                    if left_variable is None and right_variable is None:
                        target -= 2 * int(left_complemented != right_complemented)
                    else:
                        if left_variable is None:
                            literal = (
                                right_variable,
                                right_complemented != left_complemented,
                            )
                        else:
                            literal = (
                                left_variable,
                                left_complemented != right_complemented,
                            )
                        target = _add_opb_affine_literal(
                            terms, target, literal, coefficient=2
                        )
                    continue

                edge = next_variable
                next_variable += 1
                edge_variable_count += 1
                xor_constraints.extend(
                    _opb_xor_constraints(edge, left_variable, right_variable)
                )
                if left_complemented == right_complemented:
                    terms.append((2, edge))
                else:
                    terms.append((-2, edge))
                    target -= 2

        target -= central_disagreements
        if target % 2 or any(coefficient % 2 for coefficient, _ in terms):
            raise AssertionError("T-fixed correlation equation did not halve")
        correlation_constraints.append(
            _opb_combined_constraint(
                [(coefficient // 2, variable) for coefficient, variable in terms],
                "=",
                target // 2,
            )
        )

    quotient_constraints: List[str] = []
    for literals, sums in ((u_literals, u_sums), (v_literals, v_sums)):
        for residue, residue_sum in enumerate(sums):
            indices = list(range(residue, len(literals), 4))
            target = (len(indices) + residue_sum) // 2
            terms = []
            for index in indices:
                target = _add_opb_affine_literal(terms, target, literals[index])
            quotient_constraints.append(_opb_combined_constraint(terms, "=", target))

    constraints = xor_constraints + correlation_constraints + quotient_constraints
    variable_count = next_variable - 1
    equality_count = len(correlation_constraints) + len(quotient_constraints)
    lines = [
        "* #variable= {} #constraint= {} #equal= {} intsize= 7".format(
            variable_count, len(constraints), equality_count
        ),
        "* fixed-q T-fixed reduced construction subfamily",
        "* x1..x41 parameterize U; x42..x82 parameterize V",
        "* quotient_modulus=4 quotient_case_index={}".format(quotient_case_index),
    ]
    lines.extend(constraints)
    metadata: Dict[str, object] = {
        "variable_count": variable_count,
        "base_sign_variable_count": 82,
        "edge_variable_count": edge_variable_count,
        "constraint_count": len(constraints),
        "equality_constraint_count": equality_count,
        "xor_link_constraint_count": len(xor_constraints),
        "correlation_equation_count": len(correlation_constraints),
        "quotient_modulus": 4,
        "quotient_case_index": quotient_case_index,
        "u_residue_sums": list(u_sums),
        "v_residue_sums": list(v_sums),
        "subfamily": "fixed_by_normalized_involution_T",
    }
    return "\n".join(lines) + "\n", metadata


def _write_text_atomic(path: Path, contents: str) -> None:
    temporary_path = path.with_name(path.name + ".tmp")
    temporary_path.write_text(contents, encoding="utf-8")
    temporary_path.replace(path)


def write_fixed_q_opb_campaign(
    directory: Path,
    quotient_modulus: int = 4,
    symmetry_reduced: bool = True,
) -> Dict[str, object]:
    if quotient_modulus != 4:
        raise ValueError("OPB campaign export supports quotient modulus 4")
    cases = fixed_q_quotient_cases(quotient_modulus)
    feasible_indices = fixed_q_parity_feasible_case_indices(quotient_modulus)
    feasible_set = set(feasible_indices)
    excluded_indices = [index for index in range(len(cases)) if index not in feasible_set]
    selected_orbits = (
        fixed_q_parity_quotient_orbits(quotient_modulus)
        if symmetry_reduced
        else [(case_index,) for case_index in range(len(cases))]
    )
    expected_filenames = {
        "fixed-q-mod{}-case-{:02d}.opb".format(quotient_modulus, orbit[0])
        for orbit in selected_orbits
    }
    directory.mkdir(parents=True, exist_ok=True)
    existing_filenames = {path.name for path in directory.glob("*.opb")}
    unexpected_filenames = existing_filenames - expected_filenames
    if unexpected_filenames:
        raise ValueError(
            "OPB campaign directory contains stale shard {}; use a new or cleaned directory".format(
                sorted(unexpected_filenames)[0]
            )
        )
    shards = []
    for orbit in selected_orbits:
        case_index = orbit[0]
        text, metadata = build_fixed_q_opb(quotient_modulus, case_index)
        filename = "fixed-q-mod{}-case-{:02d}.opb".format(quotient_modulus, case_index)
        path = directory / filename
        _write_text_atomic(path, text)
        shards.append(
            {
                "case_index": case_index,
                "file": filename,
                "sha256": hashlib.sha256(text.encode("utf-8")).hexdigest(),
                "u_residue_sums": metadata["u_residue_sums"],
                "v_residue_sums": metadata["v_residue_sums"],
                "covered_case_indices": list(orbit),
            }
        )
    _, common = build_fixed_q_opb(quotient_modulus, 0)
    manifest = {
        "format": "modular-golay-fixed-q-opb-campaign-v3",
        "order": ORDER,
        "quotient_modulus": quotient_modulus,
        "symmetry_reduced": symmetry_reduced,
        "symmetry_action": {
            "name": "reverse_v",
            "sequence_index_map": "V[i] -> V[80-i]",
            "quotient_residue_permutation": [0, 3, 2, 1],
            "base_variable_map": "x1..x83 fixed; x(84+i) -> x(164-i), 0<=i<=80",
            "edge_variable_map": (
                "within every lag block, U-edge variables are fixed and "
                "V-edge variables are reversed"
            ),
        }
        if symmetry_reduced
        else None,
        "full_case_count": len(cases),
        "parity_prefilter_applied": symmetry_reduced,
        "parity_feasible_case_count": len(feasible_indices),
        "parity_feasible_case_indices": feasible_indices,
        "parity_excluded_case_indices": excluded_indices,
        "parity_lemma": {
            "u_residue_1_plus_count": "odd",
            "v_residue_0_plus_count": "odd",
            "derivation": "mod-2 sums of exact lag-disagreement equations",
        },
        "shard_count": len(shards),
        "variable_count_per_shard": common["variable_count"],
        "constraint_count_per_shard": common["constraint_count"],
        "edge_variable_count_per_shard": common["edge_variable_count"],
        "shards": shards,
    }
    _write_text_atomic(
        directory / "manifest.json",
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
    )
    return manifest


def write_fixed_q_t_fixed_opb_campaign(directory: Path) -> Dict[str, object]:
    """Export the exhaustive quotient campaign inside the T-fixed subfamily."""
    cases = fixed_q_quotient_cases(4)
    feasible_indices = fixed_q_parity_feasible_case_indices(4)
    excluded_indices = [
        case_index for case_index in range(len(cases)) if case_index not in feasible_indices
    ]
    orbits = fixed_q_parity_quotient_orbits(4)
    expected_filenames = {
        "fixed-q-t-fixed-mod4-case-{:02d}.opb".format(orbit[0]) for orbit in orbits
    }
    directory.mkdir(parents=True, exist_ok=True)
    existing_filenames = {path.name for path in directory.glob("*.opb")}
    unexpected_filenames = existing_filenames - expected_filenames
    if unexpected_filenames:
        raise ValueError(
            (
                "T-fixed OPB campaign directory contains stale shard {}; "
                "use a new or cleaned directory"
            ).format(sorted(unexpected_filenames)[0])
        )

    shards = []
    for orbit in orbits:
        case_index = orbit[0]
        text, metadata = build_fixed_q_t_fixed_opb(case_index)
        filename = "fixed-q-t-fixed-mod4-case-{:02d}.opb".format(case_index)
        _write_text_atomic(directory / filename, text)
        shards.append(
            {
                "case_index": case_index,
                "file": filename,
                "sha256": hashlib.sha256(text.encode("utf-8")).hexdigest(),
                "u_residue_sums": metadata["u_residue_sums"],
                "v_residue_sums": metadata["v_residue_sums"],
                "covered_case_indices": list(orbit),
            }
        )

    _, common = build_fixed_q_t_fixed_opb(orbits[0][0])
    manifest = {
        "format": "modular-golay-fixed-q-t-fixed-opb-campaign-v1",
        "order": ORDER,
        "quotient_modulus": 4,
        "subfamily": "fixed_by_normalized_involution_T",
        "subfamily_warning": (
            "SAT gives an unconditional fixed-q construction; UNSAT excludes only "
            "the T-fixed subfamily"
        ),
        "involution": {
            "u": "U[i] -> -(-1)^i U[82-i]",
            "v": "V[i] -> (-1)^i V[80-i]",
        },
        "parity_lemma": {
            "u_residue_1_plus_count": "odd",
            "v_residue_0_plus_count": "odd",
            "derivation": "mod-2 sums of exact lag-disagreement equations",
        },
        "symmetry_reduced": True,
        "symmetry_action": {
            "name": "reverse_v",
            "sequence_index_map": "V[i] -> V[80-i]",
            "commutes_with_T": True,
            "quotient_residue_permutation": [0, 3, 2, 1],
            "reduced_base_map": (
                "x1..x41 fixed; x(42+i) is fixed for even i and complemented "
                "for odd i, 0<=i<40; x82 fixed"
            ),
        },
        "coverage_basis": "exact lag parity plus V reversal commuting with T",
        "full_case_count": len(cases),
        "parity_feasible_case_count": len(feasible_indices),
        "parity_feasible_case_indices": feasible_indices,
        "parity_excluded_case_indices": excluded_indices,
        "reversal_representative_count": len(orbits),
        "shard_count": len(shards),
        "variable_count_per_shard": common["variable_count"],
        "constraint_count_per_shard": common["constraint_count"],
        "edge_variable_count_per_shard": common["edge_variable_count"],
        "shards": shards,
    }
    _write_text_atomic(
        directory / "manifest.json",
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
    )
    return manifest


def _opb_model_assignments(path: Path) -> Dict[int, bool]:
    assignments: Dict[int, bool] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped.startswith("v "):
            continue
        for token in stripped.split()[1:]:
            if token == "0":
                continue
            value = not token.startswith("-")
            name = token[1:] if token.startswith("-") else token
            if not name.startswith("x") or not name[1:].isdigit():
                raise ValueError("unsupported OPB model token {!r}".format(token))
            variable = int(name[1:])
            if variable <= 0 or name != "x{}".format(variable):
                raise ValueError("noncanonical OPB model token {!r}".format(token))
            if variable in assignments:
                relation = "contradictory" if assignments[variable] != value else "duplicate"
                raise ValueError(
                    "{} OPB model assignment for x{}".format(relation, variable)
                )
            assignments[variable] = value
    return assignments


def signs_from_opb_model(path: Path) -> Tuple[List[int], List[int]]:
    assignments = _opb_model_assignments(path)
    missing = [index for index in range(1, U_LENGTH + V_LENGTH + 1) if index not in assignments]
    if missing:
        raise ValueError(
            "OPB model is missing {} base assignments; first missing x{}".format(
                len(missing), missing[0]
            )
        )
    signs = [1 if assignments[index] else -1 for index in range(1, U_LENGTH + V_LENGTH + 1)]
    return signs[:U_LENGTH], signs[U_LENGTH:]


def signs_from_t_fixed_opb_model(path: Path) -> Tuple[List[int], List[int]]:
    assignments = _opb_model_assignments(path)
    missing = [index for index in range(1, 83) if index not in assignments]
    if missing:
        raise ValueError(
            "T-fixed OPB model is missing {} base assignments; first missing x{}".format(
                len(missing), missing[0]
            )
        )
    u_literals, v_literals = _fixed_q_t_fixed_literals()

    def expand(literals: Sequence[Tuple[Optional[int], bool]]) -> List[int]:
        values = []
        for variable, complemented in literals:
            bit = complemented if variable is None else assignments[variable] != complemented
            values.append(1 if bit else -1)
        return values

    return expand(u_literals), expand(v_literals)


def build_fixed_q_model():
    solver = z3.Solver()
    u = [z3.Bool("u_{}".format(index)) for index in range(U_LENGTH)]
    v = [z3.Bool("v_{}".format(index)) for index in range(V_LENGTH)]

    solver.add(z3.PbEq([(u[index], 1) for index in range(0, U_LENGTH, 2)], 21))
    solver.add(z3.PbEq([(u[index], 1) for index in range(1, U_LENGTH, 2)], 25))
    solver.add(z3.PbEq([(v[index], 1) for index in range(0, V_LENGTH, 2)], 25))
    solver.add(z3.PbEq([(v[index], 1) for index in range(1, V_LENGTH, 2)], 20))
    solver.add(u[0])
    solver.add(z3.Not(u[-1]))

    # Redundant but propagation-useful consequences of the lag equations.
    # Nesting avoids Z3Py version-dependent arity limits on Xor().
    u_residue_one_parity = z3.BoolVal(False)
    for variable in u[1::4]:
        u_residue_one_parity = z3.Xor(u_residue_one_parity, variable)
    v_residue_zero_parity = z3.BoolVal(False)
    for variable in v[0::4]:
        v_residue_zero_parity = z3.Xor(v_residue_zero_parity, variable)
    solver.add(u_residue_one_parity)
    solver.add(v_residue_zero_parity)

    for shift in range(1, 82):
        disagreements = [z3.Xor(u[index], u[index + shift]) for index in range(U_LENGTH - shift)]
        disagreements.extend(
            z3.Xor(v[index], v[index + shift]) for index in range(max(0, V_LENGTH - shift))
        )
        expected_edges = 164 - 2 * shift
        if len(disagreements) != expected_edges:
            raise AssertionError("fixed-q edge count derivation is inconsistent")
        solver.add(z3.PbEq([(edge, 1) for edge in disagreements], 82 - shift))

    return solver, u, v


def signs_from_model(model, variables: Sequence[z3.BoolRef]) -> List[int]:
    return [1 if z3.is_true(model.eval(variable, model_completion=True)) else -1 for variable in variables]


def reconstruct_solution(u: Sequence[int], v: Sequence[int]) -> Tuple[List[int], ...]:
    if len(u) != U_LENGTH or len(v) != V_LENGTH:
        raise ValueError("expected U length 83 and V length 81")
    if u[0] * u[-1] != -1:
        raise ValueError("U endpoint product must be -1")
    q, _ = published_sequences()
    s = list(u) + [1, 1] + list(v) + [1]
    quadruple = special_quadruple(s, q)
    defects = aggregate_defects(quadruple)
    if defects:
        raise ValueError("solver assignment failed exact reconstruction: {}".format(defects))
    return quadruple


def sequence_line(values: Sequence[int]) -> str:
    return "".join("+" if value == 1 else "-" for value in values)


def audit_published_seed() -> Dict[int, int]:
    q, s = published_sequences()
    defects = aggregate_defects(special_quadruple(s, q))
    if defects != PUBLISHED_DEFECTS:
        raise AssertionError("published seed defects do not match the source")
    u, v = fixed_q_components(s)
    reduced = reduced_correlations(u, v)
    reduced_defects = {shift: 4 * value for shift, value in reduced.items() if value != 0}
    if reduced_defects != PUBLISHED_DEFECTS:
        raise AssertionError("fixed-q reduction does not reproduce the full defects")
    return defects


def write_solution(path: Path, u: Sequence[int], v: Sequence[int], quadruple: Sequence[Sequence[int]]) -> None:
    lines = [
        "format=modular-golay-fixed-q-v1",
        "solver_status=sat",
        "u={}".format(sequence_line(u)),
        "v={}".format(sequence_line(v)),
    ]
    for label, values in zip(("a", "b", "c", "d"), quadruple):
        lines.append("{}={}".format(label, sequence_line(values)))
    lines.append("row_sums={}".format(",".join(str(sum(values)) for values in quadruple)))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def parse_args(argv: Optional[Sequence[str]] = None):
    parser = argparse.ArgumentParser(
        description="Audit and exactly solve the fixed-q lift of the length-167 64-modular Golay quadruple."
    )
    parser.add_argument("--audit-only", action="store_true", help="reproduce the published defects without solving")
    parser.add_argument("--timeout-seconds", type=float, default=300.0, help="Z3 wall-clock limit")
    parser.add_argument("--seed", type=int, default=20260710, help="Z3 random seed")
    parser.add_argument(
        "--solver-backend",
        choices=("sat", "smt"),
        default="sat",
        help="use Z3's direct Boolean SAT engine or its generic SMT solver",
    )
    parser.add_argument(
        "--quotient-modulus",
        type=int,
        choices=(0, 4),
        default=4,
        help="add the exact residue-compression table (0 disables it)",
    )
    parser.add_argument(
        "--quotient-case-index",
        type=int,
        help="solve one zero-based quotient case instead of the complete table",
    )
    parser.add_argument("--output", type=Path, help="write a verified satisfying quadruple")
    parser.add_argument("--smt2-out", type=Path, help="write the exact solver model before solving")
    parser.add_argument(
        "--export-opb-dir",
        type=Path,
        help="write the parity-filtered, symmetry-reduced modulo-4 OPB campaign and exit",
    )
    parser.add_argument(
        "--opb-all-cases",
        action="store_true",
        help="export all 42 raw cases instead of 14 parity-filtered reversal representatives",
    )
    parser.add_argument(
        "--export-t-fixed-opb-dir",
        type=Path,
        help="export the exact 14-shard OPB campaign restricted to T-fixed solutions",
    )
    parser.add_argument(
        "--verify-opb-model",
        type=Path,
        help="independently verify a complete signed x1..x164 OPB model and exit",
    )
    parser.add_argument(
        "--verify-t-fixed-opb-model",
        type=Path,
        help="expand and independently verify a complete T-fixed x1..x82 OPB model",
    )
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = parse_args(argv)
    defects = audit_published_seed()
    print("published_defect_count={}".format(len(defects)))
    print("published_defects={}".format(",".join("{}:{}".format(k, v) for k, v in defects.items())))
    print("active_sign_variables={}".format(U_LENGTH + V_LENGTH))
    print("xor_edge_incidences={}".format(sum(164 - 2 * shift for shift in range(1, 82))))
    print("correlation_equations=81")
    print("z3_version={}".format(z3.get_version_string()))
    if args.opb_all_cases and args.export_opb_dir is None:
        raise SystemExit("--opb-all-cases requires --export-opb-dir")
    selected_actions = int(args.audit_only) + sum(
        value is not None
        for value in (
            args.export_opb_dir,
            args.export_t_fixed_opb_dir,
            args.verify_opb_model,
            args.verify_t_fixed_opb_model,
        )
    )
    if selected_actions > 1:
        raise SystemExit("OPB export and verification actions are mutually exclusive")
    if args.audit_only:
        return 0
    if args.export_opb_dir is not None:
        try:
            manifest = write_fixed_q_opb_campaign(
                args.export_opb_dir, symmetry_reduced=not args.opb_all_cases
            )
        except ValueError as error:
            raise SystemExit(str(error)) from error
        print("opb_campaign_dir={}".format(args.export_opb_dir))
        print("opb_shards={}".format(manifest["shard_count"]))
        print("opb_full_quotient_cases={}".format(manifest["full_case_count"]))
        print(
            "opb_parity_feasible_cases={}".format(
                manifest["parity_feasible_case_count"]
            )
        )
        print("opb_symmetry_reduced={}".format(manifest["symmetry_reduced"]))
        print("opb_variables_per_shard={}".format(manifest["variable_count_per_shard"]))
        print("opb_constraints_per_shard={}".format(manifest["constraint_count_per_shard"]))
        print("opb_manifest={}".format(args.export_opb_dir / "manifest.json"))
        return 0
    if args.export_t_fixed_opb_dir is not None:
        try:
            manifest = write_fixed_q_t_fixed_opb_campaign(args.export_t_fixed_opb_dir)
        except ValueError as error:
            raise SystemExit(str(error)) from error
        print("t_fixed_opb_campaign_dir={}".format(args.export_t_fixed_opb_dir))
        print("t_fixed_opb_shards={}".format(manifest["shard_count"]))
        print(
            "t_fixed_opb_variables_per_shard={}".format(
                manifest["variable_count_per_shard"]
            )
        )
        print(
            "t_fixed_opb_constraints_per_shard={}".format(
                manifest["constraint_count_per_shard"]
            )
        )
        print(
            "t_fixed_opb_manifest={}".format(
                args.export_t_fixed_opb_dir / "manifest.json"
            )
        )
        return 0
    if args.verify_opb_model is not None:
        try:
            u, v = signs_from_opb_model(args.verify_opb_model)
            quadruple = reconstruct_solution(u, v)
        except ValueError as error:
            raise SystemExit(str(error)) from error
        print("verified_opb_model={}".format(args.verify_opb_model))
        print("verified_aperiodic_defects=0")
        print("row_sums={}".format(",".join(str(sum(values)) for values in quadruple)))
        if args.output is not None:
            write_solution(args.output, u, v, quadruple)
            print("artifact_written={}".format(args.output))
        return 0
    if args.verify_t_fixed_opb_model is not None:
        try:
            u, v = signs_from_t_fixed_opb_model(args.verify_t_fixed_opb_model)
            quadruple = reconstruct_solution(u, v)
        except ValueError as error:
            raise SystemExit(str(error)) from error
        print("verified_t_fixed_opb_model={}".format(args.verify_t_fixed_opb_model))
        print("verified_aperiodic_defects=0")
        print("row_sums={}".format(",".join(str(sum(values)) for values in quadruple)))
        if args.output is not None:
            write_solution(args.output, u, v, quadruple)
            print("artifact_written={}".format(args.output))
        return 0
    if args.timeout_seconds <= 0:
        raise SystemExit("--timeout-seconds must be positive")

    model, u_variables, v_variables = build_fixed_q_model()
    quotient_case_count = 0
    selector_count = 0
    if args.quotient_modulus:
        try:
            quotient_case_count, selector_count = add_fixed_q_quotient_constraints(
                model,
                u_variables,
                v_variables,
                args.quotient_modulus,
                args.quotient_case_index,
            )
        except ValueError as error:
            raise SystemExit(str(error)) from error
    elif args.quotient_case_index is not None:
        raise SystemExit("--quotient-case-index requires --quotient-modulus 4")
    print("solver_backend={}".format(args.solver_backend))
    print("quotient_modulus={}".format(args.quotient_modulus))
    print("quotient_cases={}".format(quotient_case_count))
    print("quotient_selector_variables={}".format(selector_count))
    if args.quotient_case_index is not None:
        print("quotient_case_index={}".format(args.quotient_case_index))
    if args.smt2_out is not None:
        args.smt2_out.parent.mkdir(parents=True, exist_ok=True)
        args.smt2_out.write_text(model.to_smt2(), encoding="utf-8")
        print("smt2_written={}".format(args.smt2_out))
    if args.solver_backend == "sat":
        solver = z3.With(
            z3.Tactic("sat"),
            **{"random_seed": args.seed, "pb.solver": "solver"}
        ).solver()
        solver.add(*model.assertions())
        solver.set(timeout=max(1, int(args.timeout_seconds * 1000)))
    else:
        solver = model
        solver.set(timeout=max(1, int(args.timeout_seconds * 1000)), random_seed=args.seed)
    print("timeout_seconds={}".format(args.timeout_seconds))
    print("random_seed={}".format(args.seed))
    started = time.perf_counter()
    result = solver.check()
    elapsed = time.perf_counter() - started
    print("solver_status={}".format(result))
    print("elapsed_seconds={:.6f}".format(elapsed))
    statistics = solver.statistics()
    for key in ("sat conflicts", "sat decisions", "pb conflicts", "pb propagations"):
        if key in statistics.keys():
            print("{}={}".format(key.replace(" ", "_"), statistics.get_key_value(key)))
    if result == z3.unknown:
        print("reason_unknown={}".format(solver.reason_unknown()))
        return 2
    if result == z3.unsat:
        return 0

    model = solver.model()
    u = signs_from_model(model, u_variables)
    v = signs_from_model(model, v_variables)
    quadruple = reconstruct_solution(u, v)
    print("verified_aperiodic_defects=0")
    print("row_sums={}".format(",".join(str(sum(values)) for values in quadruple)))
    if args.output is not None:
        write_solution(args.output, u, v, quadruple)
        print("artifact_written={}".format(args.output))
    else:
        for label, values in zip(("a", "b", "c", "d"), quadruple):
            print("{}={}".format(label, sequence_line(values)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
