# Experiment Log

This file records implementation directions that were tried, measured, and then rejected or deprioritized.

## Compressed LP search

### Rejected: independent compressed dihedral canonicalization

Attempt:
- quotient compressed candidates by rotation and reversal before pair matching

Why it looked promising:
- compressed PSD and compressed residual checks are invariant under dihedral actions
- removing orbit duplicates could have reduced the candidate pool substantially

Why it was rejected:
- the search stage does not only care about single-sequence invariants; downstream pair matching and decompression still depend on the relative orientations that survive into the bucket artifact
- in practice, this change removed known-valid length-`15` compressed representatives and caused decompression recovery to fail

Outcome:
- reverted
- compressed symmetry should only be factored out in a pair-aware way, not independently per side

### Rejected: partial pair-space canonicalization

Attempt:
- quotient the experimental joint compressed `(A, B)` prefix search by a partial A/B swap rule and a "first assigned pair-column is lexicographically minimal" common-rotation rule

Why it looked promising:
- the direct pair probe has obvious common symmetries, so removing them early looked like a cheap way to shrink the joint search tree

Why it was rejected:
- the partial rule was not complete: it pruned the known length-`15`, factor-`3` compressed projection before the full pair was visible
- the issue is that local prefix order is not a safe proxy for global common-rotation canonicality

Outcome:
- reverted after the known-case recovery test failed
- any future symmetry reduction in joint space must be justified against full completed pairs, not guessed from prefixes

### Tried: full nonzero-frequency prefix checks up to reduced length `33`

Attempt:
- expand the incremental spectral pruning set from a small low-frequency subset to all nonzero frequencies for reduced lengths up to `33`

Why it looked promising:
- if the current lower bound was missing decisive high-frequency obstructions, checking all frequencies should have cut the generated pool noticeably

Why it was not pursued further:
- on the current length-`33`, factor-`3` smoke probe, the measured pruning counts were unchanged
- this increased per-node work without changing the frontier on the observed benchmark

Outcome:
- deprioritized as a primary scaling lever
- low-frequency pruning remains useful, but stronger structural constraints are needed

### Rejected: count-profile-first compressed generator

Attempt:
- enumerate feasible compressed symbol count profiles first, then generate sequences within each profile using exact remaining squared norm and exact remaining spectral budget

Why it looked promising:
- row-sum feasibility becomes exact by construction
- spectral bounds can use profile-exact remaining mass instead of a coarse per-slot bound

Why it was rejected:
- the added profile-management overhead made the larger length-`33`, factor-`3` smoke probe slower than the simpler branch-and-bound generator
- it preserved correctness on the small known cases, but it did not improve the practical frontier enough to justify replacing the simpler implementation

Outcome:
- reverted
- exact count profiles may still be useful later inside a pair-aware or meet-in-the-middle generator, but not as the top-level single-sequence search driver

### Adopted: exact joint squared-norm feasibility in pair-aware compressed search

Attempt:
- use the identity
  - `sq(A) + sq(B) = 2 * row_sum^2 + 2f(n - 1)`
  for compressed length `n`, compression factor `f`, and equal row-sum target on both sides
- precompute exact remaining `(row_sum, squared_norm)` tables and prune pair prefixes that cannot complete to the required total squared norm

Why it worked:
- unlike the rejected symmetry rule, this is a consequence of the compressed Legendre-pair equations themselves
- it cuts impossible branches before full residual evaluation without discarding known-valid projections

Observed effect:
- preserved the known length-`15`, factor-`3` compressed projection
- reduced the experimental length-`33`, factor-`3` pair-aware probe from `780752` branches to `441584` branches when combined with the refined autocorrelation bound below

Outcome:
- kept
- this is now part of the experimental direct compressed-pair probe

### Adopted: endpoint-aware partial autocorrelation intervals

Attempt:
- replace the coarse unresolved-shift slack `unresolved * 2f^2` with exact per-index intervals based on whether each endpoint is:
  - assigned on both sides
  - assigned on one side only
  - unassigned on both sides

Why it worked:
- partially assigned columns do not have full `2f^2` freedom; their remaining contribution is limited by the magnitudes already fixed in the prefix
- this gives a materially tighter bound in the joint `(A, B)` search tree

Observed effect:
- together with exact joint squared-norm pruning, the direct length-`33`, factor-`3` probe now reaches its first compressed pair after `441584` branches, `166986` row-sum prunes, `222387` norm prunes, and `24532` autocorrelation prunes

