#!/usr/bin/env python3

import hashlib
import sys


def _bit_count(value):
    # int.bit_count() was added after the project's minimum Python version.
    return value.bit_count() if hasattr(value, "bit_count") else bin(value).count("1")


def load_matrix(path: str):
    rows = []
    with open(path, "r", encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, start=1):
            line = line.strip()
            if not line:
                continue
            invalid = sorted(set(line) - {"+", "-"})
            if invalid:
                symbols = "".join(invalid)
                raise ValueError(f"invalid matrix symbol(s) {symbols!r} on line {line_number}")
            rows.append([1 if ch == "+" else -1 for ch in line])
    return rows


def check_hadamard(matrix):
    """Return ``(is_hadamard, checked_pairs, failure)`` using exact bitsets."""
    n = len(matrix)
    if n == 0:
        return False, 0, "matrix is empty"
    for row_index, row in enumerate(matrix):
        if len(row) != n:
            return False, 0, f"row {row_index} has length {len(row)}, expected {n}"
        if any(value not in (-1, 1) for value in row):
            return False, 0, f"row {row_index} contains an entry outside {{-1,+1}}"

    # For +/-1 rows, a zero dot product is equivalent to Hamming distance n/2.
    # Packing each row into a Python integer moves the O(n^3 / word_size) inner
    # work into exact arbitrary-precision bit operations implemented in C.
    packed_rows = []
    for row in matrix:
        packed = 0
        for column, value in enumerate(row):
            if value == 1:
                packed |= 1 << column
        packed_rows.append(packed)

    checked_pairs = 0
    for left in range(n):
        for right in range(left + 1, n):
            distance = _bit_count(packed_rows[left] ^ packed_rows[right])
            checked_pairs += 1
            if 2 * distance != n:
                dot = n - 2 * distance
                return (
                    False,
                    checked_pairs,
                    f"rows {left} and {right} have dot product {dot}",
                )
    return True, checked_pairs, None


def is_hadamard(matrix):
    return check_hadamard(matrix)[0]


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: validate_matrix.py path")
    try:
        matrix = load_matrix(sys.argv[1])
    except ValueError as error:
        raise SystemExit(str(error)) from error
    valid, checked_pairs, failure = check_hadamard(matrix)
    digest = hashlib.sha256()
    with open(sys.argv[1], "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    print(f"order={len(matrix)}")
    print(f"is_hadamard={valid}")
    print(f"off_diagonal_pairs_checked={checked_pairs}")
    print(f"sha256={digest.hexdigest()}")
    if failure is not None:
        print(f"failure={failure}")
        raise SystemExit(1)


if __name__ == "__main__":
    main()
