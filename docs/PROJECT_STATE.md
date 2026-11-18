# Project State

This is the best starting point for a new reader.

It answers:

- What is the project trying to do?
- What plan are we following?
- What is already complete?
- What is in progress?
- What is next?

## Mission

Current goal:
- prioritize constructing `H(4412)` in the order-19 multiplier family over
  `Z_1103`. Track `2396 = 4*599` as the smallest open order
- preserve and independently validate the now-known Hadamard matrix of order `668`
- preserve and analyze its now-identified cyclic `H_4^*` / bordered construction
- determine how its particular `Z_166` seed was discovered or searched for
- retain the repository's search work as reproducible computational-methods research

Target status:
- the public `2026-08-12` payload fills the twelve former gaps below `2000`
- Schneider's `H(2060)` and Cooley's `H(2092)` passed exact local matrix checks
- Egan et al. and Kumar's later construction register both identify `2396` as
  the smallest open order. See the [frontier note](research/hm-2396-frontier.md)
- existence at order `668` is solved
- the repository fixture is exact and independently verified
- the generic Wallis–Whiteman array and the exact cyclic seed parameters are
  explained and reproduced in-tree
- the public attribution account and discovery provenance of the particular seed
  remain provisional or unknown
- the general Hadamard conjecture remains open

Historical primary mathematical route:
- search for a **Legendre pair of length `333`**
- use the **two-circulant-core (2cc)** construction to build `HM(668)`

Historical secondary route:
- search for structured **SDS / Goethals-Seidel** constructions over `Z_167`

Why this repo exists:
- it began because direct search at order `668` was infeasible and structured search
  might have become feasible with strong pruning, compression, and reproducible
  artifacts
- it now also preserves and interrogates an exact witness from the public result
- the engineering pipeline and its negative results may still become useful
  computational-methods work if they materially change what is feasible elsewhere

Background documents:
- [deep-research-report.md](/home/nate/projects/hadamard/docs/research/deep-research-report.md)
- [legendre-pair.md](/home/nate/projects/hadamard/docs/algorithms/legendre-pair.md)
- [sds-167.md](/home/nate/projects/hadamard/docs/algorithms/sds-167.md)
- [modular-golay-167.md](/home/nate/projects/hadamard/docs/research/modular-golay-167.md)
- [hm-668-construction.md](/home/nate/projects/hadamard/docs/research/hm-668-construction.md)
- [hm-2396-frontier.md](research/hm-2396-frontier.md)
- [Order-4412 campaign](research/hm-4412-campaign.md)
- [Open orders and research problems](research/open-orders-and-problems.md),
  including the `4412` and `4748` alternatives and subgroup obstructions at `7572`
- [hm-2060-frontier.md](/home/nate/projects/hadamard/docs/research/hm-2060-frontier.md)
- [RESEARCH_STATUS.md](/home/nate/projects/hadamard/docs/RESEARCH_STATUS.md)
- [EXPERIMENT_INDEX.md](/home/nate/projects/hadamard/docs/EXPERIMENT_INDEX.md)

## Current Snapshot

Current state:
- at `4412`, the first joint search reached class defect `1024`. Exact joins
  exhausted the saved incumbent's neighborhoods of up to four swaps total and
  two per row. The sampled joins and local repair found no witness. The same
  join verified injected and repaired controls at `2092`
- the `H(2396)` row-sum screen excludes every common multiplier subgroup of
  `Z_599` of order greater than `2` for cyclic Goethals–Seidel seeds
- for normalized bordered seeds over `Z_598`, the parity quotient excludes 12
  subgroups, the mod-11 fixed-slice screen excludes `<3>` and `<25>`, and exact
  enumeration excludes `<25,31>`. The smallest remaining subgroup in this
  screen is `<45,47>`, with 48 orbits
- the historical `H(2060)` attack was a cyclic Goethals--Seidel search over `Z_515`
  invariant under its order-`17` multiplier subgroup; this reduces the four sign
  sequences to `140` orbit-membership bits and the nonzero correlations to `17`
  independent shift-orbit equations
- the transparent exact model has `2,520` variables and `9,541` constraints; an
  exhaustively tested singleton normalization plus four safe reflection leaders
  yields one full-family order-`17` shard with `1,860` variables and `6,985`
  constraints
