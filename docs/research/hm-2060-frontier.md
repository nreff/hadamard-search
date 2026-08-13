# The post-2000 frontier: order 2060

Status date: `2026-08-13`

## Claim boundary

On the current public, verifiable record, the least admissible order for which no
Hadamard matrix has been verified is

\[
2060 = 4\cdot 515 = 4\cdot 5\cdot 103.
\]

This is a statement about the public construction record, not a nonexistence
claim.  The Hadamard conjecture predicts that an order-`2060` matrix exists.

The qualification matters because this frontier changed on `2026-08-12`.
Levent Alpöge's public payload and decoder encode witnesses for the twelve
previously missing admissible orders below `2000`:

```text
668, 716, 892, 1132, 1244, 1388,
1436, 1676, 1772, 1916, 1948, 1964
```

The decoded instruction stream is

```text
h664 h712 g892 g1132 g1244 v1368
u1408 h1672 h1768 w1904 g1948 g1964
```

and consumes all `23,828` signs in the primary payload.  Epoch AI reports that
the puzzle encodes all previously missing admissible orders through `2000`.
This repository has independently reconstructed and exactly verified the
order-`668` member; the remaining public matrices should receive the same durable
artifact treatment.

The 2025-revised Cati--Pasechnik table writes `n(m)` when `m` is the least known
exponent such that an order-`2^m n` Hadamard matrix exists.  Its first entries are

```text
167(3), 179(3), 223(3), 283(3), 311(3), 347(3),
359(4), 419(3), 443(3), 479(4), 487(3), 491(5),
515(3), 523(3), ...
```

The first twelve odd parts are exactly the twelve cases filled by the new
payload.  The next entry is `515(3)`: order `8*515 = 4120` is known, while order
`4*515 = 2060` is not verifiably known.  The next few surviving gaps are

```text
2060, 2092, 2148, 2284, 2292, 2396, 2572, 2588,
2636, 2676, 2876, 2884, 2956, 3004, ...
```

Sources:

