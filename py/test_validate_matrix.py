#!/usr/bin/env python3

import hashlib
from pathlib import Path
import tempfile
import unittest

from validate_matrix import check_hadamard, is_hadamard, load_matrix


REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
HM668_PATH = REPOSITORY_ROOT / "fixtures" / "known" / "hm-668" / "matrix.txt"
HM668_SHA256 = "bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0"


class ValidateMatrixTests(unittest.TestCase):
    def test_sylvester_four_is_hadamard(self):
        matrix = [
            [1, 1, 1, 1],
            [1, -1, 1, -1],
            [1, 1, -1, -1],
            [1, -1, -1, 1],
        ]
        self.assertTrue(is_hadamard(matrix))
        self.assertEqual(check_hadamard(matrix), (True, 6, None))

    def test_nonorthogonal_rows_report_the_first_exact_dot_product(self):
        matrix = [[1, 1], [1, 1]]
        self.assertEqual(
            check_hadamard(matrix),
            (False, 1, "rows 0 and 1 have dot product 2"),
        )

    def test_loader_rejects_invalid_symbols(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "matrix.txt"
            path.write_text("+-\n+x\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "invalid matrix symbol"):
                load_matrix(str(path))

    def test_order_668_fixture_hash_shape_and_gram_matrix(self):
        self.assertEqual(hashlib.sha256(HM668_PATH.read_bytes()).hexdigest(), HM668_SHA256)
        matrix = load_matrix(str(HM668_PATH))
        self.assertEqual((len(matrix), len(matrix[0])), (668, 668))
        self.assertEqual(check_hadamard(matrix), (True, 222778, None))


if __name__ == "__main__":
    unittest.main()
