# Methods Note Outline

Working title:

- `Joint Compressed Search and Exact-Tail Completion for Legendre-Pair Search Toward Hadamard Order 668`

Purpose:

- provide a paper-ready skeleton for a computational-methods note
- separate the method contribution from the still-open `HM(668)` existence question
- keep future claims disciplined and benchmark-backed

## 1. Abstract

Target shape:

- one paragraph on the `HM(668)` motivation
- one paragraph on the search bottleneck in compressed Legendre-pair search
- one paragraph stating the method contribution:
  - direct joint compressed `(A, B)` search
  - exact joint norm pruning
  - endpoint-aware autocorrelation bounds
  - exact-tail completion and factorized tail completion
- one paragraph on the main measured result:
  - reduced length `11` changes from branch-dominated to tail-candidate-dominated

Claims that are safe here:

- method-oriented improvement
- verified smaller-case correctness ladder
- experimentally documented failed alternatives

Claims to avoid here:

- any implication that `HM(668)` is solved
- any implication of nonexistence from negative runs

## 2. Problem Setting

Explain:

- Hadamard order `668`
- Legendre pair of length `333`
- 2cc construction
- why compressed search is necessary

Keep this section short. The contribution is computational method, not a survey.

## 3. Baseline Search Pipeline

Describe the pre-novel baseline:

- single-sequence compressed candidate generation
- row-sum / PSD / residual filters
- pairing after generation
- decompression after compressed acceptance

Include:

- what the baseline already does well
- where it explodes

Recommended table:

- length / compression / candidates / bucket counts / pair counts
- include the verified length-`15`, factor-`3` pipeline numbers

## 4. Joint Compressed Search Formulation

Core contribution section.

Explain:

- why searching directly in compressed `(A, B)` space is different from generating sides independently
- exact joint squared-norm identity
- row-sum feasibility
- endpoint-aware partial autocorrelation intervals
- selected-frequency pair-PSD bounds

What to formalize:

- exact state definition
- branching alphabet
- completeness-safe pruning conditions

What to avoid overstating:

- novelty of each ingredient in isolation
- any theorem-strength language unless formally proved in the note

## 5. Exact-Tail Completion

This is likely the real centerpiece.

Subsections:

- fixed-depth exact tail completion
- packed tail encoding
- factorized deeper-tail completion
- tail-side spectral filtering

Key story:

- the method moves the bottleneck from branching to keyed exact completion

Recommended figure:

- branch count versus tail depth on reduced length `11`

Recommended table:

- tail depth
- branches considered
- tail candidates checked
- tail spectral prunes
- elapsed time

Use the verified reduced length-`11` progression:

- no tail oracle: `435296` branches
- depth `3`: `60704`
- depth `4`: `20000`
- depth `5`: `5712`
- depth `6`: `1360`
- factorized depth `7`: `272`
- factorized depth `8`: `64`
- factorized depth `11`: `0` branches, `996305` tail candidates, `996304` tail spectral prunes, `14.19s`

## 6. Negative Results and Design Rejections

This section should stay in the methods note.

Include:

- independent compressed dihedral canonicalization
- partial prefix-based pair canonicalization
- full-spectrum prefix checks in the single-sequence generator
- count-profile-first generator
- contiguous MITM
- parity MITM
- parity even-shift signatures
- generator-`2` ordering

Why this section matters:

- it demonstrates that the final method is not a cherry-picked anecdote
- it provides guidance for future researchers

Primary source for this section:

- [docs/EXPERIMENT_LOG.md](/home/nate/projects/hadamard/docs/EXPERIMENT_LOG.md)

## 7. Correctness Protocol

Describe the trust model:

- known exact ladder through lengths `5, 7, 9, 11, 13`
- final 2cc and Hadamard verification
- backend agreement tests
- compressed-to-exact recovery tests
- schema/version rejection tests

This section is important if the note is to be taken seriously as computational mathematics.

## 8. Experimental Results

Structure:

- exact known-case validation
- production compressed/decompression baseline at length `15`, factor `3`
- direct-joint benchmark at reduced length `11`
- next anchor at reduced length `15`

Safe current statement for the last item:

- the `length=45`, `compression=3`, `tail_depth=11` direct-joint benchmark did not finish within `120` seconds on the audit run and remains the next scaling barrier

Do not pad this section with speculative extrapolation to `333`.

## 9. Interpretation

Core message:

- the method does not yet solve the `333` problem
- but it changes the computational regime on a nontrivial reduced benchmark
- the current dominant cost is tail-key multiplicity, not DFS branching

This is where to say:

- the method contribution is algorithmic and computational
- not a nonexistence proof
- not yet a large-scale success claim

## 10. Limitations

Be explicit:

- scaling to reduced length `15` is not yet routine
- production compressed search still blows up too early
- current evidence is strongest on reduced length `11`
- no claim of asymptotic superiority
- no claim that the method will reach `333` unchanged

## 11. Next Research Directions

These should mirror the code reality:

1. stronger exact tail keys
2. factorized tail representations with lower candidate multiplicity
3. cheaper exact tail-side filters
4. making reduced length `15` a routine benchmark

Avoid weaker directions:

- more search-order experiments
- naive new MITM splits without correlation-aware invariants

## 12. Reproducibility Appendix

Include:

- commit hash once checkpointed
- machine description
- Rust version
- exact commands for:
  - `cargo test`
  - reduced length `11` benchmark
  - reduced length `15` capped run

Recommended command block:

```bash
cargo test
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 33 --compression 3 --ordering natural --spectral-frequencies 4 --tail-depth 11 --max-pairs 1
timeout 120s cargo run -p hadamard-cli -- benchmark compressed-pairs --length 45 --compression 3 --ordering natural --spectral-frequencies 4 --tail-depth 11 --max-pairs 1
```

## Data And Figure Checklist

Before drafting the full note, gather:

- one table for the exact known-case ladder
- one table for the length-`15`, factor-`3` production compressed/decompression pipeline
- one table for direct-joint tail-depth sweep
- one table for rejected alternatives and their measured outcomes
- one figure showing the branch-to-tail-candidate regime shift

## Current Thesis

If this methods note were written now, the central thesis should be:

- exact-tail completion in joint compressed Legendre-pair search can qualitatively change the search regime on meaningful reduced instances, and this is more important than further local pruning of ordinary DFS branches

That is the strongest claim the current evidence can support cleanly.
