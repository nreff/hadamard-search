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
  - concentration is real but not extreme:
    - top `1` dihedral-swap class carries about `2.38%` of the full norm-refined lifted mass
    - top `5` carry about `11.9%`
    - top `10` carry about `23.8%`
    - top `20` carry about `47.6%`
    - it takes the top `21` dihedral-swap classes to reach half the residual mass
  - those top `21` classes are built from only `11` distinct bundled row orbits
  - that `11`-orbit core is:
    - `[-15, 3, 13]`
    - `[-15, 5, 11]`
    - `[-13, -1, 15]`
    - `[-9, -5, 15]`
    - `[-5, -3, 9]`
    - `[-5, -1, 7]`
    - `[-5, 1, 5]`
    - `[-5, 3, 3]`
    - `[-5, 5, 1]`
    - `[-5, 7, -1]`
    - `[-5, 9, -3]`
  - that half-mass core is not uniform across the `11` bundle orbits:
    - `[-15, 5, 11]`, `[-13, -1, 15]`, and `[-9, -5, 15]` each occur `6` times
    - each of the other `8` bundle orbits occurs `3` times
  - so those three bundle orbits are the current hub states of the half-mass frontier
  - the half-mass frontier also has only a small set of repeated symmetry-reduced pairings:
    - `[-15, 3, 13] | [-5, 3, 3]`
    - `[-15, 5, 11] | [-5, -1, 7]`
    - `[-15, 5, 11] | [-5, 7, -1]`
    - `[-13, -1, 15] | [-5, 1, 5]`
    - `[-13, -1, 15] | [-5, 5, 1]`
    - `[-9, -5, 15] | [-5, -3, 9]`
    - `[-9, -5, 15] | [-5, 9, -3]`
  - each of those pair families appears `3` times inside the top-`21` dihedral-swap half-mass prefix
    so the residual row-bundle structure now looks more like a small hub-and-spoke orbit graph
    than a broad undifferentiated residual family
  - that graph description is now exact:
    - connected-component sizes are `3, 3, 3, 2`
    - `[-15, 5, 11]`, `[-13, -1, 15]`, and `[-9, -5, 15]` each have graph degree `2`
    - every other bundle orbit in the half-mass core has degree `1`
  - so the half-mass residual is not merely "small"; it splits into three tiny two-spoke hub components
    plus one separate edge `[-15, 3, 13] | [-5, 3, 3]`
  - the component masses make the staging decision cleaner still:
    - `719,418,635,712`: `[-9, -5, 15]`, `[-5, -3, 9]`, `[-5, 9, -3]`
    - `719,395,139,232`: `[-15, 5, 11]`, `[-5, -1, 7]`, `[-5, 7, -1]`
    - `719,395,139,232`: `[-13, -1, 15]`, `[-5, 1, 5]`, `[-5, 5, 1]`
    - `359,709,317,856`: `[-15, 3, 13]`, `[-5, 3, 3]`
  - so the three `3`-node hub components are effectively tied in mass, while the separate `2`-node edge
    carries exactly half of a `3`-node component
  - a separate `hadamard analyze lp333-crt-component` diagnostic now tests whether one representative
    `3`-node hub component creates shared exact-lift state across its two spoke families
  - on the representative component `[-15, 5, 11]` with spokes `[-5, -1, 7]` and `[-5, 7, -1]`,
    the answer is negative at every tested level:
    - hub `UV` signature count: `1,106,079`
    - spoke `UV` signature counts: `1,166,391` and `1,153,467`
    - spoke `UV` overlap: `0`
    - spoke coefficient-only overlap: `0`
    - spoke coarse `W`-frontier counts: `4,210` and `4,218`
    - spoke coarse `W`-frontier overlap: `0`
  - so the current `UV -> W` exact-lift model does not create any cross-spoke reuse even at the
    coarsest tested `W`-frontier projection on a representative hub component
  - the same zero-overlap pattern also holds on the slightly heaviest `3`-node component
    `[-9, -5, 15]` with spokes `[-5, -3, 9]` and `[-5, 9, -3]`:
    - spoke `UV` overlap: `0`
    - spoke coefficient-only overlap: `0`
    - spoke coarse `W`-frontier overlap: `0`
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
  - a separate `hadamard analyze lp333-crt-bundle` probe now shows that this is not
    merely a top-orbit artifact:
    - on the representative hub bundle `[-15, 5, 11]`, the heaviest bundle `[-15, 3, 13]`,
      and the slightly heaviest hub bundle `[-9, -5, 15]`, every cyclic `2+1` split
      is still injective at the full `UV` level
    - the coefficient-only signatures are also injective on all of those tested splits
  - but the same bundle probe also shows where the current factorization does compress:
    - `[-15, 5, 11]` has coarse `W`-frontier counts `4,194`, `4,155`, `4,215`
    - `[-15, 3, 13]` has coarse `W`-frontier counts `4,183`, `4,155`, `4,218`
    - `[-9, -5, 15]` has coarse `W`-frontier counts `4,170`, `4,194`, `4,210`
    - that is a stable collapse of about `253x` to `275x` from raw `UV` pairs
    - and only `6` frontier keys are singletons on each tested split, with the heaviest
      frontier bucket carrying roughly `930` to `1015` raw `UV` states
  - so the current `UV -> W` split does not buy anything by deduplicating `UV` states
    themselves, but it may still be usable if the exact lift is organized directly
    around `W`-frontier batching or a stronger downstream exact key
  - a subsequent `hadamard analyze lp333-crt-pair` probe then tested the first concrete
    one-shift version of that idea
  - on the representative hub pair `[-15, 5, 11] | [-5, -1, 7]`, shifts `1`, `2`, and `4`
    all look the same at the summary level:
    - left reduced `(norm_uv, base_shift, W-coeff)` states: `1,106,079`
    - right reduced states: `1,116,807` from `1,166,391` raw `UV` pairs
    - left total distinct per-coefficient `W` signatures: `4,344,024`
    - right total distinct per-coefficient `W` signatures: `4,437,827`
    - naive norm-plus-one-shift materialization estimate: about `1.132e9` left-side combinations
      and `1.164e9` right-side ones
  - the heaviest pair `[-15, 3, 13] | [-5, 3, 3]` is comparable rather than better:
    - left reduced states: `1,044,424`
    - right reduced states: `1,164,237`
    - materialization estimates: about `1.068e9` and `1.223e9`
  - coefficient-level symmetry gives a useful implementation cache, but not a new search regime:
    - coefficient permutation orbits reduce distinct `W` histogram profiles by about `5.84x`
      on both representative and heaviest tested pairs
    - the heaviest sampled local `W` buckets still require about `0.93M` to `1.04M`
      reduced-state-by-`W` products each
    - those local bucket products collapse only to about `0.20M` to `0.22M` row signatures,
      a local compression of roughly `4.1x` to `4.9x`
    - so local bucket convolution is not hiding an order-of-magnitude reduction by itself
  - a compact left/right frontier-join diagnostic gives a useful but still insufficient
    staging filter:
    - it tests exact row-norm and one-shift marginal compatibility between coefficient buckets
      without materializing full row-signature histograms
    - on `[-15, 5, 11] | [-5, -1, 7]`, bucket pairs drop from `17,656,740` to `79,398`
      under exact norm compatibility
    - on `[-15, 3, 13] | [-5, 3, 3]`, bucket pairs drop from `17,643,894` to `80,322`
    - the one-shift marginal itself prunes no bucket pairs on either test
    - active side materialization remains around `266M` to `326M` combinations per side,
      so the filter helps staging but does not change the regime by itself
    - however, sampled exact joins inside the largest survivor bucket pairs are sparse:
      `967,079,856,834` raw row-pair products collapse to `13,194`, `13,180`, and `13,180`
      exact joins on the representative pair's top three samples, while `962,719,464,960`
      collapse to `11,971`, `12,095`, and `12,095` on the heaviest pair's top three samples
    - materializing only the active frontier buckets now recovers the full exact one-shift
      join count offline:
      - representative pair `[-15, 5, 11] | [-5, -1, 7]`: `124,923,897`
      - heaviest pair `[-15, 3, 13] | [-5, 3, 3]`: `124,940,502`
      - both are about `1.25%` of the norm-only count, with active-frontier row-signature
        histograms around `0.79M` to `0.84M` unique signatures per side
      - on the representative pair, shifts `1`, `2`, and `4` give exactly the same
        active-frontier exact count and the same active histogram sizes
  - carrying any two independent row-shift equations together already fails in the
    over-constrained direction:
    - for `(1,2)`, `(1,4)`, and `(2,4)`, the representative pair's two-shift
      coefficient buckets are injective on both sides
    - left: `1,106,079` raw `UV` pairs, `1,106,079` coefficient buckets, max bucket mass `1`,
      raw `W` materialization about `1.165e9`
    - right: `1,166,391` raw `UV` pairs, `1,166,391` coefficient buckets, max bucket mass `1`,
      raw `W` materialization about `1.249e9`
    - the heaviest pair `[-15, 3, 13] | [-5, 3, 3]` shows the same singleton-bucket shape,
      with raw `W` materialization about `1.156e9` and `1.259e9`
    - sampled two-shift buckets have local compression `1.0000`
  - carrying all three independent row-shift equations together fails in the opposite direction:
    - on the representative pair `[-15, 5, 11] | [-5, -1, 7]`, all-shift coefficient buckets
      are completely injective on both sides
    - left: `1,106,079` raw `UV` pairs, `1,106,079` coefficient buckets, max bucket mass `1`
    - right: `1,166,391` raw `UV` pairs, `1,166,391` coefficient buckets, max bucket mass `1`
    - sampled all-shift buckets have only one `UV` state each and no local `W` completion collapse
    - so adding all shifts at once destroys the downstream `W` aggregation instead of making it stronger
  - so one exact row shift is still too expensive in the naive `W`-batched form:
    the current factorization has found the right region of the state space, but not yet a
    cheap enough exact join inside it

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
- the top half of that symmetry-reduced mass is carried by pair classes built around only `11` bundle orbits, with three clear hub orbits:
  - `[-15, 5, 11]`
  - `[-13, -1, 15]`
  - `[-9, -5, 15]`
