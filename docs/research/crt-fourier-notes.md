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

- at the trivial character, `|A(1)|^2 + |B(1)|^2 = 2`
- at every nontrivial character `chi`, `|A(chi)|^2 + |B(chi)|^2 = 2 * 333 + 2 = 668`

For the current row-sum-`1` normalization used by the code, this becomes

- `A(1) = B(1) = 1`
- therefore every nontrivial Fourier mode must satisfy `|A(chi)|^2 + |B(chi)|^2 = 668`

That is the exact spectral target for an LP(333) pair.

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

- `|A(r, s)|^2 + |B(r, s)|^2 = 668`

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
  `|DFT_9(R_A)(r)|^2 + |DFT_9(R_B)(r)|^2 = 668`
- for every `s != 0`,
  `|DFT_37(C_A)(s)|^2 + |DFT_37(C_B)(s)|^2 = 668`

This is the first concrete CRT-derived necessary condition that looks directly usable:

- any valid LP(333) pair induces a pair of integer row-sum vectors of length `9`
- and a pair of integer column-sum vectors of length `37`
- both of which satisfy exact nontrivial PSD identities with the same target `668`

That is stronger than a vague product-group intuition. It is an explicit lower-dimensional sieve candidate.

## 5a. Equivalent Row-Sum And Column-Sum LP Identities

The pure-family Fourier identities can be rewritten as exact periodic-autocorrelation identities for the row-sum and column-sum vectors themselves.

### Row sums

Let `R_A, R_B` be the length-`9` row-sum vectors.

Because

- `sum_u R_A(u) = A(1) = 1`
- `sum_u R_B(u) = B(1) = 1`
- `|DFT_9(R_A)(r)|^2 + |DFT_9(R_B)(r)|^2 = 668` for every nontrivial `r`

Parseval gives

- `sum_u R_A(u)^2 + sum_u R_B(u)^2 = (1 + 1 + 8 * 668) / 9 = 594`

and inverse Fourier transform gives the exact periodic-autocorrelation target

- `PAF_{R_A}(alpha) + PAF_{R_B}(alpha) = -74` for every nonzero `alpha mod 9`

So any valid LP(333) pair induces an exact integer pair of length `9` with:

- entries odd and in `[-37, 37]`
- total sums `1`
- combined squared norm `594`
- nonzero periodic autocorrelation target `-74`

This is a true compressed LP-style problem, not merely a heuristic marginal summary.

### Column sums

Let `C_A, C_B` be the length-`37` column-sum vectors.

Because

- `sum_v C_A(v) = A(1) = 1`
- `sum_v C_B(v) = B(1) = 1`
- `|DFT_37(C_A)(s)|^2 + |DFT_37(C_B)(s)|^2 = 668` for every nontrivial `s`

Parseval gives

- `sum_v C_A(v)^2 + sum_v C_B(v)^2 = (1 + 1 + 36 * 668) / 37 = 650`

and inverse Fourier transform gives

- `PAF_{C_A}(beta) + PAF_{C_B}(beta) = -18` for every nonzero `beta mod 37`

So any valid LP(333) pair also induces an exact integer pair of length `37` with:

- entries odd and in `[-9, 9]`
- total sums `1`
- combined squared norm `650`
- nonzero periodic autocorrelation target `-18`

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

In other words, the CRT marginals are not only compatible with compression by `37` and `9`; they are exact compressed LP-type objects with fixed autocorrelation targets `-74` and `-18`.

Practical note from the current codebase:

- treating the row-sum marginal naively as the existing generic `compression 37` benchmark does not yet make it tractable
- capped probe runs at `30s` with both `spectral_frequencies=0` and `spectral_frequencies=2` did not reach a first pair
- the dedicated `hadamard analyze lp333-crt` utility shows that norm feasibility alone is weak:
  - row marginal: `73` feasible per-sequence norm splits toward combined norm `594`
  - column marginal: `73` feasible per-sequence norm splits toward combined norm `650`
- on the row marginal, sum-only filtering is far too weak:
  - there are `1,969,631,230,590` single length-`9` sequences over odd entries in `[-37, 37]` with total sum `1`
  - so any practical dedicated solver has to exploit exact autocorrelation / Fourier structure very early, not just row-sum and norm bookkeeping
