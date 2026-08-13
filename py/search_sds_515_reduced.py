#!/usr/bin/env python3
"""Exact symmetry-reduced H17-invariant cyclic SDS attack at core order 515.

This module is deliberately separate from :mod:`search_sds_515`: the latter is
the transparent baseline model, while this exporter substitutes one rigorously
exhaustive representative of the singleton-orbit choices and adds four safe
reflection lex leaders.

The reduction uses only equivalences of periodic autocorrelation:

* each sequence may be translated independently by a multiple of 103;
* all four sequences may be multiplied by the same unit modulo 515; and
* each sequence may be reflected independently after its fixed support has
  been normalized.

Consequently, SAT still constructs an H(2060), and UNSAT still excludes the
entire H17-invariant cyclic family encoded by the baseline model.
"""

from __future__ import annotations

import argparse
from collections import Counter
from functools import lru_cache
from itertools import combinations
import json
from math import gcd
from pathlib import Path
from typing import Dict, Iterable, List, Mapping, Optional, Sequence, Tuple

from search_sds_515 import (
    MODULUS,
    MULTIPLIER_ORDER,
    SEQUENCE_COUNT,
    TARGET_FIXED_COUNTS,
    TARGET_NONFIXED_COUNTS,
    TARGET_ROW_SUMS,
    TARGET_SUPPORT_SIZES,
    TARGET_TOTAL_DISAGREEMENTS,
    _opb_constraint,
    _opb_xor_constraints,
    h_orbits,
    parse_signed_model_text,
    shift_pair_multiplicities,
    verify_orbit_assignment,
)


FIXED_MODULUS = 5

# Singleton residues are 103*j for j in Z_5.  The first and fourth supports
# represent the two opposite cyclic distance types forced by the t=103,206
# equations.  The other orientation is carried here by the common unit 2.
NORMALIZED_FIXED_SUPPORTS: Tuple[Tuple[int, ...], ...] = (
    (0, 1),
    (0,),
    (1, 2, 3, 4),
    (1, 3, 4),
)

# j -> reflection_translation-j stabilizes the corresponding fixed support.
REFLECTION_TRANSLATIONS: Tuple[int, ...] = (1, 0, 0, 2)


@lru_cache(maxsize=1)
def fixed_orbit_indices() -> Tuple[int, ...]:
    return tuple(index for index, orbit in enumerate(h_orbits()) if len(orbit) == 1)


@lru_cache(maxsize=1)
def nonfixed_orbit_indices() -> Tuple[int, ...]:
    return tuple(index for index, orbit in enumerate(h_orbits())
                 if len(orbit) == MULTIPLIER_ORDER)


def fixed_difference_profile(support: Iterable[int]) -> Tuple[int, int]:
    """Return singleton disagreements at the two nonzero signed shifts of Z_5."""
    selected = {value % FIXED_MODULUS for value in support}
    return tuple(
        sum((value in selected) != ((value + shift) % FIXED_MODULUS in selected)
            for value in range(FIXED_MODULUS))
        for shift in (1, 2)
    )  # type: ignore[return-value]


def fixed_supports_pass_shift_sieve(
    supports: Sequence[Iterable[int]],
) -> bool:
    """Apply the exact modulo-17 consequence of the t=103 and t=206 equations."""
    if len(supports) != SEQUENCE_COUNT:
        return False
    normalized = tuple({value % FIXED_MODULUS for value in support} for support in supports)
    if tuple(map(len, normalized)) != TARGET_FIXED_COUNTS:
        return False
    profiles = tuple(fixed_difference_profile(support) for support in normalized)
    required = TARGET_TOTAL_DISAGREEMENTS % MULTIPLIER_ORDER
    return all(sum(profile[shift_index] for profile in profiles) == required
               for shift_index in range(2))


def normalize_fixed_supports(
    supports: Sequence[Iterable[int]],
) -> Tuple[int, Tuple[int, ...]]:
    """Return a common dilation and four translations reaching the representative.

    The returned witness satisfies

    ``{(unit*x + translations[i]) mod 5: x in supports[i]}``

    equals ``NORMALIZED_FIXED_SUPPORTS[i]`` for every sequence.  Failure means
    that the raw singleton choice does not pass the two exact fixed-shift
    equations (or has the wrong cardinalities).
    """
    raw = tuple({value % FIXED_MODULUS for value in support} for support in supports)
    if not fixed_supports_pass_shift_sieve(raw):
        raise ValueError("fixed supports fail the t=103,206 modulo-17 sieve")
    targets = tuple(set(support) for support in NORMALIZED_FIXED_SUPPORTS)
    for unit in range(1, FIXED_MODULUS):
        translations: List[int] = []
        for support, target in zip(raw, targets):
            witness = next(
                (
                    translation
                    for translation in range(FIXED_MODULUS)
                    if {(unit * value + translation) % FIXED_MODULUS
                        for value in support} == target
                ),
                None,
            )
            if witness is None:
                break
            translations.append(witness)
        if len(translations) == SEQUENCE_COUNT:
            return unit, tuple(translations)
    raise AssertionError("the fixed-support orbit classification is incomplete")