- inside that half-mass core, the repeated pair structure is already small:
  - three hub-centered two-spoke families
  - plus the separate `[-15, 3, 13] | [-5, 3, 3]` family
- more strongly, that half-mass orbit graph decomposes exactly into connected-component sizes `3, 3, 3, 2`
- and the three `3`-node components are essentially equal in mass, so any one of them is a defensible first prototype target
- but the first representative component probe now rules out the easiest hoped-for win:
  - under the current `UV -> W` model, the two spoke families of a `3`-node hub component do not share
  `UV` states, coefficient-only states, or even coarse `W`-frontier states
- and the same zero-overlap pattern already reproduces on a second top-mass hub component
- the new single-bundle probe then shows that the lack of `UV` collisions is broader than the top orbit:
  - the heaviest bundle and two representative hub bundles are all injective at the `UV` layer across every cyclic `2+1` split
  - so pre-`W` `UV` deduplication is not the right compression target here
- however, those same tested bundles collapse to only about `4.1k` coarse `W`-frontier states, a stable `~250x` to `275x` reduction with bucket multiplicities near `10^3`
- the follow-up pair probe shows why that is not sufficient by itself:
  - even a one-shift exact join still wants on the order of `10^9` reduced-state-by-`W` combinations per side on representative and heaviest pairs
