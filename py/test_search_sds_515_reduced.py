#!/usr/bin/env python3

from itertools import combinations, product
import random
import unittest

from search_sds_515 import (
    MODULUS,
    TARGET_FIXED_COUNTS,
    TARGET_NONFIXED_COUNTS,
    TARGET_TOTAL_DISAGREEMENTS,
    h_orbits,
)
from search_sds_515_reduced import (
    NORMALIZED_FIXED_SUPPORTS,
    build_reduced_opb,
    fixed_difference_profile,
    fixed_orbit_indices,
    fixed_support_normalization_audit,
    fixed_supports_pass_shift_sieve,
    nonfixed_orbit_indices,
    normalize_fixed_supports,
    normalized_fixed_bits,
    reduced_shift_equations,
    reduced_variable_layout,
    reflection_lex_terms,
    reflection_orbit_pairs,
)


class ReducedSds515Tests(unittest.TestCase):
    def test_singleton_normalization_exhaustively_covers_every_feasible_case(self):
        choices = [tuple(combinations(range(5), count)) for count in TARGET_FIXED_COUNTS]
        raw_count = 0
        feasible_count = 0
        witnesses = set()
        for supports in product(*choices):
            raw_count += 1
            profiles = tuple(fixed_difference_profile(support) for support in supports)
            independently_feasible = all(
                sum(profile[shift] for profile in profiles)
                == TARGET_TOTAL_DISAGREEMENTS % 17
                for shift in range(2)
            )
            self.assertEqual(fixed_supports_pass_shift_sieve(supports), independently_feasible)
            if not independently_feasible:
                with self.assertRaises(ValueError):
                    normalize_fixed_supports(supports)
                continue
            feasible_count += 1
            unit, translations = normalize_fixed_supports(supports)
            transformed = tuple(
                tuple(sorted((unit * value + translation) % 5 for value in support))
                for support, translation in zip(supports, translations)
            )
            self.assertEqual(transformed, NORMALIZED_FIXED_SUPPORTS)
            witnesses.add(unit)

        self.assertEqual(raw_count, 2500)
        self.assertEqual(feasible_count, 1250)
        # The deterministic normalizer needs only the identity orientation and
        # multiplication by 2; units 4 and 3 differ by a reflected orientation.
        self.assertEqual(witnesses, {1, 2})
        self.assertEqual(
            fixed_support_normalization_audit(),
            {
                "raw_fixed_support_count": 2500,
                "fixed_shift_feasible_count": 1250,
                "independent_translation_orbit_count": 2,
                "common_dilation_orbit_count": 1,
            },
        )

    def test_normalized_singletons_have_the_required_opposite_profiles(self):
        profiles = tuple(fixed_difference_profile(support)
                         for support in NORMALIZED_FIXED_SUPPORTS)
        self.assertEqual(profiles, ((2, 4), (2, 2), (2, 2), (4, 2)))
        self.assertEqual(tuple(map(sum, zip(*profiles))), (10, 10))
        self.assertTrue(fixed_supports_pass_shift_sieve(NORMALIZED_FIXED_SUPPORTS))

    def test_reflections_stabilize_fixed_bits_and_pair_every_big_orbit(self):
        self.assertEqual(len(fixed_orbit_indices()), 5)
        self.assertEqual(len(nonfixed_orbit_indices()), 30)
        for sequence in range(4):
            pairs = reflection_orbit_pairs(sequence)
            self.assertEqual(len(pairs), 15)
            self.assertEqual({orbit for pair in pairs for orbit in pair},
                             set(nonfixed_orbit_indices()))

            terms = reflection_lex_terms(sequence)
            coefficient_by_variable = dict((variable, coefficient)
                                           for coefficient, variable in terms)
            base, _, _ = reduced_variable_layout()
            for pair_index, (left, right) in enumerate(pairs):
                weight = 1 << (14 - pair_index)
                self.assertEqual(coefficient_by_variable[base[sequence][left]], -weight)
                self.assertEqual(coefficient_by_variable[base[sequence][right]], weight)

            # Reflection negates the score, so at least one member of every
            # two-element orbit satisfies the leader; fixed points score zero.
            rng = random.Random(515 + sequence)
            for _ in range(100):
                values = {variable: rng.randrange(2) for variable in base[sequence].values()}
                score = sum(coefficient * values[variable]
                            for coefficient, variable in terms)
                reflected = dict(values)
                for left, right in pairs:
                    reflected[base[sequence][left]] = values[base[sequence][right]]
                    reflected[base[sequence][right]] = values[base[sequence][left]]
                reflected_score = sum(coefficient * reflected[variable]
                                      for coefficient, variable in terms)
                self.assertEqual(reflected_score, -score)
                self.assertTrue(score >= 0 or reflected_score >= 0)

    def test_reduced_substitution_matches_raw_disagreements(self):
        orbits = h_orbits()
        base, xor_maps, _ = reduced_variable_layout()
        equations = reduced_shift_equations()
        rng = random.Random(2060)
        for _ in range(20):
            rows = [[False] * len(orbits) for _ in range(4)]
            variable_values = {}
            for sequence in range(4):
                for orbit, selected in normalized_fixed_bits()[sequence].items():
                    rows[sequence][orbit] = selected
                for orbit, variable in base[sequence].items():
                    rows[sequence][orbit] = bool(rng.randrange(2))
                    variable_values[variable] = rows[sequence][orbit]
                for (left, right), variable in xor_maps[sequence].items():
                    variable_values[variable] = rows[sequence][left] != rows[sequence][right]

            signs = []
            for row in rows:
                expanded = [1] * MODULUS
                for selected, orbit in zip(row, orbits):
                    if selected:
                        for residue in orbit:
                            expanded[residue] = -1
                signs.append(expanded)

            for shift, terms, _, constant, divisor in equations:
                raw_disagreements = sum(
                    row[index] != row[(index + shift) % MODULUS]
                    for row in signs
                    for index in range(MODULUS)
                )
                reduced_value = sum(coefficient * int(variable_values[variable])
                                    for coefficient, variable in terms)
                self.assertEqual(raw_disagreements, constant + divisor * reduced_value)

    def test_reduced_opb_has_the_exact_substituted_shape(self):
        text, metadata = build_reduced_opb()
        self.assertEqual(metadata["base_variable_count"], 120)
        self.assertEqual(metadata["xor_variable_count"], 1740)
        self.assertEqual(metadata["variable_count"], 1860)
        self.assertEqual(metadata["xor_link_constraint_count"], 6960)
        self.assertEqual(metadata["support_equation_count"], 4)
        self.assertEqual(metadata["shift_equation_count"], 17)
        self.assertEqual(metadata["reflection_lex_constraint_count"], 4)
        self.assertEqual(metadata["equality_constraint_count"], 21)
        self.assertEqual(metadata["constraint_count"], 6985)
        self.assertEqual(metadata["shift_equation_divisors"]["103"], 17)
        self.assertEqual(metadata["shift_equation_divisors"]["206"], 17)
        self.assertTrue(text.startswith(
            "* #variable= 1860 #constraint= 6985 #equal= 21 intsize= 32\n"))

        constraints = [line for line in text.splitlines()
                       if line and not line.startswith("*")]
        self.assertEqual(len(constraints), 6985)
        self.assertEqual(sum(" = " in line for line in constraints), 21)
        self.assertEqual(sum(" >= " in line for line in constraints), 6964)
        referenced = {
            int(token[1:])
            for line in constraints
            for token in line.split()
            if token.startswith("x")
        }
        self.assertEqual(referenced, set(range(1, 1861)))
        self.assertEqual(len(reduced_variable_layout()[1][0]), 30 * 29 // 2)
        self.assertEqual(tuple(TARGET_NONFIXED_COUNTS), (15, 15, 14, 14))


if __name__ == "__main__":
    unittest.main()
