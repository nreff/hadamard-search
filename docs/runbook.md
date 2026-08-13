# Runbook

## Known-case validation

Run:

```bash
cargo test
cargo run -p hadamard-cli -- test-known lp-small
cargo run -p hadamard-cli -- test-known lp-seven
cargo run -p hadamard-cli -- test-known lp-nine
cargo run -p hadamard-cli -- test-known lp-eleven
cargo run -p hadamard-cli -- test-known lp-thirteen
cargo run -p hadamard-cli -- build 2cc --a +--++ --b +-+-+ --output outputs/examples/hm12.txt
uv run python py/validate_matrix.py outputs/examples/hm12.txt
cargo run -p hadamard-cli -- search lp --config configs/baselines/lp15-compressed3-smoke.cfg --max-attempts 8
```

## Small exact LP search

```bash
cargo run -p hadamard-cli -- search lp --length 5 --max-attempts 1024 --artifact-out outputs/examples/lp5.txt --checkpoint-out outputs/examples/lp5.chk
```

Resume:

```bash
cargo run -p hadamard-cli -- search lp --length 5 --resume outputs/examples/lp5.chk --max-attempts 1024
```

Checkpoint schema `v2` records the row-sum target. If an older ignored example
checkpoint reports `unsupported checkpoint version 1`, regenerate it with the first
command before resuming; it is intentionally not guessed or silently migrated.

## Fixed-`q` modular Golay campaign

```bash
uv run python py/search_modular_golay.py --audit-only
uv run python py/search_modular_golay.py --timeout-seconds 300
uv run python py/search_modular_golay.py --quotient-case-index 0 --timeout-seconds 300
uv run python py/search_modular_golay.py --export-opb-dir outputs/fixed-q-opb
uv run python py/search_modular_golay.py --export-t-fixed-opb-dir outputs/fixed-q-opb-t-fixed
uv run python py/search_modular_golay.py --export-opb-dir outputs/fixed-q-opb-all --opb-all-cases
uv run python py/test_search_modular_golay.py
roundingsat --lp=0 --bits-learned=1 --print-sol=1 --proof-log=outputs/fixed-q-case-00.pbp outputs/fixed-q-opb/fixed-q-mod4-case-00.opb > outputs/fixed-q-case-00.model
veripb outputs/fixed-q-opb/fixed-q-mod4-case-00.opb outputs/fixed-q-case-00.pbp
uv run python py/search_modular_golay.py --verify-opb-model outputs/fixed-q-case-00.model --output outputs/modular-golay-fixed-q.txt
roundingsat --lp=0 --bits-learned=1 --print-sol=1 outputs/fixed-q-opb-t-fixed/fixed-q-t-fixed-mod4-case-00.opb > outputs/fixed-q-t-fixed-case.model
uv run python py/search_modular_golay.py --verify-t-fixed-opb-model outputs/fixed-q-t-fixed-case.model --output outputs/modular-golay-fixed-q.txt
```

The cyclic quotient table has `42` exhaustive raw cases. Exact mod-`2` consequences
of the lag cardinalities reject `20`, leaving `22` parity-compatible cases; reversal
of the length-`81` sequence leaves `14` representatives. The default v3 manifest
records the parity exclusions and reversal coverage without renumbering the raw case
IDs. `--opb-all-cases` reproduces all `42` raw shards. Treat `unknown` and
`TIMELIMIT` as unresolved. A SAT model must pass `--verify-opb-model`; an UNSAT
campaign is complete only after all `14` representative proofs, the tested reversal
isomorphism, and the documented lag-parity lemma are checked. Use all `42` raw
shards when standalone per-case certificates are required.

`--quotient-case-index` uses the stable raw IDs and rejects a parity-excluded ID
before solving. RoundingSat timeout transcripts should verify as `NO CONCLUSION`;
only a terminal proof accepted as `UNSATISFIABLE` discharges a shard.

The `--export-t-fixed-opb-dir` campaign is an exact construction probe with `82`
independent signs, `1,642` total variables, and `6,288` constraints per shard. A
verified SAT model solves the original fixed-`q` problem; an UNSAT campaign excludes
only the explicitly labeled `T`-fixed subfamily.

## Compressed search and decompression demo

