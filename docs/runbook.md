# Runbook

## Known-case validation

Run:

```bash
cargo test
cargo run -p hadamard-cli -- test-known lp-small
cargo run -p hadamard-cli -- test-known lp-seven
cargo run -p hadamard-cli -- test-known lp-nine
cargo run -p hadamard-cli -- test-known lp-eleven
cargo run -p hadamard-cli -- build 2cc --a +--++ --b +-+-+ --output outputs/examples/hm12.txt
uv run python py/validate_matrix.py outputs/examples/hm12.txt
```

## Small exact LP search

```bash
cargo run -p hadamard-cli -- search lp --length 5 --max-attempts 1024 --artifact-out outputs/examples/lp5.txt --checkpoint-out outputs/examples/lp5.chk
```

Resume:

```bash
cargo run -p hadamard-cli -- search lp --length 5 --resume outputs/examples/lp5.chk --max-attempts 1024
```

## Compressed search and decompression demo

```bash
cargo run -p hadamard-cli -- search lp --length 9 --compression 3 --max-attempts 256 --bucket-out outputs/examples/lp9-buckets.txt
cargo run -p hadamard-cli -- decompress lp --bucket-in outputs/examples/lp9-buckets.txt --max-pairs 64 --artifact-out outputs/examples/lp9-decompressed.txt
```

## Artifact handling

- checkpoint files are plain text and versioned
- artifact files are line-based and intended to remain human-inspectable during early research
- generated files should be written under `outputs/`, not the repository root
- Python-side helper scripts should be run via `uv run ...`, not plain `python ...`

## Production guidance

- Do not treat compressed-mode hits as proofs.
- Do not trust any negative result at larger lengths until the same code path has passed known smaller cases.
- Keep configs, checkpoints, and final artifacts together for reproducibility.
