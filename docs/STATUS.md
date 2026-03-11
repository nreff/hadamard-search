# Status

This file is now a short companion to [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md).

## Implemented

- Rust workspace with `core`, `search`, `construct`, and `cli` crates
- `uv`-managed Python helper project for validation scripts
- dedicated `outputs/` directory for generated matrices, artifacts, and checkpoints
- exact `±1` sequence math and periodic autocorrelation
- pluggable PSD backend layer with `direct` and `autocorrelation` implementations
- Legendre-pair verification
- 2cc construction and Hadamard verification
- checkpoint/artifact text formats
- exact small-order LP search
- compressed candidate generation with candidate-compatibility, exact PSD-signature, residual, and PSD-consistency stages
- reusable compressed bucket artifact output for later decomposition work
- first exact decompression prototype from compressed bucket artifacts
- canonical exact pair output and multi-layer expansion-pruning metrics in decompression artifacts
- exact Legendre-signature bucketing for decompressed candidates before pair validation
- SDS-167 parameter enumeration
- three known LP fixtures with order-12, order-16, and order-20 validation paths

## Next

- genuinely faster PSD/DFT backend behind the current abstraction
- stronger compressed LP filters
- incremental autocorrelation-style decompression pruning
- more known structured fixtures
- SDS matcher implementation
