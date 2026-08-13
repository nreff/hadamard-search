#!/usr/bin/env python3

from contextlib import redirect_stderr, redirect_stdout
import hashlib
import io
from pathlib import Path
import tempfile
import unittest

from reconstruct_hm668 import (
    build_spence_hadamard,
    load_seed,
    main,
    parse_seed,
    serialize_matrix,
    validate_h4_star,
)
from validate_matrix import check_hadamard


REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
HM668_DIRECTORY = REPOSITORY_ROOT / "fixtures" / "known" / "hm-668"
HM668_SEED_PATH = HM668_DIRECTORY / "spence-sequences.txt"
HM668_MATRIX_PATH = HM668_DIRECTORY / "matrix.txt"
HM668_SHA256 = "bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0"
HM668_SEED_FILE_SHA256 = (
    "bff1d35f7b8ebbbeada18e6d765bf53ded9574edd5b2bc55fa200da069d05e8a"
)
HM668_SEED_CONCATENATED_SHA256 = (
    "6671d64e5118226a7b2b0e7f7be6fe52a371860984fd6152a925a0d73c5919a7"
)


class ReconstructHm668Tests(unittest.TestCase):
    def test_length_two_seed_builds_order_twelve_hadamard(self):
        sequences = parse_seed("++\n+-\n+-\n+-\n")
        parameters = validate_h4_star(sequences)
        self.assertEqual(parameters.order, 2)
        self.assertEqual(parameters.row_sums, (2, 0, 0, 0))
        self.assertEqual(parameters.negative_support_sizes, (0, 1, 1, 1))
        self.assertEqual(parameters.lambda_value, 0)

        matrix = build_spence_hadamard(sequences)
        self.assertEqual((len(matrix), len(matrix[0])), (12, 12))
        self.assertEqual(check_hadamard(matrix), (True, 66, None))
        encoded = serialize_matrix(matrix)
        self.assertTrue(encoded.endswith(b"\n"))
        self.assertEqual(len(encoded.splitlines()), 12)

    def test_parser_and_structural_validation_reject_bad_seeds(self):
        with self.assertRaisesRegex(ValueError, "exactly four nonblank seed rows"):
            parse_seed("++\n+-\n+-\n")
        with self.assertRaisesRegex(ValueError, "invalid seed symbol"):
            parse_seed("++\n+-\n+-\n+x\n")
        with self.assertRaisesRegex(ValueError, "has length 3, expected 2"):
            parse_seed("++\n+-+\n+-\n+-\n")

        odd = parse_seed("+++\n++-\n+-+\n-++\n")
        with self.assertRaisesRegex(ValueError, "length must be even"):
            validate_h4_star(odd)

        wrong_row_sums = parse_seed("++\n++\n+-\n+-\n")
        with self.assertRaisesRegex(ValueError, "row sums must be"):
            validate_h4_star(wrong_row_sums)

        bad_correlation = parse_seed("+++-\n++--\n++--\n++--\n")
        with self.assertRaisesRegex(
            ValueError, "periodic autocorrelation must be -4 at shift 1, got 0"
        ):
            validate_h4_star(bad_correlation)

    def test_hm668_seed_reconstructs_fixture_byte_for_byte(self):
        seed_bytes = HM668_SEED_PATH.read_bytes()
        self.assertEqual(
            hashlib.sha256(seed_bytes).hexdigest(), HM668_SEED_FILE_SHA256
        )
        self.assertEqual(
            hashlib.sha256(b"".join(seed_bytes.splitlines())).hexdigest(),
            HM668_SEED_CONCATENATED_SHA256,
        )
        sequences = load_seed(str(HM668_SEED_PATH))
        parameters = validate_h4_star(sequences)
        self.assertEqual(parameters.order, 166)
        self.assertEqual(parameters.row_sums, (2, 0, 0, 0))
        self.assertEqual(parameters.negative_support_sizes, (82, 83, 83, 83))
        self.assertEqual(parameters.lambda_value, 164)

        encoded = serialize_matrix(build_spence_hadamard(sequences))
        self.assertEqual(encoded, HM668_MATRIX_PATH.read_bytes())
        self.assertEqual(hashlib.sha256(encoded).hexdigest(), HM668_SHA256)

    def test_cli_writes_identified_output_and_rejects_invalid_seed(self):
        with tempfile.TemporaryDirectory() as directory:
            directory_path = Path(directory)
            valid_seed = directory_path / "valid.txt"
            valid_seed.write_text("++\n+-\n+-\n+-\n", encoding="ascii")
            output = directory_path / "hm12.txt"
            stdout = io.StringIO()
            stderr = io.StringIO()
            with redirect_stdout(stdout), redirect_stderr(stderr):
                exit_code = main([str(valid_seed), str(output)])
            self.assertEqual(exit_code, 0)
            self.assertEqual(stderr.getvalue(), "")
            self.assertTrue(output.read_bytes().endswith(b"\n"))
            self.assertIn("construction=wallis-whiteman", stdout.getvalue())
            self.assertIn("negative_support_sizes=0,1,1,1", stdout.getvalue())
            self.assertIn("lambda=0", stdout.getvalue())
            self.assertIn("order=12", stdout.getvalue())
            self.assertIn("sha256=", stdout.getvalue())

            invalid_seed = directory_path / "invalid.txt"
            invalid_seed.write_text("++\n+-\n+-\n", encoding="ascii")
            invalid_output = directory_path / "invalid-output.txt"
            stdout = io.StringIO()
            stderr = io.StringIO()
            with redirect_stdout(stdout), redirect_stderr(stderr):
                exit_code = main([str(invalid_seed), str(invalid_output)])
            self.assertEqual(exit_code, 1)
            self.assertEqual(stdout.getvalue(), "")
            self.assertIn("expected exactly four nonblank seed rows", stderr.getvalue())
            self.assertFalse(invalid_output.exists())


if __name__ == "__main__":
    unittest.main()
