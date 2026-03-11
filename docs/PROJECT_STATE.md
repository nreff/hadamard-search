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

## Current Snapshot

Current state:
- working research scaffold
- not yet a production `LP(333)` solver
- compressed search and first decompression prototype exist
- decompression now emits canonical exact pair representatives
- decompression now applies multi-layer canonical-prefix pruning during expansion
- decompression now reuses exact candidates via complementary Legendre-signature buckets
- exact small known cases are validated end to end

Current known-case ladder:
- [x] order `12` from a length-`5` Legendre pair
- [x] order `16` from a length-`7` Legendre pair
- [x] order `20` from a length-`9` Legendre pair
- [x] order `24` from a length-`11` Legendre pair
- [x] compressed length-`9`, factor-`3` bucket/decompression demo
- [ ] larger known compressed benchmarks
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
- [ ] genuinely faster backend

Status:
- partially complete

### M2. Strong compressed LP filtering

- [x] compressed candidate generation
- [x] candidate compatibility pruning
- [x] exact PSD-signature bucket pruning
- [x] compressed residual filtering
- [x] PSD consistency filtering
- [x] staged metrics in artifacts
- [x] reusable bucket artifact output
- [ ] larger compressed benchmark cases

Status:
- substantially complete for small cases

### M3. Decompression engine

- [x] decompression interface
- [x] bucket artifact parser
- [x] exact small-case decompression prototype
- [x] canonical exact representative filtering
- [x] pair-level common-shift canonicalization
- [x] multi-layer prefix pruning during exact expansion
- [x] exact Legendre-signature bucket matching after expansion
- [x] end-to-end compressed-to-exact recovery test
- [ ] scalable decompressor for larger compressed instances
- [ ] SAT/CAS or meet-in-the-middle decompressor

Status:
- prototype complete, production version not started

### M4. Known-case ladder expansion

- [x] order `12` fixture
- [x] order `16` fixture
- [x] order `20` fixture
- [x] order `24` fixture
- [x] more nontrivial known constructions
- [x] independent exported-matrix checks for the current future fixture set

Status:
- in progress

### M5. SDS infrastructure

- [x] parameter enumeration for `Z_167`
- [ ] SDS block representation
- [ ] difference-profile math
- [ ] matcher primitives
- [ ] shardable SDS search

Status:
- not started beyond parameter encoding

### M6. First serious `LP(333)` campaign

- [ ] production search configs
- [ ] artifact version hardening
- [ ] benchmark baselines on larger compressed sizes
- [ ] long-run operational runbook
- [ ] real `333` search campaign

Status:
- not started

## What Works Today

You can do these things right now:

- run the full Rust test suite
- run the known exact fixtures
- build and verify small Hadamard matrices from known pairs
- run compressed search on the known length-`9` demo
- emit reusable bucket artifacts
- decompress those bucket artifacts back to canonical exact pairs
- observe branch-pruning metrics during decompression and recover a single canonical exact representative on the length-`9` demo with only `2` exact pair checks

Most useful commands:

```bash
cargo test
cargo run -p hadamard-cli -- test-known lp-small
cargo run -p hadamard-cli -- test-known lp-seven
cargo run -p hadamard-cli -- test-known lp-nine
cargo run -p hadamard-cli -- test-known lp-eleven
cargo run -p hadamard-cli -- search lp --length 9 --compression 3 --max-attempts 256 --bucket-out outputs/examples/lp9-buckets.txt
cargo run -p hadamard-cli -- decompress lp --bucket-in outputs/examples/lp9-buckets.txt --max-pairs 64 --artifact-out outputs/examples/lp9-decompressed.txt
uv run python py/validate_matrix.py fixtures/known/hadamard-small/order20.txt
uv run python py/validate_matrix.py fixtures/known/hadamard-small/order24.txt
```

## Immediate Next Work

Highest-value next steps:

1. Add a genuinely faster PSD/DFT backend behind the current abstraction.
2. Add incremental autocorrelation-style pruning so decompression branches less before exact candidate construction.
3. Add larger known compressed benchmarks so pruning stages can be measured on something more realistic than length `9`.
4. Start the first real SDS primitives rather than leaving that track at parameter enumeration.

## How To Read The Repo

Recommended order for a new contributor:

1. Read [README.md](/home/nate/projects/hadamard/README.md).
2. Read this file.
3. Read [docs/MILESTONES.md](/home/nate/projects/hadamard/docs/MILESTONES.md).
4. Read [docs/runbook.md](/home/nate/projects/hadamard/docs/runbook.md).
5. Read [docs/algorithms/legendre-pair.md](/home/nate/projects/hadamard/docs/algorithms/legendre-pair.md).
6. Run the known-case commands and inspect `outputs/examples/`.