```bash
cargo run -p hadamard-cli -- search lp --length 9 --compression 3 --max-attempts 256 --bucket-out outputs/examples/lp9-buckets.txt
cargo run -p hadamard-cli -- decompress lp --bucket-in outputs/examples/lp9-buckets.txt --max-pairs 64 --artifact-out outputs/examples/lp9-decompressed.txt
```

## Larger compressed benchmark

```bash
cargo run -p hadamard-cli -- search lp --length 15 --compression 3 --max-attempts 32768 --artifact-out outputs/examples/lp15-compressed.txt --bucket-out outputs/examples/lp15-buckets.txt
cargo run -p hadamard-cli -- decompress lp --bucket-in outputs/examples/lp15-buckets.txt --max-pairs 4096 --artifact-out outputs/examples/lp15-decompressed.txt
cargo run -p hadamard-cli -- benchmark psd --sequence ++-++-+--++-+-- --backend fft
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 15 --compression 3 --max-pairs 32
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 33 --compression 3 --max-pairs 1
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 33 --compression 3 --ordering generator2 --max-pairs 1
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 33 --compression 3 --spectral-frequencies 1 --max-pairs 1
cargo run -p hadamard-cli -- benchmark compressed-pairs --length 33 --compression 3 --spectral-frequencies 10 --max-pairs 1
cargo run -p hadamard-cli -- benchmark compressed-pairs-mitm --length 15 --compression 3 --max-pairs 32
cargo run -p hadamard-cli -- benchmark compressed-pairs-mitm --length 33 --compression 3 --split contiguous --max-pairs 1
cargo run -p hadamard-cli -- search sds --order 5 --block-sizes 2,2,0,0 --lambda 1 --max-matches 4
cargo run -p hadamard-cli -- search lp --config configs/baselines/lp33-compressed3-smoke.cfg
```

Current baseline:

- the compressed benchmark produces `135` generated candidates and `55` signature-bucket candidates per side
- the length-`15` generator explores `940` branches per side, with `551` row-sum prunes and `20` spectral prunes
- the compressed stage leaves `215` unique PSD-consistent pairs after unordered compressed-pair deduplication
- the current decompressor expands `43` exact candidates per side across `39` exact-signature buckets, checks `47` complementary pairs, and recovers `24` common-dihedral-canonical exact matches from the broad length-`15` bucket artifact
- the default PSD backend is `fft`, with `direct` retained as the correctness reference backend
- the small SDS matcher over `Z_5` recovers normalized `(5;2,2,0,0;1)` examples through a shardable meet-in-the-middle path
- the length-`33`, factor-`3` smoke probe already reaches `293139` generated candidates per side after `2774492` branch extensions, so further pruning is still required before `333`
- the experimental direct compressed-pair probe recovers the known length-`15`, factor-`3` projection after `4960` joint branches
- with exact joint squared-norm pruning, endpoint-aware autocorrelation intervals, selected-frequency pair-PSD bounds, packed exact-tail lookup, and tail depth `6`, the natural-order direct probe reaches its first length-`33`, factor-`3` compressed pair after `1360` branches
- with the shift-`1` seam-aware factorized join and `1` monitored frequency, the reduced length-`11` benchmark now checks only `8399` tails and completes in `9.77s`
- the best recorded packed reduced length-`15` anchor is `length=45`,
  `compression=3`, `tail_depth=12`, `spectral_frequencies=1`, which reaches a first
  pair in `12.60` seconds after `48` branches and `129335` checked tail candidates
- natural order remains the preferred direct-probe benchmark; generator-`2` did not outperform it on the measured reduced length-`11` runs
- frequency sweep on the same benchmark suggests `4` monitored frequencies is the current sweet spot: `1` is slightly weaker and `10` is no better than `4`
- tail-depth sweep on the same benchmark:
  - depth `3`: `60704` branches
  - depth `4`: `20000` branches
  - depth `5`: `5712` branches
  - depth `6`: `1360` branches
  - factorized depth `7`: `272` branches
  - factorized depth `8`: `64` branches
  - historical factorized depth `11`: `0` branches but `996305` exact tail candidates checked
  - current separate-norm factorized depth `11`: `0` branches, `124981` exact tail candidates checked
