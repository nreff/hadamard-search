#!/usr/bin/env python3
"""Reconstruct a bordered Wallis-Whiteman Hadamard matrix.

The input is the compact four-sequence seed, not the public shell generator.
This module uses only the type-H4* correlation conditions and the documented
bordered block array.
"""

import argparse
from dataclasses import dataclass
import hashlib
from pathlib import Path
import sys
from typing import Iterable, List, Optional, Sequence, Tuple


SignRow = Tuple[int, ...]
SignSequences = Tuple[SignRow, ...]


@dataclass(frozen=True)
class H4StarParameters:
    order: int
    row_sums: Tuple[int, int, int, int]
    negative_support_sizes: Tuple[int, int, int, int]
    lambda_value: int


def _normalize_sequences(sequences: Iterable[Sequence[int]]) -> SignSequences:
    rows = tuple(tuple(row) for row in sequences)
    if len(rows) != 4:
        raise ValueError(f"expected exactly four seed rows, got {len(rows)}")
    if not rows[0]:
        raise ValueError("seed rows must be nonempty")

    order = len(rows[0])
    for index, row in enumerate(rows):
        if len(row) != order:
            raise ValueError(
                f"seed row {index} has length {len(row)}, expected {order}"
            )
        if any(value not in (-1, 1) for value in row):
            raise ValueError(f"seed row {index} contains an entry outside {{-1,+1}}")
    return rows


def parse_seed(text: str) -> SignSequences:
    """Parse exactly four nonblank, equal-length ``+/-`` rows."""
    rows = []
    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        line = raw_line.strip()
        if not line:
            continue
        invalid = sorted(set(line) - {"+", "-"})
        if invalid:
            symbols = "".join(invalid)
            raise ValueError(
                f"invalid seed symbol(s) {symbols!r} on line {line_number}"
            )
        rows.append(tuple(1 if symbol == "+" else -1 for symbol in line))

    if len(rows) != 4:
        raise ValueError(f"expected exactly four nonblank seed rows, got {len(rows)}")
    return _normalize_sequences(rows)


def load_seed(path: str) -> SignSequences:
    return parse_seed(Path(path).read_text(encoding="ascii"))


def periodic_autocorrelation(sequence: Sequence[int], shift: int) -> int:
    order = len(sequence)
    offset = shift % order
    return sum(
        sequence[index] * sequence[(index + offset) % order]
        for index in range(order)
    )


def validate_h4_star(sequences: Iterable[Sequence[int]]) -> H4StarParameters:
    """Validate the normalized cyclic type-H4* conditions used by the lift."""
    rows = _normalize_sequences(sequences)
    order = len(rows[0])
    if order % 2 != 0:
        raise ValueError(f"type-H4* seed length must be even, got {order}")

    row_sums = tuple(sum(row) for row in rows)
    expected_row_sums = (2, 0, 0, 0)
    if row_sums != expected_row_sums:
        raise ValueError(
            f"type-H4* seed row sums must be {expected_row_sums}, got {row_sums}"
        )

    for shift in range(1, order):
        correlation_sum = sum(
            periodic_autocorrelation(row, shift) for row in rows
        )
        if correlation_sum != -4:
            raise ValueError(
                "type-H4* seed combined periodic autocorrelation must be -4 "
                f"at shift {shift}, got {correlation_sum}"
            )

    support_sizes = tuple(sum(value == -1 for value in row) for row in rows)
    lambda_value = sum(support_sizes) - (order + 1)
    return H4StarParameters(
        order=order,
        row_sums=row_sums,
        negative_support_sizes=support_sizes,
        lambda_value=lambda_value,
    )