def fixed_support_normalization_audit() -> Dict[str, int]:
    """Count raw and symmetry-reduced singleton cases by exhaustive enumeration."""
    choices = tuple(
        tuple(combinations(range(FIXED_MODULUS), count))
        for count in TARGET_FIXED_COUNTS
    )
    raw_count = 0
    feasible_count = 0
    translation_types = set()
    for first in choices[0]:
        for second in choices[1]:
            for third in choices[2]:
                for fourth in choices[3]:
                    raw_count += 1
                    supports = (first, second, third, fourth)
                    if not fixed_supports_pass_shift_sieve(supports):
                        continue
                    feasible_count += 1
                    # Translation leaves each difference profile unchanged, so
                    # this pair is a complete label for the two translation orbits.
                    translation_types.add((fixed_difference_profile(first),
                                           fixed_difference_profile(fourth)))
                    normalize_fixed_supports(supports)
    return {
        "raw_fixed_support_count": raw_count,
        "fixed_shift_feasible_count": feasible_count,
        "independent_translation_orbit_count": len(translation_types),
        "common_dilation_orbit_count": 1,
    }


@lru_cache(maxsize=1)
def normalized_fixed_bits() -> Tuple[Mapping[int, bool], ...]:
    """Map original H-orbit indices to selected singleton bits for each row."""
    coordinate_to_orbit = {
        (orbit[0] // 103) % FIXED_MODULUS: index
        for index, orbit in enumerate(h_orbits())
        if len(orbit) == 1
    }
    result: List[Mapping[int, bool]] = []
    for support in NORMALIZED_FIXED_SUPPORTS:
        selected = set(support)
        result.append({coordinate_to_orbit[coordinate]: coordinate in selected
                       for coordinate in range(FIXED_MODULUS)})
    return tuple(result)


def reduced_variable_layout() -> Tuple[
    Tuple[Mapping[int, int], ...],
    Tuple[Mapping[Tuple[int, int], int], ...],
    int,
]:
    """Return variables for 30 base bits and C(30,2) XORs in each row."""
    nonfixed = nonfixed_orbit_indices()
    next_variable = 1
    base: List[Mapping[int, int]] = []
    for _ in range(SEQUENCE_COUNT):
        mapping = {orbit: next_variable + offset for offset, orbit in enumerate(nonfixed)}
        next_variable += len(nonfixed)
        base.append(mapping)
    xor_maps: List[Mapping[Tuple[int, int], int]] = []
    for _ in range(SEQUENCE_COUNT):
        mapping: Dict[Tuple[int, int], int] = {}
        for left, right in combinations(nonfixed, 2):
            mapping[(left, right)] = next_variable
            next_variable += 1
        xor_maps.append(mapping)
    return tuple(base), tuple(xor_maps), next_variable - 1


@lru_cache(maxsize=None)
def reflection_orbit_pairs(sequence: int) -> Tuple[Tuple[int, int], ...]:
    """Return the 15 transpositions induced on big H-orbits by a row reflection."""
    if not 0 <= sequence < SEQUENCE_COUNT:
        raise ValueError("sequence index must be in 0..3")
    orbits = h_orbits()
    orbit_of = {residue: index for index, orbit in enumerate(orbits) for residue in orbit}
    translation = 103 * REFLECTION_TRANSLATIONS[sequence]
    permutation = []
    for orbit in orbits:
        images = {orbit_of[(translation - residue) % MODULUS] for residue in orbit}
        if len(images) != 1:
            raise AssertionError("reflection does not normalize the multiplier orbits")
        permutation.append(images.pop())
    fixed_bits = normalized_fixed_bits()[sequence]
    if any(fixed_bits[index] != fixed_bits[permutation[index]] for index in fixed_bits):
        raise AssertionError("reflection does not stabilize the normalized fixed support")
    nonfixed = set(nonfixed_orbit_indices())
    if any(permutation[permutation[index]] != index for index in range(len(orbits))):
        raise AssertionError("reflection permutation is not an involution")
    pairs = tuple(sorted(
        (index, permutation[index])
        for index in nonfixed
        if index < permutation[index]
    ))
    if len(pairs) != 15 or {index for pair in pairs for index in pair} != nonfixed:
        raise AssertionError("expected 15 fixed-point-free big-orbit transpositions")
    return pairs


def reflection_lex_terms(sequence: int) -> Tuple[Tuple[int, int], ...]:
    """Encode one representative of a row's residual reflection orbit.

    With transposition pairs ``(a_j,b_j)``, the first unequal pair is oriented
    as x[a_j] <= x[b_j].  Binary weights make that lexicographic condition one
    exact pseudo-Boolean inequality, without auxiliary variables.
    """
    base, _, _ = reduced_variable_layout()
    pairs = reflection_orbit_pairs(sequence)
    terms: List[Tuple[int, int]] = []
    for pair_index, (left, right) in enumerate(pairs):
        weight = 1 << (len(pairs) - pair_index - 1)
        terms.append((-weight, base[sequence][left]))
        terms.append((weight, base[sequence][right]))
    return tuple(terms)


def reduced_shift_equations() -> Tuple[
    Tuple[int, Tuple[Tuple[int, int], ...], int, int, int], ...
]:
    """Return ``(shift, terms, target, constant, divisor)`` after substitution.

    ``constant + divisor*sum(terms)`` is the original total disagreement
    expression.  Dividing by the coefficient gcd is exact and reduces the two
    fixed-direction equations by 17.
    """
    base, xor_maps, _ = reduced_variable_layout()
    fixed_bits = normalized_fixed_bits()
    nonfixed = set(nonfixed_orbit_indices())
    equations = []
    for shift, multiplicities in shift_pair_multiplicities():
        coefficients: Counter[int] = Counter()
        constant = 0
        for sequence in range(SEQUENCE_COUNT):
            for left, right, multiplicity in multiplicities:
                left_big = left in nonfixed
                right_big = right in nonfixed
                if left_big and right_big:
                    coefficients[xor_maps[sequence][(left, right)]] += multiplicity
                elif left_big or right_big:
                    big = left if left_big else right
                    fixed = right if left_big else left
                    if fixed_bits[sequence][fixed]:
                        constant += multiplicity
                        coefficients[base[sequence][big]] -= multiplicity
                    else:
                        coefficients[base[sequence][big]] += multiplicity
                elif fixed_bits[sequence][left] != fixed_bits[sequence][right]:
                    constant += multiplicity
        coefficients = Counter({variable: coefficient
                                for variable, coefficient in coefficients.items()
                                if coefficient})
        target = TARGET_TOTAL_DISAGREEMENTS - constant
        divisor = 0
        for value in tuple(coefficients.values()) + (target,):
            divisor = gcd(divisor, abs(value))
        divisor = max(divisor, 1)
        terms = tuple((coefficient // divisor, variable)
                      for variable, coefficient in sorted(coefficients.items()))
        equations.append((shift, terms, target // divisor, constant, divisor))
    return tuple(equations)


def build_reduced_opb() -> Tuple[str, Dict[str, object]]:
    """Build the exhaustive singleton-normalized and reflection-reduced model."""
    base, xor_maps, variable_count = reduced_variable_layout()
    constraints: List[str] = []

    for sequence, count in enumerate(TARGET_NONFIXED_COUNTS):
        constraints.append(_opb_constraint(
            ((1, base[sequence][orbit]) for orbit in nonfixed_orbit_indices()), "=", count))

    for sequence in range(SEQUENCE_COUNT):
        for (left, right), edge in xor_maps[sequence].items():
            constraints.extend(_opb_xor_constraints(
                edge, base[sequence][left], base[sequence][right]))

    for _, terms, target, _, _ in reduced_shift_equations():
        constraints.append(_opb_constraint(terms, "=", target))

    for sequence in range(SEQUENCE_COUNT):
        constraints.append(_opb_constraint(reflection_lex_terms(sequence), ">=", 0))

    base_variable_count = SEQUENCE_COUNT * len(nonfixed_orbit_indices())
    xor_variable_count = SEQUENCE_COUNT * len(xor_maps[0])
    xor_link_constraint_count = 4 * xor_variable_count
    support_equation_count = SEQUENCE_COUNT
    shift_equation_count = len(reduced_shift_equations())
    reflection_lex_constraint_count = SEQUENCE_COUNT
    equality_constraint_count = support_equation_count + shift_equation_count
    fixed_audit = fixed_support_normalization_audit()
    lines = [
        "* #variable= {} #constraint= {} #equal= {} intsize= 32".format(
            variable_count, len(constraints), equality_constraint_count),
        "* exact symmetry-reduced H17-invariant cyclic SDS search over Z_515",
        "* singleton supports normalized by independent Z_5 translations and a common unit",
        "* x1..x120 select the thirty nonfixed H-orbits in four sequences",
        "* remaining variables are exact XORs for pairs of nonfixed H-orbits",
        "* four final inequalities are safe independent-reflection lex leaders",
        "* SAT constructs H(2060); UNSAT excludes the full H17-invariant cyclic family",
    ]
    lines.extend(constraints)
    metadata: Dict[str, object] = {
        "format": "h17-invariant-cyclic-sds-515-reduced-opb-v1",
        "modulus": MODULUS,
        "hadamard_order": 4 * MODULUS,
        "row_sums": list(TARGET_ROW_SUMS),
        "negative_support_sizes": list(TARGET_SUPPORT_SIZES),
        "normalized_fixed_supports_z5": [list(support)
                                            for support in NORMALIZED_FIXED_SUPPORTS],
        "reflection_translations_z5": list(REFLECTION_TRANSLATIONS),
        **fixed_audit,
        "base_variable_count": base_variable_count,
        "xor_variable_count": xor_variable_count,
        "variable_count": variable_count,
        "support_equation_count": support_equation_count,
        "shift_equation_count": shift_equation_count,
        "reflection_lex_constraint_count": reflection_lex_constraint_count,
        "xor_link_constraint_count": xor_link_constraint_count,
        "equality_constraint_count": equality_constraint_count,
        "constraint_count": len(constraints),
        "shift_equation_divisors": {
            str(shift): divisor for shift, _, _, _, divisor in reduced_shift_equations()
        },
        "scope_caveat": (
            "UNSAT excludes H17-invariant cyclic Goethals--Seidel/SDS seeds over Z_515 only"
        ),
    }
    return "\n".join(lines) + "\n", metadata


def export_reduced_opb(path: Path) -> Dict[str, object]:
    text, metadata = build_reduced_opb()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return metadata


def verify_reduced_signed_model(path: Path) -> Dict[str, object]:
    """Expand a complete reduced solver model and verify all 514 raw shifts."""
    assignments = parse_signed_model_text(path.read_text(encoding="utf-8"))
    base, xor_maps, variable_count = reduced_variable_layout()
    missing = [variable for variable in range(1, variable_count + 1)
               if variable not in assignments]
    extras = sorted(variable for variable in assignments if variable > variable_count)
    if missing or extras:
        raise ValueError("model must assign exactly x1..x{} ({} missing, {} extra)".format(
            variable_count, len(missing), len(extras)))

    rows = [[False] * len(h_orbits()) for _ in range(SEQUENCE_COUNT)]
    for sequence in range(SEQUENCE_COUNT):
        for orbit, selected in normalized_fixed_bits()[sequence].items():
            rows[sequence][orbit] = selected
        for orbit, variable in base[sequence].items():
            rows[sequence][orbit] = assignments[variable]
        for (left, right), edge in xor_maps[sequence].items():
            if assignments[edge] != (rows[sequence][left] != rows[sequence][right]):
                raise ValueError("x{} is not the required XOR auxiliary".format(edge))
        lex_score = sum(coefficient * int(assignments[variable])
                        for coefficient, variable in reflection_lex_terms(sequence))
        if lex_score < 0:
            raise ValueError("sequence {} violates its reflection lex leader".format(sequence))
    result = verify_orbit_assignment(rows)
    result["assigned_variable_count"] = len(assignments)
    result["symmetry_reduced"] = True
    return result


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Export/verify the exact reduced H17-invariant SDS(515) model."
    )
    parser.add_argument("--export-opb", type=Path, help="write the reduced OPB instance")
    parser.add_argument("--verify-model", type=Path, help="independently verify a solver model")
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = parse_args(argv)
    if args.export_opb is not None and args.verify_model is not None:
        raise SystemExit("choose at most one of --export-opb and --verify-model")
    if args.verify_model is not None:
        output = verify_reduced_signed_model(args.verify_model)
    elif args.export_opb is not None:
        output = dict(export_reduced_opb(args.export_opb))
        output["exported_opb"] = str(args.export_opb)
    else:
        _, output = build_reduced_opb()
    print(json.dumps(output, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
