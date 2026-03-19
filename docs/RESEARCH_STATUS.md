# Research Status

This document is the detailed "where are we really?" memo for the current search program.

It is intentionally more candid than the README. It separates:

- what is verified now
- what seems promising but is still experimental
- what feels novel
- what would and would not yet be defensible in a paper or report

If this progresses toward a formal write-up, the paper-style skeleton now lives in [METHODS_NOTE_OUTLINE.md](/home/nate/projects/hadamard/docs/METHODS_NOTE_OUTLINE.md).

## Verification Stamp

Last re-checked on `2026-03-14`.

Verified during this audit:

- `cargo test` passes across the workspace
- the current best reduced-length-`11` direct joint benchmark reproduces exactly:
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

- the current best reduced-length-`15` anchor is now measured rather than merely capped
  - `cargo run -p hadamard-cli -- benchmark compressed-pairs --length 45 --compression 3 --ordering natural --spectral-frequencies 1 --tail-depth 12 --max-pairs 1`
  completed in `14.13` seconds
  - output summary:
    - `branches_considered=48`
    - `tail_candidates_checked=129335`
    - `tail_spectral_pruned=71722`
    - `tail_residual_pruned=57612`
    - `pairs_emitted=1`
  - reduced length `15` is therefore no longer an inaccessible anchor, but it remains the next serious scaling barrier because the cost is dominated by tail-candidate multiplicity

## Executive State

The repository is now beyond "research scaffold only" but still short of a production `LP(333)` search engine.

The strongest current result is not just a constant-factor improvement. The experimental direct compressed-pair probe has changed regimes:

- earlier, reduced length `11` was dominated by branching
- now, with factorized exact-tail completion, the same benchmark is dominated by exact tail-candidate volume

That distinction matters. It means the code is no longer merely shaving branches off a DFS. It has started converting the search into a keyed exact-completion problem.

That is the most important technical change in the repo right now.

## Verified Accomplishments

### Correctness ladder

The exact `LP -> 2cc -> Hadamard` path is validated on:

- length `5` -> order `12`
- length `7` -> order `16`
- length `9` -> order `20`
- length `11` -> order `24`
- length `13` -> order `28`

This matters because every experimental pruning change is being judged against known-valid smaller instances, not only against runtime.

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

For the itemized history, see [docs/EXPERIMENT_LOG.md](/home/nate/projects/hadamard/docs/EXPERIMENT_LOG.md).

## Current Intuition

My current intuition is:

1. The main bottleneck is no longer global branching at the reduced-length-`11` benchmark.
2. The main bottleneck is now candidate multiplicity inside exact tail keys.
3. That suggests the right abstraction is becoming "exact completion under strong keyed invariants" rather than "deeper DFS with more inequalities."

More concretely:

- the direct joint path is now strongest when it commits early work to a front prefix and pushes the rest into an exact keyed tail oracle
- once that happens, the useful next ideas are not generic branch-order tweaks
- they are better exact tail keys, better factorization, or cheap exact tail-side filters before full residual evaluation

I would summarize the current mathematical-computational picture like this:

- branch pruning got us into the right region
- exact completion is what is now changing the regime

That is the strongest signal in the whole project at the moment.

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

- the repo contains an original experimental search pipeline around the `HM(668)` Legendre-pair route
- several joint-space pruning and exact-tail ideas were implemented, benchmarked, and compared against alternatives
- the best current direct-joint benchmark on reduced length `11` changed regime from branch-dominated to exact-tail-candidate-dominated
- multiple intuitive alternatives were tested and rejected for precise reasons

### What would not yet be defensible to claim

Not yet justified:

- "we have a publishable breakthrough"
- "this is definitely a new theorem"
- "this will scale to `LP(333)`"
- "negative computational results here would imply nonexistence"

The gap is still scaling evidence. Reduced length `11` is now strong and reduced length `15` is now measurable, but the latter already requires checking about `96` million tail candidates for the first hit, so we should not overstate the reach of the current method.

## What Would Make This Publishable

The lowest realistic publication bar is not discovery of `HM(668)` itself. It is a computational-methods result with a clean experimental story.

That would likely need:

- a stable and clearly described algorithmic core
- a measured comparison against credible alternatives
- a strong correctness ladder
- honest negative results
- evidence that the new method changes the practical frontier on a nontrivial family of reduced instances

In other words, the most plausible near-term publishable object is:

- "a joint compressed search and exact-tail completion method for Legendre-pair search,"

not yet:

- "the solution to order `668`."

The project is close enough to that methods territory that the documentation now needs to be publication-aware, which is why this file exists.

## Current Bottlenecks

The bottlenecks now split by path.

Production compressed pipeline:

- single-side candidate generation still explodes too early for `333`

Experimental direct-joint path:

- exact tail candidate volume per key is now the dominant cost on the best reduced-length-`11` configuration
- the reduced-length-`15` anchor confirms the same regime more strongly: only `160` branches, but about `96` million tail candidates checked

MITM line:

- naive splits preserve too little of the correlation structure and push too much work into the join

## Immediate Next Research Questions

The best next questions are:

1. What exact necessary conditions does the LP identity impose after the CRT decomposition `Z_333 ~= Z_9 x Z_37`?
2. Does LP(333) admit any nontrivial multiplier or automorphism action that can be turned into a hard search-space normalization or sieve?
3. Can product-group Fourier structure over `Z_9 x Z_37` expose sharper exact or near-exact invariants than the current undifferentiated PSD view over `Z_333`?
4. Can the polynomial identity `A(x)A(x^-1) + B(x)B(x^-1)` be turned into a practical propagation engine or SAT/CAS side-constraint rather than only a residual check?
5. If none of the algebraic-sieve lines pay off, what genuinely different exact factorization lowers multiplicity better than the current `6+6` tail join?

Questions that currently look less attractive:

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

The project is not yet in "we solved 668" territory.

It is, however, in a more serious place than a generic exploratory codebase:

- correctness infrastructure is strong
- multiple negative and positive experiments are documented
- the direct joint search line has changed computational regime on a nontrivial benchmark
- the next steps are mathematically and algorithmically sharper than before

That is enough to justify much more careful documentation and claim discipline from this point onward.
