![hadamard banner](docs/assets/hadamard-banner.png)

# hadamard

Research-grade tooling for computational search toward new Hadamard matrices, starting with the open order **668**.

If you are new to the repo, read these first:

1. [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md)
2. [`docs/RESEARCH_STATUS.md`](docs/RESEARCH_STATUS.md)
3. [`docs/METHODS_NOTE_OUTLINE.md`](docs/METHODS_NOTE_OUTLINE.md)
4. [`docs/MILESTONES.md`](docs/MILESTONES.md)
5. [`docs/runbook.md`](docs/runbook.md)
6. [`docs/EXPERIMENT_LOG.md`](docs/EXPERIMENT_LOG.md)

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
- mixed-radix FFT PSD backend with direct-backend agreement tests
- Legendre-pair verification
- 2cc construction and Hadamard verification
- checkpointed search command for exact small-order LP search
- compressed candidate generation with staged candidate-compatibility, exact PSD-signature, residual, and PSD-consistency filters for future LP(333) decompression work
- a larger length-`15`, factor-`3` compressed benchmark artifact with checked pool metrics
- an experimental joint compressed-pair probe with exact norm feasibility and partial autocorrelation pruning
- corrected decompression prefix pruning that now recovers exact matches from the length-`15` benchmark artifact
- SDS block and difference-profile primitives plus a small meet-in-the-middle search scaffold
- reproducible LP search config-file support for benchmark and campaign setup
- five known-case fixtures proving the end-to-end `LP -> 2cc -> Hadamard` path

It is not yet a production-strength `LP(333)` solver. The design is ready for stronger pruning and Phase 2 SAT/CAS decompression.

For the clearest current snapshot with checkboxes and next steps, see:

- [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md)
- [`docs/RESEARCH_STATUS.md`](docs/RESEARCH_STATUS.md) for the detailed audit of what is verified, what feels novel, and what is still speculative
- [`docs/METHODS_NOTE_OUTLINE.md`](docs/METHODS_NOTE_OUTLINE.md) for the paper-style structure of the current computational-methods story
- [`docs/EXPERIMENT_LOG.md`](docs/EXPERIMENT_LOG.md) for tried, rejected, and adopted experimental search ideas

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
cargo run -p hadamard-cli -- test-known lp-eleven
cargo run -p hadamard-cli -- test-known lp-thirteen
```

Search for small exact Legendre pairs:

```bash
cargo run -p hadamard-cli -- search lp --length 5 --max-attempts 1024 --artifact-out outputs/examples/lp5.txt
```

Run the config-driven compressed baseline:

```bash
cargo run -p hadamard-cli -- search lp --config configs/baselines/lp15-compressed3-smoke.cfg --max-attempts 8
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

Run the small SDS matcher demo:

```bash
cargo run -p hadamard-cli -- search sds --order 5 --block-sizes 2,2,0,0 --lambda 1 --max-matches 4
```

Campaign-template configs for future `LP(333)` work live under [`configs/lp333`](configs/lp333).
Search and decompression artifacts now record run metadata such as `length`, `compression`, shard info, attempt bounds, and the active PSD backend.

Compare PSD backends:

```bash
cargo run -p hadamard-cli -- benchmark psd --sequence +--++ --backend direct
cargo run -p hadamard-cli -- benchmark psd --sequence +--++ --backend fft
cargo run -p hadamard-cli -- benchmark psd --sequence +--++ --backend autocorrelation
```

The default backend is now `fft`, while `direct` remains the correctness reference for backend-agreement tests.

Probe the experimental joint compressed search directly:

```bash
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 15 --compression 3 --max-pairs 32
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 33 --compression 3 --max-pairs 1
cargo run -p hadamard-cli -- benchmark compressed-pairs-mitm --length 15 --compression 3 --max-pairs 32
```