def build_spence_hadamard(
    sequences: Iterable[Sequence[int]],
) -> List[List[int]]:
    """Build the exact bordered Wallis-Whiteman block array.

    With ``A, B, C, D`` the four right-circulants and ``R`` the
    back-diagonal permutation matrix, the lower-right core is::

        [ A    BR      CR      DR   ]
        [-BR   A       D^T R  -C^T R]
        [-CR  -D^T R   A       B^T R]
        [-DR   C^T R  -B^T R   A   ]

    The first four rows and columns are the fixed bordering matrices below.
    """
    rows = _normalize_sequences(sequences)
    parameters = validate_h4_star(rows)
    order = parameters.order
    size = 4 * (order + 1)
    matrix = [[0] * size for _ in range(size)]

    top_left = (
        (-1, 1, 1, -1),
        (1, -1, 1, -1),
        (1, 1, -1, -1),
        (-1, -1, -1, -1),
    )
    top_right = (
        (-1, -1, -1, 1),
        (-1, -1, 1, -1),
        (-1, 1, -1, -1),
        (1, -1, -1, -1),
    )
    bottom_left = (
        (1, 1, 1, -1),
        (-1, -1, 1, -1),
        (-1, 1, -1, -1),
        (1, -1, -1, -1),
    )

    for row in range(4):
        matrix[row][:4] = top_left[row]
        for block_column in range(4):
            start = 4 + block_column * order
            matrix[row][start : start + order] = [top_right[row][block_column]] * order

    for block_row, border in enumerate(bottom_left):
        for row in range(order):
            matrix[4 + block_row * order + row][:4] = border

    # Each tuple is (source circulant, transpose, multiply by R on the right, sign).
    core_blocks = (
        (
            (0, False, False, 1),
            (1, False, True, 1),
            (2, False, True, 1),
            (3, False, True, 1),
        ),
        (
            (1, False, True, -1),
            (0, False, False, 1),
            (3, True, True, 1),
            (2, True, True, -1),
        ),
        (
            (2, False, True, -1),
            (3, True, True, -1),
            (0, False, False, 1),
            (1, True, True, 1),
        ),
        (
            (3, False, True, -1),
            (2, True, True, 1),
            (1, True, True, -1),
            (0, False, False, 1),
        ),
    )

    for block_row, block_specs in enumerate(core_blocks):
        for block_column, (source, transpose, reverse_columns, sign) in enumerate(
            block_specs
        ):
            sequence = rows[source]
            for row in range(order):
                target_row = 4 + block_row * order + row
                for column in range(order):
                    source_column = order - 1 - column if reverse_columns else column
                    if transpose:
                        source_index = (row - source_column) % order
                    else:
                        source_index = (source_column - row) % order
                    target_column = 4 + block_column * order + column
                    matrix[target_row][target_column] = sign * sequence[source_index]

    return matrix


def serialize_matrix(matrix: Sequence[Sequence[int]]) -> bytes:
    if not matrix:
        raise ValueError("matrix must be nonempty")
    width = len(matrix[0])
    encoded_rows = []
    for row_index, row in enumerate(matrix):
        if len(row) != width:
            raise ValueError(
                f"matrix row {row_index} has length {len(row)}, expected {width}"
            )
        if any(value not in (-1, 1) for value in row):
            raise ValueError(
                f"matrix row {row_index} contains an entry outside {{-1,+1}}"
            )
        encoded_rows.append("".join("+" if value == 1 else "-" for value in row))
    return ("\n".join(encoded_rows) + "\n").encode("ascii")


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Reconstruct a Hadamard matrix from four type-H4* seed rows."
    )
    parser.add_argument("seed_path")
    parser.add_argument("output_path")
    arguments = parser.parse_args(argv)

    seed_path = Path(arguments.seed_path)
    output_path = Path(arguments.output_path)
    try:
        if seed_path.resolve() == output_path.resolve():
            raise ValueError("seed and output paths must differ")
        sequences = load_seed(str(seed_path))
        parameters = validate_h4_star(sequences)
        matrix = build_spence_hadamard(sequences)
        encoded = serialize_matrix(matrix)
        output_path.write_bytes(encoded)
    except (OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    print("construction=wallis-whiteman")
    print(f"seed_order={parameters.order}")
    print(
        "negative_support_sizes="
        + ",".join(str(value) for value in parameters.negative_support_sizes)
    )
    print(f"lambda={parameters.lambda_value}")
    print(f"order={len(matrix)}")
    print(f"sha256={hashlib.sha256(encoded).hexdigest()}")
    print(f"output={output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
