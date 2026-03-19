# CRT Fourier Notes For LP(333)

This note records the first exact consequences of viewing a length-`333` Legendre pair through

- `333 = 9 * 37`
- `Z_333 ~= Z_9 x Z_37`

The goal is not a full proof development. The goal is to put the CRT line on explicit algebraic footing so future code work can target exact statements rather than intuition.

## 1. Basic LP Identity

For `A = (a_i)` and `B = (b_i)` with entries in `{+1, -1}`, define

- `A(x) = sum_i a_i x^i`
- `B(x) = sum_i b_i x^i`

in `Z[x] / (x^333 - 1)`.

The periodic Legendre-pair condition is equivalent to

- `PAF_A(s) + PAF_B(s) = -2` for every nonzero shift `s mod 333`
- `PAF_A(0) + PAF_B(0) = 666`

Equivalently,

- `A(x) A(x^-1) + B(x) B(x^-1) = 333 - 2 * sum_{j=1}^{332} x^j`

or, after combining the constant and nonconstant terms,

- `A(x) A(x^-1) + B(x) B(x^-1) = 335 - 2 * sum_{j=0}^{332} x^j`

inside `Z[x] / (x^333 - 1)`.

The Fourier form is the standard one:

- at the trivial character, `|A(1)|^2 + |B(1)|^2 = 4 * 333 + 2 = 1334`
- at every nontrivial character `chi`, `|A(chi)|^2 + |B(chi)|^2 = 335`

For the current row-sum-`1` normalization used by the code, this becomes

- `A(1) = B(1) = 1`
- therefore every nontrivial Fourier mode must satisfy `|A(chi)|^2 + |B(chi)|^2 = 335`

That is the exact spectral target already used by the search.

## 2. CRT Indexing Of Coordinates

Under the CRT map

- `i mod 333 <-> (u mod 9, v mod 37)`

each sequence becomes a `9 x 37` signed table:

- `a[u, v]`
- `b[u, v]`

with indices interpreted modulo `9` and `37`.

Then the periodic autocorrelation at shift `(alpha, beta)` is

- `PAF_A(alpha, beta) = sum_{u in Z_9} sum_{v in Z_37} a[u, v] a[u + alpha, v + beta]`

and similarly for `B`.

The LP condition becomes:

- `PAF_A(alpha, beta) + PAF_B(alpha, beta) = -2` for every nonzero `(alpha, beta) in Z_9 x Z_37`

This already separates the shifts into three algebraically distinct families.

## 3. Three Shift Families

The nonzero shifts split into:

1. `pure-9` shifts: `(alpha, 0)` with `alpha != 0`
2. `pure-37` shifts: `(0, beta)` with `beta != 0`
3. `mixed` shifts: `(alpha, beta)` with `alpha != 0` and `beta != 0`

Counts:

- pure-9: `8`
- pure-37: `36`
- mixed: `8 * 36 = 288`

These are not equivalent from the product-group point of view. Any future CRT sieve should ask whether some necessary conditions can be derived separately for these three families rather than treating all `332` nontrivial shifts uniformly.

## 4. Three Frequency Families

The dual group also factors:

- `widehat(Z_333) ~= widehat(Z_9) x widehat(Z_37)`

So Fourier characters are indexed by pairs `(r, s)` with

- `r in Z_9`
- `s in Z_37`

and

- `chi_{r,s}(u, v) = omega_9^(r u) omega_37^(s v)`

where `omega_9 = exp(2 pi i / 9)` and `omega_37 = exp(2 pi i / 37)`.

Again the nontrivial characters split into three families:

1. `pure-9` characters: `(r, 0)` with `r != 0`
2. `pure-37` characters: `(0, s)` with `s != 0`
3. `mixed` characters: `(r, s)` with `r != 0` and `s != 0`

For every nontrivial `(r, s)`, the LP condition gives the same exact scalar equation:

- `|A(r, s)|^2 + |B(r, s)|^2 = 335`

But the decomposition suggests the families should not be treated as interchangeable when deriving secondary invariants.

## 5. Explicit Pure-Family Transforms

Define row and column sums of `A` by

- `R_A(u) = sum_v a[u, v]`
- `C_A(v) = sum_u a[u, v]`

and similarly for `B`.

Then the pure-character transforms reduce exactly to lower-dimensional transforms:

- `A(r, 0) = sum_u R_A(u) omega_9^(r u)`
- `A(0, s) = sum_v C_A(v) omega_37^(s v)`

and similarly for `B`.

So the exact LP equations imply:

- for every `r != 0`,
  `|DFT_9(R_A)(r)|^2 + |DFT_9(R_B)(r)|^2 = 335`
- for every `s != 0`,
  `|DFT_37(C_A)(s)|^2 + |DFT_37(C_B)(s)|^2 = 335`

This is the first concrete CRT-derived necessary condition that looks directly usable:

