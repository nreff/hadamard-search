![hadamard banner](docs/assets/hadamard-banner.png)

# hadamard

Research-grade tooling for computational Hadamard-matrix research, originally
centered on the then-open order **668**.

This repository preserves an independently verified `668 x 668` matrix at
[`fixtures/known/hm-668/matrix.txt`](fixtures/known/hm-668/matrix.txt). Public
constructions have since closed orders `2060` and `2092`. The smallest order
without a known Hadamard matrix in the current public record is `2396 = 4*599`.
The active construction search targets `4412 = 4*1103` because its multiplier
family is better suited to our search tools. The general Hadamard conjecture
remains open. See the [order-2396 frontier](docs/research/hm-2396-frontier.md) and
the [order-4412 campaign](docs/research/hm-4412-campaign.md).

If you are new to the repo, read these first:

1. [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md)
2. [`docs/RESEARCH_STATUS.md`](docs/RESEARCH_STATUS.md)
3. [`docs/METHODS_NOTE_OUTLINE.md`](docs/METHODS_NOTE_OUTLINE.md)
4. [`docs/MILESTONES.md`](docs/MILESTONES.md)
5. [`docs/runbook.md`](docs/runbook.md)
6. [`docs/EXPERIMENT_LOG.md`](docs/EXPERIMENT_LOG.md)

The repo is built around a practical research stance:

- preserve the exact order-`668` artifact and its primary-source reconstruction
- analyze the identified construction and investigate the discovery provenance of
  its cyclic seed
- retain the **Legendre-pair / 2cc** and **cyclic SDS / Goethals-Seidel** search
  programs as reproducible historical method work and possible tools for other orders
- require every serious computational claim to have independent exact checks

## Why 668

Order `668 = 2 * 333 + 2` is a natural 2cc target: a Legendre pair of length `333` would yield a Hadamard matrix of order `668`.

The included research report also identifies a second route through Goethals-Seidel / SDS constructions over `Z_167`, and lists the feasible row-sum parameter sets forced by the square-sum constraint.