- Cati and Pasechnik, [A database of constructions of Hadamard
  matrices](https://arxiv.org/abs/2411.18897), especially Section 6 and Table 4
- [machine-readable source for version 2](https://arxiv.org/src/2411.18897v2)
- Epoch AI's [solution update](https://epoch.ai/frontiermath/open-problems/hadamard)
- Alpöge's primary [`+/-` payload](https://x.com/__alpoge__/status/2087504785952182273)
  and [decoder/generator](https://x.com/__alpoge__/status/2087504788938510427)

## The 1991 claim does not close the case

Miyamoto's 1991 paper lists `515` among factors claimed to yield Hadamard
matrices.  Cati and Pasechnik explicitly retain `515(3)`, explaining that they
could not verify the order-`4*515` claim because the necessary construction data
are missing.

A closer audit makes the apparent route less promising, not more.  The proposed
Baumert--Hall multiplication needs an order-`103` M-partition seed.  The cited
Corollary 2 produces only odd factors congruent to `1 mod 4`, so it cannot produce
`103`.  Correcting the apparent dimension typo in the likely intended Corollary 4
sets its internal parameter to `51`.  That specialization asks for a symmetric,
zero-diagonal `51 x 51` sign block with row sum zero.  Its `+1` off-diagonal
entries would form a `25`-regular graph on `51` vertices, impossible because the
degree sum `51*25` is odd.

This invalidates that cited specialization; it does **not** prove that `H(2060)`
does not exist.  If one accepts the old bare claim without a checkable witness,
then `2092` would be called the next gap.  This project follows the modern,
reproducible standard and therefore treats `2060` as the frontier.

Primary historical source: Masahiko Miyamoto, [A construction of Hadamard
matrices](https://doi.org/10.1016/0097-3165(91)90008-5).

## Selected attack: an orbit-invariant cyclic GS family

Let `v=515`.  Four cyclic `+/-1` sequences `A,B,C,D` of length `v` whose
combined periodic autocorrelation is zero at every nonzero shift give a Hadamard
matrix of order `4v=2060` through the Goethals--Seidel array already implemented
in this repository.

This route is evidence-led.  The public puzzle's `g` constructor is an unbordered
four-circulant Goethals--Seidel construction.  Its five seeds have core orders
`223`, `283`, `311`, `487`, and `491`, and the decoded seeds are stabilized by
small multiplicative subgroups.  The nearest extrapolation is therefore a
subgroup-orbit search at core order `515`, rather than a search over all
`2060^2` matrix entries.

### The order-17 multiplier shard

The unit group modulo `515=5*103` has a canonical subgroup `H` of order `17`.
Multiplication by `H` partitions `Z_515` into

- `5` fixed points (the multiples of `103`), and
- `30` orbits of size `17`.

For any four complementary odd-length sequences, their row sums satisfy

\[
r_A^2+r_B^2+r_C^2+r_D^2=4v=2060.
\]

There are `29` nondecreasing positive odd row-sum quadruples satisfying this
equation.  Requiring each negative support to be a union of `H`-orbits leaves
exactly one:

```text
row sums:              1,   3,  31,  33
negative support size: 257, 256, 242, 241
fixed points selected:   2,   1,   4,   3
size-17 orbits selected: 15,  15,  14,  14
```

Because periodic autocorrelation is even in the shift, the `514` nonzero shifts
collapse to `17` orbits under multiplication by `+/-H`.  An exact
pseudo-Boolean model can therefore use one Boolean for each sequence/orbit,
weighted cardinality equations for the four supports, XOR indicators for orbit
pairs, and only `17` weighted disagreement equations.

The result has a deliberately narrow interpretation:

- `SAT`, followed by independent full-length correlation and matrix checks,
  proves `H(2060)` exists;
- a proof-checked `UNSAT` result excludes the `H`-invariant cyclic
  Goethals--Seidel subfamily only;
- a timeout means no conclusion.

### Implemented exact campaign

[`py/search_sds_515.py`](../../py/search_sds_515.py) is the transparent baseline
encoder and independent full-shift model checker.  Its OPB instance has `2,520`
Boolean variables and `9,541` constraints.  The exact symmetry reduction in
[`py/search_sds_515_reduced.py`](../../py/search_sds_515_reduced.py) then uses:

- independent translations of the four rows by multiples of `103`;
- one common dilation by a unit modulo `515`; and
- one residual reflection lex leader per row.

An exhaustive test of all `2,500` singleton-support choices leaves `1,250` that
pass the two fixed-shift equations, and maps every one to the single normalized
representative.  The final campaign therefore remains exhaustive for the whole
order-`17`-invariant cyclic family while shrinking to:

```text
variables:    1,860 = 120 orbit bits + 1,740 XOR bits
constraints:  6,985
equalities:      21 = 4 support + 17 shift equations
OPB SHA-256:  4f13cfd9dd69ad580a5b7343e8bacf5199eaa7e8824c50523c17b6f2232e434c
```

Reproduce the instance and its structural tests with:

```bash
uv run python py/search_sds_515_reduced.py \
  --export-opb /tmp/sds515-h17-reduced.opb
uv run python py/test_search_sds_515.py
uv run python py/test_search_sds_515_reduced.py
```

A `30`-second RoundingSat calibration on the reduced instance reached `672,083`
conflicts and returned `TIMELIMIT`: **no conclusion**.

### Implemented heuristic companion

[`py/search_sds_515_heuristic.py`](../../py/search_sds_515_heuristic.py) performs
seeded tabu search using swaps that preserve all eight fixed/nonfixed orbit-count
conditions exactly.  Its objective is the exact sum of squared residuals in the
`17` shift equations; objective zero invokes the independent all-`514`-shift
checker before any witness is written.

A bounded four-seed, `170`-second calibration found no witness.  Its best objective
was `24`: eleven equations were exact and the remaining six had residuals `+/-2`.
This is a useful search baseline but has no exclusion meaning.

## Parallel lanes

The selected shard should not become a single point of failure.  The next lanes
are:

1. Find base sequences `BS(52,51)`, which yield T-sequences of length `103`,
   then combine them with known Williamson matrices of order `5` through the
   Cooper--Wallis construction.
2. Search directly for a checkable order-`103` amicable M-partition or
   Williamson/near-Williamson seed, then use a Baumert--Hall array of order `5`.
3. Relax the multiplier subgroup in the cyclic `Z_515` GS search, exploiting
   `Z_515 ~= Z_5 x Z_103` for exact quotient and Fourier propagation.
4. Explore the bordered type-`H_4^*` route at seed length `514`; higher-order
   multiplier shards are small enough to eliminate first, with order `32` the
   first substantial meet-in-the-middle case.

Direct search over an unrestricted `2060 x 2060` sign matrix is not a credible
first attack.