- a `30`-second RoundingSat calibration on that reduced shard returned only
  `TIMELIMIT` after `672,083` conflicts; a `170`-second deterministic orbit-swap
  heuristic found no witness but reached squared shift-equation defect `24`
- multiplier compatibility uniquely selects row sums `[1,3,31,33]`, negative
  support sizes `[257,256,242,241]`, fixed-point counts `[2,1,4,3]`, and
  nonfixed-orbit counts `[15,15,14,14]` for this shard
- [`fixtures/known/hm-668/matrix.txt`](/home/nate/projects/hadamard/fixtures/known/hm-668/matrix.txt)
  is a verified `668 x 668` matrix with entries in `{+1,-1}` and SHA-256
  `bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0`
- the fixture was reconstructed from the primary X
  [`+/-` payload](https://x.com/__alpoge__/status/2087504785952182273) and
  [`decoder/generator`](https://x.com/__alpoge__/status/2087504788938510427)
- the first `664` signs of the extracted primary sign stream split into four
  length-`166` sequences with row sums `[2,0,0,0]` and combined periodic
  autocorrelation `-4` at every nonzero shift;
  their negative supports form a cyclic type-`H_4^*` difference family
  `(166;82,83,83,83;164)`, equivalently the positive supports form
  `(166;84,83,83,83;166)`
- the standard Wallis–Whiteman bordered array is implemented by
  `build_spence_hadamard`; the compact seed reconstructs `matrix.txt` byte for byte
- a separate standard-library Python implementation independently reconstructs the
  same bytes without the public shell generator or Rust builder
- a direct Python row-dot verifier and an independent optimized Rust
  Hamming-distance verifier each checked all `222,778` unordered off-diagonal row
  pairs exactly and confirmed `H H^T = 668 I`
- [Epoch AI reports order `668` as solved](https://epoch.ai/frontiermath/open-problems/hadamard);
  its same-day public account may still be revised, but that provisional public
  status does not weaken the exact artifact verification recorded here
- the array theorem and orthogonality mechanism are now explained; the immediate
  provenance question is how this particular cyclic `Z_166` family was discovered
  or searched for. The artifact does not resolve the general Hadamard conjecture
- working research scaffold
- not yet a production `LP(333)` solver
- compressed search and first decompression prototype exist
- a larger length-`15`, factor-`3` compressed benchmark is now recorded
- corrected prefix pruning now lets the prototype decompressor recover exact matches from the length-`15` benchmark
- common-dihedral pair canonicalization now trims a small symmetry layer from decompression output
- sequence-level dihedral canonicalization now cuts the exact candidate frontier substantially before pair matching
- exact-signature complement pruning now cuts the recovered exact frontier sharply before pair matching
- decompression now emits canonical exact pair representatives
- decompression now applies multi-layer canonical-prefix pruning during expansion
- decompression now reuses exact candidates via complementary Legendre-signature buckets
- SDS block representation, difference-profile math, and a small meet-in-the-middle search scaffold now exist
- a checked Goethals-Seidel constructor now closes the `four complementary sequences / SDS -> Hadamard` path
- the published length-`167` 64-modular Golay quadruple now has a reproducible audit,
  an exact fixed-`q` reduction to `164` signs and `81` XOR-cardinality equations,
  an exact `42`-case modulo-`4` quotient, a lag-parity reduction to `22` feasible
  cases, a `14`-representative reversal quotient, and a restricted-OPB
  proof-campaign export; an optional exact `T`-fixed construction campaign reduces
  further to `1,642` variables and `6,288` constraints per shard but is not a
  full-family refutation path;
  all current bounded solver runs remain unresolved
- checkpoint and bucket parsing now reject unknown schema versions, and LP artifacts carry explicit run metadata
- compressed checkpoints now validate run metadata and resume without repeating consumed pairs; shard-local attempt metrics no longer confuse the global resume cursor with the number of attempts processed by one shard; compressed row-sum and spectral bounds now honor the configured factor and target
- compressed generation now has branch-and-bound row-sum and low-frequency DFT pruning, with a first length-`33` scaling baseline recorded
- compressed pair scoring now deduplicates unordered candidate pairs before residual/PSD evaluation
- an experimental pair-aware compressed-prefix probe now exists, with exact joint squared-norm pruning and endpoint-aware partial autocorrelation bounds
- an experimental contiguous-split MITM probe also exists, but its first reduced-length-`11` benchmark is not yet competitive with the direct joint probe
- a parity-split MITM variant was benchmarked and found equivalent to the contiguous split at reduced length `11`, so that line is not currently promising
- a generator-`2` assignment order for the direct joint probe was benchmarked and also matched the natural-order frontier at reduced length `11`
- selected-frequency pair-PSD pruning now improves the natural-order direct joint probe modestly at reduced length `11`
- the current pair-PSD frequency sweep suggests a small selected set is sufficient on the measured benchmark; larger full-spectrum checks do not help yet
- a small exact tail oracle now gives the strongest measured direct-joint improvement, cutting the reduced length-`11` benchmark down to `60704` branches
- packed exact-tail lookup now pushes that same reduced length-`11` benchmark down further to `1360` branches at tail depth `6`
- factorized exact-tail lookup can eliminate branching entirely on the reduced length-`11` benchmark, but then exact tail-candidate volume becomes the new bottleneck
- the current direct tail path now carries separate per-side norm keys, which cuts reduced length-`11` checked tails from about `996k` to about `125k`, though with extra overhead on that small benchmark
- the factorized tail join now also uses an exact shift-`1` seam filter in the natural-order suffix case, cutting reduced length-`11` to `8399` checked tails and the best reduced length-`15` anchor to `129335` checked tails
- the current best reduced length-`15` direct-joint anchor is now `length 45`, factor `3`, tail depth `12`, `spectral_frequencies=1`: `48` branches and `12.60` seconds to the first pair
- the next measured anchor is now reduced length `17` (`length 51`, factor `3`, tail depth `12`, `spectral_frequencies=1`): `223664` checked tails and `103.67` seconds to the first pair
- reduced length `17` is slightly faster with `spectral_frequencies=0` than with `1`
- an exact tail-side small-shift prefilter for shifts `2..4` is now enabled only for reduced lengths `>= 17`; it is a modest win at reduced length `17` (`95.87s` versus `97.27s`) but not at reduced length `15`
- a direct-indexed packed shift-`1` seam-bucket representation plus unified tail-summary caching now trims join overhead further without changing the search counts, improving the reduced length-`17`, `K=0` anchor to a best measured `79.28s`
- reduced length `21` (`length 63`, factor `3`) still exceeds a `5`-minute cap with the current best tail-depth-`12`, `spectral_frequencies=0` regime
- a dedicated `hadamard analyze lp333-crt` utility now anchors the algebraic CRT line
- the current CRT row-bundle sieve picture is:
  - `504` exact bundled row-pair solutions at the induced length-`3` level
  - norm-refined lifted upper bound `5,035,801,219,344`
  - `90` active bundled states
  - `30` active bundle orbits
  - `168` active bundled pair orbits
  - `42` active bundled pair orbits after quotienting by swap plus common dihedral symmetry
  - the top `21` dihedral-swap pair orbits already carry half the residual mass and are built from only `11` distinct bundle orbits
  - inside that `11`-orbit core, `[-15,5,11]`, `[-13,-1,15]`, and `[-9,-5,15]` are the current hub bundle orbits, each appearing twice as often as the others
  - the half-mass frontier is also organized by only `7` repeated symmetry-reduced pair families, centered on those hub orbits plus the separate `[-15,3,13] | [-5,3,3]` family
  - more sharply, that half-mass core decomposes into connected-component sizes `3,3,3,2`, so the next exact-lift prototype can be staged on tiny orbit components rather than a monolithic residual set
  - the three `3`-node components are essentially equal in mass, so any one of them is a representative first exact-lift target; the remaining `2`-node edge carries exactly half of a `3`-node component
  - a separate `lp333-crt-component` analyzer now shows that a representative `3`-node hub component has zero cross-spoke overlap under the current full `UV`, coefficient-only, and coarse `W`-frontier signatures, so shared-state reuse in the present `UV -> W` model is not the next win
  - the same zero-overlap pattern already reproduces on the slightly heaviest `3`-node hub component `[-9,-5,15]`
  - a separate `lp333-crt-bundle` analyzer now shows that the heaviest bundle `[-15,3,13]` and representative hub bundles `[-15,5,11]` and `[-9,-5,15]` are injective at the full `UV` and coefficient-only levels across all cyclic `2+1` splits, but still collapse to only about `4.1k` coarse `W`-frontier states, a stable `~250x` to `275x` reduction
  - a newer `lp333-crt-pair` analyzer now shows that even after fixing one row shift at a time, the representative hub pair `[-15,5,11] | [-5,-1,7]` still has essentially no useful pre-`W` state collapse, and a naive norm-plus-one-shift materialization would still require about `1.13e9` left-side combinations and `1.16e9` right-side ones; the heaviest pair `[-15,3,13] | [-5,3,3]` is similar at about `1.07e9` and `1.22e9`
  - coefficient-permutation caching gives a real but bounded `~5.84x` reuse factor for `W` histograms, and sampled heavy `W` buckets only collapse locally by about `4.1x` to `4.9x`, so this is not enough to rescue the naive one-shift materialization path
  - an opt-in frontier-join diagnostic now shows that exact norm compatibility between left/right coefficient buckets is a real staging filter (`~17.6M` bucket pairs down to about `79k` to `80k`), but row-shift marginal compatibility prunes nothing and active side materialization remains in the hundreds of millions; top survivor bucket-pair samples are sparse (`~9.6e11` raw row-pair products down to about `12k` to `13k` exact joins)
  - a newer `--frontier-exact-join` pass now materializes only those active frontier buckets and recovers the full exact one-shift join count offline: `124,923,897` on `[-15,5,11] | [-5,-1,7]` and `124,940,502` on `[-15,3,13] | [-5,3,3]`, both about `1.25%` of the norm-only count
  - on the representative pair, the active-frontier exact one-shift count is exactly the same for shifts `1`, `2`, and `4`, so the current one-shift analyzer is not hiding a better cyclic shift choice
  - an opt-in two-shift pair diagnostic now closes the middle case: each of `(1,2)`, `(1,4)`, and `(2,4)` makes the representative pair's coefficient buckets injective (`1,106,079` left buckets and `1,166,391` right buckets, max bucket mass `1`), and the heaviest pair shows the same singleton-bucket shape
  - an opt-in all-shifts pair diagnostic now shows the opposite failure mode: carrying shifts `1,2,4` together makes the representative pair's coefficient buckets injective (`1,106,079` left buckets and `1,166,391` right buckets, max bucket mass `1`), so the `W`-frontier batching disappears entirely
- a dedicated `lp333-multiplier` analyzer now separates unconditional multiplier equivalence from conditional stabilizer assumptions
  - the full column-preserving stabilizer hypothesis is incompatible with the row-bundle sieve (`18` allowed bundles, `0` ordered bundled pairs)
  - among `39` cyclic subgroups with nontrivial row action, `15` survive the row-bundle sieve
  - the best nontrivial row-action hypotheses all have `row_units={1,4,7}`, leave `1064` allowed bundles and `12` ordered bundled pairs, and carry norm-refined mass `119,903,105,952`
  - the first full row-marginal lift for `row_units={1,4,7}` now leaves `6,048` exact length-`9` row-sum marginal pairs after norm and pure row autocorrelation checks
  - the column-trivial order-`3` representative `{1,112,223}` is now rejected by an actual shift-`(alpha,0)` row-dot marginal feasibility check: `0` of those `6,048` row-marginal pairs can realize the required row-dot equations when rows in each orbit are identical; this uses the actual nonzero shift target `-2`, not the row-compressed aggregate target `-74`
  - for the non-column-trivial order-`3` cases with `col_units={1,10,26}`, a fixed-row column-orbit representability check now keeps `1,296` of the `6,048` exact row-marginal pairs, with aggregate fixed-row-pattern mass `log10 ~= 60.818`
  - an exact column-`10` orbit dot calculation for the actual shift `(3,0)` does not prune those survivors further: all `1,296` fixed-row-compatible row-marginal pairs remain feasible at that marginal level
  - an exact `(0,1)` column-shift frontier-DP scaffold now exists behind `hadamard analyze lp333-multiplier --col10-shift1`, but it is skipped by default because the current implementation is too slow for a routine checkpoint
  - a previous fast `2^13` `(0,1)` run has been invalidated as a recorded result: it enumerated rows fixed by column multiplication by `10`, which is an extra assumption for the nonfixed order-`3` row orbits; the corrected arbitrary-row table takes about `176s` in the targeted test and leaves all `1,296` fixed-row-compatible row-marginal pairs alive, so `(0,1)` adds no pruning at this marginal level
  - a relaxed coupled `(3,0)+(0,1)` diagnostic now exists behind `hadamard analyze lp333-multiplier --col10-coupled`; it couples the arbitrary nonfixed row-orbit base rows through a joint table but still relaxes fixed-row coupling, and it also keeps all `1,296` fixed-row-compatible row-marginal pairs (`147` cached sequence joint sets, max set size `2,426`)
  - exact-fixed-row coupling over the three fixed rows was attempted twice: first as a naive 13-variable frontier, then as a shared fixed-triple target table narrowed to the exact fixed-row-compatible survivors; neither produced a full stats line in a useful debug window, so exact fixed-row coupling still needs a more compressed state representation before it should become a public analyzer path
  - an opt-in CRT component orbit view is now available behind `hadamard analyze lp333-multiplier --crt-components`, showing the multiplier orbits in `(mod 9, mod 37)` form and emitting a focused `row_units={1,4,7}` hypothesis summary instead of only flat residue classes; that family currently has `15` surviving hypotheses split across orders `3,6,9,12,18,36`, with order-`3` column units `[1]` / `[1,10,26]` and order-`6` column units `[1,36]` / `[1,10,11,26,27,36]`; the nontrivial mixed-CRT target set is now the four hypotheses in `crt_component_mixed_crt_targets` with order split `3:2,6:2`, and the summary now includes `pairs`, `mass`, `pair_samples`, and `col_orbits`, but those four targets still tie on `pairs=12`, `mass=119903105952`, `row_orbits=0|1,4,7|2,5,8|3|6`, `allowed_bundles=1064`, and the same top pair samples; `col_orbits` only splits them into the two order-based classes, and the remaining distinction is the explicit `subgroup_crt` label
  - the same CRT component view now runs an actual invariant-row-stabilizer lift for those four mixed targets: the two order-`3` targets have `113` invariant table orbits (`1:3,3:110`) and reproduce the `1,296` exact row-pair survivors with `log10 ~= 60.818`, while the two order-`6` targets have `59` invariant table orbits (`1:3,3:2,6:54`) but their stronger row stabilizer leaves `0` row-pair candidates after the active bundle-pair join
  - a bounded `hadamard analyze lp333-multiplier --invariant-shift31` graph diagnostic now profiles the first actual mixed shift `(3,1)` for the two surviving order-`3` targets: it keeps the same `1,296` exact row-pair survivors, finds only `42` fixed-row triple targets and `6` nonfixed row-sum targets, and now prints both min-fill and linear-frontier orders; the fixed graph has min-fill width `5` but exact frontier width `6` (`8^6 = 262,144` states), while the nonfixed graphs have min-fill width `10` / `9` but frontier width `11` (`2^11 = 2,048` states)
  - the fixed-row `(3,1)` value/profile builders now use exact suffix target reachability and the new frontier order; `hadamard analyze lp333-multiplier --invariant-shift31-profile` also reports active-assignment capacity and fill rate (`fill_ppm = assignments / capacity * 1,000,000`). With the `100,000` value-state cap, the fixed-row side stops after `7` processed variables at active width `4` (`~2.8k` assignments, max `93` / `95` values per assignment), while the nonfixed raw table reaches the width-`11` frontier and still caps at `100,000` values
  - `hadamard analyze lp333-multiplier --invariant-shift31-compressed-profile` now profiles a row-signature plus bounded-energy-bitset representation using that frontier order; the cap is configurable with `--invariant-shift31-profile-limit N`. At the default `100,000` signature cap, both nonfixed profiles now complete exactly (`31,020` / `26,598` signatures and `267,044` / `162,860` energy bits), while fixed stops after `7` variables. In release mode with the direct fixed assignment representation, cap `30,000,000` completes the fixed side exactly in about `22s`, with peak `20,849,958` row signatures and `58,360,095` energy bits.
  - an opt-in exact value-table scaffold for that same shift now exists behind `hadamard analyze lp333-multiplier --invariant-shift31-exact`; in release mode it completes in about `189s`, with fixed targets `42 / 42`, nonfixed targets `6 / 6`, max row energy values `84`, and `1,296` feasible row pairs. Therefore exact single shift `(3,1)` is non-pruning: aggregate pattern mass remains `log10 ~= 60.818`, and the next multiplier discriminator needs coupled mixed shifts or a stronger invariant, not another single-shift marginal.
  - a new opt-in coupled mixed-shift graph sweep now exists behind `hadamard analyze lp333-multiplier --invariant-mixed-shift-pairs`; it ranks labeled pairs of column-shift orbits before a heavier exact representation is attempted. The `66` labeled pairs reduce to `6` multiplier-ratio classes. `(1,11)` is a representative of the cheapest observed class for both surviving order-`3` mixed targets, not a unique or proved-strongest choice; it keeps the fixed-row union graph at frontier width `6` while raising the nonfixed union graph to frontier width `18` (`2^18 = 262,144` states).
  - a reproducible PB-geometry audit reduces the `1,296` exact row-marginal pairs to `19` safe symmetry orbits per surviving subgroup (`2` orbits of size `36`, `17` of size `72`). Each of the resulting `38` planned shards has `226` base signs, `12,656` shared XOR auxiliaries, and `56` independent weighted shift equations. This supports a streamed PB export; it does not make the two-energy coupled frontier practical in memory.
  - the remaining multiplier-subfamily targets are now the two non-column-trivial order-`3` `row_units={1,4,7}` subgroups, where row orbits are tied by column permutations rather than identical row sequences; the next missing check is mixed CRT compatibility on actual invariant sign tables
- the heaviest surviving bundled pair orbit still has raw exact lift space about `1.4545e18`, so a naive exact row-shift `1,2,4` lift is not viable
- but the first compressed `UV -> W` transition state on that same orbit is only about `1.11e6` / `1.16e6` signatures per side, and the measured collapse now appears concentrated at the final `W` frontier rather than inside the `UV` table itself, so an optimized orbit-level exact lift still looks plausible if it batches on that downstream key
- exact small known cases are validated end to end

Current known-case ladder:
- [x] order `12` from a length-`5` Legendre pair
- [x] order `16` from a length-`7` Legendre pair
- [x] order `20` from a length-`9` Legendre pair
- [x] order `24` from a length-`11` Legendre pair
- [x] order `28` from a length-`13` Legendre pair
- [x] order `668` public artifact reconstructed and checked by two independent exact
  verifiers
- [x] order `668` compact `4 x 166` seed reconstructed through the native
  Wallis–Whiteman builder byte for byte
- [x] compressed length-`9`, factor-`3` bucket/decompression demo
- [x] length-`15`, factor-`3` compressed benchmark artifact
- [ ] any serious `333`-scale decompression (retained as historical methods work,
  not needed to establish order-`668` existence)

## Milestone Board

### M0. Foundation

- [x] exact `±1` sequence math
- [x] periodic autocorrelation
- [x] Hadamard verification
- [x] 2cc construction
- [x] checkpoint format
- [x] artifact format
- [x] known small-case validation path

Status:
- complete

### M1. Fast spectral backend layer

- [x] PSD backend abstraction
- [x] reference direct backend
- [x] independent autocorrelation backend
- [x] backend agreement tests
- [x] mixed-radix FFT backend

Status:
- complete

### M2. Strong compressed LP filtering

- [x] compressed candidate generation
- [x] candidate compatibility pruning
- [x] exact PSD-signature bucket pruning
- [x] compressed residual filtering
- [x] PSD consistency filtering
- [x] staged metrics in artifacts
- [x] reusable bucket artifact output
- [x] larger compressed benchmark cases

Status:
- complete for current small benchmark scope, larger-scale work still open

### M3. Decompression engine

- [x] decompression interface
- [x] bucket artifact parser
- [x] exact small-case decompression prototype
- [x] canonical exact representative filtering
- [x] pair-level common-shift canonicalization
- [x] multi-layer prefix pruning during exact expansion
- [x] exact Legendre-signature bucket matching after expansion
- [x] end-to-end compressed-to-exact recovery test
- [x] larger length-`15` compressed bucket decompression recovery
- [ ] scalable decompressor for larger compressed instances
- [ ] SAT/CAS or meet-in-the-middle decompressor

Status:
- prototype complete, production version not started

### M4. Known-case ladder expansion

- [x] order `12` fixture
- [x] order `16` fixture
- [x] order `20` fixture
- [x] order `24` fixture
- [x] order `28` fixture
- [x] order `668` fixture with primary-source provenance and two independent exact
  all-pairs checks
- [x] more nontrivial known constructions
- [x] independent exported-matrix checks for the current future fixture set

Status:
- complete

### M5. SDS infrastructure

- [x] parameter enumeration for `Z_167`
- [x] SDS block representation
- [x] difference-profile math
- [x] matcher primitives
- [x] shardable SDS search

Status:
- complete for small exact instances, not yet scaled toward `Z_167`

### M6. First serious `LP(333)` campaign (historical methods track)

- [x] production search config-file support
- [x] initial campaign-template configs
- [x] artifact version hardening
- [x] benchmark baselines on larger compressed sizes
- [ ] long-run operational runbook
- [ ] real `333` search campaign
- [ ] productionize or replace the experimental pair-aware compressed search path

Status:
- partially complete and still blocked on `333`-scale compressed candidate
  generation; no longer a blocker to order-`668` existence or the top active priority

### M7. Preserve, verify, and identify `HM(668)`

- [x] exact `668 x 668` fixture and primary-source hashes
- [x] two independent exact all-pairs verifiers
- [x] compact four-sequence seed extracted from the primary payload
- [x] cyclic type-`H_4^*` parameters and combined PAF identity checked exactly
- [x] native Wallis–Whiteman construction
- [x] byte-identical reconstruction of the stored fixture
- [x] independent standard-library Python reconstruction of the same bytes
- [ ] documented discovery/search provenance for the particular `Z_166` family

Status:
- artifact preservation, structural identification, and reconstruction complete;
  seed-discovery provenance remains open

## What Works Today

You can do these things right now:

- run the full Rust test suite
- run the known exact fixtures
- re-check the exact order-`668` fixture and its recorded SHA-256
- reconstruct the fixture byte for byte from its compact cyclic `4 x 166` seed
- build and verify small Hadamard matrices from known pairs
- run compressed search on the known length-`9` demo
- run compressed search on the length-`15` benchmark and inspect the larger bucket metrics
- emit reusable bucket artifacts
- decompress those bucket artifacts back to canonical exact pairs
- observe branch-pruning metrics during decompression, including the reduced `43`-candidate, `39`-bucket, `47`-pair, `24`-match length-`15` recovery set
- run a small SDS meet-in-the-middle recovery over `Z_5`
- build a Goethals-Seidel matrix from four complementary sequences
- reproduce the published 64-modular seed and build its exact fixed-`q` solver model
- enumerate or select its `42` exact modulo-`4` quotient cases, eliminate `20` by
  the exact lag-parity lemma, export the complete `14`-representative restricted-OPB
  campaign, and independently verify an external signed model
- run a larger compressed smoke probe via `configs/baselines/lp33-compressed3-smoke.cfg`
- run the experimental joint compressed-pair probe on reduced lengths `5` and `11`
- read the detailed verification/novelty audit in [RESEARCH_STATUS.md](/home/nate/projects/hadamard/docs/RESEARCH_STATUS.md)

Most useful commands:

```bash
cargo test
cargo run -p hadamard-cli -- test-known lp-small
cargo run -p hadamard-cli -- test-known lp-seven
cargo run -p hadamard-cli -- test-known lp-nine
cargo run -p hadamard-cli -- test-known lp-eleven
cargo run -p hadamard-cli -- test-known lp-thirteen
cargo run -p hadamard-cli -- search lp --length 9 --compression 3 --max-attempts 256 --bucket-out outputs/examples/lp9-buckets.txt
cargo run -p hadamard-cli -- decompress lp --bucket-in outputs/examples/lp9-buckets.txt --max-pairs 64 --artifact-out outputs/examples/lp9-decompressed.txt
cargo run -p hadamard-cli -- search lp --length 15 --compression 3 --max-attempts 32768 --bucket-out outputs/examples/lp15-buckets.txt --artifact-out outputs/examples/lp15-compressed.txt
cargo run -p hadamard-cli -- decompress lp --bucket-in outputs/examples/lp15-buckets.txt --max-pairs 4096 --artifact-out outputs/examples/lp15-decompressed.txt
cargo run -p hadamard-cli -- search lp --config configs/baselines/lp15-compressed3-smoke.cfg --max-attempts 8
cargo run -p hadamard-cli -- search lp --config configs/baselines/lp33-compressed3-smoke.cfg
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 15 --compression 3 --max-pairs 32
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 33 --compression 3 --max-pairs 1
cargo run -p hadamard-cli -- search sds --order 5 --block-sizes 2,2,0,0 --lambda 1 --max-matches 4
uv run python py/search_modular_golay.py --audit-only
uv run python py/search_modular_golay.py --export-opb-dir outputs/fixed-q-opb
uv run python py/search_modular_golay.py --export-t-fixed-opb-dir outputs/fixed-q-opb-t-fixed
uv run python py/test_search_modular_golay.py
uv run python py/validate_matrix.py fixtures/known/hadamard-small/order20.txt
uv run python py/validate_matrix.py fixtures/known/hadamard-small/order24.txt
uv run python py/validate_matrix.py fixtures/known/hadamard-small/order28.txt
uv run python py/validate_matrix.py fixtures/known/hm-668/matrix.txt
cargo run -p hadamard-cli -- verify matrix --input fixtures/known/hm-668/matrix.txt
cargo run -p hadamard-cli -- build spence --sequences-in fixtures/known/hm-668/spence-sequences.txt --output /tmp/hm668-spence.txt
cmp /tmp/hm668-spence.txt fixtures/known/hm-668/matrix.txt
uv run python py/reconstruct_hm668.py fixtures/known/hm-668/spence-sequences.txt /tmp/hm668-python.txt
cmp /tmp/hm668-python.txt fixtures/known/hm-668/matrix.txt
```

## Immediate Next Work

Highest-value next steps:

1. Replay identical `2092` pools through the central and bucket-ranked
   partial-sum schedules at equal pair budgets. Use the
   [order-4412 handoff](research/hm-4412-campaign.md#handoff-where-to-look-next)
   for controls, run metrics, and when to scale the search.
2. For the smallest-order lane, apply the `Z_46` quotient and mod-3 fixed-slice
   correlations to `<45,47>` over `Z_598`, using Cooley's seed as a control.
3. Preserve the primary X payload and decoder provenance, their hashes, and the
   audited reconstruction steps in a durable, reproducible record independent of
   the live social-media presentation.
4. Determine the discovery or search provenance of the particular cyclic
   `(166;82,83,83,83;164)` family. Do not infer that it came from a named Spence
   relative-difference-set family merely because it uses the generic bordered array.
5. Extend structural analysis beyond the now-proved circulant blocks and border to
   normalization, automorphisms, ranks, and useful equivalence fingerprints.
6. Preserve both compact-seed reconstruction paths and their exact digests: the
   native builder and the independent standard-library Python implementation.
7. Keep the LP(333), fixed-`q`, CRT, and SDS results as reproducible historical
   search work. Resume expensive searches only for a distinct methods or structural
   question, with the existing exact benchmarks and proof discipline intact.

Historical search note:

- see [docs/research/crt-multiplier-roadmap.md](/home/nate/projects/hadamard/docs/research/crt-multiplier-roadmap.md) for the current algebraic-sieve-first plan

## How To Read The Repo

Recommended order for a new contributor:

1. Read [README.md](/home/nate/projects/hadamard/README.md).
2. Read this file.
3. Read [docs/RESEARCH_STATUS.md](/home/nate/projects/hadamard/docs/RESEARCH_STATUS.md).
4. Read [docs/MILESTONES.md](/home/nate/projects/hadamard/docs/MILESTONES.md).
5. Read [docs/runbook.md](/home/nate/projects/hadamard/docs/runbook.md).
6. Read [docs/algorithms/legendre-pair.md](/home/nate/projects/hadamard/docs/algorithms/legendre-pair.md).
7. Run the known-case commands and inspect `outputs/examples/`.