Outcome:
- kept
- joint-space pruning is now strong enough to benchmark directly against the older "generate single candidates then pair them" pipeline on reduced length `11`

### Tried: contiguous split meet-in-the-middle joint compressed search

Attempt:
- split the compressed joint `(A, B)` search into a left half and right half
- enumerate half-states with exact global row-sum and combined squared-norm feasibility
- join halves by exact `(sum_a, sum_b, sq_a + sq_b)` completion, then check the full compressed residual only on joined candidates

Why it looked promising:
- it changes the search shape more substantially than another DFS prune
- exact join keys can collapse many half-prefixes before final residual checks

What happened:
- it is complete on the known length-`15`, factor-`3` case and reaches `32` emitted pairs after only `2656` branch extensions, `132` left states, `670` right states, and `196` join checks
- but on length `33`, factor `3`, the first implementation is not competitive yet:
  - `10540384` branch extensions
  - `545724` left states
  - `4582578` right states
  - `25536` joined candidates for the first emitted pair
- this is much worse than the current direct joint probe at the same size (`441584` branches for the first pair)

Why it was not promoted:
- the contiguous split does not preserve enough cross-half autocorrelation structure, so most of the pruning happens too late

Outcome:
- kept as an experimental benchmark path only
- future MITM work should use a better split, likely by autocorrelation-coupled coordinates or hashed boundary signatures rather than a naive left/right cut

### In progress: non-contiguous parity-split MITM

Hypothesis:
- splitting the compressed pair by index parity instead of by a contiguous left/right cut may preserve more of the cyclic autocorrelation structure inside each half-state
- if true, that should reduce late join explosion and give a more realistic time/memory estimate for larger reduced lengths

Planned evaluation:
- keep the same exact row-sum and combined squared-norm join keys
- expose emitted state counts and rough memory estimates before attempting any large job
- compare parity split directly against the existing contiguous split at reduced lengths `5` and `11`

Decision rule:
- only promote parity-split follow-up work if it improves the reduced length-`11` frontier materially over the contiguous split

Observed result:
- on reduced length `11` (`length 33`, factor `3`), parity split is not an improvement
- it produces the same branch count and join pressure as the contiguous split, only with left/right state counts swapped:
  - `10540384` branches
  - `4753310` norm prunes
  - `25536` join checks

Conclusion:
- parity alone is too weak a notion of "autocorrelation-coupled" structure
- future non-contiguous splits need to be driven by the correlation equations themselves, not by a cosmetic partition like even/odd indices

### Rejected: parity MITM join on exact even-shift signatures

Attempt:
- strengthen the parity-split MITM join by hashing half-states on exact compressed autocorrelation totals for even shifts

Why it looked promising:
- if even shifts stayed inside a parity class, those autocorrelation components would be exact half-state invariants and would make an excellent join key

Why it was rejected:
- the reduced compressed lengths of interest are odd, and cyclic wraparound by an even shift modulo an odd length does not preserve ordinary index parity
- the proposed key therefore over-pruned valid joins; on reduced length `11`, factor `3`, it dropped join checks from `25536` to `3712` but also eliminated all emitted pairs

Outcome:
- reverted
- any future correlation-aware join key has to respect the actual cyclic action modulo odd order, not an ambient integer parity heuristic

### In progress: cycle-generator assignment order for the direct joint probe

Hypothesis:
- the direct joint probe may prune earlier if assigned compressed columns are spread around the cycle instead of filled in natural index order
- for odd reduced lengths, stepping by a generator such as `2 mod n` visits every position and exposes more wraparound constraints earlier to the partial autocorrelation bounds

Planned evaluation:
- refactor the direct joint probe to assign into arbitrary position orders rather than only a contiguous prefix
- compare `natural` order against a generator walk on reduced lengths `5` and `11`
- treat this as promising only if the branch count drops without breaking the known length-`15` projection

Observed result:
- the sparse-order refactor preserved the known length-`15`, factor-`3` projection for both `natural` and generator-`2` orderings
- on reduced length `11` (`length 33`, factor `3`), generator-`2` produced the same frontier as natural order:
  - `441584` branches
  - `166986` row-sum prunes
  - `222387` norm prunes
  - `24532` autocorrelation prunes

Conclusion:
- merely spreading assignments around the cycle is not enough by itself
- the current bounds are effectively permutation-invariant under this change, so future progress needs stronger constraints, not just a different DFS order

### In progress: selected-frequency pair-PSD bounds in the direct joint probe

