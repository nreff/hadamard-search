# TODO

Primary dashboard:

- see [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md) for current state and priority order

Execution rule:

- Do not mark any item complete until its milestone gates in `docs/MILESTONES.md` pass.

## Current frontier attack: order 2060

- Treat `2060 = 4*515` as the least unknown order on the current public,
  verifiable record after the twelve new sub-`2000` witnesses. Keep the precise
  source derivation and the old unverified Miyamoto claim documented in
  [`docs/research/hm-2060-frontier.md`](docs/research/hm-2060-frontier.md).
- [x] Export and independently audit the exact pseudo-Boolean shard for cyclic
  Goethals--Seidel seeds over `Z_515` invariant under the order-`17` multiplier
  subgroup. A SAT model must pass full-length support, periodic-correlation, GS,
  and final matrix verification before any existence claim.
- [x] Normalize all feasible singleton-support choices by exact translations and a
  common dilation, add safe residual-reflection leaders, and test that the reduced
  single OPB instance still covers the complete order-`17`-invariant family.
- [x] Add a deterministic orbit-swap tabu baseline whose zero objective is checked
  independently over all `514` nonzero shifts. Its first bounded run reached exact
  squared defect `24` but found no witness.
- Run a proof-producing PB portfolio on that shard. A checked UNSAT proof excludes
  only this multiplier-invariant construction family; a timeout is no conclusion.
- In parallel, prototype the `BS(52,51) -> T(103) -> Cooper--Wallis with W(5)`
  route and a direct order-`103` amicable/Williamson-family search.
- Do not spend compute on the apparent symmetric-block Corollary-4 route in
  Miyamoto: at the required internal order `51`, it would imply a `25`-regular
  graph on `51` vertices and is parity-impossible.

## Current priorities after the order-668 result

- Preserve a durable, deterministic reconstruction trail from the primary X
  [`+/-` payload](https://x.com/__alpoge__/status/2087504785952182273) and
  [`decoder/generator`](https://x.com/__alpoge__/status/2087504788938510427), with
  hashes for source text, decoded instructions, intermediates, and the final matrix.
- Determine and document how the particular cyclic type-`H_4^*` difference family
  `(166;82,83,83,83;164)` was discovered or searched for. Do not infer that it came
  from a named Spence relative-difference-set family merely from use of the generic
  Wallis–Whiteman bordered array.
- Extend the already-documented circulant/back-diagonal block structure with
  normalization, automorphisms, ranks, and equivalence fingerprints of
  `fixtures/known/hm-668/matrix.txt`.
- Track updates to the same-day public construction and attribution account. Keep
  that public-history uncertainty separate from the exact two-verifier check of all
  `222,778` unordered off-diagonal pairs and from the still-open general Hadamard
  conjecture.

Completed structural gate:

- [x] split the first `664` signs of the extracted primary sign stream into four
  length-`166` sequences; verify row sums `[2,0,0,0]`, combined nonzero-shift PAF
  `-4`, cyclic type-`H_4^*`
  parameters, the bordered-array proof, and byte-identical native reconstruction
- [x] independently reconstruct the same bytes with the standard-library Python
  implementation, without the public shell generator or Rust builder, and reproduce
  SHA-256 `bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0`

## Historical search and methods backlog

- Preserve the LP(333), fixed-`q`, CRT, and SDS artifacts and benchmarks as
  reproducible research history; resume expensive witness searches only when they
  answer a distinct structural or algorithmic question.
- Complete the exported `14`-shard fixed-`q` OPB campaign with proof-producing PB
  solvers; preserve a verified model or one independently checked UNSAT proof per
  reversal representative, with the exact lag-parity lemma and tested OPB
  isomorphism covering all `42` raw cases (or retain `42` standalone proofs).
- Prototype the staged `128 -> 256 -> 512 -> 1024` modular lift described in `docs/research/modular-golay-167.md`.
- Benchmark exact finite-field root-of-unity fingerprints inside the factorized LP tail join at reduced lengths `11`, `15`, and `17`.
- Reduce the `1,296` multiplier row-marginal pairs to their `19` safe symmetry
  representatives per subgroup, then export the resulting `38` weighted-PB shards
  with independent full-length model verification.
- Enumerate invariant column marginals and join them to the `1,296` surviving row marginals with an exact signed-table realization test.
- Prototype a correlation-graph separator factorization before implementing a fixed contiguous `4+4+4` tail split.
- Replace the materializing SDS toy matcher with a measurable `Z_167` campaign and exact final projection.
- Bound compressed candidate generation itself in campaign configs; `max_attempts` currently limits only post-generation pairing.
- Replace the saturated `u64` exact-pair cursor before claiming exhaustive coverage above length `32`.
- Replace rounded floating PSD "exact signatures" with exact autocorrelation or finite-field signatures in correctness-critical filters.
