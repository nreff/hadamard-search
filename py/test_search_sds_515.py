#!/usr/bin/env python3

from collections import Counter
from itertools import product
from pathlib import Path
import tempfile
import unittest

from search_sds_515 import (
    HADAMARD_ORDER,
    MODULUS,
    TARGET_FIXED_COUNTS,
    TARGET_NONFIXED_COUNTS,
    TARGET_ROW_SUMS,
    TARGET_SUPPORT_SIZES,
    _opb_xor_constraints,
    build_opb,
    compatible_row_sum_cases,
    export_opb,
    h_orbits,
    multiplier_subgroup,
    orbit_pairs,
    parse_signed_model_text,
    row_sum_quadruples,
    shift_pair_multiplicities,
    signed_shift_orbits,
    variable_layout,
    verify_orbit_assignment,
)


def evaluate_constraint(line, assignment):
    tokens = line.rstrip(" ;").split()
    relation_index = next(index for index, token in enumerate(tokens) if token in ("=", ">="))
    total = sum(
        int(tokens[index]) * int(assignment[int(tokens[index + 1][1:])])
        for index in range(0, relation_index, 2)
    )
    target = int(tokens[relation_index + 1])
    return total == target if tokens[relation_index] == "=" else total >= target


class Sds515Tests(unittest.TestCase):
    def test_multiplier_group_and_residue_orbits(self):
        generator, subgroup = multiplier_subgroup()
        self.assertEqual(generator, 61)
        self.assertEqual(
            subgroup,
            (1, 61, 66, 76, 81, 111, 116, 126, 196, 236, 306, 381, 421, 426, 446, 476, 491),
        )
        self.assertEqual({left * right % MODULUS for left in subgroup for right in subgroup}, set(subgroup))

        orbits = h_orbits()
        self.assertEqual(len(orbits), 35)
        self.assertEqual(Counter(map(len, orbits)), Counter({17: 30, 1: 5}))
        self.assertEqual(sorted(value for orbit in orbits for value in orbit), list(range(MODULUS)))
        self.assertEqual(
            [(index, orbit) for index, orbit in enumerate(orbits) if len(orbit) == 1],
            [(0, (0,)), (31, (103,)), (32, (206,)), (33, (309,)), (34, (412,))],
        )

    def test_signed_shift_orbits_are_the_17_required_equation_classes(self):
        orbits = signed_shift_orbits()
        self.assertEqual(len(orbits), 17)
        self.assertEqual(Counter(map(len, orbits)), Counter({34: 15, 2: 2}))
        self.assertEqual(sorted(value for orbit in orbits for value in orbit), list(range(1, MODULUS)))
        self.assertEqual([orbit[0] for orbit in orbits],
                         [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 16, 18, 20, 36, 103, 206])

    def test_h17_compatibility_uniquely_selects_the_target_parameters(self):
        candidates = row_sum_quadruples()
        self.assertEqual(len(candidates), 29)
        self.assertTrue(all(tuple(sorted(case)) == case for case in candidates))
        self.assertTrue(all(sum(value * value for value in case) == HADAMARD_ORDER
                            for case in candidates))
        self.assertEqual(
            compatible_row_sum_cases(),
            ((TARGET_ROW_SUMS, (TARGET_SUPPORT_SIZES, TARGET_FIXED_COUNTS,
                                TARGET_NONFIXED_COUNTS)),),
        )

    def test_pair_compression_is_complete_and_shared(self):
        pairs = orbit_pairs()
        self.assertEqual(len(pairs), 35 * 34 // 2)
        self.assertEqual(pairs, tuple((left, right) for left in range(35)
                                      for right in range(left + 1, 35)))
        shifts = shift_pair_multiplicities()
        self.assertEqual(len(shifts), 17)
        self.assertEqual([shift for shift, _ in shifts],
                         [orbit[0] for orbit in signed_shift_orbits()])
        self.assertTrue(all(multiplicity > 0 for _, terms in shifts
                            for _, _, multiplicity in terms))

        base, xor_maps, variable_count = variable_layout()
        self.assertEqual(len(base), 4)
        self.assertTrue(all(len(row) == 35 for row in base))
        self.assertTrue(all(set(mapping) == set(pairs) for mapping in xor_maps))
        self.assertEqual(variable_count, 2520)

    def test_opb_shape_and_exact_counts(self):
        text, metadata = build_opb()
        self.assertEqual(metadata["base_variable_count"], 140)
        self.assertEqual(metadata["xor_variable_count"], 2380)
        self.assertEqual(metadata["variable_count"], 2520)
        self.assertEqual(metadata["support_equation_count"], 4)
        self.assertEqual(metadata["shift_equation_count"], 17)
        self.assertEqual(metadata["xor_link_constraint_count"], 9520)
        self.assertEqual(metadata["equality_constraint_count"], 21)
        self.assertEqual(metadata["constraint_count"], 9541)
        self.assertEqual(metadata["row_sum_candidate_count"], 29)
        self.assertEqual(metadata["compatible_row_sum_case_count"], 1)
        self.assertEqual(metadata["target_total_disagreements"], 1030)
        self.assertTrue(text.startswith("* #variable= 2520 #constraint= 9541 #equal= 21 intsize= 16\n"))

        constraints = [line for line in text.splitlines() if line and not line.startswith("*")]
        equalities = [line for line in constraints if " = " in line]
        inequalities = [line for line in constraints if " >= " in line]
        self.assertEqual(len(equalities), 21)
        self.assertEqual(len(inequalities), 9520)
        self.assertEqual([int(line.rstrip(" ;").split()[-1]) for line in equalities[:4]],
                         list(TARGET_SUPPORT_SIZES))
        self.assertTrue(all(line.endswith("= 1030 ;") for line in equalities[4:]))
        referenced = {
            int(token[1:])
            for line in constraints
            for token in line.split()
            if token.startswith("x")
        }
        self.assertEqual(referenced, set(range(1, 2521)))

    def test_four_inequalities_are_an_exact_xor_fixture(self):
        constraints = _opb_xor_constraints(3, 1, 2)
        for left, right, edge in product((False, True), repeat=3):
            assignment = {1: left, 2: right, 3: edge}
            satisfies = all(evaluate_constraint(line, assignment) for line in constraints)
            self.assertEqual(satisfies, edge == (left != right))

    def test_export_and_signed_model_parser(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "sds515.opb"
            metadata = export_opb(path)
            self.assertEqual(path.read_text(encoding="utf-8"), build_opb()[0])
            self.assertEqual(metadata["format"], "h17-invariant-cyclic-sds-515-opb-v1")
        self.assertEqual(parse_signed_model_text("s SATISFIABLE\nv x1 -x2 ~x3 0\n"),
                         {1: True, 2: False, 3: False})
        with self.assertRaisesRegex(ValueError, "both truth values"):
            parse_signed_model_text("v x1 -x1 0\n")

    def test_independent_verifier_rejects_a_wrong_support_before_correlations(self):
        with self.assertRaisesRegex(ValueError, "support 0, expected 257"):
            verify_orbit_assignment([[False] * 35 for _ in range(4)])


if __name__ == "__main__":
    unittest.main()