Hypothesis:
- for compressed LP pairs, the nonzero-frequency PSD target is fixed once the row-sum target and total squared norm are fixed
- if a partial assignment already forces `PSD_A(k) + PSD_B(k)` above that target for some selected frequencies, the branch can be pruned safely

Planned evaluation:
- add a small selected-frequency set to the direct joint probe
- use current Fourier accumulators plus remaining amplitude budget to derive a lower bound on attainable pair PSD
- compare the existing direct probe against the same probe with pair-PSD pruning on reduced lengths `5` and `11`

Observed result:
- preserved the known length-`15`, factor-`3` compressed projection
- had no effect on reduced length `5`
- on reduced length `11` (`length 33`, factor `3`), natural-order direct search improved from
  - `441584` branches
  to
  - `435296` branches
  with `393` spectral prunes
- frequency-count sweep on reduced length `11`:
  - `1` frequency: `435360` branches, `389` spectral prunes
  - `4` frequencies: `435296` branches, `393` spectral prunes
  - `10` frequencies: identical to `4` on this benchmark
- the bound interacts with assignment order:
  - `natural`: `435296` branches, `393` spectral prunes
  - `generator2`: `441520` branches, `4` spectral prunes

Conclusion:
- selected-frequency pair-PSD pruning is a real, completeness-safe improvement
- unlike the earlier order-only experiment, this bound makes natural-order assignment slightly preferable to generator-`2`
- `4` frequencies is currently the right default for this probe: better than `1`, no worse-case gain from `10` on the measured benchmark

### In progress: exact remaining-sum reachability

Hypothesis:
- the current row-sum interval check is leaving some impossible branches alive because it ignores the exact set of reachable remaining sums from the compressed alphabet
- replacing that interval with exact remaining-sum reachability should be a cheap, completeness-safe improvement that stacks with the newer spectral and autocorrelation bounds

Planned evaluation:
- add exact remaining-sum tables to the compressed generation and direct joint probe paths
- benchmark the natural-order direct probe again at reduced length `11`

Observed result:
- preserved the known length-`15`, factor-`3` compressed projection
- did not change the reduced length-`11` natural-order direct benchmark relative to the current pair-PSD baseline

Conclusion:
- exact remaining-sum reachability is harmless and mathematically cleaner than the old interval check
- but on the current direct benchmark it does not unlock any additional pruning beyond the stronger norm/autocorrelation/PSD filters already in place

### In progress: exact tail-completion oracle for the direct joint probe

Hypothesis:
- once only a few compressed columns remain, generic DFS branching is wasteful
- a precomputed table of exact tail pairs keyed by remaining sums and combined norm can replace the last part of the search tree with a small exact lookup

Planned evaluation:
- precompute tail pair tables up to a small remaining depth
- splice that oracle into the direct joint probe
- compare the reduced length-`11` benchmark against the current best natural-order direct probe

Observed result:
- preserved the known length-`15`, factor-`3` compressed projection
- reduced length `5` (`length 15`, factor `3`) dropped from `4960` branches to `96`
- reduced length `11` (`length 33`, factor `3`) dropped from `435296` branches to `60704`
- increasing the exact tail depth continued to pay off on reduced length `11`:
  - tail depth `3`: `60704` branches
  - tail depth `4`: `20000` branches
  - tail depth `5`: `5712` branches
  - tail depth `6`: `1360` branches
  - factorized tail depth `7`: `272` branches
  - factorized tail depth `8`: `64` branches
  - factorized tail depth `11`: `0` branches, but `996305` exact tail candidates checked
- on the reduced length-`11` benchmark the new profile is:
  - `60704` branches
  - `312` row-sum prunes
  - `50186` norm prunes
  - `0` autocorrelation prunes
  - `0` spectral prunes
  - exact tail lookup handles the rest

Conclusion:
- this is the strongest improvement so far on the direct joint probe
- a small exact tail oracle is much more effective than additional local prefix inequalities at the current reduced sizes
- compact tail storage makes depth `6` feasible and very effective
- factorized tail completion makes depth `7+` feasible without storing monolithic raw tables
- but by depth `11` the regime changes: branch count disappears, and exact tail-candidate volume becomes the dominant cost instead

### In progress: factorized depth-7 tail oracle

Hypothesis:
- depth `7` tail completion is still promising, but it should be expressed as an exact join of smaller tail tables rather than a monolithic raw table