- any valid LP(333) pair induces a pair of integer row-sum vectors of length `9`
- and a pair of integer column-sum vectors of length `37`
- both of which satisfy exact nontrivial PSD identities with the same target `335`

That is stronger than a vague product-group intuition. It is an explicit lower-dimensional sieve candidate.

## 5a. Equivalent Row-Sum And Column-Sum LP Identities

The pure-family Fourier identities can be rewritten as exact periodic-autocorrelation identities for the row-sum and column-sum vectors themselves.

### Row sums

Let `R_A, R_B` be the length-`9` row-sum vectors.

Because

- `sum_u R_A(u) = A(1) = 1`
- `sum_u R_B(u) = B(1) = 1`
- `|DFT_9(R_A)(r)|^2 + |DFT_9(R_B)(r)|^2 = 335` for every nontrivial `r`

Parseval gives

- `sum_u R_A(u)^2 + sum_u R_B(u)^2 = (1 + 1 + 8 * 335) / 9 = 298`

and inverse Fourier transform gives the exact periodic-autocorrelation target

- `PAF_{R_A}(alpha) + PAF_{R_B}(alpha) = -37` for every nonzero `alpha mod 9`

So any valid LP(333) pair induces an exact integer pair of length `9` with:

- entries odd and in `[-37, 37]`
- total sums `1`
- combined squared norm `298`
- nonzero periodic autocorrelation target `-37`

This is a true compressed LP-style problem, not merely a heuristic marginal summary.

### Column sums

Let `C_A, C_B` be the length-`37` column-sum vectors.

Because

- `sum_v C_A(v) = A(1) = 1`
- `sum_v C_B(v) = B(1) = 1`
- `|DFT_37(C_A)(s)|^2 + |DFT_37(C_B)(s)|^2 = 335` for every nontrivial `s`

Parseval gives

- `sum_v C_A(v)^2 + sum_v C_B(v)^2 = (1 + 1 + 36 * 335) / 37 = 326`

and inverse Fourier transform gives

- `PAF_{C_A}(beta) + PAF_{C_B}(beta) = -9` for every nonzero `beta mod 37`

So any valid LP(333) pair also induces an exact integer pair of length `37` with:

- entries odd and in `[-9, 9]`
- total sums `1`
- combined squared norm `326`
- nonzero periodic autocorrelation target `-9`

This is the second exact lower-dimensional sieve candidate.

## 6. Immediate Computational Interpretation

The current code already works with compressed sequences. The row/column-sum viewpoint suggests two exact sieve directions.

### A. Row-bundle sieve

Treat the length-`333` pair as `9` blocks of length `37`. Then:

- the `9` row sums for `A`
- the `9` row sums for `B`

must satisfy the pure-`Z_9` Fourier identities exactly.

These are integer constraints on values in `[-37, 37]` with parity fixed mod `2`.

### B. Column-bundle sieve

Treat the same pair as `37` blocks of length `9`. Then:

- the `37` column sums for `A`
- the `37` column sums for `B`

must satisfy the pure-`Z_37` Fourier identities exactly.

These are integer constraints on values in `[-9, 9]` with parity fixed mod `2`.

Either sieve could be used:

- as a standalone verifier for candidate decompositions
- as a pre-search enumeration problem
- or as a future join key if a factorization is aligned to the `9 x 37` layout

In other words, the CRT marginals are not only compatible with compression by `37` and `9`; they are exact compressed LP-type objects with fixed autocorrelation targets `-37` and `-9`.

## 7. Why This May Matter More Than The Current Compression

The current compression factors `3`, `9`, `37`, and `111` collapse coordinates arithmetically, but they do not yet explicitly distinguish:

- pure `Z_9` structure
- pure `Z_37` structure
- mixed `Z_9 x Z_37` structure

The CRT view does distinguish them, and the pure-character constraints above are exact.

That does not prove they will prune well. But it means the CRT line already yields more than philosophy: it yields concrete lower-dimensional objects whose spectra must satisfy the same exact target `335`.

## 8. Next Questions

The next derivations to try are:

1. characterize all possible row-sum pairs `(R_A, R_B)` of length `9` satisfying the pure-`Z_9` conditions
2. characterize all possible column-sum pairs `(C_A, C_B)` of length `37` satisfying the pure-`Z_37` conditions
3. determine whether mixed-character equations impose compatibility constraints between those two marginal views
4. test whether any of these necessary conditions are strong enough to act as a practical pre-search sieve

## 9. Multiplier Connection

The multiplier line naturally interacts with the CRT decomposition.

Any unit `t in U(333)` acts on indices by

- `(u, v) -> (t mod 9) * u, (t mod 37) * v`

So a multiplier subgroup would act simultaneously on:

- row indices in `Z_9`
- column indices in `Z_37`
- and frequency indices in the dual product group

That makes the CRT basis the natural place to study multipliers. If a useful multiplier exists, its orbit structure is likely to be clearer in `Z_9 x Z_37` coordinates than in the flat cyclic indexing.
