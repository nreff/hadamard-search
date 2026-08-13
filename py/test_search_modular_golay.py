#!/usr/bin/env python3

from collections import Counter
import hashlib
from pathlib import Path
import tempfile
import unittest

import z3

from search_modular_golay import (
    PUBLISHED_DEFECTS,
    PUBLISHED_Q_RUNS,
    PUBLISHED_S_RUNS,
    U_LENGTH,
    V_LENGTH,
    _opb_xor_constraints,
    aggregate_defects,
    add_fixed_q_quotient_constraints,
    aperiodic_autocorrelation,
    audit_published_seed,
    build_fixed_q_model,
    build_fixed_q_opb,
    build_fixed_q_t_fixed_opb,
    expand_runs,
    fixed_q_components,
    fixed_q_opb_v_reversal_variable_map,
    fixed_q_parity_case_is_feasible,
    fixed_q_parity_feasible_case_indices,
    fixed_q_parity_quotient_orbits,
    fixed_q_quotient_cases,
    fixed_q_quotient_orbits,
    fixed_q_quotient_target,
    periodic_autocorrelation,
    published_sequences,
    reduced_correlations,
    signs_from_opb_model,
    signs_from_t_fixed_opb_model,
    special_quadruple,
    write_fixed_q_opb_campaign,
    write_fixed_q_t_fixed_opb_campaign,
)


def normalized_opb_constraints(text, variable_map=None):
    normalized = []
    for line in text.splitlines():
        if not line or line.startswith("*"):
            continue
        tokens = line.split()
        relation_index = next(
            index for index, token in enumerate(tokens) if token in ("=", ">=")
        )
        terms = []
        for offset in range(0, relation_index, 2):
            coefficient = int(tokens[offset])
            variable = int(tokens[offset + 1][1:])
            if variable_map is not None:
                variable = variable_map[variable]
            terms.append((variable, coefficient))
        normalized.append(
            (tuple(sorted(terms)), tokens[relation_index], int(tokens[relation_index + 1]))
        )
    return Counter(normalized)


