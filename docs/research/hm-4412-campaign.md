# Order 4412 construction campaign

## Decision, 2026-11-18

The active construction target is `4412 = 4 × 1103`. The common order-19
multiplier family is a tractable starting point for our Goethals–Seidel tooling
and can be calibrated against a published witness at order `2092`. The raw row
pools are still large and Kumar's pilot estimated a very low yield per partial
sum. Order `2396` remains the [smallest open order](hm-2396-frontier.md).
[Target comparison](open-orders-and-problems.md#search-shortlist) ·
[Kumar's pilot](https://raw.githubusercontent.com/vatskishan03/Hadamard/d885022/METHOD.md#limitations).

The search uses four sign rows on `Z_1103`, each invariant under the order-19
subgroup of the units. There are 59 position orbits, including the singleton
at zero, and 29 nonzero shift classes under the subgroup and negation. The
unique sorted positive row sums are `(1,37,39,39)`. These force signs
`(+,-,+,+)` at zero and positive nonzero-orbit counts `(29,30,30,30)`.
The objective in the joint search is the sum of squares of the 29 combined
periodic autocorrelations. Each shift class has 38 members. A construction
requires combined autocorrelation zero on all `1102` nonzero shifts, followed
by exact verification of the resulting `4412 × 4412` matrix.

## Positive control and bounded blind search

The control is Kumar's cyclic order-9 multiplier seed at length `523`, with
row sums `(1,19,19,37)`. The source sign file has SHA-256
`af66d60f4961d1c2f1ef3241fe597481a76eac08d461a6dc2bb6eaac02e4a1d5`.
The script recovered a witness from sampled pools seeded with these four rows
and their matching partial sum. Two additional controls repaired corrupted
copies by exact joins, one with a swap in every row and one with swap counts
`(1,0,2,1)`. The corruptions exchanged the first positive and negative
nonzero orbits in order of least representative, using the first two of each
when two swaps were required. Both recovered the original rows byte for byte.
The native Goethals–Seidel builder reconstructed the first repaired
control. An independent Python matrix check verified all `2,187,186` row pairs.
The bounded blind searches at `2092` missed, so the controls establish the
arithmetic and repair path without a measured blind recovery rate.
[Pinned control seed](https://raw.githubusercontent.com/vatskishan03/Hadamard/d885022/hadamard/2092/sequences.txt).

The [row-pool script](../../py/search_gs_prime_join.py) generates independent
fixed-cardinality rows and attempts random orbit swaps that reduce their
autocorrelation dispersion. It indexes each pair on prescribed shift
coordinates, then checks all remaining coordinates with an exact additive
integer encoding. A hit goes through a scalar full-shift verifier before being
written to disk. The control and target used seed `11` and `12000` profiles for
each distinct row sum. With five prescribed coordinates and 64 swap proposals
per start, it checked `52603` and `8801` ordered pair records, respectively,
across 32 tried partial sums at each order. Neither run hit its pair cap or
returned a witness.

A second run used 128 proposals per start and two prescribed coordinates.
At `4412` it checked `5,452,362` pair records across four partial sums in
about 76 seconds, with no cap reached and no witness in those pools and sums.
The comparable blind `2092` run checked `6,400,000` pair records and hit the
cap on both sides at every partial sum. It produced no witness in the
records examined. The target's mean squared profile sums fell from about
`578–596` in the first run to `488–506` in the second.

## Joint search and exact repair

The existing native `gs_anneal.c` from the companion repository at commit
`c2436c66fd078f1b4c420777d9cba32a2728734d` supplied the joint-search baseline.
Three 65-second annealing runs at seeds `11`, `29`, and `47` ended at defects
`1376`, `1568`, and `1536`. A 65-second tabu run at seed `11` reached `1024`.
Four further 95-second tabu runs saved their best rows using a temporary variant
that only added incumbent output. The companion source SHA-256 is
`4f0c21c5537efcddabba89611185b83cb7f2db4bda10aa59d77adb57df090351`.

| Seed | Tabu tenure | Stall limit | Iterations | Best defect |
| --- | ---: | ---: | ---: | ---: |
| `11` | 8 | 1200 | 1050556 | **1024** |
| `29` | 5 | 400 | 1038191 | 1152 |
| `47` | 12 | 2500 | 1051682 | 1216 |
| `73` | 3 | 280 | 1017759 | 1056 |

The [saved seed-11 incumbent](../../fixtures/search/hm-4412-incumbent.txt) has
SHA-256 `b804deaee2b6b148053837bebd369f24e723630ee6eeb87aeef2459d71e3b8ef`.
Directly recomputing all `1102` nonzero shifts confirmed a defect of `38912`
over all shifts, or `1024` over class representatives. The 29 representative
correlations had the following multiplicities.

| Correlation | −16 | −12 | −8 | −4 | 0 | +4 | +12 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Count | 1 | 2 | 1 | 4 | 7 | 13 | 1 |

The exact repair search joined every profile within one swap per row around
the four saved incumbents. Around seed `11`, it also exhausted all 19 allocations
of four swaps with at most two per row, including all smaller swap counts within
those allocations. The cap of `800000` pairs per side was never reached.
There was no witness in the tested neighborhoods. The complete local checks and
the heuristic misses leave existence in the order-19 family open.

## Reproduce the sampled search and local repair

```bash
uv run python py/search_gs_prime_join.py 1103 19 1 37 39 39 \
  --seed 11 --samples 12000 --proposals 128 --prefix-width 2 --pair-limit 800000 \
  --witness-out outputs/hm4412-sampled-witness.txt

uv run python - <<'PY'
from itertools import product
import subprocess
import sys

for radii in product(range(3), repeat=4):
    if sum(radii) != 4:
        continue
    subprocess.run([
        sys.executable, 'py/search_gs_prime_join.py', '1103', '19', '1', '37', '39', '39',
        '--around-seed', 'fixtures/search/hm-4412-incumbent.txt',
        '--radii', *map(str, radii), '--pair-limit', '800000',
        '--witness-out', 'outputs/hm4412-repair-' + ''.join(map(str, radii)) + '.txt',
    ], check=True)
PY
```

To replay the native 65-second tabu baseline from a sibling checkout at the
recorded commit, run the following commands. The program's status `2` reports
that its time budget expired.

```bash
cc -O3 -march=native ../hadamard-workspace/code/search/gs_anneal.c -lm \
  -o /tmp/hadamard-prime-gs-anneal
/tmp/hadamard-prime-gs-anneal 1103 19 1 37 39 39 11 65 8 1200 0 \
  outputs/hm4412-native-witness.txt tabu
```

## Handoff, where to look next

Start with the partial-sum scheduler in
[`sampled_search` and `exact_join`](../../py/search_gs_prime_join.py). The
two-coordinate blind `2092` run hit all four pair caps. Its tested partial sums
used only `-2` and `-1`, whereas the published control's first two coordinates
sum to `(-5,-2)` in the current pairing. That leaves two plausible bottlenecks:
which partial sums we visit and whether the sampled pools contain useful rows.
I would test the scheduler first because it can reuse one set of row pools.

1. **Measure coverage on frozen `2092` pools.** Save the orbit assignments and
   profiles with length, subgroup, row sums, shift order, RNG seed and proposal
   count. For two-coordinate prefixes, count the pairs on each side from bucket
   sizes before joining. Compare the current four central sums with a schedule
   that ranks sums by the product of the left and right pair counts. Spend the
   same pair-record budget on both and record completed and truncated sums.
   Split heavy buckets into resumable slices or add a coordinate and visit
   its sub-buckets so a cap does not silently discard the rest of a sum.
2. **Separate scheduler recovery from blind recovery.** Run the scheduler once
   with the four published `2092` rows in the pools and their partial sum
   withheld. Then try blind pools at `2092` across at least three fixed seeds
   under the same budget. Log verified hits, distinct profiles per second,
   candidate pairs checked and cap hits. Count only full-shift matches as hits.
   The targeted injection and swap repairs above serve as quick arithmetic
   checks before this comparison.
3. **Improve row generation if the scheduler passes and the blind runs miss.**
   Compare longer descent runs and iterated orbit swaps at equal compute.
   Add the distinct multiplier images of sampled rows, then deduplicate on the
   full correlation profile. Measure the additional profiles and matching
   throughput. Individual row dilations permute profile coordinates, so keep
   their distinct images available to the join. Port the row generator or
   matching loop to native code once timings establish which one dominates.
4. **Transfer the promising configuration to `4412`.** A blind `2092` hit at a
   documented cost would justify a larger `4412` run. Compare identical metrics
   and preserve completed slices, RNG seeds and any candidate rows. A hit at
   `4412` needs an independent check of all `1102` shifts and a full matrix
   check. If the scheduler and generation changes still give little evidence
   for scaling, compare a bounded `4748` quotient experiment before allocating
   substantially more compute. [Alternative targets](open-orders-and-problems.md#search-shortlist).

The saved incumbent and the exhaustive repair commands above capture the last
exact local result. The next experiment uses fresh row pools. Its results belong
here and in the [project state](../PROJECT_STATE.md).