- coefficient-profile caching and sampled local bucket convolution only recover small constant factors, not the missing algorithmic reduction
- exact norm compatibility between left/right frontier buckets is a better staging filter, cutting bucket pairs by about `220x`, but it still leaves hundreds of millions of active side combinations and the shift marginal adds no pruning
- sampled survivor bucket pairs have extremely sparse exact joins, so an indexed active-bucket join is now a more plausible next prototype than either full materialization or more marginal filters
- more strongly, the active-frontier exact join now completes for one shift as an offline analyzer, so the next question is not "can this be done at all?" but "can the active-bucket exact path be indexed and accelerated enough to matter in search?"
- carrying two or all row shifts simultaneously is not the answer either, because it makes the `UV` side injective at the coefficient-bucket level
- but that exact lift needs an optimized orbit-level DP or signature join, not a naive distribution build inside the main analysis command
- the `UV -> W` transition signature counts suggest that such an optimized join may actually be viable, because they are around `10^6` rather than `10^18`
- but the next compression is unlikely to come from a trivial coefficient-only collapse of the `UV` states
- and the present `UV -> W` split is probably not the right place to expect early-state compression, because all tested bundles show no `UV` collisions at all before the final `W` stage

So the next real refinement has to come from either a more structural exact invariant or a more aggressively cached / factorized way to exploit that downstream `W`-frontier batching, not another norm-only filter.

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