class ModularGolayTests(unittest.TestCase):
    def test_published_run_lengths_and_defects(self):
        self.assertEqual(len(expand_runs(PUBLISHED_Q_RUNS)), 167)
        self.assertEqual(len(expand_runs(PUBLISHED_S_RUNS)), 167)
        self.assertEqual(audit_published_seed(), PUBLISHED_DEFECTS)

    def test_fixed_q_reduction_matches_full_quadruple(self):
        q, s = published_sequences()
        full = aggregate_defects(special_quadruple(s, q))
        u, v = fixed_q_components(s)
        reduced = {shift: 4 * value for shift, value in reduced_correlations(u, v).items() if value}
        self.assertEqual(full, reduced)

    def test_fixed_q_identity_holds_away_from_the_published_seed(self):
        q, _ = published_sequences()
        s = [1 if (index * index + 3 * index + 1) % 7 < 3 else -1 for index in range(167)]
        quadruple = special_quadruple(s, q)
        u, v = fixed_q_components(s)
        reduced = reduced_correlations(u, v)
        for shift in range(1, 82):
            full_value = sum(aperiodic_autocorrelation(values, shift) for values in quadruple)
            self.assertEqual(full_value, 4 * reduced[shift])
            edge_count = 164 - 2 * shift
            disagreements = sum(u[index] != u[index + shift] for index in range(U_LENGTH - shift))
            disagreements += sum(
                v[index] != v[index + shift] for index in range(max(0, V_LENGTH - shift))
            )
            self.assertEqual(reduced[shift], edge_count - 2 * disagreements)
        lag_82 = sum(aperiodic_autocorrelation(values, 82) for values in quadruple)
        self.assertEqual(lag_82, 4 * (u[0] * u[-1] + s[84] * s[166]))
        for shift in range(83, 167):
            full_value = sum(aperiodic_autocorrelation(values, shift) for values in quadruple)
            self.assertEqual(full_value, 0)

    def test_model_has_the_derived_shape(self):
        solver, u, v = build_fixed_q_model()
        self.assertEqual(len(u), U_LENGTH)
        self.assertEqual(len(v), V_LENGTH)
        self.assertEqual(len(solver.assertions()), 89)
        solver.set(timeout=1)
        self.assertIn(solver.check(), (z3.sat, z3.unsat, z3.unknown))

    def test_modulo_four_quotient_has_42_exact_cases(self):
        cases = fixed_q_quotient_cases(4)
        target = fixed_q_quotient_target(4)
        self.assertEqual(len(cases), 42)
        self.assertEqual(len(set(cases)), 42)
        for u_sums, v_sums in cases:
            self.assertEqual(sum(u_sums[::2]), 0)
            self.assertEqual(sum(u_sums[1::2]), 9)
            self.assertEqual(sum(v_sums[::2]), 9)
            self.assertEqual(sum(v_sums[1::2]), 0)
            signature = tuple(
                periodic_autocorrelation(u_sums, shift)
                + periodic_autocorrelation(v_sums, shift)
                for shift in range(4)
            )
            self.assertEqual(signature, target)

    def test_modulo_four_table_is_added_as_an_exact_selector(self):
        solver, u, v = build_fixed_q_model()
        case_count, selector_count = add_fixed_q_quotient_constraints(solver, u, v, 4)
        self.assertEqual(case_count, 42)
        self.assertEqual(selector_count, 22)
        self.assertEqual(len(solver.assertions()), 266)
        with self.assertRaisesRegex(ValueError, "excluded by the exact lag-parity lemma"):
            add_fixed_q_quotient_constraints(solver, u, v, 4, 6)

    def test_lag_parity_lemma_and_quotient_filter(self):
        def xor_lag_rows(shifts, sequence_lengths):
            support = set()
            right_hand_side = 0
            for shift in shifts:
                for label, length in sequence_lengths:
                    for index in range(max(0, length - shift)):
                        for variable in ((label, index), (label, index + shift)):
                            if variable in support:
                                support.remove(variable)
                            else:
                                support.add(variable)
                right_hand_side ^= (82 - shift) % 2
            return support, right_hand_side

        sequence_lengths = (("u", U_LENGTH), ("v", V_LENGTH))
        u_shifts = [shift for shift in range(1, 82) if shift % 8 in (3, 4, 5, 6)]
        v_shifts = [shift for shift in range(1, 82) if shift % 8 in (4, 5, 6, 7)]
        self.assertEqual(
            xor_lag_rows(u_shifts, sequence_lengths),
            ({("u", index) for index in range(3, U_LENGTH, 4)}, 0),
        )
        self.assertEqual(
            xor_lag_rows(v_shifts, sequence_lengths),
            ({("v", index) for index in range(2, V_LENGTH, 4)}, 0),
        )

        feasible = fixed_q_parity_feasible_case_indices(4)
        self.assertEqual(
            feasible,
            [0, 1, 2, 3, 4, 5, 9, 11, 14, 15, 20, 23, 24, 29, 31, 33, 34, 35, 36, 37, 38, 39],
        )
        cases = fixed_q_quotient_cases(4)
        self.assertEqual(
            [index for index, case in enumerate(cases) if fixed_q_parity_case_is_feasible(case)],
            feasible,
        )
        self.assertEqual(
            fixed_q_parity_quotient_orbits(4),
            [
                (0, 1),
                (2, 3),
                (4, 5),
                (9,),
                (11,),
                (14, 15),
                (20,),
                (23, 24),
                (29,),
                (31,),
                (33,),
                (34, 35),
                (36, 37),
                (38, 39),
            ],
        )

    def test_v_reversal_reduces_the_quotient_campaign_to_26_orbits(self):
        cases = fixed_q_quotient_cases(4)
        orbits = fixed_q_quotient_orbits(4)
        self.assertEqual(len(orbits), 26)
        self.assertEqual(sum(len(orbit) == 1 for orbit in orbits), 10)
        self.assertEqual(sum(len(orbit) == 2 for orbit in orbits), 16)
        self.assertEqual(sorted(index for orbit in orbits for index in orbit), list(range(42)))
        for orbit in orbits:
            representative_u, representative_v = cases[orbit[0]]
            for case_index in orbit:
                u_sums, v_sums = cases[case_index]
                self.assertEqual(u_sums, representative_u)
                self.assertIn(
                    v_sums,
                    (
                        representative_v,
                        (
                            representative_v[0],
                            representative_v[3],
                            representative_v[2],
                            representative_v[1],
                        ),
                    ),
                )

    def test_v_reversal_is_an_exact_opb_formula_isomorphism(self):
        cases = fixed_q_quotient_cases(4)
        variable_map = fixed_q_opb_v_reversal_variable_map()
        self.assertEqual(len(variable_map), 6807)
        self.assertEqual(
            tuple(variable_map[variable_map[index]] for index in range(len(variable_map))),
            tuple(range(len(variable_map))),
        )
        for orbit in fixed_q_quotient_orbits(4):
            left_text, _ = build_fixed_q_opb(4, orbit[0])
            right_text, _ = build_fixed_q_opb(4, orbit[-1])
            self.assertEqual(
                normalized_opb_constraints(left_text, variable_map),
                normalized_opb_constraints(right_text),
            )

            left_u, left_v = cases[orbit[0]]
            right_u, right_v = cases[orbit[-1]]
            self.assertEqual(left_u, right_u)
            self.assertEqual(right_v, (left_v[0], left_v[3], left_v[2], left_v[1]))

    def test_opb_campaign_manifest_hashes_and_stale_shard_guard(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            with self.assertRaisesRegex(ValueError, "quotient modulus 4"):
                write_fixed_q_opb_campaign(path, quotient_modulus=6)
            self.assertEqual(list(path.iterdir()), [])

            reduced = write_fixed_q_opb_campaign(path)
            self.assertEqual(reduced["format"], "modular-golay-fixed-q-opb-campaign-v3")
            self.assertEqual(reduced["shard_count"], 14)
            self.assertEqual(reduced["full_case_count"], 42)
            self.assertEqual(reduced["parity_feasible_case_count"], 22)
            self.assertEqual(len(reduced["parity_excluded_case_indices"]), 20)
            self.assertEqual(
                sorted(
                    case_index
                    for shard in reduced["shards"]
                    for case_index in shard["covered_case_indices"]
                ),
                reduced["parity_feasible_case_indices"],
            )
            self.assertEqual(
                reduced["symmetry_action"]["quotient_residue_permutation"],
                [0, 3, 2, 1],
            )
            for shard in reduced["shards"]:
                contents = (path / shard["file"]).read_bytes()
                self.assertEqual(hashlib.sha256(contents).hexdigest(), shard["sha256"])
            self.assertEqual(len(list(path.glob("fixed-q-mod4-case-*.opb"))), 14)
            self.assertEqual(list(path.glob("*.tmp")), [])

            full = write_fixed_q_opb_campaign(path, symmetry_reduced=False)
            self.assertEqual(full["shard_count"], 42)
            self.assertEqual(len(list(path.glob("fixed-q-mod4-case-*.opb"))), 42)
            with self.assertRaisesRegex(ValueError, "stale shard"):
                write_fixed_q_opb_campaign(path)

    def test_opb_shard_has_the_exact_linearized_shape(self):
        text, metadata = build_fixed_q_opb(4, 0)
        self.assertEqual(metadata["base_sign_variable_count"], 164)
        self.assertEqual(metadata["edge_variable_count"], 6642)
        self.assertEqual(metadata["variable_count"], 6806)
        self.assertEqual(metadata["xor_link_constraint_count"], 26568)
        self.assertEqual(metadata["constraint_count"], 26663)
        self.assertEqual(metadata["equality_constraint_count"], 95)
        self.assertEqual(metadata["integer_size"], 8)
        self.assertEqual(metadata["correlation_equation_count"], 81)
        self.assertTrue(
            text.startswith(
                "* #variable= 6806 #constraint= 26663 #equal= 95 intsize= 8\n"
            )
        )
        self.assertEqual(sum(line.endswith(" ;") for line in text.splitlines()), 26663)

    def test_t_fixed_reduced_shard_and_expansion(self):
        text, metadata = build_fixed_q_t_fixed_opb(0)
        self.assertEqual(metadata["base_sign_variable_count"], 82)
        self.assertEqual(metadata["edge_variable_count"], 1560)
        self.assertEqual(metadata["variable_count"], 1642)
        self.assertEqual(metadata["xor_link_constraint_count"], 6240)
        self.assertEqual(metadata["correlation_equation_count"], 40)
        self.assertEqual(metadata["equality_constraint_count"], 48)
        self.assertEqual(metadata["constraint_count"], 6288)
        self.assertTrue(
            text.startswith(
                "* #variable= 1642 #constraint= 6288 #equal= 48 intsize= 7\n"
            )
        )
        with self.assertRaisesRegex(ValueError, "excluded by the exact lag-parity lemma"):
            build_fixed_q_t_fixed_opb(6)

        with tempfile.TemporaryDirectory() as directory:
            model_path = Path(directory) / "model.txt"
            base_values = {
                index: (index * 37 + 11) % 5 < 2 for index in range(1, 83)
            }
            rendered_assignments = " ".join(
                ("" if base_values[index] else "-") + "x{}".format(index)
                for index in range(1, 83)
            )
            model_path.write_text(
                "v {} 0\n".format(rendered_assignments), encoding="utf-8"
            )
            u, v = signs_from_t_fixed_opb_model(model_path)
            self.assertEqual((u[0], u[-1]), (1, -1))
            for index in range(U_LENGTH):
                self.assertEqual(u[index], -((-1) ** index) * u[82 - index])
            for index in range(V_LENGTH):
                self.assertEqual(v[index], ((-1) ** index) * v[80 - index])
            for shift in range(1, 82, 2):
                self.assertEqual(aperiodic_autocorrelation(u, shift), 0)
                if shift < V_LENGTH:
                    self.assertEqual(aperiodic_autocorrelation(v, shift), 0)

            def accepts(constraint, values):
                tokens = constraint.split()
                relation_index = next(
                    index
                    for index, token in enumerate(tokens)
                    if token in ("=", ">=")
                )
                left_hand_side = sum(
                    int(tokens[offset]) * int(values[int(tokens[offset + 1][1:])])
                    for offset in range(0, relation_index, 2)
                )
                target = int(tokens[relation_index + 1])
                return (
                    left_hand_side == target
                    if tokens[relation_index] == "="
                    else left_hand_side >= target
                )

            constraints = [
                line for line in text.splitlines() if line and not line.startswith("*")
            ]
            values = dict(base_values)
            for offset in range(0, 6240, 4):
                first_link = constraints[offset].split()
                edge = int(first_link[1][1:])
                left = int(first_link[3][1:])
                right = int(first_link[5][1:])
                values[edge] = values[left] != values[right]
                self.assertTrue(
                    all(
                        accepts(constraints[line_index], values)
                        for line_index in range(offset, offset + 4)
                    )
                )

            full_correlations = reduced_correlations(u, v)
            for equation_index, shift in enumerate(range(2, 82, 2)):
                self.assertEqual(
                    accepts(constraints[6240 + equation_index], values),
                    full_correlations[shift] == 0,
                )
            case = fixed_q_quotient_cases(4)[0]
            actual_sums = [
                sum(sequence[residue::4])
                for sequence in (u, v)
                for residue in range(4)
            ]
            target_sums = list(case[0]) + list(case[1])
            for equation_index, (actual, target) in enumerate(
                zip(actual_sums, target_sums)
            ):
                self.assertEqual(
                    accepts(constraints[6280 + equation_index], values),
                    actual == target,
                )

            for case_index in fixed_q_parity_feasible_case_indices(4):
                case_text, _ = build_fixed_q_t_fixed_opb(case_index)
                case_constraints = [
                    line
                    for line in case_text.splitlines()
                    if line and not line.startswith("*")
                ]
                u_sums, v_sums = fixed_q_quotient_cases(4)[case_index]
                case_targets = list(u_sums) + list(v_sums)
                for equation_index, (actual, target) in enumerate(
                    zip(actual_sums, case_targets)
                ):
                    self.assertEqual(
                        accepts(case_constraints[6280 + equation_index], values),
                        actual == target,
                    )

        for u_sums, v_sums in fixed_q_quotient_cases(4):
            transformed_u = tuple(
                -((-1) ** residue) * u_sums[(2 - residue) % 4]
                for residue in range(4)
            )
            transformed_v = tuple(
                ((-1) ** residue) * v_sums[(-residue) % 4]
                for residue in range(4)
            )
            self.assertEqual(transformed_u, u_sums)
            self.assertEqual(transformed_v, v_sums)

        sample_u = [1 if (index * 5 + 1) % 7 < 3 else -1 for index in range(U_LENGTH)]
        sample_v = [1 if (index * 3 + 2) % 11 < 5 else -1 for index in range(V_LENGTH)]

        def transform_t(pair):
            left, right = pair
            return (
                [-((-1) ** index) * left[82 - index] for index in range(U_LENGTH)],
                [((-1) ** index) * right[80 - index] for index in range(V_LENGTH)],
            )

        def reverse_v(pair):
            left, right = pair
            return list(left), list(reversed(right))

        self.assertEqual(
            transform_t(reverse_v((sample_u, sample_v))),
            reverse_v(transform_t((sample_u, sample_v))),
        )

    def test_t_fixed_campaign_manifest_is_explicitly_conditional(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            manifest = write_fixed_q_t_fixed_opb_campaign(path)
            self.assertEqual(
                manifest["format"],
                "modular-golay-fixed-q-t-fixed-opb-campaign-v1",
            )
            self.assertEqual(manifest["shard_count"], 14)
            self.assertEqual(manifest["variable_count_per_shard"], 1642)
            self.assertEqual(manifest["constraint_count_per_shard"], 6288)
            self.assertIn("UNSAT excludes only", manifest["subfamily_warning"])
            self.assertEqual(manifest["quotient_modulus"], 4)
            self.assertTrue(manifest["symmetry_action"]["commutes_with_T"])
            self.assertIn("lag parity", manifest["coverage_basis"])
            self.assertEqual(
                sorted(
                    case_index
                    for shard in manifest["shards"]
                    for case_index in shard["covered_case_indices"]
                ),
                manifest["parity_feasible_case_indices"],
            )
            for shard in manifest["shards"]:
                contents = (path / shard["file"]).read_bytes()
                self.assertEqual(hashlib.sha256(contents).hexdigest(), shard["sha256"])
            with self.assertRaisesRegex(ValueError, "stale shard"):
                write_fixed_q_opb_campaign(path)

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            write_fixed_q_opb_campaign(path)
            with self.assertRaisesRegex(ValueError, "stale shard"):
                write_fixed_q_t_fixed_opb_campaign(path)

    def test_opb_xor_linearization_has_the_exact_truth_table(self):
        constraints = _opb_xor_constraints(3, 1, 2)
        parsed = []
        for constraint in constraints:
            tokens = constraint.split()
            terms = []
            for offset in range(0, 6, 2):
                terms.append((int(tokens[offset]), int(tokens[offset + 1][1:])))
            self.assertEqual(tokens[6], ">=")
            parsed.append((terms, int(tokens[7])))

        for left in (0, 1):
            for right in (0, 1):
                for edge in (0, 1):
                    values = {1: left, 2: right, 3: edge}
                    accepted = all(
                        sum(coefficient * values[variable] for coefficient, variable in terms)
                        >= target
                        for terms, target in parsed
                    )
                    self.assertEqual(accepted, edge == (left ^ right))

    def test_opb_model_parser_rejects_aliases_and_repeated_assignments(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "model.txt"
            base = " ".join("x{}".format(index) for index in range(1, 165))
            path.write_text("v {} 0\n".format(base), encoding="utf-8")
            u, v = signs_from_opb_model(path)
            self.assertEqual(u, [1] * U_LENGTH)
            self.assertEqual(v, [1] * V_LENGTH)

            path.write_text("v x01 {} 0\n".format(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "noncanonical"):
                signs_from_opb_model(path)

            path.write_text("v {} -x1 0\n".format(base), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "contradictory"):
                signs_from_opb_model(path)


if __name__ == "__main__":
    unittest.main()