Planned evaluation:
- compose depth `7` tails from exact `(3,4)` or `(4,3)` sub-tail joins keyed by remaining sums and combined norm
- benchmark the reduced length-`11` direct probe again before considering any deeper exact tail

Observed result:
- factorized tail completion made deeper exact tails practical without building monolithic raw tables
- on reduced length `11` (`length 33`, factor `3`):
  - factorized depth `7`: `272` branches
  - factorized depth `8`: `64` branches
  - factorized depth `11`: `0` branches, `996305` exact tail candidates checked

Conclusion:
- factorization is the reason the direct joint probe can now reach the tail-dominated regime at all
- but once depth `11` is reached, the next bottleneck is no longer branching or table construction
- it is candidate multiplicity inside exact tail keys

### Adopted: cached segment spectral rejection inside factorized tail completion

Attempt:
- in the factorized exact-tail path, reject joined left/right tail candidates using cached segment-level spectral contributions before decoding and stitching the full tail values

Why it looked promising:
- once depth `11` removed branching on the reduced length-`11` benchmark, most work was being spent materializing candidates that were about to fail the tail spectral check

Observed result:
- the search counts stayed exactly the same on the re-verified reduced length-`11` benchmark:
  - `0` branches
  - `996305` tail candidates checked
  - `996304` tail spectral prunes
  - `1` emitted pair
- runtime improved from `14.19s` to `12.64s`
- the `length=45`, `compression=3`, `tail_depth=11` capped anchor still timed out at `120s`

Conclusion:
- this is a useful implementation improvement in the tail-dominated regime
- it is not a new pruning idea
- the main remaining barrier is still tail-key multiplicity, not per-candidate decoding overhead

### Adopted: raw exact residual check before `CompressedSequence` construction

Attempt:
- after the tail candidate survives spectral filtering, compute the full exact compressed Legendre residual directly from the assignment buffers and only construct `CompressedSequence` values for actual zero-residual hits

Why it looked promising:
- in the tail-dominated regime, almost every candidate is rejected
- allocating and collecting full sequence objects for those rejects is unnecessary overhead

Observed result:
- the exact reduced length-`11` search counts stayed the same:
  - `0` branches
  - `996305` tail candidates checked
  - `996304` tail spectral prunes
  - `tail_residual_pruned=0`
  - `1` emitted pair
- runtime improved further from `12.64s` to `11.80s`
- the `length=45`, `compression=3`, `tail_depth=11` capped anchor still timed out at `120s`

Conclusion:
- this is another valid implementation improvement in the tail-dominated regime
- it reduces per-candidate overhead but still does not change the next scaling barrier

### Measured: first reduced-length-15 direct-joint anchor

Run:
- `cargo run -p hadamard-cli -- benchmark compressed-pairs --length 45 --compression 3 --ordering natural --spectral-frequencies 4 --tail-depth 11 --max-pairs 1`

Observed result:
- completed in `158.94s`
- `branches_considered=160`
- `norm_pruned=20`
- `tail_candidates_checked=96096005`
- `tail_spectral_pruned=96095826`
- `tail_residual_pruned=178`
- `pairs_emitted=1`

Conclusion:
- reduced length `15` is no longer merely a capped or speculative anchor
- the method clearly remains tail-dominated there too
- the next useful experiments should target reducing tail-key multiplicity rather than branch count

### Adopted: separate per-side norm keys in exact tail tables

Attempt:
- strengthen the exact tail key from `(sum_a, sum_b, norm_a + norm_b)` to `(sum_a, sum_b, norm_a, norm_b)`
- use the row-sum/squared-norm reachability tables to enumerate only feasible per-side norm splits before exact tail lookup or factorized join

Why it looked promising:
- the old combined-norm key collapsed many tail states that were not actually interchangeable
- the search already had exact per-side norm reachability information available, so this was a completeness-safe way to cut multiplicity at the source

Observed result:
- reduced length `11`, tail depth `11`:
  - tail candidates dropped from `996305` to `124981`
  - tail spectral prunes dropped to `122670`
  - tail residual prunes rose to `2310`
  - runtime got worse on this small case: `11.80s` -> `13.65s`
- reduced length `15`, tail depth `11`:
  - tail candidates dropped from `96096005` to `91241732`
  - runtime improved slightly: `158.94s` -> `158.31s`
- reduced length `15`, tail depth `12`:
  - now better than the old depth-`11` baseline
  - `48` branches
  - `90668636` tail candidates checked
  - `151.08s`

