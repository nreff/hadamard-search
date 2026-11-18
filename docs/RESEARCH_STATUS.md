# Research Status

This document is the detailed "where are we really?" memo for the current search program.

It is intentionally more candid than the README. It separates:

- what is verified now
- what seems promising but is still experimental
- what feels novel
- what would and would not yet be defensible in a paper or report

If this progresses toward a formal write-up, the paper-style skeleton now lives in [METHODS_NOTE_OUTLINE.md](/home/nate/projects/hadamard/docs/METHODS_NOTE_OUTLINE.md).

## Verification Stamp

The [November 2026 frontier update](research/hm-2396-frontier.md) records the
verified witnesses at `2060` and `2092`, the public frontier at `2396`, and the
first exact multiplier-family screens for the new target. The audit below
records the order-`668` artifact and earlier methods work.

Last re-checked on `2026-08-12`.

Verified during this audit:

- [`fixtures/known/hm-668/matrix.txt`](/home/nate/projects/hadamard/fixtures/known/hm-668/matrix.txt)
  is a `668 x 668` `{+1,-1}` matrix with SHA-256
  `bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0`
- it was reconstructed from Levent Alpöge's primary X
  [`+/-` payload](https://x.com/__alpoge__/status/2087504785952182273) and
  [`decoder/generator`](https://x.com/__alpoge__/status/2087504788938510427),
  without executing the posted wrapper as opaque code
- the first `664` signs of the extracted primary sign stream are four length-`166`
  sequences whose negative supports form the cyclic type-`H_4^*` difference family
  `(166;82,83,83,83;164)`; their row sums are `[2,0,0,0]` and their combined PAF is
  `-4` at every nonzero shift
- the native Wallis–Whiteman builder reconstructs the stored matrix byte for
  byte from those four sequences
- an independent standard-library Python implementation reconstructs the same bytes
  from the compact seed without invoking the public shell generator or Rust builder
- the initial audit used a scalar Python integer-dot checker; the current Python and
  Rust paths independently implement exact bit-packed Hamming checks and each checks
  all `222,778` unordered row pairs. A separate audit also checked all `222,778`
  unordered column pairs
- [Epoch AI's primary-ish problem-status page reports order `668` as
  solved](https://epoch.ai/frontiermath/open-problems/hadamard). Its same-day
  classification and attribution are explicitly provisional and may change as the
  authors publish details; the repository's finite artifact verification is exact
- `cargo test --workspace` passes across the workspace
- `uv run python py/test_search_modular_golay.py` passes all `15` published-seed,
  quotient-table, exact-model, and OPB-shape audits
- the Goethals-Seidel constructor builds and verifies a small `Z_5 -> HM(20)` instance
- the exact fixed-`q` lift of the published 64-modular length-`167` seed remains
  unresolved; the exact modulo-`4` decomposition has `42` raw cases, of which an
  exact lag-parity lemma excludes `20`, and bounded Z3 and RoundingSat portfolios
  decided none of the remaining cases
- each fixed-`q` quotient case now exports as restricted OPB with `6,806` variables,
  `26,663` constraints, a hash manifest, and an independent external-model verifier;
  lag parity and reversal of `V` reduce the proof campaign from `42` raw cases to
  `14` representatives without losing coverage
- a separately labeled normalized-symmetry construction probe reduces the `T`-fixed
  subfamily to `1,642` variables and `6,288` constraints per shard; SAT would solve
  the original problem, while UNSAT applies only to that subfamily
- the current reduced-length-`11` direct joint benchmark with the newer seam-aware tail join reproduces exactly:
  - command:
    - `cargo run -p hadamard-cli -- benchmark compressed-pairs --length 33 --compression 3 --ordering natural --spectral-frequencies 1 --tail-depth 11 --max-pairs 1`
  - output summary:
    - `branches_considered=0`
    - `tail_spectral_pruned=5789`
    - `tail_residual_pruned=2609`
    - `tail_candidates_checked=8399`
    - `pairs_emitted=1`
    - `elapsed_seconds=9.77`

Not yet resolved during this audit:

- the current best reduced-length-`15` anchor is measured rather than merely capped
  - `cargo run -p hadamard-cli -- benchmark compressed-pairs --length 45 --compression 3 --ordering natural --spectral-frequencies 1 --tail-depth 12 --max-pairs 1`
  has the current packed-join best measurement of `12.60` seconds
  - output summary:
    - `branches_considered=48`
    - `tail_candidates_checked=129335`
    - `tail_spectral_pruned=71722`
    - `tail_residual_pruned=57612`
    - `pairs_emitted=1`
  - reduced length `15` is therefore no longer an inaccessible anchor, but it remains the next serious scaling barrier because the cost is dominated by tail-candidate multiplicity

Fixed-`q` campaign calibration on this audit:

- the complete modulo-`4` selector processed `108,654` Z3 SAT conflicts in five
  seconds, versus `50,287` without the quotient table; both results were `unknown`
- `42` selected Z3 shards at five seconds each were all `unknown`
- the same `42` restricted-OPB shards under RoundingSat 2 at five seconds each were
  all `TIMELIMIT`
- a tuned `26 x 30s` RoundingSat campaign using `--bits-learned=1` also left every
  reversal representative at `TIMELIMIT` after `5,503,589` aggregate conflicts
- after the parity sieve, the reduced `14 x 30s` campaign also left every surviving
  representative at `TIMELIMIT` after `2,657,005` aggregate conflicts
- the reduced `T`-fixed construction subfamily likewise produced `14 TIMELIMIT`
  results at 30 seconds under RoundingSat (`3,386,209` conflicts) and `14 unknown`
  results at ten seconds under Z3
- RoundingSat proof logging is wired to pinned VeriPB `3.0.2`: a real timeout
  transcript verifies as `NO CONCLUSION`, while a learned-clause UNSAT smoke test
  verifies as `UNSATISFIABLE`
- no timeout is counted as evidence for SAT or UNSAT

## Executive State

The order-`668` existence target is solved, and the repository now contains an
independently verified witness. This is a finite, exact claim about the checked
artifact; it is not a claim that this repository discovered the construction, that
the current public account is final, or that the general Hadamard conjecture has
been resolved.

The remaining order-`668` work concerns provenance and structural analysis.
The array, exact seed parameters, and orthogonality proof have
now been recovered; the principal provenance gap is how this particular cyclic
`Z_166` family was discovered or searched for. The
repository remains short of a production `LP(333)` search engine, but that is now a
historical methods limitation rather than a blocker to existence at order `668`.

Within the retained search program, the strongest method result is not just a
constant-factor improvement. The experimental direct compressed-pair probe changed
regimes:

- earlier, reduced length `11` was dominated by branching
- now, with factorized exact-tail completion, the same benchmark is dominated by exact tail-candidate volume

That distinction matters. It means the code is no longer merely shaving branches off a DFS. It has started converting the search into a keyed exact-completion problem.

That remains an important historical technical change, but it is no longer the
project's top-level result or immediate priority.

## Verified Accomplishments

### Exact order-`668` artifact

The repository fixture has the following exact identity:

- path: `fixtures/known/hm-668/matrix.txt`
- dimensions: `668 x 668`
- alphabet: exactly `+1` and `-1`
- SHA-256:
  `bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0`
- orthogonality: all `222,778 = 668 * 667 / 2` unordered off-diagonal row
  pairs checked exactly by each of two independent verifiers
- result: `H H^T = 668 I`

The native verifier reports the exact identifying labels:

```bash
cargo run -p hadamard-cli -- verify matrix \
  --input fixtures/known/hm-668/matrix.txt
```

```text
rows=668
columns=668
is_hadamard=true
off_diagonal_pairs_checked=222778
```

The structural reconstruction is:

```bash
cargo run -p hadamard-cli -- build spence \
  --sequences-in fixtures/known/hm-668/spence-sequences.txt \
  --output /tmp/hm668-spence.txt
cmp /tmp/hm668-spence.txt fixtures/known/hm-668/matrix.txt
```

The `cmp` is empty and the reconstructed file has the same matrix SHA-256 above.
The independent mathematical reconstruction gives the same result:

```bash
uv run python py/reconstruct_hm668.py \
  fixtures/known/hm-668/spence-sequences.txt /tmp/hm668-python.txt
cmp /tmp/hm668-python.txt fixtures/known/hm-668/matrix.txt
```

The provenance starts from the public X
[`+/-` payload](https://x.com/__alpoge__/status/2087504785952182273) and
[`decoder/generator`](https://x.com/__alpoge__/status/2087504788938510427).
The posted wrapper was decoded as data and its generated shell instructions were
audited; only the constructor invocation needed for order `668` was run. The full
chain and source-text hashes are recorded in the fixture
[README](/home/nate/projects/hadamard/fixtures/known/hm-668/README.md).

The structural explanation is now exact. The first `664` signs split into four
length-`166` sequences with row sums `[2,0,0,0]`; their negative supports form a
cyclic type-`H_4^*` family `(166;82,83,83,83;164)`, equivalently their positive
supports form `(166;84,83,83,83;166)`. Their combined nonzero-shift PAF is `-4`, so
for the corresponding circulants

```text
sum_i C_i C_i^T = 668 I_166 - 4 J_166.
```

Substitution into the standard Wallis–Whiteman bordered array, including the
four border rows and columns, cancels the `-4J` term and proves
`H H^T = 668 I_668`. The native `build_spence_hadamard` implementation reproduces
the stored fixture byte for byte. See
[hm-668-construction.md](/home/nate/projects/hadamard/docs/research/hm-668-construction.md)
for the exact formula and proof.

This identifies the difference-family type and generic bordered array. It does not
justify attributing the particular seed to either of Spence's
relative-difference-set families; its actual discovery/search provenance remains
unknown. A single order also cannot establish the general Hadamard conjecture.
Epoch currently reports the problem as solved while labeling its same-day
attribution provisional; these public-history caveats do not turn the exact
reconstruction or either verifier into a provisional computation.

### Correctness ladder

The exact `LP -> 2cc -> Hadamard` path is validated on:

- length `5` -> order `12`
- length `7` -> order `16`
- length `9` -> order `20`
- length `11` -> order `24`
- length `13` -> order `28`

This matters because every experimental pruning change is being judged against known-valid smaller instances, not only against runtime.

### Published modular seed exact lift

The closest explicit order-`668` near-construction now has a solver-neutral exact
campaign path:

- the fixed published `q` reduces to `164` signs and `81` correlation equalities
- reducing its norm identity modulo `x^4-1` leaves exactly `42` residue-sum cases
- exact mod-`2` sums of the lag-disagreement equations exclude `20` cases by forcing
  the `U` residue-`1` and `V` residue-`0` plus counts to be odd
- the surviving `22` cases are closed under independent reversal of the length-`81`
  sequence and form `14` orbits: six fixed cases and eight two-case orbits
- every raw case can export to restricted OPB with shared XOR variables; the default
  v3 campaign stores `14` representatives, the `20` parity exclusions, and explicit
  reversal coverage without renumbering raw cases
- SAT models are reconstructed and checked against all `166` aperiodic shifts
- a complete UNSAT claim requires either `42` standalone checked proofs, or `14`
  checked representative proofs plus the exact lag-parity lemma and separately
  verified OPB reversal-isomorphism lemma
- the normalized involution
  `T(U)_i=-(-1)^i U_(82-i)`, `T(V)_i=(-1)^i V_(80-i)` stabilizes every raw shard;
  fixing it leaves `82` independent signs, `40` nontrivial even-lag equations, and
  an exact `14`-shard construction subcampaign. Its manifest states explicitly that
  subfamily UNSAT is not full fixed-`q` UNSAT

This route is narrower than unrestricted SDS or LP(333), but it is unconditional
inside the published special family. Before the independently verified fixture was
available it was the shortest search path to a possible `HM(668)` witness; it now
remains valuable as historical method work and as possible context for
understanding the public construction.

### Spectral backend

The PSD layer is now a real backend abstraction with:

- `direct` reference backend
- `autocorrelation` backend
- in-tree mixed-radix `fft` backend

Agreement tests pass, and `fft` is the operational default.

### Production compressed/decompression path

The "main" compressed LP pipeline currently has:

- branch-and-bound compressed generation
- row-sum and low-frequency DFT pruning
- shared A/B generation and stat reuse
- unordered compressed-pair deduplication
- reusable bucket artifacts with explicit run metadata
- hardened artifact/checkpoint version rejection
- decompression with corrected prefix pruning
- common-dihedral exact-candidate canonicalization
- exact-signature complement pruning

Verified working benchmark:

- length `15`, factor `3`
- compressed stage:
  - `135` candidates per side
  - `55` signature buckets per side
  - `215` unique PSD-consistent compressed pairs
- decompression stage:
  - `43` exact candidates per side
  - `39` exact-signature buckets
  - `47` complementary pair checks
  - `24` exact matches recovered

This path is useful and reproducible, but it is still not the path that looks most likely to break the `333` barrier.

### Experimental direct joint compressed-pair path

The most important experimental line is the direct joint search over compressed `(A, B)` columns.

Additions that are currently kept because they are correct and measured:

- exact joint squared-norm feasibility
- endpoint-aware partial autocorrelation intervals
- selected-frequency pair-PSD bounds
- exact remaining-sum reachability
- exact tail completion
- packed tail encoding
- factorized tail completion for deeper tails
- exact tail-side spectral filtering
- exact shift-1 seam filtering inside factorized tail joins

Best verified reduced-length-`11` progression so far:

- no tail oracle, selected-frequency PSD:
  - `435296` branches
- exact tail depth `3`:
  - `60704` branches
- depth `4`:
  - `20000` branches
- depth `5`:
  - `5712` branches
- depth `6`:
  - `1360` branches
- factorized depth `7`:
  - `272` branches
- factorized depth `8`:
  - `64` branches
- factorized depth `11`:
  - `0` branches
  - historical combined-norm key: `996305` tail candidates checked, `996304` tail-side spectral prunes, `11.80s`
  - current separate per-side norm key: `124981` tail candidates checked, `122670` tail-side spectral prunes, `2310` tail residual prunes, `13.65s`
  - current shift-1 seam-aware join with `1` monitored frequency: `8399` tail candidates checked, `5789` tail spectral prunes, `2609` tail residual prunes, `9.77s`
  - `1` emitted pair

Reduced-length-`15` anchors with the same method family:

- historical combined-norm key, tail depth `11`:
  - `160` branches
  - `96096005` tail candidates checked
  - `96095826` tail-side spectral prunes
  - `178` tail residual prunes
  - `1` emitted pair
  - `158.94` seconds
- current separate per-side norm key, tail depth `11`:
  - `160` branches
  - `91241732` tail candidates checked
  - `91241643` tail-side spectral prunes
  - `88` tail residual prunes
  - `1` emitted pair
  - `158.31` seconds
- current separate per-side norm key, tail depth `12`:
  - `48` branches
  - `90668636` tail candidates checked
  - `90668033` tail-side spectral prunes
  - `602` tail residual prunes
  - `1` emitted pair
  - `151.08` seconds
- current shift-1 seam-aware join, tail depth `12`, `1` monitored frequency:
  - `48` branches
  - `129335` tail candidates checked
  - `71722` tail-side spectral prunes
  - `57612` tail residual prunes
  - `1` emitted pair
  - `14.13` seconds
- current packed shift-1 seam-bucket join, tail depth `12`, `1` monitored frequency:
  - reduced length `15`
  - search counts unchanged in practice
  - `12.60` seconds
- current shift-1 seam-aware join, tail depth `12`, `0` monitored frequencies, plus the larger-instance-only exact small-shift tail filter:
  - reduced length `17`
  - `768` branches
  - `223670` tail candidates checked
  - `223531` tail shift prunes
  - `138` tail residual prunes
  - `1` emitted pair
  - `95.87` seconds
- current packed shift-1 seam-bucket join with the same `K=0` regime:
  - reduced length `17`
  - search counts unchanged in practice
  - timed reruns now reach `85.86` seconds
- current packed shift-1 seam-bucket join plus unified tail-summary caching:
  - reduced length `17`
  - search counts unchanged in practice
  - timed reruns now reach a best measured `79.28` seconds

Interpretation:

- the latest change did not alter the search counts
- recent implementation work improved the factorized tail path by:
  - doing spectral rejection from cached segment contributions before decoding and stitching full tail candidates
  - checking the full exact compressed residual on raw assignment buffers before constructing `CompressedSequence` values
  - carrying separate `norm_a` / `norm_b` values in the exact tail keys instead of only the combined norm
- the newest step is more substantial:
  - using the exact shift-`1` seam equation as a factorized join filter in natural-order suffix tails
- the first two are implementation improvements
- the separate per-side norm key is a genuine multiplicity reduction, but currently shows a tradeoff:
  - it reduces checked tails sharply on reduced length `11`, but increases runtime there because of extra bookkeeping
  - it helps more on reduced length `15`, where the best current anchor is now tail depth `12` at `151.08s`
- the shift-`1` seam filter is the first follow-up that changes the reduced length-`15` anchor qualitatively again, bringing it down to about `14` seconds
- the newer exact small-shift tail filter is much weaker:
  - it is not worth using at reduced length `15`
  - it is only a modest win at reduced length `17`
  - it does not yet move reduced length `21` under a `300s` cap
- the newer packed shift-`1` seam-bucket layout is a cleaner implementation improvement:
  - it preserves the search counts
  - it lowers runtime in the current best exact-tail regime
  - it still does not move reduced length `21` under a `300s` cap

This is the clearest evidence that the research direction has genuinely shifted shape.

The newer CRT row-bundle line is now specific enough to stage implementation against:

- `42` dihedral-swap pair-orbit classes remain after the bundled exact and norm refinements
- the top `21` of those classes already carry half the residual mass
- those `21` classes are built from only `11` distinct bundle orbits
- within that `11`-orbit core, three bundle orbits are the current hubs:
  - `[-15,5,11]`
  - `[-13,-1,15]`
  - `[-9,-5,15]`
- the half-mass frontier is also organized by only `7` repeated symmetry-reduced pair families, so the residual CRT row problem is starting to look like a small hub-and-spoke orbit graph rather than a broad residual cloud
- more strongly, that half-mass graph decomposes exactly into connected-component sizes `3,3,3,2`, with only the three hub orbits having degree `2` and every other orbit having degree `1`
- the three `3`-node components are essentially tied in mass, so the first exact-lift prototype does not need a delicate component choice; any representative `3`-node hub component is defensible
- a new representative-component probe then closed the easiest continuation of that idea:
  - for `[-15,5,11]` with spokes `[-5,-1,7]` and `[-5,7,-1]`, the two spoke families have zero overlap under the current full `UV` signatures, zero overlap under coefficient-only signatures, and zero overlap even after collapsing to the coarse `W` frontier
  - so the current `UV -> W` model offers no component-level shared-state reuse on a representative `3`-node hub component
- the same zero-overlap pattern already reproduces on the slightly heaviest `3`-node hub component `[-9,-5,15]`, so this no longer looks like a representative-only quirk
- a newer single-bundle probe then clarified where the remaining compression actually sits:
  - on the heaviest bundle `[-15,3,13]` and representative hub bundles `[-15,5,11]` and `[-9,-5,15]`, every cyclic `2+1` split is still injective at the full `UV` level and again at the coefficient-only level
  - but the same splits collapse to only about `4.1k` coarse `W`-frontier states, a stable `~250x` to `275x` reduction from raw `UV` pairs, with maximum frontier bucket multiplicity around `930` to `1015`
- so the current `UV -> W` model does not help by deduplicating `UV` states, but it may still help if an orbit-level exact lift is staged directly on `W`-frontier keys or a sharper downstream invariant
- a newer pair-level one-shift probe then put a scale on that next step:
  - on the representative hub pair `[-15,5,11] | [-5,-1,7]`, shift `1`, `2`, and `4` all have the same summary shape:
    - left side stays fully injective at the reduced `(norm_uv, base_shift, W-coeff)` level with `1,106,079` states
    - right side only drops mildly from `1,166,391` raw `UV` pairs to `1,116,807` reduced states
    - the per-coefficient `W` completion signatures total about `4.34e6` on the left and `4.44e6` on the right
    - a naive materialization of full norm-plus-one-shift row signatures would still require about `1.132e9` left-side combinations and `1.164e9` right-side ones
  - the heaviest pair `[-15,3,13] | [-5,3,3]` is comparable:
    - left reduced states: `1,044,424`
    - right reduced states: `1,164,237`
    - materialization estimates: about `1.068e9` on the left and `1.223e9` on the right
- a follow-up cache/profile probe does not change that conclusion:
  - coefficient permutation symmetry reduces distinct `W` histogram profiles by about `5.84x` on both representative and heaviest pairs
  - the largest sampled local `W` buckets still have around `0.93M` to `1.04M` local products each
  - those local products collapse only to about `0.20M` to `0.22M` row signatures, a `~4.1x` to `4.9x` local compression
  - this is useful as an analyzer implementation cache, but not strong enough to make the full one-shift materialization cheap
- a compact frontier-join probe gives one more calibration point:
  - it compares left/right coefficient buckets using exact possible row norms and exact possible one-shift values, without materializing full row-signature histograms
  - on the representative pair, exact norm compatibility drops bucket pairs from `17,656,740` to `79,398`
  - on the heaviest pair, exact norm compatibility drops bucket pairs from `17,643,894` to `80,322`
  - shift-value marginal compatibility prunes no bucket pairs on either test, so the pruning is entirely norm-driven
  - active side materialization is still about `302M` / `291M` on the representative pair and `266M` / `326M` on the heaviest pair
  - sampled exact joins inside the heaviest surviving bucket pairs are much sparser than the local materialization counts:
    - representative top-three samples: `967,079,856,834` raw row-pair products, with `13,194`, `13,180`, and `13,180` exact norm-plus-shift joins
    - heaviest-pair top-three samples: `962,719,464,960` raw row-pair products, with `11,971`, `12,095`, and `12,095` exact joins
  - a follow-up active-frontier exact join now completes offline:
    - representative pair exact join count: `124,923,897`
    - heaviest pair exact join count: `124,940,502`
    - both are about `1.25%` of the norm-only count, a reduction factor just under `80x`
    - active-frontier row-signature histograms are around `0.79M` to `0.84M` unique signatures per side from `266M` to `326M` active raw rows
    - on the representative pair, shifts `1`, `2`, and `4` give exactly the same active-frontier exact count and the same active histogram sizes
  - so this is no longer only a staging filter: it is a viable offline exact analyzer path for one shift, though still too heavy to treat as a default search primitive
- an opt-in two-shift pair diagnostic closes the middle version of the idea:
  - carrying any of `(1,2)`, `(1,4)`, or `(2,4)` makes the representative pair's coefficient buckets injective
  - representative pair summary:
    - left: `1,106,079` raw `UV` pairs, `1,106,079` two-shift coefficient buckets, max bucket mass `1`, raw `W` materialization about `1.165e9`
    - right: `1,166,391` raw `UV` pairs, `1,166,391` two-shift coefficient buckets, max bucket mass `1`, raw `W` materialization about `1.249e9`
  - the heaviest pair `[-15,3,13] | [-5,3,3]` has the same singleton-bucket shape:
    - left: `1,110,187` coefficient buckets, raw `W` materialization about `1.156e9`
    - right: `1,164,237` coefficient buckets, raw `W` materialization about `1.259e9`
  - sampled two-shift buckets have no local compression because each bucket has one `UV` state and the tested `W` completions remain unique
- an opt-in all-shifts pair diagnostic closes the other obvious version of the idea:
  - carrying shifts `1,2,4` together makes the representative hub pair's coefficient buckets injective
  - representative pair summary:
    - left: `1,106,079` raw `UV` pairs, `1,106,079` all-shift coefficient buckets, max bucket mass `1`
    - right: `1,166,391` raw `UV` pairs, `1,166,391` all-shift coefficient buckets, max bucket mass `1`
  - sampled all-shift local buckets have no local compression because each bucket has one `UV` state and all tested `W` completions stay unique
- so two-shift and all-shift batching are worse than one-shift batching for this purpose: they carry more exact information, but destroy the only useful `W`-frontier aggregation
- so even a single exact row shift is not yet cheap enough in a naive `W`-batched implementation; the next exact-lift prototype still needs another layer of caching, convolution, or a sharper invariant inside those `W` buckets

The multiplier line now has a first careful screen rather than only a roadmap item:

- the `lp333-multiplier` analyzer now treats multiplier actions as equivalences unless an explicit stabilizer hypothesis is being tested
- this matters because subgroup invariance is an optional search assumption, not an unconditional theorem for arbitrary `LP(333)` pairs
- the full column-preserving stabilizer hypothesis is already too strong:
  - it allows only `18` row-bundle triples
  - it leaves `0` ordered bundled exact pairs after the row-bundle sieve
- row-preserving cyclic subgroups are harmless but not useful at this level:
  - they leave all `504` ordered bundled pairs alive because their row action is trivial
- among `39` cyclic subgroups with nontrivial row action, `15` survive the row-bundle sieve
- the strongest surviving nontrivial row-action family is uniform:
  - row action `row_units={1,4,7}`
  - row orbits `0 | 1,4,7 | 2,5,8 | 3 | 6`
  - `1064` allowed row-bundle triples
  - `12` ordered bundled exact pairs
  - norm-refined mass `119,903,105,952`
- a new full row-marginal lift for `row_units={1,4,7}` now pushes through the remaining pure `Z_9` row equations:
  - `7` active bundle triples
  - `7,467` active invariant row-sum marginals
  - `13,764,060` row-marginal pair candidates
  - `6,048` norm-compatible row-marginal pairs
  - all `6,048` norm-compatible pairs also satisfy the exact nonzero row autocorrelation target `-74`
  - for the column-trivial representative `{1,112,223}`, the invariant row-pattern multiplicity across those exact row marginals is about `10^103.315`
- a new actual shift-`(alpha,0)` row-dot marginal diagnostic rejects the column-trivial representative `{1,112,223}`:
  - it assumes the row action is implemented literally by identical rows on the orbits `1,4,7` and `2,5,8`
  - under that assumption, `0` of the `6,048` exact row-marginal pairs can realize the required row-dot equations for actual CRT shifts `(alpha,0)`
  - the diagnostic uses the actual nonzero shift target `-2`; the pure row-sum autocorrelation target remains the aggregated value `-74`
- for the non-column-trivial order-`3` cases with column action `col_units={1,10,26}`, a first fixed-row representability check now narrows the row-marginal set:
  - rows `0`, `3`, and `6` must be fixed by column multiplication by `10`
  - each fixed row is determined by column `0` plus `12` length-`3` nonzero-column orbits
  - `1,296` of the `6,048` exact row-marginal pairs survive this necessary check
  - the aggregate fixed-row-pattern multiplicity is about `10^60.818`
- an exact column-`10` orbit calculation for actual CRT shift `(3,0)` was added next:
  - it accounts for fixed-row pair dots and self-dot terms induced by the two nonfixed row orbits
  - it does not prune beyond fixed-row representability: all `1,296` fixed-row-compatible row-marginal pairs pass this marginal check
- the next mixed marginal, actual CRT shift `(0,1)`, now has an opt-in exact frontier-DP scaffold:
  - command: `hadamard analyze lp333-multiplier --col10-shift1`
  - the default analyzer deliberately reports this path as skipped because the current exact DP is still too slow for routine checkpointing
  - a previous fast `2^13` table result is no longer treated as recorded: it only enumerated rows fixed by column multiplication by `10`, while the nonfixed order-`3` row orbits require arbitrary base rows whose copies are column-multiplied
  - the arbitrary-row table has been restored; the corrected targeted run takes about `176s` and leaves all `1,296` fixed-row-compatible row-marginal pairs alive, so `(0,1)` adds no sound pruning at this marginal level
- the first coupled mixed-shift relaxation now exists:
  - command: `hadamard analyze lp333-multiplier --col10-coupled`
  - it couples the arbitrary nonfixed row-orbit base rows through a joint `(multiplier self-dot, additive-autocorrelation)` table, while still relaxing the fixed rows by combining their `(3,0)` and `(0,1)` possibilities independently
  - it also keeps all `1,296` fixed-row-compatible row-marginal pairs; the run built `147` cached sequence joint sets, with maximum joint-set size `2,426`
  - direct exact-fixed-row coupling was tried next, first with a naive three-fixed-row frontier and then with a shared fixed-triple target table narrowed to the exact fixed-row-compatible survivors; neither produced a full stats line in a useful debug window, so no public exact-fixed analyzer flag was kept
- the multiplier analyzer now also has an opt-in CRT component orbit view:
  - command: `hadamard analyze lp333-multiplier --crt-components`
  - it prints the multiplier orbits in `(mod 9, mod 37)` form and a focused `row_units={1,4,7}` hypothesis summary so the branch can be inspected directly in product-group coordinates
  - the focused `row_units={1,4,7}` family currently has `15` surviving hypotheses, split across subgroup orders `3,6,9,12,18,36`
  - the order-`3` survivors split into column units `[1]` and `[1,10,26]`; the order-`6` survivors split into `[1,36]` and `[1,10,11,26,27,36]`
  - the mixed-CRT next target is now the four hypotheses in `crt_component_mixed_crt_targets`, with order split `3:2,6:2`; the summary now also carries `pairs`, `mass`, `pair_samples`, and `col_orbits`, but those four targets currently tie on `pairs=12`, `mass=119903105952`, `row_orbits=0|1,4,7|2,5,8|3|6`, `allowed_bundles=1064`, and the same top pair samples; `col_orbits` only separates the order-`3` and order-`6` buckets, and the remaining distinction is the explicit `subgroup_crt` label
  - a new actual invariant-row-stabilizer lift now separates those four mixed targets: the two order-`3` targets have `113` invariant table orbits (`1:3,3:110`) and reproduce the `1,296` exact row-pair survivors with aggregate mass about `10^60.818`; the two order-`6` targets have `59` invariant table orbits (`1:3,3:2,6:54`) but their stronger row stabilizer leaves `0` row-pair candidates after the active bundle-pair join
- a bounded actual mixed-shift diagnostic now profiles shift `(3,1)` for the two
  surviving order-`3` targets:
  - command: `hadamard analyze lp333-multiplier --invariant-shift31`
  - it is a graph/target-size diagnostic, not a full exact feasibility enumerator
  - both order-`3` targets still have `1,296` exact row-pair survivors
  - the survivor set induces only `42` distinct fixed-row triple targets and `6`
    distinct nonfixed row-sum targets
  - the fixed-row shift graph is modest:
    `vars=13,domain=8,edges=32,weighted_edges=35,self_terms=2,max_degree=6,min_fill_width=5`
  - the fixed-row min-fill profile adds only `7` fill edges, has bag-size
    distribution `1:1,2:1,3:2,4:3,5:5,6:1`, and a max bag domain-state
    bound of `262,144`
  - because the current DP is a linear frontier rather than a tree-decomposition
    DP, the analyzer now also reports a frontier order; for the fixed graph this
    order is exact and has frontier width `6` (`8^6 = 262,144` states)
  - the nonfixed row-orbit shift `(3,1)` graph is the hard side:
    `vars=37,domain=2,edges=105,weighted_edges=108,self_terms=3,max_degree=6`,
    with min-fill width `10` for column generator `10` and `9` for column
    generator `26`
  - the nonfixed min-fill profiles have max bag domain-state bounds `2,048`
    and `1,024`, respectively, and the analyzer now prints the concrete
    elimination orders needed for a bag-DP implementation
  - the frontier-order heuristic gives frontier width `11` for both nonfixed
    graphs, so the actual compressed-profile frontier is bounded by `2,048`
    assignments for both column generators
- an opt-in exact value-table scaffold now exists for that shift:
  - command: `hadamard analyze lp333-multiplier --invariant-shift31-exact`
  - it uses the frontier orders and target row sums to build exact fixed-row and
    nonfixed row-orbit value sets for shift `(3,1)`
  - this is deliberately skipped by default; the first integrated attempt was too
    heavy for the routine checkpoint path, so the normal graph diagnostic reports
    `value_profile=skipped`, `compressed_profile=skipped`, and
    `exact_shift31=skipped`
  - no exact shift-`(3,1)` pruning count is recorded yet
- a bounded profile mode now makes that slow exact scaffold observable without
  claiming a pruning result:
  - command: `hadamard analyze lp333-multiplier --invariant-shift31-profile`
  - the fixed-row side now uses exact min-fill suffix target reachability instead
    of the older loose per-row remaining-column screen
  - profile summaries now report active-assignment capacity and fill rate
    (`fill_ppm = assignments / capacity * 1,000,000`) alongside the capped value
    or signature counts
  - with the current `100,000` value-state cap, the fixed-row profile stops after
    `7` processed variables at active width `4`, with about `2.8k` active
    assignments and max `93` / `95` values per assignment
  - the nonfixed raw profile reaches the width-`11` frontier and still caps at
    `100,000` values: generator `10` stops after `20` variables with all `2,048`
    assignments active, while generator `26` stops after `22` variables with
    `1,531` assignments active
- a compressed profile mode now groups each bag assignment by row-sum signature
  and stores its energy alphabet as a bounded bitset:
  - command: `hadamard analyze lp333-multiplier --invariant-shift31-compressed-profile`
  - the cap can be changed with `--invariant-shift31-profile-limit N`
  - with a `100,000` row-signature cap and the same exact suffix reachability, the
    fixed-row profile stops after `7` processed variables at active width `4`;
    the fixed side now has many signatures per active assignment rather than one
    signature per assignment
  - the nonfixed compressed profile now completes exactly at the default cap:
    generator `10` has `31,020` signatures and `267,044` energy bits, while
    generator `26` has `26,598` signatures and `162,860` energy bits
  - in release mode, cap `30,000,000` now completes the fixed side exactly:
    `steps=13`, peak `262,144 / 262,144` assignments, `20,849,958` row
    signatures, and `58,360,095` energy bits; the final fixed table has `42`
    row-sum signatures and `1,068` energy bits
- the exact shift-`(3,1)` value lift now completes in release mode:
  - command: `hadamard analyze lp333-multiplier --invariant-shift31-exact`
  - fixed targets: `42 / 42`, max fixed energy values `28`
  - nonfixed targets: `6 / 6`, max nonfixed energy values `29`
  - row-value cache entries: `147`, max row energy values `84`
  - feasible row pairs: `1,296`
  - feasible pattern mass: `log10 ~= 60.818`
  - conclusion: exact single shift `(3,1)` is non-pruning for both surviving
    non-column-trivial order-`3` targets
- a coupled mixed-shift graph sweep now exists before committing to a heavier exact
  coupled DP:
  - command: `hadamard analyze lp333-multiplier --invariant-mixed-shift-pairs`
  - it scans the `12` nonzero column-shift orbits under `{1,10,26}` and ranks the
    `66` two-shift union graphs for each surviving order-`3` mixed target
  - the `66` labeled pairs reduce to `6` multiplier-ratio classes; `(1,11)` is one
    representative of the cheapest observed class for both column generators `10`
    and `26`, not a unique or proved-strongest pair
  - the fixed-row union graph remains at frontier width `6`
    (`8^6 = 262,144` states), while the nonfixed union graph rises to frontier
    width `18` (`2^18 = 262,144` states)
  - although the frontier widths look moderate, the two-energy value state is still
    expected to be memory-heavy; this ranking is a label-sensitive heuristic, not
    evidence that a coupled in-process DP is the best next implementation
- the planned invariant PB campaign now has a reproducible geometry audit:
  - the `1,296` row-marginal pairs form `19` safe symmetry orbits per surviving
    subgroup (`2 x 36` and `17 x 72`), hence `38` planned shards
  - each shard has `226` base signs, `12,656` shared XOR auxiliaries, `56`
    independent shift equations, and `12,882` variables total
  - this geometry is tested, but the exporter and proof campaign are not yet built
- representative order-`3` row-action subgroups that remain worth testing are therefore the non-column-trivial cases such as `{1,121,322}` and `{1,211,232}`
- representative oriented bundle samples include `[-5,-9,15] | [-5,-3,9]` and `[-5,-9,15] | [-5,9,-3]`
- this gives a concrete optional multiplier-invariant row-action family to test next, while also ruling out the tempting full column-preserving assumption, the cleanest column-trivial order-`3` representative, and the nontrivial order-`6` mixed targets
- the next hard question is whether the two non-column-trivial order-`3` `row_units={1,4,7}` subgroups survive an actual invariant sign-table assignment; corrected `(3,0)`, `(0,1)`, exact `(3,1)`, the first relaxed two-shift coupling, and the graph ranking do not decide that, so the next implementation should stream the `38` weighted-PB shards rather than materialize the two-energy coupled frontier

So the next exact-lift prototype does not need to start as a generic residual-bundle
solver. The better unrestricted framing remains an orbit-level lift staged on the
`11`-orbit core and batched at the downstream `W` frontier. For the conditional
multiplier branch, the immediate implementation should instead export the tested
`38`-shard weighted-PB geometry and reserve the `(1,11)` coupled frontier for a
bounded propagator comparison.

## What Failed, And Why That Matters

The repo now has enough experimentation behind it that negative results are informative, not just discarded.

Rejected or deprioritized lines include:

- independent compressed dihedral canonicalization
- partial pair-space canonicalization from prefixes
- full nonzero-frequency prefix checks in the single-sequence generator
- count-profile-first compressed generation
- contiguous MITM as a production path
- parity MITM as a production path
- exact even-shift parity signatures on odd cyclic lengths
- generator-`2` assignment ordering as the main lever

This matters for publishability because it shows a search program, not a single lucky trick. The repo is accumulating both:

- successful constraints
- reasons certain plausible ideas fail

That is often the difference between "interesting code" and a defensible computational methods story.

For the counted experiment registry, see
[docs/EXPERIMENT_INDEX.md](/home/nate/projects/hadamard/docs/EXPERIMENT_INDEX.md).
For the detailed itemized history, see
[docs/EXPERIMENT_LOG.md](/home/nate/projects/hadamard/docs/EXPERIMENT_LOG.md).

## Current Intuition

My current overall intuition is that the finite existence question no longer needs
more search. The highest leverage is to make the source-to-fixture reconstruction
durable, recover the mathematical ideas hidden in the generator, and characterize
the verified matrix well enough that others can compare and reproduce it.

For the retained historical search program, my current intuition is:

1. The main bottleneck is no longer global branching at the reduced-length-`11` benchmark.
2. The main bottleneck is now candidate multiplicity inside exact tail keys.
3. That suggests the right abstraction is becoming "exact completion under strong keyed invariants" rather than "deeper DFS with more inequalities."

More concretely:

- the direct joint path is now strongest when it commits early work to a front prefix and pushes the rest into an exact keyed tail oracle
- once that happens, the useful next ideas are not generic branch-order tweaks
- they are better exact tail keys, better factorization, or cheap exact tail-side filters before full residual evaluation
- the strongest new algebraic object is the CRT-derived row-bundle marginal:
  - `504` exact bundled row-pair solutions
  - norm-refined lifted upper bound `5,035,801,219,344`
  - `90` active bundled states
  - `30` active bundle orbits
  - `168` active bundled pair orbits
  - `42` active bundled pair orbits after quotienting by swap plus common dihedral symmetry
- the heaviest surviving bundled pair orbit still has raw lift space about `1.4545e18`
- but its natural `UV -> W` transition state for row shifts `1,2,4` is only about `1.11e6` / `1.16e6` signatures per side
- that is the current reason the CRT line is still credible:
  - naive exact lifting is dead
  - orbit-level exact lifting may still be viable

I would summarize that search-specific mathematical-computational picture like this:

- branch pruning got us into the right region
- exact completion is what is now changing the regime

That is still the strongest signal from the repository's search experiments. The
strongest project-level result is now the exact independently verified order-`668`
artifact.

## Novelty Assessment

The careful answer is: parts of this now look plausibly novel as computational method work, but not yet as a theorem.

### What is probably not novel by itself

These are standard or at least unsurprising ingredients in the literature or in search engineering:

- Legendre-pair search as a route to `HM(668)`
- compression
- PSD filtering
- meet-in-the-middle as a general tactic
- using known smaller cases as correctness gates

### What may be novel in this repo-specific combination

These are the parts that now look plausibly original enough to be worth documenting as method contributions:

- direct joint compressed `(A, B)` search rather than only "generate singles then pair"
- exact joint squared-norm pruning derived from the compressed LP identities inside that joint search
- endpoint-aware partial autocorrelation interval bounds in the joint compressed space
- exact-tail completion keyed by remaining sums and combined norm
- factorized deeper-tail completion that turns the reduced-length-`11` benchmark into a tail-candidate problem rather than a branching problem
- explicit experimental comparison against failed symmetry and MITM variants, with known-case preservation as the acceptance criterion

### What would be defensible to claim now

Reasonable present-tense claims:

- the repo contains an exact independently verified order-`668` artifact reconstructed
  from the cited primary payload and decoder
- the repo contains an original experimental search pipeline developed around the
  formerly open `HM(668)` Legendre-pair route
- several joint-space pruning and exact-tail ideas were implemented, benchmarked, and compared against alternatives
- the best current direct-joint benchmark on reduced length `11` changed regime from branch-dominated to exact-tail-candidate-dominated
- multiple intuitive alternatives were tested and rejected for precise reasons

### What would not yet be defensible to claim

Not yet justified:

- "this repository discovered the order-`668` construction"
- "the public construction or attribution account is final"
- "the general Hadamard conjecture is solved"
- "we have a publishable breakthrough"
- "this is definitely a new theorem"
- "this will scale to `LP(333)`"
- "negative computational results here would imply nonexistence"

The gap is still scaling evidence. Reduced length `11` is strong and reduced length
`15` is measurable, but the current seam-aware anchor still checks `129,335` exact
tail candidates for the first hit and reduced length `21` remains beyond the
five-minute cap, so we should not overstate the reach of the current method.

## What Would Make This Publishable

The lowest realistic publication bar is not rediscovery of the now-known
`HM(668)` witness. It is either a computational-methods result with a clean
experimental story or a rigorous reconstruction and structural account of the new
construction.

That would likely need:

- a stable and clearly described algorithmic core
- a measured comparison against credible alternatives
- a strong correctness ladder
- honest negative results
- evidence that the new method changes the practical frontier on a nontrivial family of reduced instances

In other words, plausible near-term publishable objects are:

- "a joint compressed search and exact-tail completion method for Legendre-pair search,"
- "an independently reproducible reconstruction and structural analysis of the new
  order-`668` construction,"

but not:

- "this repository's discovery of order `668`," or
- "a resolution of the general Hadamard conjecture."

The project is close enough to that methods territory that the documentation now needs to be publication-aware, which is why this file exists.

## Current Bottlenecks

The bottlenecks now split by path.

Artifact and construction line:

- the generic Wallis–Whiteman bordered array, its `H_4^*` input identity, and
  its orthogonality proof are now documented and implemented
- the discovery/search provenance of this particular cyclic
  `(166;82,83,83,83;164)` family is still unknown
- structural invariants beyond the proved block/circulant description, including
  automorphisms, ranks, and equivalence fingerprints, have not yet been
  systematically documented

Production compressed pipeline:

- single-side candidate generation still explodes too early for `333`

Experimental direct-joint path:

- exact tail candidate volume per key is now the dominant cost on the best reduced-length-`11` configuration
- the reduced-length-`15` anchor confirms the same regime more strongly: only `48`
  branches and `129,335` tail candidates after the exact shift-`1` seam filter

MITM line:

- naive splits preserve too little of the correlation structure and push too much work into the join

## Immediate Next Research Questions

The best next questions are:

1. How was the particular cyclic type-`H_4^*` family on `Z_166` discovered or
   searched for, and what reproducible search provenance can the authors provide?
2. Does the seed have additional multipliers, automorphisms, or orbit structure that
   explains how it was found?
3. What normalization, rank, and equivalence fingerprints best characterize the
   verified fixture and connect it to other Wallis–Whiteman constructions?
4. What additional independent distribution or reconstruction record would
   materially improve long-term provenance beyond the native and standard-library
   Python builders already reproducing the exact fixture hash?
5. Which parts of the LP(333), fixed-`q`, CRT, and SDS programs remain worth
   developing as general computational methods or as clues to the new construction?

Questions that currently look less attractive:

- long blind search runs whose only purpose is another existence witness at order
  `668`
- another DFS ordering experiment
- another naive MITM split
- another cache layer or symmetric handling tweak on the current `6+6` join
- broadening spectral checks without evidence they improve the reduced-length-`11` frontier

Lower-priority speculative direction:

- learn statistical regularities from known smaller Legendre pairs only after the exact algebraic lines above are better understood

Focused working note:

- [docs/research/crt-multiplier-roadmap.md](/home/nate/projects/hadamard/docs/research/crt-multiplier-roadmap.md)

## Long-Run Policy

Before starting long jobs, estimate first.

Required pre-launch evidence:

- the exact variant has a measured reduced benchmark
- branch counts or tail-candidate counts are recorded
- rough memory footprint is estimated for any MITM-style job

This is now especially important because the search can fail in different ways:

- branch explosion
- state-memory explosion
- tail-candidate explosion

Those are different failure modes and need different responses.

## Bottom Line

Order `668` is solved, and this repository has exactly verified the reconstructed
artifact. The public account is same-day and may evolve, but the matrix dimensions,
alphabet, digest, and `222,778` checked off-diagonal orthogonality conditions are not
provisional.

The project is now in a different phase:

- correctness infrastructure is strong
- primary provenance and an exact target fixture are recorded
- the exact cyclic `H_4^*` family, bordered array, and orthogonality proof are
  documented and reproduced byte for byte
- the particular seed's discovery provenance and deeper structural classification
  remain open
- multiple negative and positive experiments are documented
- the direct joint search line has changed computational regime on a nontrivial benchmark
- the historical search work remains useful, but provenance, reconstruction, and
  structural analysis now have priority

None of this resolves the general Hadamard conjecture.
