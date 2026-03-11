![hadamard banner](docs/assets/hadamard-banner.png)

# hadamard

Research-grade tooling for computational search toward new Hadamard matrices, starting with the open order **668**.

If you are new to the repo, read these first:

1. [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md)
2. [`docs/MILESTONES.md`](docs/MILESTONES.md)
3. [`docs/runbook.md`](docs/runbook.md)

The repo is built around a practical research stance:

- target structured constructions first, especially **Legendre pairs of length 333** followed by the **two-circulant-core (2cc)** construction
- keep **cyclic SDS over `Z_167`** available as a secondary track
- make room for **novel algorithms** without rewriting the whole codebase
- require every serious search pipeline to reproduce **known smaller cases** before trusting any result at 668

## Why 668

Order `668 = 2 * 333 + 2` is a natural 2cc target: a Legendre pair of length `333` would yield a Hadamard matrix of order `668`.

The included research report also identifies a second route through Goethals-Seidel / SDS constructions over `Z_167`, and lists the feasible row-sum parameter sets forced by the square-sum constraint.

Primary background material lives in:

- [`docs/research/deep-research-report.md`](docs/research/deep-research-report.md)

## Current status

This first implementation is a **working research scaffold**:

- Rust workspace with separated `core`, `search`, `construct`, and `cli` crates
- exact sequence math, PAF/autocorrelation, and a pluggable PSD backend layer
- Legendre-pair verification
- 2cc construction and Hadamard verification
- checkpointed search command for exact small-order LP search
- compressed candidate generation with staged candidate-compatibility, exact PSD-signature, residual, and PSD-consistency filters for future LP(333) decompression work
- SDS-167 parameter enumeration from the report
- two known-case fixtures proving the end-to-end `LP -> 2cc -> Hadamard` path

It is not yet a production-strength `LP(333)` solver. The design is ready for faster FFT backends, stronger pruning, and Phase 2 SAT/CAS decompression.

For the clearest current snapshot with checkboxes and next steps, see:

- [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md)

## Repository layout

- `crates/core`: exact math, sequences, matrices, checkpoint and artifact formats
- `crates/search`: search runners, sharding, compressed candidate enumeration
- `crates/construct`: 2cc builder and validation
- `crates/cli`: `hadamard` command-line interface
- `docs/`: roadmap, algorithm notes, runbook
- `docs/research/`: imported reports and source research material
- `py/`: `uv`-managed independent validation helpers
- `fixtures/`: known cases and future artifacts
- `outputs/`: generated matrices, checkpoints, and search artifacts

## Quick start

Build the workspace:

```bash
cargo build
```

Set up the Python helper environment with `uv`:

```bash
uv sync
```

Run the known small case:

```bash
cargo run -p hadamard-cli -- test-known lp-small
cargo run -p hadamard-cli -- test-known lp-seven
cargo run -p hadamard-cli -- test-known lp-nine
```

Search for small exact Legendre pairs:

```bash
cargo run -p hadamard-cli -- search lp --length 5 --max-attempts 1024 --artifact-out outputs/examples/lp5.txt
```

Verify a candidate pair:

```bash
cargo run -p hadamard-cli -- verify lp --a +--++ --b +-+-+
```

Build a Hadamard matrix from a verified 2cc-ready pair:

```bash
cargo run -p hadamard-cli -- build 2cc --a +--++ --b +-+-+ --output outputs/examples/hm12.txt
```

Validate an exported matrix through the `uv`-managed Python helper:

```bash
uv run python py/validate_matrix.py outputs/examples/hm12.txt
```

Enumerate the SDS-167 parameter targets:

```bash
cargo run -p hadamard-cli -- enumerate sds-167
```

Compare PSD backends:

```bash
cargo run -p hadamard-cli -- benchmark psd --sequence +--++ --backend direct
cargo run -p hadamard-cli -- benchmark psd --sequence +--++ --backend autocorrelation
```

Run the staged compressed search demo:

```bash
cargo run -p hadamard-cli -- search lp --length 9 --compression 3 --max-attempts 256 --artifact-out outputs/examples/lp9-compressed.txt
```

In the current length-9 demonstration, the exact PSD-signature bucket stage reduces pair attempts from the full `12 x 12 = 144` product down to `18` exact bucket-compatible pairs.

Write the reusable bucket artifact too:

```bash
cargo run -p hadamard-cli -- search lp --length 9 --compression 3 --max-attempts 256 --artifact-out outputs/examples/lp9-compressed.txt --bucket-out outputs/examples/lp9-buckets.txt
```

Decompress exact pairs from the bucket artifact:

```bash
cargo run -p hadamard-cli -- decompress lp --bucket-in outputs/examples/lp9-buckets.txt --max-pairs 64 --artifact-out outputs/examples/lp9-decompressed.txt
```

The current prototype decompressor now emits canonical exact pair representatives, applies multi-layer canonical-prefix pruning during expansion, reuses exact candidates through complementary Legendre-signature buckets, and reduces the length-`9` demo to a single canonical exact representative after only `2` exact pair checks.

## Research workflow

1. Prove the pipeline on smaller known cases.
2. Harden artifact/checkpoint formats and shard execution.
3. Improve spectral filtering and compression search.
4. Add Phase 2 decompression and stronger LP(333) pruning.
5. Expand the experimental API for novel methods and the SDS track.

## Test-first rule

Every major algorithmic step has to land with explicit correctness gates before it is trusted for larger searches. The milestone checklist lives in:

- [`docs/MILESTONES.md`](docs/MILESTONES.md)

In practice that means:

- new math or pruning logic gets unit tests first
- each stage must preserve known-valid smaller cases
- every serious path must still pass end-to-end construction and final Hadamard verification
- no 668-scale result is trusted unless the smaller-case ladder is green

## Known small case

The repo now ships with two known Legendre-pair fixtures:

- length `5`: `A = +--++`, `B = +-+-+`
- length `7`: `A = +--+-++`, `B = +--+-++`
- length `9`: `A = +---+-+++`, `B = +--++-+-+`

It also has a known compressed-valid projection at length `9` with factor `3`:

- `A = +---+-+++ -> [1, 1, -1]`
- `B = +--++-+-+ -> [3, -1, -1]`

They satisfy the Legendre-pair autocorrelation conditions and build Hadamard matrices of orders `12`, `16`, and `20` through the implemented 2cc construction. These fixtures are the current correctness ladder for the code.

## Roadmap

Short-term goals:

- add a genuinely faster PSD backend behind the new abstraction
- add stronger compressed LP filters
- introduce a decompressor interface for SAT/CAS and meet-in-the-middle approaches
- add more known structured cases to the fixtures
- turn the SDS route into a real matcher instead of parameter enumeration only

## Limitations

- exact LP search is intentionally limited to small sizes
- compressed search now applies staged candidate-compatibility, exact PSD-signature, residual, and PSD-consistency filters
- decompression exists for small exact recovery from bucket artifacts and now canonicalizes exact pair outputs with branch-pruning and exact-signature metrics, but it is still a prototype rather than a production decomposer
- no external solver or FFT dependency is required in this first pass, which keeps builds simple but leaves performance on the table

## Development notes

See:

- [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md)
- [`docs/roadmap.md`](docs/roadmap.md)
- [`docs/MILESTONES.md`](docs/MILESTONES.md)
- [`docs/TODO.md`](docs/TODO.md)
- [`docs/algorithms/legendre-pair.md`](docs/algorithms/legendre-pair.md)
- [`docs/algorithms/sds-167.md`](docs/algorithms/sds-167.md)
- [`docs/runbook.md`](docs/runbook.md)
