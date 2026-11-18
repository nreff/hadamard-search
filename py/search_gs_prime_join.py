#!/usr/bin/env python3
"""Bounded row-pool and exact partial-signature joins for cyclic GS quadruples."""

import argparse
from collections import defaultdict
from itertools import combinations, islice, product
from math import isqrt
from pathlib import Path
from random import Random
import time

from reconstruct_hm668 import load_seed, periodic_autocorrelation
from search_sds_515 import multiplicative_orbits, multiplier_subgroup


def count_bits(bits):
    return bin(bits).count("1")


popcount = getattr(int, "bit_count", count_bits)


class OrbitModel:
    def __init__(self, length, order):
        if length < 3 or any(length % d == 0 for d in range(2, isqrt(length) + 1)):
            raise ValueError("the core length must be an odd prime")
        if order <= 1 or order % 2 == 0 or (length - 1) % order:
            raise ValueError("the subgroup order must be an odd divisor of p-1 greater than one")
        self.length = length
        self.order = order
        self.mask = (1 << length) - 1
        self.generator, units = multiplier_subgroup(length, order)
        orbits = multiplicative_orbits(range(length), units, length)
        self.orbit_masks = tuple(sum(1 << x for x in orbit) for orbit in orbits[1:])
        signed_units = set(units) | {(-unit) % length for unit in units}
        self.shifts = tuple(orbit[0] for orbit in multiplicative_orbits(range(1, length), signed_units, length))

    def row_parameters(self, row_sum):
        # r = s(0) + h(2k - number of nonzero orbits).
        for fixed in (1, -1):
            count, rem = divmod(row_sum - fixed + self.length - 1, 2 * self.order)
            if rem == 0 and 0 <= count <= len(self.orbit_masks):
                return int(fixed == 1), count
        raise ValueError(f"row sum {row_sum} is impossible in these orbits")

    def profile(self, bits):
        p = self.length
        # All PAF values are congruent to p modulo 4. Work in that lattice.
        profile = []
        for shift in self.shifts:
            rotated = ((bits << shift) | (bits >> (p - shift))) & self.mask
            profile.append(p // 4 - popcount(bits ^ rotated) // 2)
        return tuple(profile)

    def check_rows(self, rows, row_sums):
        for row, wanted_sum in zip(rows, row_sums):
            if len(row) != self.length or sum(row) != wanted_sum:
                raise ValueError("wrong witness length or row sum")
            if any(sign != row[i * self.generator % self.length] for i, sign in enumerate(row)):
                raise ValueError("witness is not invariant under the prescribed multiplier")

    def check_witness(self, rows, row_sums):
        self.check_rows(rows, row_sums)
        if any(sum(periodic_autocorrelation(row, shift) for row in rows)
               for shift in range(1, self.length)):
            raise ValueError("witness failed the full autocorrelation check")


def sample_pool(model, row_sum, samples, proposals, rng):
    fixed, plus_count = model.row_parameters(row_sum)
    masks = list(model.orbit_masks)
    pool = {}
    for _ in range(samples):
        rng.shuffle(masks)
        plus, minus = masks[:plus_count], masks[plus_count:]
        bits = fixed | sum(plus)
        profile = model.profile(bits)
        score = sum(x * x for x in profile)
        for _ in range(proposals if plus and minus else 0):
            a, b = rng.randrange(len(plus)), rng.randrange(len(minus))
            candidate = bits ^ plus[a] ^ minus[b]
            new_profile = model.profile(candidate)
            new_score = sum(x * x for x in new_profile)
            if new_score <= score:
                plus[a], minus[b] = minus[b], plus[a]
                bits, profile, score = candidate, new_profile, new_score
        pool.setdefault(profile, bits)
    return pool


def swap_pool(model, row, radius):
    bits = sum(1 << i for i, sign in enumerate(row) if sign == 1)
    plus = [mask for mask in model.orbit_masks if bits & mask]
    minus = [mask for mask in model.orbit_masks if not bits & mask]
    pool = {}
    for swaps in range(radius + 1):
        removed = [sum(masks) for masks in combinations(plus, swaps)]
        added = [sum(masks) for masks in combinations(minus, swaps)]
        for a, b in product(removed, added):
            candidate = bits ^ a ^ b
            pool.setdefault(model.profile(candidate), candidate)
    return pool


def encode(values, field_width):
    # An additive encoding of signed coordinates. Equality of two encoded
    # sums implies coordinate equality whenever each coordinate difference
    # has magnitude less than the base, by reducing one digit at a time.
    return sum(value << (index * field_width) for index, value in enumerate(values))


def split_pool(pool, width, field_width):
    index = defaultdict(list)
    for profile, bits in pool.items():
        index[profile[:width]].append((encode(profile[width:], field_width), bits))
    return index


def pairs_with_prefix(left, right, target):
    for prefix, lhs in left.items():
        complement = tuple(wanted - value for wanted, value in zip(target, prefix))
        for (left_tail, left_bits), (right_tail, right_bits) in product(lhs, right.get(complement, ())):
            yield left_tail + right_tail, (left_bits, right_bits)


def exact_join(model, pools, targets, width, total, pair_limit):
    # Each row coordinate has magnitude at most p. This bound prevents
    # collisions between any four-row sum and the required constant vector.
    field_width = (4 * model.length + 4).bit_length()
    goal = encode([total] * (len(model.shifts) - width), field_width)
    a, b, c, d = (split_pool(pool, width, field_width) for pool in pools)
    for target in targets:
        left = {}
        left_count = 0
        capped = False
        for left_count, (tail, pair) in enumerate(islice(pairs_with_prefix(a, b, target), pair_limit + 1), 1):
            if left_count > pair_limit:
                capped = True
                break
            left[tail] = pair
        opposite_target = tuple(total - value for value in target)
        right_count = 0
        witness = None
        for right_count, (tail, pair) in enumerate(islice(pairs_with_prefix(c, d, opposite_target), pair_limit + 1), 1):
            if right_count > pair_limit:
                capped = True
                break
            complement = goal - tail
            if complement in left:
                witness = (*left[complement], *pair)
                break
        print(f"prefix={target} left_pairs={min(left_count, pair_limit)} "
              f"right_pairs={min(right_count, pair_limit)} capped={capped}", flush=True)
        if witness is not None:
            return witness
    return None


def repair(model, rows, radii, total, pair_limit):
    pools = [swap_pool(model, row, radius) for row, radius in zip(rows, radii)]
    a, b, c, d = sorted(range(4), key=lambda i: len(pools[i]))
    order = (a, d, b, c)
    if len(pools[a]) * len(pools[d]) > len(pools[b]) * len(pools[c]):
        order = (b, c, a, d)
    print(f"radii={tuple(radii)} profiles={tuple(map(len, pools))} "
          f"join_order={tuple(i + 1 for i in order)}", flush=True)
    joined = exact_join(model, [pools[i] for i in order], [()], 0, total, pair_limit)
    if joined is not None:
        return tuple(joined[order.index(i)] for i in range(4))
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("length", type=int)
    parser.add_argument("subgroup_order", type=int)
    parser.add_argument("row_sums", nargs=4, type=int)
    parser.add_argument("--seed", type=int, default=11)
    parser.add_argument("--samples", type=int, default=2000)
    parser.add_argument("--proposals", type=int, default=64)
    parser.add_argument("--prefix-width", type=int, default=4)
    parser.add_argument("--pair-limit", type=int, default=100000)
    seeds = parser.add_mutually_exclusive_group()
    seeds.add_argument("--control-seed", type=Path, help="inject a verified positive control")
    seeds.add_argument("--around-seed", type=Path, help="join bounded swap neighborhoods of the input rows")
    parser.add_argument("--radii", nargs=4, type=int, choices=range(3),
                        help="per-row swap limits, totaling at most four (default: 1 1 1 1)")
    parser.add_argument("--witness-out", type=Path, required=True)
    args = parser.parse_args()
    start = time.monotonic()
    model = OrbitModel(args.length, args.subgroup_order)
    if sum(r * r for r in args.row_sums) != 4 * model.length:
        parser.error("row-sum squares must add to 4p")
    if min(args.samples, args.pair_limit) < 1 or args.proposals < 0:
        parser.error("samples and pair limit must be positive and proposals nonnegative")
    if args.radii is not None and (args.around_seed is None or sum(args.radii) > 4):
        parser.error("--radii requires --around-seed and a total of at most four swaps")
    total = -(model.length % 4)
    if args.around_seed is not None:
        rows = load_seed(str(args.around_seed))
        model.check_rows(rows, args.row_sums)
        witness = repair(model, rows, args.radii or (1, 1, 1, 1), total, args.pair_limit)
    else:
        width = args.prefix_width
        if not 1 <= width <= min(12, len(model.shifts)):
            parser.error("the prefix width must be between 1 and the lesser of 12 and the shift count")
        pools, targets = sampled_search(args, model, width, total)
        witness = exact_join(model, pools, targets, width, total, args.pair_limit)
    if witness is not None:
        rows = [tuple(1 if bits >> i & 1 else -1 for i in range(model.length)) for bits in witness]
        model.check_witness(rows, args.row_sums)
        args.witness_out.write_text("".join("".join("+" if x == 1 else "-" for x in row) + "\n" for row in rows))
        print(f"verified_witness={args.witness_out} elapsed={time.monotonic() - start:.2f}s")
    else:
        print(f"no_match_in_candidate_pools elapsed={time.monotonic() - start:.2f}s")


def sampled_search(args, model, width, total):
    # Across the shift classes the average PAF is (r²-p)/(p-1).
    # Try the floor and ceiling of the resulting mean sum, independently per coordinate.
    a, b = args.row_sums[:2]
    shift_count = model.length - 1
    mean_numerator = a * a + b * b - 2 * model.length + 2 * total * shift_count
    mean_floor = mean_numerator // (4 * shift_count)
    targets = list(product((mean_floor, mean_floor + 1), repeat=width))
    rng = Random(args.seed)
    by_sum = {}
    for row_sum in dict.fromkeys(args.row_sums):
        pool = sample_pool(model, row_sum, args.samples, args.proposals, rng)
        by_sum[row_sum] = pool
        squares = [sum(x * x for x in profile) for profile in pool]
        print(f"row_sum={row_sum} profiles={len(pool)} min_sum_squares={min(squares)} "
              f"mean_sum_squares={sum(squares) / len(squares):.1f}", flush=True)
    if args.control_seed is not None:
        rows = load_seed(str(args.control_seed))
        model.check_witness(rows, args.row_sums)
        profiles = []
        for row, row_sum in zip(rows, args.row_sums):
            bits = sum(1 << i for i, sign in enumerate(row) if sign == 1)
            profile = model.profile(bits)
            by_sum[row_sum][profile] = bits
            profiles.append(profile)
        target = tuple(x + y for x, y in zip(profiles[0][:width], profiles[1][:width]))
        targets = [target]
    pools = [by_sum[row_sum] for row_sum in args.row_sums]
    return pools, targets


if __name__ == "__main__":
    main()