Conclusion:
- this is a real multiplicity reduction, not just a runtime polish
- the benefit is mixed on small cases because the stronger key adds overhead
- on the more relevant reduced length-`15` anchor, it is directionally helpful and makes deeper exact tail depth `12` worthwhile again

### Rejected: adaptive factorized split selection by estimated join volume

Attempt:
- instead of always splitting a factorized tail at `remaining/2`, estimate the keyed join volume for each valid split and choose the smallest one

Why it looked promising:
- after strengthening the tail key, split choice looked like a natural next lever
- in principle, different valid `(left_len, right_len)` choices could expose much smaller keyed joins

Observed result:
- on the measured reduced length-`11` and reduced length-`15` anchors, the chosen split produced the same search counts as the fixed half split
- runtime got slightly worse because the estimator added overhead without reducing candidate multiplicity

Outcome:
- reverted
- for the current tail sizes, split selection is not the bottleneck; key strength is

### Adopted: exact shift-1 seam key in factorized natural-order tail joins

Attempt:
- use the exact compressed shift-`1` identity at the factorized seam as a join filter before candidate-level spectral and residual checks
- for a natural-order suffix tail, this depends only on:
  - prefix boundary values
  - left/right segment boundary values
  - left/right internal adjacent-pair sums

Why it looked promising:
- this is the first exact shift constraint that is both:
  - strong enough to cut multiplicity sharply
  - cheap enough to use before candidate-level spectral filtering

Observed result:
- reduced length `11`, tail depth `11`, `K=1`:
  - `tail_candidates_checked=8399`
  - `tail_spectral_pruned=5789`
  - `tail_residual_pruned=2609`
  - `elapsed_seconds=9.77`
- reduced length `15`, tail depth `12`, `K=1`:
  - `tail_candidates_checked=129335`
  - `tail_spectral_pruned=71722`
  - `tail_residual_pruned=57612`
  - `elapsed_seconds=14.13`

Comparison to the previous best reduced length-`15` anchor:
- before seam key: `90668636` checked tails and `151.08s`
- after seam key: `129335` checked tails and `14.13s`

Conclusion:
- this is a major exact-tail-key improvement
- it changes the reduced length-`15` anchor by more than an order of magnitude in runtime and by roughly three orders of magnitude in checked tail candidates
- in the new regime, `1` monitored frequency is better than `4`, and `0` is slightly worse than `1`

### Measured: next anchors after the shift-1 seam key