- spectral-frequency sweep in the new seam-aware regime:
  - reduced length `15`, tail depth `12`, `K=4`: `17.29s`, `129356` checked tails
  - reduced length `15`, tail depth `12`, `K=1`: `14.13s`, `129335` checked tails
  - reduced length `15`, tail depth `12`, `K=0`: `14.29s`, `129337` checked tails
  - reduced length `17`, tail depth `12`, `K=1`: `103.67s`, `223664` checked tails
  - reduced length `17`, tail depth `12`, `K=0`: `97.27s`, `223676` checked tails
- exact small-shift tail prefilter in the seam-aware regime:
  - the direct probe now reports `effective_tail_depth`, because the exact factorized tail path is only exact through depth `12`
  - reduced length `15`, tail depth `12`, `K=1`: the new `shift 2..4` prefilter is worse and is therefore disabled on this size (`16.11s`, `tail_shift_pruned=0`)
  - reduced length `17`, tail depth `12`, `K=0`: the same prefilter is enabled and is a modest win (`95.87s`, `tail_shift_pruned=223531`)
- staged timing anchor beyond reduced length `11`:
  - reduced length `15` (`length 45`, factor `3`, tail depth `11`, combined norm key): `158.94s`, `160` branches, `96096005` tail candidates checked
  - reduced length `15` (`length 45`, factor `3`, tail depth `12`, separate per-side norm key): `151.08s`, `48` branches, `90668636` tail candidates checked
  - reduced length `15` (`length 45`, factor `3`, tail depth `12`, shift-1 seam-aware join, `K=1`): `14.13s`, `48` branches, `129335` tail candidates checked
  - reduced length `17` (`length 51`, factor `3`, tail depth `12`, shift-1 seam-aware join, `K=1`): `103.67s`, `768` branches, `223664` tail candidates checked
  - reduced length `17` (`length 51`, factor `3`, tail depth `12`, shift-1 seam-aware join, `K=0`, plus the new exact small-shift tail filter): `95.87s`, `768` branches, `223670` tail candidates checked
  - reduced length `21` (`length 63`, factor `3`, tail depth `12`, shift-1 seam-aware join, `K=1`): exceeded a `300s` cap
  - reduced length `21` (`length 63`, factor `3`, tail depth `12`, shift-1 seam-aware join, `K=0`): exceeded a `300s` cap
  - reduced length `21` (`length 63`, factor `3`, tail depth `11` or `13`, `K=0`): also exceeded a `300s` cap
  - reduced length `21` (`length 63`, factor `3`, tail depth `12`, `K=0`, with the new exact small-shift tail filter): still exceeded a `300s` cap
- the experimental contiguous-split MITM probe is complete on length `15` but not yet competitive at length `33`; keep it as a benchmark path, not a preferred search mode
- the MITM benchmark now emits rough state-memory estimates; on the reduced length-`11` contiguous split, the stored half-state payload is already about `121 MB` before map/vector overhead, so use these estimates before starting long jobs

## Artifact handling

- checkpoint files are plain text and versioned
- artifact files are line-based and intended to remain human-inspectable during early research
- config files under `configs/` are line-based `key=value` inputs for reproducible LP searches
- LP artifacts now carry explicit run metadata such as `length`, `compression`, shard info, attempt bounds, and `psd_backend`
- bucket artifact parsing rejects unknown schema versions instead of silently accepting them
- generated files should be written under `outputs/`, not the repository root
- Python-side helper scripts should be run via `uv run ...`, not plain `python ...`

## Config-driven workflow

For repeatable runs, prefer:

```bash
cargo run -p hadamard-cli -- search lp --config configs/baselines/lp15-compressed3-smoke.cfg --max-attempts 8 --artifact-out outputs/examples/lp15-config-smoke.txt
```

For future `LP(333)` campaign setup, start from the templates under `configs/lp333/` and override only the flags that need to change for a specific shard or attempt cap.

## Production guidance

- Do not treat compressed-mode hits as proofs.
- Do not treat the `benchmark compressed-pairs` path as production search yet; it is an experimental joint-space probe.
- Before launching long MITM-style experiments, record the benchmarked branch counts and estimated state bytes from a smaller matching split strategy; do not start an hours-long job without that estimate.
- Do not trust any negative result at larger lengths until the same code path has passed known smaller cases.
- Keep configs, checkpoints, and final artifacts together for reproducibility.
- If you need the detailed audit of what is verified versus speculative, read [RESEARCH_STATUS.md](/home/nate/projects/hadamard/docs/RESEARCH_STATUS.md) before launching a long run.