- a naive `3+3+3` exact-block signature split does not obviously save the day:
  - raw length-`3` row blocks: `54,872`
  - distinct internal signatures `(sum, norm, paf1, paf2)`: `28,158`
  - but once endpoint data is carried, the signature count goes back to the full `54,872`
  - so a small-block exact seam join will only help if it finds a stronger boundary invariant than "carry the endpoints"

So the CRT marginal line should not be interpreted as "the current compressed engine already solves this." It points instead to the need for a dedicated marginal solver or sieve that uses the exact row/column identities directly.

## 6a. A Second Compression Inside The Row Marginal

The row marginal has a further exact compression coming from the special pure-`Z_9` frequencies `r = 3, 6`.

If the row-sum vector is `R_A(u)`, define the three residue-class bundle sums

- `T_A(j) = R_A(j) + R_A(j + 3) + R_A(j + 6)` for `j mod 3`

and similarly `T_B`.

Then the frequency `r = 3` only sees these bundled sums:

- `DFT_9(R_A)(3) = DFT_3(T_A)(1)`
- `DFT_9(R_A)(6) = DFT_3(T_A)(2)`

and similarly for `B`.

So the pair `(T_A, T_B)` is itself an exact length-`3` LP-type marginal:

- entries odd and in `[-111, 111]`
- total sums `1`
- nontrivial Fourier target `668`
- combined squared norm `(2 + 2 * 668) / 3 = 446`
- nonzero periodic autocorrelation target `(2 - 668) / 3 = -222`

This induced subproblem is much smaller than the full row marginal:

- single length-`3` sum-`1` sequences: `9408`
- feasible per-sequence norm splits toward combined norm `446`: `16`
- distinct exact `PAF(1)` values on the bundled marginal: `1186`
- ordered exact pair solutions to the bundled length-`3` problem: `504`

So while the full row marginal is still huge, the mod-`3` bundled marginal is the first CRT-derived exact subproblem that already looks solver-sized.

The dedicated `hadamard analyze lp333-crt` utility now also measures what happens when the bundled exact pair solutions are lifted back through the true row-marginal norm target `594`:

- all `504` ordered bundled exact pair solutions remain norm-compatible at the full row-marginal level
- but the lifted ordered-pair upper bound drops from `733,366,929,773,393,867,016` to `5,035,801,219,344`
- the heaviest surviving bundled row pairs are already highly structured, for example
  `[-15, 3, 13]` paired with permutations of `[-5, 3, 3]`, each carrying
  `9,991,925,496` norm-compatible lifts
- the active bundled state set is small and symmetry-heavy:
  - `90` active bundled states
  - only `30` cyclic rotation orbits among those `90`
  - the heaviest active orbit is the rotation family of `[-9, -1, 11]`
    with orbit-level lifted mass `719,395,139,232`
  - the heaviest per-state mass inside that orbit is `239,798,379,744`
- the surviving bundled pair set is also symmetry-compressed:
  - `504` surviving bundled exact pairs
  - only `168` simultaneous cyclic pair orbits among those `504`
  - those `168` ordered pair orbits collapse exactly to `84` under the `A/B` swap symmetry
  - and those `84` unordered pair orbits collapse again to `42` under common dihedral symmetry
  - the heaviest unordered pair orbits are exactly the doubled versions of the
    heaviest ordered ones, each with mass `59,951,552,976`
  - the heaviest dihedral-swap orbit classes double again to mass `119,903,105,952`
    and include both the `[-15, 3, 13] | [-5, 3, 3]` family and the
    `[-9, -5, 15] | [-5, -3, 9]` family
  - a naive exact lift of even the single heaviest pair orbit through row shifts `1, 2, 4`
    was too slow to keep inside the default `analyze lp333-crt` path
  - the reason is scale: the heaviest surviving pair orbit
    `[-15, 3, 13] | [-5, 3, 3]` is built from six component-sum classes of sizes
    `1027, 1081, 1041, 1077, 1081, 1081`, so its raw exact lift space is already
    `1,454,500,779,279,999,399`
  - but the first useful compression is real: under the natural `UV -> W` factorization
    for row shifts `1, 2, 4`, the heaviest pair orbit has only about
    `1,110,187` left-side transition signatures and `1,164,237` right-side ones
  - however, on that top orbit the coefficient-only transition signature count is the same
    as the full transition signature count, so there is little extra redundancy to win
    by stripping off the scalar base terms alone
  - in fact the raw `U,V` pair counts on that top orbit are exactly the same as the
    `UV` transition signature counts on both sides, so the current `UV -> W` split
    shows essentially no collision/compression before the final `W` stage
  - trying the two other cyclic `2+1` splits on that same top orbit only changes the
    transition counts modestly:
    - left side: `1,110,187`, `1,125,321`, `1,069,107`
    - right side: `1,164,237`, `1,168,561`, `1,164,237`
    so there is no obvious "good split" hiding among the three cyclic variants

