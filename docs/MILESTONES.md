# Milestones

This project advances only when each milestone has both:

- implementation work completed
- the corresponding correctness gates passing

No milestone is considered complete based only on runtime improvements or promising large-order search output.

## M0. Foundation

Goal:
- keep the current exact small-order pipeline stable

Implementation:
- exact `±1` sequence math
- periodic autocorrelation
- portable PSD
- Legendre-pair verification
- 2cc construction
- checkpoint and artifact text formats

Required gates:
- `cargo test`
- `cargo run -p hadamard-cli -- test-known lp-small`
- `uv run python py/validate_matrix.py fixtures/known/hadamard-small/order12.txt`

Completion standard:
- known length-5 pair still builds a valid order-12 Hadamard matrix
- checkpoint round-trips remain stable

## M1. Fast spectral backend

Goal:
- introduce a pluggable fast PSD/DFT backend without changing correctness

Implementation:
- backend trait or abstraction layer
- current naive backend retained as reference
- optional faster implementation wired into the same interface

Required gates:
- backend-vs-reference tests on many small sequences
- deterministic CLI benchmark output shape
- all M0 gates remain green

Completion standard:
- fast backend agrees with the reference backend within explicit tolerance
- no valid known case is rejected because of spectral backend differences

Current status:
- complete via the in-tree mixed-radix `fft` backend, with direct-backend agreement tests across many small normalized sequences

## M2. Strong compressed LP filtering

Goal:
- turn compressed search into a meaningful pruning stage instead of a placeholder scorer

Implementation:
- stronger compressed necessary conditions
- deterministic candidate filtering order
- improved metrics and artifact contents

Required gates:
- unit tests per filter on toy compressed inputs
- tests that known-valid small cases survive the filter stack
- shard determinism tests
- all M1 gates remain green

Completion standard:
- filters remove work while preserving known-valid cases
- repeated runs with the same config produce identical accepted candidates

## M3. Decompression engine

Goal:
- recover exact pairs from compressed candidates

Implementation:
- decompressor interface
- first concrete decompressor implementation
- explicit handling for unsat / exhausted / partial states

Required gates:
- compressed-to-exact round-trip tests on toy cases
- end-to-end decompression on at least one known smaller case
- resume/checkpoint tests for interrupted decompression jobs
- all M2 gates remain green

Completion standard:
- at least one known compressed valid case is recovered exactly
- failure modes are explicit and reproducible

## M4. Known-case ladder expansion

Goal:
- strengthen trust in the pipeline before serious 333-scale work

Implementation:
- add more known Legendre-pair or related structured fixtures
- add corresponding export artifacts and validation commands

Required gates:
- each new fixture gets:
  - sequence verification
  - construction test
  - final Hadamard validation
- independent Python validation for exported matrices
- all M3 gates remain green

Completion standard:
- the project no longer depends on a single known case
- at least one nontrivial case beyond length 5 / order 12 is in the test ladder

Current status:
- complete through the length-`13` / order-`28` ladder, with independent Python validation of the exported matrices

## M5. SDS infrastructure

Goal:
- move SDS-167 from parameter enumeration to actual search primitives

Implementation:
- block representation
- difference-profile math
- matcher primitives
- shardable search scaffolding

Required gates:
- exact tests for difference-profile computations
- tests for the encoded `Z_167` parameter table
- at least one small known SDS-style instance end to end
- all M4 gates remain green

Completion standard:
- SDS code is mathematically test-backed, not just structurally present

Current status:
- complete for the current small-instance scope via cyclic block representation, difference-profile tests, `Z_167` table validation, and a shardable `Z_5` meet-in-the-middle recovery path

## M6. First serious LP(333) campaign

Goal:
- run reproducible, checkpointed LP(333) search jobs

Implementation:
- production configs
- artifact versioning hardening
- benchmark baselines
- runbook updates for long jobs

Required gates:
- smoke tests on reduced search slices
- artifact round-trip tests
- benchmark snapshots recorded in docs
- all M5 gates remain green

Completion standard:
- a long LP(333) run can be launched, resumed, inspected, and independently validated

Current status:
- started via `search lp --config ...` support, checked baseline and campaign-template configs, hardened artifact version rejection, and experimental direct-joint benchmark baselines through reduced length `11`, but not close to completion because the current production compressed enumeration does not scale to `333`

## M7. Preserve, Verify, And Identify `HM(668)`

Goal:
- preserve the public order-`668` construction as a reproducible, independently
  checked repository artifact and identify its exact mathematical array

Implementation:
- exact `668 x 668` `+/-` fixture
- primary-source URLs and content hashes
- sign-strict, ragged-row-checking matrix parser and exact Gram verification
- independent reconstruction from the published payload
- standard-library Python mathematical reconstruction independent of the public
  shell generator and native Rust implementation
- compact four-sequence seed extracted from the first `664` signs of the primary
  sign stream
- native Wallis–Whiteman bordered-array constructor
- exact cyclic type-`H_4^*` and PAF validation

Required gates:
- fixture SHA-256 is
  `bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0`
- exactly `668` rows of `668` signs are present
- all `222,778` unordered row pairs have dot product zero
- a separately reconstructed artifact is byte-identical
- four seed rows have length `166`, row sums `[2,0,0,0]`, and combined PAF `-4`
  at all `165` nonzero shifts
- their negative supports have parameters `(166;82,83,83,83;164)` and satisfy the
  type-`H_4^*` identity
- `cargo run -p hadamard-cli -- build spence --sequences-in fixtures/known/hm-668/spence-sequences.txt --output OUTPUT`
  reproduces the fixture byte for byte
- `uv run python py/reconstruct_hm668.py fixtures/known/hm-668/spence-sequences.txt OUTPUT`
  independently reproduces the same bytes and SHA-256
- all earlier milestone gates remain green

Completion standard:
- `H H^T = 668 I` is recomputed from the stored signs, not trusted from a
  reported score or screenshot
- the identity `sum_i C_i C_i^T = 668 I - 4 J` and the border cancellation provide
  a structural proof independent of exhaustive pairwise verification
- provenance distinguishes the now-explained generic bordered array from the still
  unknown discovery/search origin of this particular cyclic `Z_166` seed

Current status:
- complete on `2026-08-12`; see the fixture
  [`README.md`](../fixtures/known/hm-668/README.md) and
  [`hm-668-construction.md`](research/hm-668-construction.md)

## Working rule

When starting any new feature branch or implementation chunk:

1. Add or update the tests for the milestone first.
2. Implement the feature behind the existing stable interfaces where possible.
3. Prove all earlier milestone gates still pass.
4. Only then run larger searches.
