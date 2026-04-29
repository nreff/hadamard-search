# Project State

This is the best starting point for a new reader.

It answers:

- What is the project trying to do?
- What plan are we following?
- What is already complete?
- What is in progress?
- What is next?

## Mission

Goal:
- build a research-grade computational pipeline that could help discover a new Hadamard matrix of order `668`

Primary mathematical route:
- search for a **Legendre pair of length `333`**
- use the **two-circulant-core (2cc)** construction to build `HM(668)`

Secondary route:
- search for structured **SDS / Goethals-Seidel** constructions over `Z_167`

Why this repo exists:
- direct search at order `668` is infeasible
- structured search might be feasible with strong pruning, compression, and reproducible artifacts
- the engineering pipeline itself may become a novel computational method if it materially changes what is feasible

Background documents:
- [deep-research-report.md](/home/nate/projects/hadamard/docs/research/deep-research-report.md)
- [legendre-pair.md](/home/nate/projects/hadamard/docs/algorithms/legendre-pair.md)
- [sds-167.md](/home/nate/projects/hadamard/docs/algorithms/sds-167.md)
- [RESEARCH_STATUS.md](/home/nate/projects/hadamard/docs/RESEARCH_STATUS.md)

## Current Snapshot

Current state:
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
- checkpoint and bucket parsing now reject unknown schema versions, and LP artifacts carry explicit run metadata
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
  - a direct opt-in run was attempted, but it did not complete quickly enough to count as a recorded analyzer result
  - the remaining multiplier-subfamily targets are still the non-column-trivial `row_units={1,4,7}` subgroups, where row orbits are tied by column permutations rather than identical row sequences; the next missing check is mixed CRT compatibility on actual invariant sign tables
- the heaviest surviving bundled pair orbit still has raw exact lift space about `1.4545e18`, so a naive exact row-shift `1,2,4` lift is not viable
- but the first compressed `UV -> W` transition state on that same orbit is only about `1.11e6` / `1.16e6` signatures per side, and the measured collapse now appears concentrated at the final `W` frontier rather than inside the `UV` table itself, so an optimized orbit-level exact lift still looks plausible if it batches on that downstream key
- exact small known cases are validated end to end

Current known-case ladder:
- [x] order `12` from a length-`5` Legendre pair
- [x] order `16` from a length-`7` Legendre pair
- [x] order `20` from a length-`9` Legendre pair
- [x] order `24` from a length-`11` Legendre pair
- [x] order `28` from a length-`13` Legendre pair
- [x] compressed length-`9`, factor-`3` bucket/decompression demo
- [x] length-`15`, factor-`3` compressed benchmark artifact
- [ ] any serious `333`-scale decompression

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

### M6. First serious `LP(333)` campaign

- [x] production search config-file support
- [x] initial campaign-template configs
- [x] artifact version hardening
- [x] benchmark baselines on larger compressed sizes
- [ ] long-run operational runbook
- [ ] real `333` search campaign
- [ ] productionize or replace the experimental pair-aware compressed search path

Status:
- started, but still blocked on `333`-scale compressed candidate generation

## What Works Today

You can do these things right now:

- run the full Rust test suite
- run the known exact fixtures
- build and verify small Hadamard matrices from known pairs
- run compressed search on the known length-`9` demo
- run compressed search on the length-`15` benchmark and inspect the larger bucket metrics
- emit reusable bucket artifacts
- decompress those bucket artifacts back to canonical exact pairs
- observe branch-pruning metrics during decompression, including the reduced `43`-candidate, `39`-bucket, `47`-pair, `24`-match length-`15` recovery set
- run a small SDS meet-in-the-middle recovery over `Z_5`
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
uv run python py/validate_matrix.py fixtures/known/hadamard-small/order20.txt
uv run python py/validate_matrix.py fixtures/known/hadamard-small/order24.txt
uv run python py/validate_matrix.py fixtures/known/hadamard-small/order28.txt
```

## Immediate Next Work

Highest-value next steps:

1. Keep the current exact-tail compressed-pair path as a runtime baseline, but treat the CRT analyzer as the main discovery path for now.
2. Turn the surviving CRT row-bundle problem into an optimized exact lift over row shifts `1,2,4`, keyed by orbit-level transition signatures rather than naive lifted rows.
3. Continue the non-column-trivial `row_units={1,4,7}` multiplier branch by deriving mixed CRT constraints for the `col_units={1,10,26}` row-orbit action, starting from the `1,296` surviving fixed-row-compatible row marginals.
4. Promote any CRT or multiplier invariant into the hot search path only after it is exact, documented, and benchmarked against the current reduced anchors.
5. If the algebraic sieve line stalls, then revisit a genuinely different exact factorization such as `4+4+4`, not more local tweaks of the current `6+6` join.

Focused note:

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
