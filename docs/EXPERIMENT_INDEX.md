# Experiment Index

Last updated: 2026-07-02

Recorded experiments: 46

Source of truth: [EXPERIMENT_LOG.md](EXPERIMENT_LOG.md). This index is the
accounting layer: it tracks count, date, current status, area, and the shortest
useful finding for each recorded experiment.

## Counting Rules

- Count each level-3 experiment section in `EXPERIMENT_LOG.md` once.
- Historical entries use stable `H###` IDs and `undated` dates unless the original
  run date is explicit in the log.
- New entries use `EXP-YYYY-MM-DD-NN` and include `Date:`, `Area:`, `Status:`,
  `Question:`, `Commands:`, `Finding:`, `Decision:`, and `Follow-up:`.
- If a broad historical section is split into a newly dated sub-experiment, add a
  new dated ID and increment the recorded count.

## Current Rollup

- Adopted: 16
- Rejected: 21
- Measured calibration: 4
- Tried but not promoted: 3
- Narrowed but not rejected: 1
- Deferred: 1

## Registry

| ID | Date | Status | Area | Experiment | Finding |
| --- | --- | --- | --- | --- | --- |
| H001 | undated | Rejected | Multiplier stabilizer | Full column-preserving subgroup | Over-constrains row bundles; `0 / 504` ordered bundled exact pairs survive. |
| H002 | undated | Adopted | Multiplier stabilizer | Row action `row_units={1,4,7}` | First viable multiplier subfamily target; leaves `6,048` exact pure-row matches. |
| H003 | undated | Rejected | Multiplier stabilizer | Column-trivial order-`3` subgroup `{1,112,223}` | Actual `(alpha,0)` row-dot marginal leaves `0` row-marginal pairs. |
| H004 | undated | Narrowed | Multiplier stabilizer | Non-column-trivial `row_units={1,4,7}` subgroups | Fixed-row marginal leaves `1,296` pairs; mixed CRT equations remain necessary. |
| H005 | undated | Deferred | Multiplier shift marginals | Exact `(0,1)` column-shift marginal | Corrected arbitrary-row table is non-pruning and exact path remains opt-in because it is too slow. |
| H006 | undated | Rejected | CRT pair lift | Naive norm-plus-one-shift materialization inside `W`-frontier buckets | Too expensive as a default path; `W`-frontier batching remains useful offline. |
| H007 | undated | Rejected | CRT component lift | Shared-state reuse in representative hub component | No cross-spoke overlap under full `UV`, coefficient-only, or coarse `W` signatures. |
| H008 | undated | Rejected | CRT row-shift lift | Naive exact row-shift lift above row-bundle sieve | Carrying more shifts destroys the useful `W`-frontier aggregation. |
| H009 | undated | Rejected | Compressed symmetry | Independent compressed dihedral canonicalization | Not safe per side; removed known-valid length-`15` representatives. |
| H010 | undated | Rejected | Joint symmetry | Partial pair-space canonicalization | Prefix canonicality over-pruned the known length-`15` compressed projection. |
| H011 | undated | Tried | Spectral pruning | Full nonzero-frequency prefix checks to reduced length `33` | No pruning improvement on the measured benchmark and higher per-node cost. |
| H012 | undated | Rejected | Compressed generator | Count-profile-first compressed generation | Correct but slower than the simpler branch-and-bound generator at length `33`. |
| H013 | undated | Adopted | Direct compressed pair probe | Exact joint squared-norm feasibility | Completeness-safe and cuts impossible pair prefixes before residual evaluation. |
| H014 | undated | Adopted | Direct compressed pair probe | Endpoint-aware partial autocorrelation intervals | Tightens partial autocorrelation bounds and became part of the direct probe baseline. |
| H015 | undated | Tried | Meet-in-the-middle | Contiguous split MITM joint compressed search | Complete on the known case but much worse than direct search at length `33`. |
| H016 | undated | Rejected | Meet-in-the-middle | Non-contiguous parity-split MITM | Parity split gives the same pressure as contiguous splitting; correlation-aware splits are needed. |
| H017 | undated | Rejected | Meet-in-the-middle | Exact even-shift parity signatures | Invalid for odd cyclic lengths; over-pruned all emitted pairs. |
| H018 | undated | Rejected | Direct compressed pair probe | Cycle-generator assignment order | Generator order preserved correctness but gave the same frontier as natural order. |
| H019 | undated | Adopted | Direct compressed pair probe | Selected-frequency pair-PSD bounds | Safe improvement; four frequencies are the useful measured default. |
| H020 | undated | Adopted | Direct compressed pair probe | Exact remaining-sum reachability | Cleaner and safe, though it did not add pruning on the current benchmark. |
| H021 | undated | Adopted | Direct compressed pair probe | Exact tail-completion oracle | Strongest branch-count improvement; shifts bottleneck to exact tail-candidate multiplicity. |
| H022 | undated | Adopted | Direct compressed pair probe | Factorized depth-`7` tail oracle | Makes deep exact tails practical without monolithic raw tables. |
| H023 | undated | Adopted | Tail oracle | Cached segment spectral rejection | Reduces tail-dominated runtime by avoiding unnecessary decode/stitch work. |
| H024 | undated | Adopted | Tail oracle | Raw exact residual check before `CompressedSequence` construction | Further reduces per-candidate overhead without changing counts. |
| H025 | undated | Measured | Tail oracle anchor | First reduced-length-`15` direct-joint anchor | Completed in `158.94s`; confirms reduced length `15` is tail-dominated. |
| H026 | undated | Adopted | Tail oracle | Separate per-side norm keys | Cuts tail multiplicity and makes depth `12` worthwhile on reduced length `15`. |
| H027 | undated | Rejected | Tail oracle | Adaptive factorized split selection | Same counts as fixed split and extra estimator overhead. |
| H028 | undated | Adopted | Tail oracle | Exact shift-`1` seam key | Major breakthrough: reduced length `15` tail checks drop from about `90.7M` to `129k`. |
| H029 | undated | Measured | Tail oracle anchor | Next anchors after shift-`1` seam key | Reduced length `17` becomes a real timing anchor; reduced length `21` is the barrier. |
| H030 | undated | Rejected | Tail oracle | Shift-`2` seam key | Correct but too expensive; slows tested anchors and is reverted. |
| H031 | undated | Measured | Tail oracle anchor | Tail-depth limit and reduced length `21` sweep | Deeper exact tail alone does not clear the reduced length `21` barrier. |
| H032 | undated | Adopted | Tail oracle | Exact small-shift tail prefilter | Kept only for larger reduced lengths; modest help at reduced length `17`, not a breakthrough. |
| H033 | undated | Rejected | Tail oracle | Small-shift signature bucketing | Great count reduction but worse wall-clock time from hash/signature overhead. |
| H034 | undated | Rejected | Tail oracle | Precompute right-tail small-shift summaries | Preserved correctness but worsened the reduced length `17` timing. |
| H035 | undated | Adopted | Tail oracle | Packed shift-`1` seam buckets | Constant-factor win by flattening the successful shift-`1` seam representation. |
| H036 | undated | Adopted | Tail oracle | Direct-indexed packed seam-boundary slots | Strong constant-factor improvement over packed buckets on reduced length `17`. |
| H037 | undated | Adopted | Tail oracle | Unified tail-join summary cache | Improves reduced length `17` anchor to `79.28s` in the measured configuration. |
| H038 | undated | Rejected | Tail oracle | Cache packed shift-`1` buckets by right-tail key | Reuse bookkeeping loses to cheaper local rebuilds. |
| H039 | undated | Rejected | Tail oracle | Direct-indexed internal-sum slots | Direct slot array does not beat the sorted-vector representation. |
| H040 | undated | Rejected | Tail oracle | Precomputed required shift-`1` seam totals | Extra per-left precompute does not pay for itself. |
| H041 | undated | Tried | Tail oracle | Exact tail-side selected-shift residual prefilter | Correctness-safe but no meaningful pruning on the measured benchmark. |
| H042 | undated | Rejected | Tail oracle | Heuristic low-energy ordering | Increases checked candidates and worsens runtime. |
| H043 | undated | Rejected | Tail oracle | Cross-branch factorized tail caches | Cross-branch cache overhead loses to per-probe local caches. |
| H044 | undated | Rejected | Tail oracle | Adaptive probe-side choice in exact `6+6` join | Symmetric probe-side logic is noncompetitive against the current anchor. |
| EXP-2026-07-02-01 | 2026-07-02 | Adopted | Multiplier shift profile | Linear-frontier order for shift-`(3,1)` profiles | Nonfixed compressed profiles complete exactly at the default cap; fixed side now needs denser signature storage, not larger caps. |
| EXP-2026-07-02-02 | 2026-07-02 | Measured | Multiplier shift lift | Exact shift-`(3,1)` lift for surviving order-`3` targets | Release-mode exact lift completes but keeps all `1,296` row pairs; single shift-`(3,1)` is non-pruning. |
