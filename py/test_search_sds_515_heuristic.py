#!/usr/bin/env python3

import random
import unittest

from search_sds_515_heuristic import (
    FIXED_ORBITS,
    NONFIXED_ORBITS,
    SearchState,
    exact_residuals,
    random_rows,
    squared_objective,
)


class Sds515HeuristicTests(unittest.TestCase):
    def test_incremental_swap_deltas_and_fields_match_exact_recomputation(self):
        rng = random.Random(515)
        state = SearchState.from_rows(random_rows(rng))
        state.assert_exact()
        for _ in range(80):
            sequence = rng.randrange(4)
            orbit_class = rng.choice((FIXED_ORBITS, NONFIXED_ORBITS))
            selected = rng.choice([index for index in orbit_class if state.rows[sequence][index]])
            unselected = rng.choice([index for index in orbit_class if not state.rows[sequence][index]])
            delta = state.swap_delta(sequence, selected, unselected)
            predicted = state.objective_after(delta)
            state.apply_swap(sequence, selected, unselected, delta)
            self.assertEqual(tuple(state.residuals), exact_residuals(state.rows))
            self.assertEqual(state.objective, predicted)
            self.assertEqual(state.objective, squared_objective(state.residuals))
            state.assert_exact()

    def test_illegal_cross_cardinality_move_is_rejected(self):
        state = SearchState.from_rows(random_rows(random.Random(17)))
        selected = next(index for index in FIXED_ORBITS if state.rows[0][index])
        with self.assertRaisesRegex(ValueError, "selected then an unselected"):
            state.swap_delta(0, selected, selected)


if __name__ == "__main__":
    unittest.main()