The current `hadamard analyze lp333-multiplier` output is now deliberately conditional:

- multiplying indices by a unit is always an equivalence action on the LP equations
- invariance under a subgroup is an additional search hypothesis, not an unconditional consequence
- therefore the analyzer reports stabilizer-hypothesis consequences separately from group-action facts

The first row-bundle compatibility screen gives a useful calibration:

- the full column-preserving subgroup, whose row action is all of `U(9)`, would force too much row-bundle symmetry
- under that full stabilizer hypothesis only `18` row-bundle triples are allowed, and `0` ordered bundled exact pairs survive
- row-preserving subgroups have trivial row action and therefore do not reduce the `504` ordered bundled exact pairs at this level
- the useful middle case is a nontrivial row action generated by `row_units={1,4,7}`
- that row action has row orbits `0 | 1,4,7 | 2,5,8 | 3 | 6`
- it leaves `1064` allowed row-bundle triples and exactly `12` ordered bundled exact pairs, with norm-refined mass `119,903,105,952`
- representative cyclic subgroups with this row action include `{1,112,223}`, `{1,121,322}`, and `{1,211,232}`
- the top oriented surviving bundle samples include `[-5,-9,15] | [-5,-3,9]` and `[-5,-9,15] | [-5,9,-3]`
- lifting those `12` bundled row-pair cases to invariant length-`9` row-sum marginals gives:
  - `7` active bundle triples
  - `7,467` active row marginals
  - `13,764,060` row-marginal pair candidates
  - `6,048` norm-compatible row-marginal pairs
  - all `6,048` satisfy the remaining pure-`Z_9` row autocorrelation equations
- in the column-trivial representative `{1,112,223}`, the resulting exact row-marginal pairs still correspond to roughly `10^103.315` invariant row-pattern choices before mixed CRT constraints
- however, that same column-trivial representative is now rejected by actual shift-`(alpha,0)` row-dot feasibility:
  - if the rows in each multiplier orbit are identical length-`37` rows, then the actual CRT shift `(alpha,0)` equations impose row-dot constraints beyond row sums
  - `0` of the `6,048` exact row-marginal pairs pass those row-dot marginal constraints
  - this uses the actual nonzero shift target `-2`, distinct from the row-compressed aggregate target `-74`
- for the non-column-trivial order-`3` cases with `col_units={1,10,26}`, rows `0`, `3`, and `6` must be fixed by column multiplication by `10`
- that fixed-row condition means each such row is a choice on column `0` plus `12` length-`3` nonzero-column orbits
- a first representability check for those fixed rows keeps `1,296` of the `6,048` exact row-marginal pairs, with aggregate pattern mass about `10^60.818`
- an exact column-`10` orbit calculation for actual shift `(3,0)` then models the fixed-row pair dots and the self-dot terms from the row orbits `1,4,7` and `2,5,8`
- that `(3,0)` marginal does not prune further: all `1,296` fixed-row-compatible row marginals remain feasible at this level
- the next shift orbit to test is `(0,1)`; an exact frontier-DP scaffold exists behind `hadamard analyze lp333-multiplier --col10-shift1`, but it is currently opt-in because it is too slow for the default analyzer
- a direct opt-in run was attempted, but it did not complete quickly enough to be a recorded result

So if the project tests a multiplier-invariant LP(333) subfamily, the current best target is not the full column-preserving subgroup and not the column-trivial representative `{1,112,223}`. It is the same smaller order-`3` style row action `row_units={1,4,7}`, but with a nontrivial column action such as `{1,121,322}` or `{1,211,232}`.

That result moves the multiplier line past the pure row-marginal stage and kills the simplest implementation of the row action. The fixed-row check narrows the non-column-trivial branch, but the `(3,0)` row-dot marginal is too weak to narrow it further. The next exact sieve has to account for column permutation inside the nonfixed row orbits and mixed CRT character constraints, because row-sum constraints alone no longer distinguish the remaining non-column-trivial subgroups.