On `2026-08-12`, the public order-`668` result was reconstructed from Levent
Alpöge's primary X
[`+/-` payload](https://x.com/__alpoge__/status/2087504785952182273) and
[`decoder/generator`](https://x.com/__alpoge__/status/2087504788938510427). Two
independent exact verifiers confirmed that the resulting file has `668` rows and
`668` columns, contains only `+1` and `-1`, and has zero dot product for all
`222,778` unordered pairs of distinct rows. Its SHA-256 is
`bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0`.
See the fixture's [provenance and verification
record](fixtures/known/hm-668/README.md).

[Epoch AI's order-668 status page](https://epoch.ai/frontiermath/open-problems/hadamard)
reports the target as solved. Because that report and its public attribution are a
same-day result, the public historical account may still be revised; that
provisional status is separate from the exact verification of the repository
artifact.

The construction itself is now identified exactly. The first `664` signs of the
extracted primary sign stream are four length-`166` sequences whose negative
supports form a cyclic type-`H_4^*`
difference family `(166;82,83,83,83;164)`; equivalently, their positive supports
form `(166;84,83,83,83;166)`. The standard Wallis–Whiteman bordered array
then gives the stored order-`668` matrix byte for byte. See [the construction
note](docs/research/hm-668-construction.md). What remains open is how this particular
`Z_166` seed was discovered or searched for, not why the resulting bordered array
is Hadamard. The general Hadamard conjecture also remains open.

Primary background material lives in:

- [`docs/research/deep-research-report.md`](docs/research/deep-research-report.md)
- [`docs/research/hm-668-construction.md`](docs/research/hm-668-construction.md) for
  the exact cyclic difference family, bordered array, and orthogonality proof
- [`docs/research/hm-2396-frontier.md`](docs/research/hm-2396-frontier.md) for the
  current public record and exact subgroup screens
- [Open orders and research problems](docs/research/open-orders-and-problems.md)
  for the Explorer, the survey's open problems, and the search shortlist
- [Order-4412 campaign](docs/research/hm-4412-campaign.md) for the active
  construction search, controls, and reproducible local checks
- [`docs/research/hm-2060-frontier.md`](docs/research/hm-2060-frontier.md) for the
  historical multiplier-orbit GS attack and verification of Schneider's witness
- [`docs/research/modular-golay-167.md`](docs/research/modular-golay-167.md) for the exact lift of the published 64-modular order-668 near-construction

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
- a Goethals-Seidel constructor, checked end to end on a small cyclic SDS
- an exact fixed-`q` pseudo-Boolean reduction of the published length-`167`
  64-modular Golay quadruple, with a `42`-case modulo-`4` partition and
  exact lag-parity sieve leaving `22` cases and solver-neutral restricted-OPB
  export reduced to `14` reversal representatives, plus a separately labeled
  `1,642`-variable `T`-fixed construction subfamily
- reproducible LP search config-file support for benchmark and campaign setup
- five known-case fixtures proving the end-to-end `LP -> 2cc -> Hadamard` path
- an independently verified order-`668` fixture with exact source provenance,
  artifact digest, and two independent all-row-pairs checks
- a native `build_spence_hadamard` path from the compact `4 x 166` seed, with exact
  `H_4^*`/PAF validation and byte-identical reconstruction of the stored fixture

It is not a production-strength `LP(333)` solver. That search work remains useful as
a documented computational-methods program, but it is no longer the primary path to
establishing existence at order `668`.

For the clearest current snapshot with checkboxes and next steps, see:

- [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md)
- [`docs/RESEARCH_STATUS.md`](docs/RESEARCH_STATUS.md) for the detailed audit of what is verified, what feels novel, and what is still speculative
- [`docs/METHODS_NOTE_OUTLINE.md`](docs/METHODS_NOTE_OUTLINE.md) for the paper-style structure of the current computational-methods story
- [`docs/EXPERIMENT_LOG.md`](docs/EXPERIMENT_LOG.md) for tried, rejected, and adopted experimental search ideas

## Repository layout

- `crates/core`: exact math, sequences, matrices, checkpoint and artifact formats
- `crates/search`: search runners, sharding, compressed candidate enumeration
- `crates/construct`: 2cc, Goethals–Seidel, and Wallis–Whiteman builders
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

Re-check the verified order-`668` fixture:

```bash
uv run python py/validate_matrix.py fixtures/known/hm-668/matrix.txt
```

Audit and export the historical exact order-`2060` multiplier shard:

```bash
uv run python py/search_sds_515.py
uv run python py/search_sds_515_reduced.py \
  --export-opb /tmp/sds515-h17-reduced.opb
uv run python py/test_search_sds_515_reduced.py
```

Reconstruct it from the compact cyclic seed through the native bordered-array
builder:

```bash
cargo run -p hadamard-cli -- build spence \
  --sequences-in fixtures/known/hm-668/spence-sequences.txt \
  --output /tmp/hm668-spence.txt
cmp /tmp/hm668-spence.txt fixtures/known/hm-668/matrix.txt
uv run python py/reconstruct_hm668.py \
  fixtures/known/hm-668/spence-sequences.txt /tmp/hm668-python.txt
cmp /tmp/hm668-python.txt fixtures/known/hm-668/matrix.txt
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

The current seam-aware reduced-length-`11` (`length 33`, factor `3`) benchmark
eliminates branching, checks `8399` exact tails, emits `1` pair, and completed in
`9.77` seconds in the recorded configuration. The packed reduced-length-`15`
anchor checks `129335` tails after `48` branches and has a best recorded time of
`12.60` seconds. See [`docs/RESEARCH_STATUS.md`](docs/RESEARCH_STATUS.md) for the
full progression and claim boundaries.

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

1. Preserve the primary-source material, decoded reconstruction, and exact digests.
2. Reproduce the order-`668` artifact through independent implementations.
3. Preserve the exact `H_4^*`/Wallis–Whiteman structural explanation and proof.
4. Determine how the particular cyclic `Z_166` seed was discovered and compute
   invariants that support classification and equivalence comparisons.
5. Preserve and selectively extend the historical LP, SDS, and fixed-`q` search
   program as computational-methods research.

## Test-first rule

Every major algorithmic step has to land with explicit correctness gates before it is trusted for larger searches. The milestone checklist lives in:

- [`docs/MILESTONES.md`](docs/MILESTONES.md)

In practice that means:

- new math or pruning logic gets unit tests first
- each stage must preserve known-valid smaller cases
- every serious path must still pass end-to-end construction and final Hadamard verification
- no large matrix artifact is trusted without an exact digest and an independent
  all-pairs orthogonality check

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

Separately, the repository ships the verified target fixture at
[`fixtures/known/hm-668/matrix.txt`](fixtures/known/hm-668/matrix.txt). It is an
externally sourced order-`668` result, not an output of the LP correctness ladder;
its own [provenance record](fixtures/known/hm-668/README.md) documents the primary
sources, exact digest, and independent verification.

## Roadmap

Short-term goals:

- improve and calibrate the `4412` order-`19` cyclic Goethals–Seidel search,
  using Kumar's order-`2092` witness as a positive control
- make the primary-source decoding and artifact provenance maximally reproducible
- preserve and cross-check the now-identified cyclic `H_4^*` bordered construction
- determine the search or discovery provenance of its particular `Z_166` seed
- analyze the verified matrix for block structure, symmetries, and relationships to
  known Hadamard constructions
- preserve the LP(333), fixed-`q`, and SDS search results as historical method work;
  extend them only when they answer a distinct structural or algorithmic question

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
