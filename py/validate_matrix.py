#!/usr/bin/env python3

import sys


def load_matrix(path: str):
    rows = []
    with open(path, "r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            rows.append([1 if ch == "+" else -1 for ch in line])
    return rows


def is_hadamard(matrix):
    n = len(matrix)
    if n == 0:
        return False
    for row in matrix:
        if len(row) != n:
            return False
    for i in range(n):
        for j in range(n):
            dot = sum(matrix[i][k] * matrix[j][k] for k in range(n))
            if dot != (n if i == j else 0):
                return False
    return True


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: validate_matrix.py path")
    matrix = load_matrix(sys.argv[1])
    print(f"order={len(matrix)}")
    print(f"is_hadamard={is_hadamard(matrix)}")


if __name__ == "__main__":
    main()