Observed result:
- reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=1`):
  - `branches_considered=768`
  - `norm_pruned=186`
  - `tail_candidates_checked=223664`
  - `tail_spectral_pruned=190445`
  - `tail_residual_pruned=33218`
  - `elapsed_seconds=103.67`
- reduced length `17`, tail depth `11`, `K=1`:
  - worse than depth `12`
  - `tail_candidates_checked=288271`
  - `elapsed_seconds=165.57`
- reduced length `21` (`length 63`, factor `3`, tail depth `12`, `K=1`):
  - did not finish within a `300s` cap

Conclusion:
- the new seam-aware regime has clearly moved the frontier beyond reduced length `15`
- reduced length `17` is now a real timing anchor instead of a speculative future case
- reduced length `21` is the next practical barrier

Follow-up frequency check:
- reduced length `17`, tail depth `12`:
  - `K=1`: `103.67s`
  - `K=0`: `97.27s`
- reduced length `21`, tail depth `12`:
  - both `K=1` and `K=0` exceed a `300s` cap

Interpretation:
- once the shift-`1` seam key is active, the usefulness of spectral monitoring becomes length-dependent
- `K=1` remains best on reduced length `15`
- `K=0` is slightly better on reduced length `17`

### Rejected: shift-`2` seam key as a second exact join constraint

Attempt:
- extend the factorized natural-order seam-aware join with a second exact local summary intended to enforce shift `2` before candidate-level filtering

Why it looked promising:
- the shift-`1` seam key was the first major multiplicity breakthrough
- a second cheap exact seam identity looked like the most natural next escalation

Observed result:
- correctness was preserved, but the added key was a bad trade
- reduced length `15` slowed down
- reduced length `17` did not finish within a `300s` cap in the tested configuration

Outcome:
- reverted
- the lesson is that exact seam statistics have to be cheap in the same way shift `1` is cheap; a more expensive local summary can easily lose more than it gains

### Measured: tail-depth limit and reduced length `21` sweep

Observed result:
- the current exact factorized tail path is only exact through depth `12`
- the direct probe now reports `effective_tail_depth` explicitly and clamps larger requests
- reduced length `21` (`length 63`, factor `3`, `K=0`):
  - tail depth `11`: did not finish within a `300s` cap
  - tail depth `12`: did not finish within a `300s` cap
  - tail depth `13`: also did not finish within a `300s` cap and is now outside the exact factorized regime, so larger requests are clamped instead of treated as distinct benchmarks

Conclusion:
- deeper exact tail alone is not the next lever
- reduced length `21` remains the active barrier

### Adopted, but only for larger reduced lengths: exact small-shift tail prefilter

Attempt:
- in the natural-order factorized tail path, check exact shifts `2..4` from cached tail-side summaries before stitching and full residual evaluation

Why it looked promising:
- after the shift-`1` seam key, the remaining cost is candidate multiplicity inside exact tail joins
- exact small shifts are stronger than spectral checks and cheaper than full residual evaluation if summarized correctly

Observed result:
- reduced length `15`, tail depth `12`, `K=1`:
  - correctness preserved
  - `tail_shift_pruned=508279`
  - runtime worsened to `18.10s`
- reduced length `17`, tail depth `12`, `K=0`:
  - correctness preserved
  - `tail_shift_pruned=223531`
  - runtime improved from `97.27s` to `95.87s`
- reduced length `21`, tail depth `12`, `K=0`:
  - still did not finish within a `300s` cap

Outcome:
- kept only for reduced lengths `>= 17`
- this is a modest larger-instance optimization, not the next major multiplicity breakthrough

### Rejected: small-shift signature bucketing inside the factorized tail join

Attempt:
- strengthen the natural-order factorized join by grouping right-tail candidates not only by the shift-`1` seam key, but also by cached exact small-shift summaries for shifts `2..4`
- for each left-tail candidate, test compatibility at the summary level first and only expand matching right-signature buckets into concrete candidates

Why it looked promising:
- the kept small-shift prefilter already rejects a very large fraction of stitched candidates on reduced length `17`
- if that rejection could happen one level earlier, candidate multiplicity inside the join loop might drop sharply without needing a full shift-`2` seam key

Observed result:
- on reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`):
  - `tail_candidates_checked` dropped from `223670` to `139`
  - `tail_shift_pruned` rose to `226375`
  - but the timed run slowed from `95.87s` to `152.65s`
- on reduced length `21` (`length 63`, factor `3`, tail depth `12`, `K=0`):
  - the benchmark still did not finish within a `300s` cap

Conclusion:
- this looked like a stronger exact tail key in terms of search counts, but not in terms of wall-clock cost
- the extra hash-map bucketing and signature-level matching overhead outweighed the saved candidate stitching

Outcome:
- reverted
- the lesson matches the earlier rejected shift-`2` seam attempt: more cyclic structure only helps if the join-time summary is nearly as cheap as the shift-`1` seam key
- the next tail-key attempt should aim for a compressed cyclic invariant with lower bookkeeping cost than full small-shift signature bucketing

### Rejected: precomputing right-tail small-shift summaries inside shift-`1` buckets

Attempt:
- keep the existing shift-`1` join key unchanged
- when building the right-tail shift-`1` buckets, also precompute and store each candidate's cached exact small-shift summary so the hot join loop can avoid a hash lookup for `right_small_shift_sig`

Why it looked promising:
- on reduced length `17`, the dominant cost is still the exact small-shift rejection path after the shift-`1` seam match
- removing one cache lookup from each of roughly `223k` checked tails looked like a cheap constant-factor win

Observed result:
- correctness was preserved
- on reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`), the timed run got worse again:
  - search counts stayed effectively unchanged
  - elapsed time worsened to `146.86s`
- reduced length `21` (`length 63`, factor `3`, tail depth `12`, `K=0`) still did not finish within a `300s` cap

Conclusion:
- even this lighter-weight small-shift optimization does not buy back the dominant runtime cost
- the current bottleneck is not a single cache lookup inside the post-join filter

Outcome:
- reverted
- further effort on the existing small-shift filter looks unlikely to change the frontier materially

### Adopted: packed shift-`1` seam buckets for the factorized join

Attempt:
- keep the exact shift-`1` seam key unchanged
- replace the hash-heavy right-tail bucket representation with a flatter layout:
  - store only the present seam-boundary buckets in a vector
  - inside each boundary bucket, store `(internal_sum, candidates)` as a sorted vector and use binary search instead of an inner hash map

Why it looked promising:
- the shift-`1` seam key is still the only exact join key that has clearly paid for itself
- recent failures suggested the next useful gain was likely lower join bookkeeping cost, not a richer join invariant

Observed result:
- reduced length `15` (`length 45`, factor `3`, tail depth `12`, `K=1`):
  - search counts unchanged in practice
  - runtime improved from about `14.13s` to `12.60s`
- reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`):
  - search counts unchanged in practice
  - runtime improved from `95.87s` to `94.53s`