That joint-space probe is not yet the production LP search path, but it already recovers the known length-`15`, factor-`3` compressed projection and, with the current selected-frequency pair-PSD bound plus packed exact-tail lookup at depth `6`, reaches a first length-`33`, factor-`3` compressed pair after `1360` joint branches. A first MITM benchmark path is also in-tree for comparison and now emits rough state-memory estimates before longer runs; see [`docs/EXPERIMENT_LOG.md`](docs/EXPERIMENT_LOG.md) for the measured tradeoffs.

The current best verified direct-joint benchmark goes further: with factorized exact-tail completion at depth `11`, the reduced length-`11` (`length 33`, factor `3`) probe now eliminates branching entirely, checks `996305` exact tails, prunes `996304` of them spectrally, emits `1` pair, and completed in `14.19` seconds during the `2026-03-13` audit. See [`docs/RESEARCH_STATUS.md`](docs/RESEARCH_STATUS.md) for the full context and claim boundaries.

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

The current prototype decompressor now emits common-dihedral canonical exact pair representatives, applies corrected canonical-prefix pruning during expansion, reuses exact candidates through complementary Legendre-signature buckets, and recovers `3` exact matches on the length-`9` demo.

Run the larger compressed benchmark:

```bash
cargo run -p hadamard-cli -- search lp --length 15 --compression 3 --max-attempts 32768 --artifact-out outputs/examples/lp15-compressed.txt --bucket-out outputs/examples/lp15-buckets.txt
cargo run -p hadamard-cli -- decompress lp --bucket-in outputs/examples/lp15-buckets.txt --max-pairs 4096 --artifact-out outputs/examples/lp15-decompressed.txt
```

At length `15`, the compressed stage is already useful: the branch-and-bound generator now emits `135` compressed candidates per side, `55` signature-bucket candidates per side, and `215` unique PSD-consistent compressed pairs after unordered pair deduplication. With corrected prefix pruning, dihedral canonicalization, and exact-signature complement pruning, the current decompressor shrinks to `43` exact candidates per side across `39` exact-signature buckets, checks `47` complementary pairs, and recovers `24` exact matches, including the known pair `++++-++-++----- / +++--++-+-+-+--`.

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

The repo now ships with five known Legendre-pair fixtures:

- length `5`: `A = +--++`, `B = +-+-+`
- length `7`: `A = +--+-++`, `B = +--+-++`
- length `9`: `A = +---+-+++`, `B = +--++-+-+`
- length `11`: `A = +++-++-+---`, `B = +++-++-+---`
- length `13`: `A = +++-+++-+----`, `B = +-+++--++-+--`

It also has a known compressed-valid projection at length `9` with factor `3`:

- `A = +---+-+++ -> [1, 1, -1]`
- `B = +--++-+-+ -> [3, -1, -1]`

They satisfy the Legendre-pair autocorrelation conditions and build Hadamard matrices of orders `12`, `16`, `20`, `24`, and `28` through the implemented 2cc construction. These fixtures are the current correctness ladder for the code.

## Roadmap

Short-term goals:

- add stronger compressed LP filters
- introduce a decompressor interface for SAT/CAS and meet-in-the-middle approaches
- add more known structured cases to the fixtures
- scale the SDS route past the current toy matcher toward `Z_167`

## Limitations

- exact LP search is intentionally limited to small sizes
- compressed search now applies staged candidate-compatibility, exact PSD-signature, residual, and PSD-consistency filters
- decompression exists for small exact recovery from bucket artifacts and now canonicalizes exact pair outputs with branch-pruning and exact-signature metrics, but it is still a prototype rather than a production decomposer
- no external solver dependency is required in this first pass, and the FFT backend is implemented in-tree to keep builds simple

## Development notes

See:

- [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md)
- [`docs/roadmap.md`](docs/roadmap.md)
- [`docs/MILESTONES.md`](docs/MILESTONES.md)
- [`docs/TODO.md`](docs/TODO.md)
- [`docs/algorithms/legendre-pair.md`](docs/algorithms/legendre-pair.md)
- [`docs/algorithms/sds-167.md`](docs/algorithms/sds-167.md)
- [`docs/runbook.md`](docs/runbook.md)