That is an important calibration point:

- the bundled length-`3` sieve is already strong enough to crush the raw row-marginal multiplicity
- the full row norm is a meaningful secondary constraint on lifted mass
- but norm compatibility alone does not distinguish among the `504` bundled exact pairs
- the surviving lifted mass is concentrated in a small-looking family of bundled patterns rather than spread uniformly
- the surviving bundled states are organized strongly by the natural cyclic action on the length-`3` bundle
- the residual bundled problem is now naturally an orbit-sieve problem on roughly `30` active bundle orbits, not a generic search over `9408` bundled states
- even the surviving bundled pair space is only on the order of `10^2` orbit classes, which is small enough to justify a direct exact lift using the remaining pure-`Z_9` constraints
- after quotienting by the obvious `A/B` swap symmetry, that residual bundled pair space is really only about `84` orbit classes
- after quotienting by common dihedral symmetry as well, the residual bundled pair space is only about `42` orbit classes
- but that exact lift needs an optimized orbit-level DP or signature join, not a naive distribution build inside the main analysis command
- the `UV -> W` transition signature counts suggest that such an optimized join may actually be viable, because they are around `10^6` rather than `10^18`
- but the next compression is unlikely to come from a trivial coefficient-only collapse of the `UV` states
- and the present `UV -> W` split is probably not the right one to bet on as the main compression mechanism, because the top orbit shows no `UV` collisions at all

So the next real refinement has to come from a more structural exact invariant, not another norm-only filter.

## 7. Why This May Matter More Than The Current Compression

The current compression factors `3`, `9`, `37`, and `111` collapse coordinates arithmetically, but they do not yet explicitly distinguish:

- pure `Z_9` structure
- pure `Z_37` structure
- mixed `Z_9 x Z_37` structure

The CRT view does distinguish them, and the pure-character constraints above are exact.

That does not prove they will prune well. But it means the CRT line already yields more than philosophy: it yields concrete lower-dimensional objects whose spectra must satisfy the same exact target `668`.

## 8. Next Questions

The next derivations to try are:

1. characterize all possible row-sum pairs `(R_A, R_B)` of length `9` satisfying the pure-`Z_9` conditions
2. characterize all possible column-sum pairs `(C_A, C_B)` of length `37` satisfying the pure-`Z_37` conditions
3. determine whether mixed-character equations impose compatibility constraints between those two marginal views
4. test whether any of these necessary conditions are strong enough to act as a practical pre-search sieve
5. on the row side specifically, derive an exact lift condition from the remaining pure-`Z_9` frequencies `r = 1, 2, 4, 5, 7, 8` or the equivalent row shifts `1, 2, 4`

## 9. Multiplier Connection

The multiplier line naturally interacts with the CRT decomposition.

Any unit `t in U(333)` acts on indices by

- `(u, v) -> (t mod 9) * u, (t mod 37) * v`

So a multiplier subgroup would act simultaneously on:

- row indices in `Z_9`
- column indices in `Z_37`
- and frequency indices in the dual product group

That makes the CRT basis the natural place to study multipliers. If a useful multiplier exists, its orbit structure is likely to be clearer in `Z_9 x Z_37` coordinates than in the flat cyclic indexing.