- reduced length `21` (`length 63`, factor `3`, tail depth `12`, `K=0`):
  - still did not finish within a `300s` cap

Conclusion:
- this is a real constant-factor improvement in the current best exact-tail regime
- it helps by making the already-successful shift-`1` seam join cheaper, not by changing the mathematical pruning frontier

Outcome:
- kept
- the next step should build on this same principle: cheaper exact join representation first, richer tail key second

### Adopted: direct-indexed packed seam-boundary slots for shift-`1` buckets

Attempt:
- keep the packed shift-`1` seam-bucket layout, but remove the remaining outer hash map during bucket construction
- encode each seam boundary as a small base-`|alphabet|` integer and place it into a direct slot table before compacting to the present boundary buckets

Why it looked promising:
- after flattening the inner `internal_sum` lookup, the remaining seam-join bookkeeping still included a hash map from boundary signature to bucket id
- the compressed alphabets here are tiny, so the full seam-boundary space is small enough to index directly

Observed result:
- reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`):
  - search counts unchanged in practice
  - timed reruns improved from `94.53s` to `88.46s`, then to `85.86s`
- reduced length `21` (`length 63`, factor `3`, tail depth `12`, `K=0`):
  - still under evaluation for a possible constant-factor improvement, but not yet known to finish under a `300s` cap

Conclusion:
- this is the strongest constant-factor improvement since the original shift-`1` seam key
- it supports the current instinct: keep compressing the exact seam-join representation before attempting another richer tail key

Outcome:
- kept

### Adopted: unified tail-join summary cache for shift-`1` and small-shift data

Attempt:
- replace the separate cached tail decodes for:
  - shift-`1` seam summaries
  - small-shift summaries
- with one cached tail-join summary per `(a_code, b_code)` pair carrying both summaries

Why it looked promising:
- the packed seam-boundary work showed the remaining cost is still join bookkeeping
- the old path decoded the same tail twice just to derive two different summary structs that are always used together in the natural-order exact-tail regime

Observed result:
- reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`):
  - search counts unchanged in practice
  - runtime improved from the direct-indexed seam-boundary version (`85.86s`) down to `79.28s`
- reduced length `21` (`length 63`, factor `3`, tail depth `12`, `K=0`):
  - still under evaluation for a possible constant-factor improvement, but not yet known to finish under a `300s` cap

Conclusion:
- this is another clean constant-factor improvement on the same successful exact join
- it reinforces the current direction: keep compressing summary representation and cache reuse before introducing richer exact tail keys

Outcome:
- kept

### Rejected: cache packed shift-`1` buckets by exact right-tail key

Attempt:
- keep the packed seam-boundary representation and unified tail-join summaries
- cache the built packed shift-`1` buckets by the exact right-tail key `(sum_a, sum_b, norm_a, norm_b)` instead of rebuilding them every time through the left-side loop nest

Why it looked promising:
- the current packed shift-`1` join is already the best exact key in the code
- after flattening its representation, bucket construction itself became a clear candidate for reuse

Observed result:
- reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`):
  - search counts unchanged in practice
  - runtime came in at `83.46s`, which is slightly worse than the unified-summary-cache anchor (`79.28s`)

Outcome:
- reverted
- the extra packed-bucket cache bookkeeping did not beat the cheaper rebuild path on the measured anchor

### Rejected: direct-indexed internal-sum slots inside packed shift-`1` buckets

Attempt:
- keep the packed seam-boundary representation and unified tail-join summary cache
- replace the remaining per-boundary binary search over `(internal_sum, candidates)` with a direct slot array indexed by the bounded exact shift-`1` internal-sum range

Why it looked promising:
- after the earlier packing steps, the inner seam lookup was still paying for a search inside each present boundary bucket
- for the current exact-tail regime, the shift-`1` internal range is small enough that direct indexing is cheap

Observed result:
- reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`):
  - search counts unchanged in practice
  - timed reruns came in at `84.03s` and `91.13s`, both worse than the unified-summary-cache anchor (`79.28s`)

