# Research Status

This document is the detailed "where are we really?" memo for the current search program.

It is intentionally more candid than the README. It separates:

- what is verified now
- what seems promising but is still experimental
- what feels novel
- what would and would not yet be defensible in a paper or report

If this progresses toward a formal write-up, the paper-style skeleton now lives in [METHODS_NOTE_OUTLINE.md](/home/nate/projects/hadamard/docs/METHODS_NOTE_OUTLINE.md).

## Verification Stamp

Last re-checked on `2026-03-13`.

Verified during this audit:

- `cargo test` passes across the workspace
- the current best reduced-length-`11` direct joint benchmark reproduces exactly:
  - command:
    - `cargo run -p hadamard-cli -- benchmark compressed-pairs --length 33 --compression 3 --ordering natural --spectral-frequencies 4 --tail-depth 11 --max-pairs 1`
  - output summary:
    - `branches_considered=0`
    - `tail_spectral_pruned=996304`
    - `tail_candidates_checked=996305`
    - `pairs_emitted=1`
    - `elapsed_seconds=14.19`

Not yet resolved during this audit:

- the next anchor run
  - `cargo run -p hadamard-cli -- benchmark compressed-pairs --length 45 --compression 3 --ordering natural --spectral-frequencies 4 --tail-depth 11 --max-pairs 1`
  remains materially harder
  - a `120`-second capped run did not finish and exited on timeout
  - so reduced length `15` should still be treated as the next real scaling barrier

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
  - `996305` tail candidates checked
  - `996304` tail-side spectral prunes
  - `1` emitted pair
  - `14.19` seconds on the current machine during this audit

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

The gap is still scaling evidence. Reduced length `11` is now impressive relative to where the code started, but reduced length `15` is still hard enough that we should not overstate the reach of the current method.

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
- the next anchor point at reduced length `15` is still materially expensive

MITM line:

- naive splits preserve too little of the correlation structure and push too much work into the join

## Immediate Next Research Questions

The best next questions are:

1. Can tail keys be strengthened without destroying completeness?
2. Can the factorized tail representation be refined so candidate multiplicity drops sharply per key?
3. Can a tail-side exact filter be made significantly cheaper than full residual checking while preserving the current large pruning rate?
4. Can the reduced-length-`15` anchor be made routine enough to serve as the new benchmark floor?

Questions that currently look less attractive:

- another DFS ordering experiment
- another naive MITM split
- broadening spectral checks without evidence they improve the reduced-length-`11` frontier

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