Outcome:
- reverted
- the direct slot array for exact shift-`1` internals did not beat the simpler sorted-vector representation

### Rejected: precomputed required shift-`1` seam totals per left candidate

Attempt:
- keep the direct-indexed packed seam-boundary slots
- for each left-tail candidate, precompute the exact required shift-`1` internal total for every present right-boundary bucket once, then let the boundary loop only binary-search the sorted per-boundary internal lists

Why it looked promising:
- after flattening the boundary representation, the boundary loop still recomputed the same exact required internal value formula for every bucket visit
- this is pure join bookkeeping, so a cheap precompute should stack cleanly with the existing packed representation

Observed result:
- reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`):
  - search counts unchanged in practice
  - runtime worsened slightly from `88.46s` to `93.03s`

Outcome:
- reverted
- the extra per-left precompute did not pay for itself even on the cheaper packed-boundary join

### Tried: exact tail-side selected-shift residual prefilter

Attempt:
- after a tail candidate passes the tail spectral check but before constructing `CompressedSequence`, test the first few exact compressed residual shifts directly against the target `-2f`

Why it looked promising:
- the next barrier is no longer branch count
- if the harder reduced length-`15` anchor leaves many spectrally admissible tails, an exact shift filter could reject some of them before the full residual computation

Observed result:
- preserved the known length-`15`, factor-`3` compressed projection
- on the reduced length-`11` full-tail benchmark:
  - `tail_residual_pruned=0`
  - the exact search counts were unchanged
- the `length=45`, `compression=3`, `tail_depth=11` capped anchor still timed out at `120s`

Conclusion:
- this filter is correctness-safe, but it is not currently a meaningful pruning lever on the measured benchmark
- the current bottleneck still appears before these selected exact shifts become decisive

### Rejected: heuristic low-energy ordering of factorized tail candidates

Attempt:
- when `max_pairs` is small, sort factorized tail candidates by a cheap selected-frequency segment-energy score and try low-energy candidates first

Why it looked promising:
- for the `max_pairs=1` benchmark, a better search order could in principle reach the first valid pair much earlier without changing completeness

Observed result:
- on the reduced length-`11` full-tail benchmark, the heuristic made the search worse:
  - `tail_candidates_checked` increased from `996305` to `999097`
  - `tail_spectral_pruned` increased correspondingly
  - elapsed time also worsened

Outcome:
- reverted
- the segment-energy heuristic is too weak a proxy for pair viability and should not be trusted as an ordering signal

### Rejected: cross-branch factorized tail caches

Attempt:
- try a larger structural cut instead of another local representation tweak
- hoist factorized-tail preprocessing out of each recursive exact-tail probe so branches can reuse prior work
- tested two variants:
  - persistent join-summary, decoded-tail, and right shift-`1` bucket caches
  - persistent join-summary and decoded-tail caches only

Why it looked promising:
- the reduced length-`17` anchor now spends most of its time inside the exact factorized tail path
- that path rebuilds tail summaries and decodes inside many recursive probes, so cross-branch reuse looked like a plausible step-change win

Observed result:
- reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`):
  - full cross-branch cache variant preserved the search counts but slowed to `95.96s`
  - the stripped-down summary/decode cache variant also ran well past the `79.28s` anchor and was stopped once it was clearly noncompetitive

Outcome:
- reverted
- in the current code shape, broader cross-branch tail caching adds enough hash/ownership overhead to lose against the cheaper per-probe local caches

### Rejected: adaptive probe-side choice in the exact `6+6` factorized join

Attempt:
- try a bigger structural change than another cache or bucket-layout tweak
- for each exact `(sum_a, sum_b, norm_a, norm_b)` tail key, choose the smaller side as the probe side instead of always probing the left `6`-tail bucket against a bucketed right side
- preserve the same exact shift-`1`, small-shift, spectral, and residual checks

Why it looked promising:
- the current factorized exact-tail join is still asymmetric even when the left/right multiplicities for a key are skewed
- in principle, probing the smaller side should reduce repeated join bookkeeping on the larger side

Observed result:
- reduced length `17` (`length 51`, factor `3`, tail depth `12`, `K=0`):
  - correctness preserved
  - runtime was clearly noncompetitive and was stopped after passing roughly `160s`, far beyond the current `79.28s` anchor

Outcome:
- reverted
- in the current implementation, the extra branching and symmetric bucket handling lose badly enough that adaptive probe-side choice is not a viable next step
