# Experiment Log

This file records implementation directions that were tried, measured, and then rejected or deprioritized.

## Compressed LP search

### Rejected as a multiplier stabilizer assumption: full column-preserving subgroup

Attempt:
- use `hadamard analyze lp333-multiplier` to test whether the full subgroup
  `t ≡ 1 mod 37` could be imposed as a multiplier stabilizer
- this subgroup acts on rows by all of `U(9)`, so it would strongly constrain the CRT
  row-sum and mod-`3` row-bundle marginals

What happened:
- the analyzer now treats this as a conditional stabilizer hypothesis, not an
  unconditional multiplier theorem
- under that hypothesis, only `18` row-bundle triples are allowed
- `0` of the `504` ordered bundled exact pairs survive
- the corresponding norm-refined mass is `0`

Conclusion:
- full column-preserving invariance is not a viable LP(333) subfamily target
- smaller cyclic row-action hypotheses remain viable, especially those with
  `row_units={1,4,7}`, which leave `12` ordered bundled pairs and mass
  `119,903,105,952`
- multiplier work should continue as optional stabilizer-subfamily search, not as a
  claimed hard sieve for all LP(333) pairs

### Adopted as the first multiplier subfamily target: row action `row_units={1,4,7}`

Attempt:
- after the full column-preserving stabilizer failed, screen cyclic multiplier
  stabilizer hypotheses with nontrivial row action against the exact row-bundle sieve
- then lift the best surviving row action through the full length-`9` row-sum marginal
  equations

What happened:
- `15` cyclic subgroups with nontrivial row action survive the row-bundle sieve
- every best survivor has row action `row_units={1,4,7}`
- the representative column-trivial order-`3` subgroup is `{1,112,223}`
- this row action leaves:
  - `1064` allowed row-bundle triples
  - `12` ordered bundled exact pairs
  - norm-refined mass `119,903,105,952`
- the full row-marginal lift then gives:
  - `7,467` active invariant length-`9` row marginals
  - `13,764,060` row-marginal pair candidates
  - `6,048` norm-compatible row-marginal pairs
  - `6,048` exact pure-row autocorrelation matches

Conclusion:
- the `row_units={1,4,7}` multiplier subfamily is not killed by the pure row equations
- this is now the first multiplier-invariant subfamily worth testing at the actual
  row-pattern / mixed-character level
- the next sieve should act on invariant `9 x 37` sign tables or mixed CRT Fourier
  constraints, not on more row-sum-only filtering

### Rejected as a multiplier stabilizer representative: column-trivial order-`3` subgroup `{1,112,223}`

Attempt:
- take the cleanest `row_units={1,4,7}` order-`3` subgroup, `{1,112,223}`
- because this subgroup has column action `1`, its row orbits force identical
  length-`37` rows at positions `1,4,7` and at positions `2,5,8`
- test actual CRT shift `(alpha,0)` feasibility using row-dot marginals after the
  pure row-sum lift

What happened:
- the pure row-sum lift left `6,048` exact row-marginal pairs
- the actual shift-`(alpha,0)` row-dot marginal check left `0`
- the check uses the actual LP target `-2` for a nonzero CRT shift, not the
  row-compressed aggregate target `-74`

Conclusion:
- the column-trivial representative `{1,112,223}` is not a viable multiplier-invariant
  subfamily
- this does not reject the whole `row_units={1,4,7}` row-action family
- the remaining viable targets are the non-column-trivial subgroups, where row orbit
  members are tied by column permutations rather than being identical rows

### Narrowed but not rejected: non-column-trivial `row_units={1,4,7}` subgroups

Attempt:
- continue the order-`3` row-action branch on the non-column-trivial representatives
  such as `{1,121,322}` and `{1,211,232}`
- these have column action `col_units={1,10,26}`, so rows fixed by the row action
  must also be invariant under column multiplication by `10`
- test the necessary row-sum representability condition for the fixed rows `0`, `3`,
  and `6`

What happened:
- a row fixed by column multiplication by `10` is determined by column `0` plus
  `12` length-`3` nonzero-column orbits
- among the `6,048` exact row-marginal pairs from the `row_units={1,4,7}` lift,
  `1,296` have fixed-row sums that can be realized by such column-orbit-invariant rows
- the aggregate fixed-row-pattern multiplicity across those survivors is
  `log10 ~= 60.818`
- an exact column-`10` orbit dot calculation for the actual CRT shift `(3,0)` was
  then added
- this `(3,0)` marginal does not prune further: all `1,296` fixed-row-compatible
  row-marginal pairs remain feasible

Conclusion:
- the non-column-trivial order-`3` row-action branch remains viable, but it is much
  narrower than the pure row-marginal count suggests
- this check is necessary only; it does not enforce mixed CRT character constraints or
  the actual column-permuted relationships among rows `1,4,7` and `2,5,8`
- the next test should derive the mixed-character equations for shifts involving the
  nonfixed row orbits under the `col_units={1,10,26}` action, rather than add more
  row-sum-only filters

### Deferred from the default analyzer: exact `(0,1)` column-shift marginal

Attempt:
- after the `(3,0)` marginal did not prune the non-column-trivial branch, start the
  next mixed CRT shift orbit `(0,1)`
- for rows in the nonfixed row orbits, this requires possible values of
  `C(1)+C(10)+C(26)` for an arbitrary length-`37` base row with fixed row sum
- the column-action quotient graph has a small enough frontier to attempt an exact DP

What happened:
- an exact frontier-DP scaffold was added behind
  `hadamard analyze lp333-multiplier --col10-shift1`
- the first generic factor-join version was too slow for tests
- the current frontier version is still kept out of the default analyzer because it is
  not yet fast enough to be a routine checkpoint
- a fast direct `2^13` table run completed, but that table enumerated only rows fixed
  by column multiplication by `10`
- that is too strong for the nonfixed row orbits in the order-`3` mixed branch: those
  rows are arbitrary base rows whose copies are tied by column multiplication, not
  rows that are themselves column-orbit-constant
- the arbitrary-row frontier DP has been restored for this table
- the corrected exact run took about `176s` in the targeted test and leaves all
  `1,296` fixed-row-compatible row-marginal pairs alive, with aggregate pattern mass
  still `log10 ~= 60.818`
- a relaxed coupled `(3,0)+(0,1)` diagnostic was added behind
  `hadamard analyze lp333-multiplier --col10-coupled`; it couples the arbitrary
  nonfixed row-orbit base rows through a joint `(multiplier self-dot,
  additive-autocorrelation)` table, but still combines the fixed-row `(3,0)` and
  `(0,1)` possibilities independently
- that relaxed coupled check is also non-pruning: it keeps all `1,296`
  fixed-row-compatible row-marginal pairs, with aggregate pattern mass
  `log10 ~= 60.818`; the run built `147` cached sequence joint sets, with maximum
  joint-set size `2,426`
- a direct exact-fixed-row coupling DP was also prototyped; the first naive
  13-variable frontier over three fixed rows did not finish after a long debug run,
  and a later shared fixed-triple target-table variant, narrowed to the exact
  fixed-row-compatible survivors, still did not produce a full stats line in the
  practical debug window; only the constant-row sanity cases are currently cheap,
  so it was not kept as a public analyzer flag
- the multiplier analyzer also gained an opt-in CRT component orbit view behind
  `--crt-components`, which prints the coordinate orbits as `(mod 9, mod 37)` pairs
  and a focused `row_units={1,4,7}` hypothesis summary; that family currently has
  `15` surviving hypotheses split across subgroup orders `3,6,9,12,18,36`, with
  order-`3` column units `[1]` / `[1,10,26]` and order-`6` column units `[1,36]`
  / `[1,10,11,26,27,36]`; the nontrivial mixed-CRT target set is the four
  hypotheses in `crt_component_mixed_crt_targets`, with order split `3:2,6:2`,
  and the summary now includes `pairs`, `mass`, `pair_samples`, and `col_orbits`,
  but those four targets currently tie on `pairs=12`, `mass=119903105952`,
  `row_orbits=0|1,4,7|2,5,8|3|6`, and `allowed_bundles=1064`; the top pair
  samples are also identical across the four targets; `col_orbits` only splits
  them into the order-`3` and order-`6` buckets, and the remaining distinction is
  the explicit `subgroup_crt` label
- the CRT component view now also runs an actual invariant-row-stabilizer lift on
  those four mixed targets:
  - the two order-`3` targets have `113` invariant table orbits with size
    distribution `1:3,3:110`; they reproduce the `1,296` exact row-pair survivors
    and `log10 ~= 60.818`
  - the two order-`6` targets have `59` invariant table orbits with size
    distribution `1:3,3:2,6:54`; their stronger row stabilizer leaves only `216`
    active row marginals and `0` row-pair candidates after the active bundle-pair
    join
- a bounded mixed-shift graph diagnostic was added behind
  `hadamard analyze lp333-multiplier --invariant-shift31`; this is intentionally
  not an exact feasibility enumerator
- for the two surviving order-`3` mixed targets, it reports:
  - `1,296` exact row-pair survivors
  - `42` distinct fixed-row triple targets
  - `6` distinct nonfixed row-sum targets
  - fixed-row shift graph:
    `vars=13,domain=8,edges=32,weighted_edges=35,self_terms=2,max_degree=6,min_fill_width=5`
  - fixed-row min-fill profile:
    `min_fill_added_edges=7`, bag-size distribution `1:1,2:1,3:2,4:3,5:5,6:1`,
    and max bag domain states `262,144`
  - nonfixed row-orbit shift `(3,1)` graph:
    `vars=37,domain=2,edges=105,weighted_edges=108,self_terms=3,max_degree=6`,
    with `min_fill_width=10` for column generator `10` and `min_fill_width=9`
    for column generator `26`
  - nonfixed max bag domain states are `2,048` for column generator `10` and
    `1,024` for column generator `26`; the analyzer now prints the concrete
    min-fill order for each case

Conclusion:
- the `(0,1)` mixed marginal is now recorded as non-pruning for the corrected
  arbitrary-row table, but the exact path remains opt-in because it is still too slow
  for routine checkpoints
- the first coupled mixed-shift relaxation is also non-pruning, so the next useful
  discriminator must either couple the fixed-row rows exactly with a better state
  representation or lift actual invariant sign tables
- the mixed target set has narrowed further: the order-`6` mixed targets are rejected
  by actual invariant row-stabilizer compatibility, leaving the two non-column-trivial
  order-`3` subgroups as the next mixed-character targets
- the first actual shift `(3,1)` structural diagnostic says the target sets are small
  and the graph is wide enough that another direct value-DP is the wrong next
  default, but the domain-aware min-fill profile is encouraging: the next exact
  attempt should use the printed bag order and couple fixed/nonfixed invariant
  sign-table assignments through bag states rather than materializing global
  row-sum/energy tables
- default `hadamard analyze lp333-multiplier` now marks this path as `skipped`

### Rejected as a default CRT pair path: naive norm-plus-one-shift materialization inside `W`-frontier buckets

Attempt:
- add a `hadamard analyze lp333-crt-pair` diagnostic for one bundle pair
- stage the next exact lift one row shift at a time by:
  - grouping `UV` states by the downstream `W` coefficient
  - enumerating all `W` completion signatures for that coefficient
  - materializing exact row signatures for norm plus a single shift

Why it looked promising:
- the earlier bundle probe had already shown a stable `~250x` to `275x` collapse from raw `UV` pairs down to only about `4.1k` coarse `W`-frontier buckets
- if one exact shift inside those buckets were already cheap enough, it could have become the first practical orbit-level lift primitive

What happened:
- on the representative hub pair `[-15,5,11] | [-5,-1,7]`, the reduced `(norm_uv, base_shift, W-coeff)` states are still:
  - `1,106,079` on the left
  - `1,116,807` on the right
- the per-coefficient `W` completion signatures total about:
  - `4,344,024` on the left
  - `4,437,827` on the right
- that implies a naive one-shift materialization workload around:
  - `1.132e9` left-side combinations
  - `1.164e9` right-side combinations
- the heaviest pair `[-15,3,13] | [-5,3,3]` is comparable rather than better:
  - `1.068e9` and `1.223e9`
- a follow-up coefficient-profile cache was measured and kept in the analyzer:
  - it gives about `5.84x` reuse for `W` histogram construction via coordinate-permutation symmetry
  - this lowers analyzer runtime but does not change the materialization scale
- sampled heavy local buckets also do not rescue the path:
  - representative sampled buckets have about `0.98M` products and collapse to about `0.20M` to `0.22M` row signatures
  - heaviest-pair sampled buckets have about `0.93M` to `1.04M` products and collapse to about `0.22M` row signatures
  - local compression is only about `4.1x` to `4.9x`
- a compact frontier-join follow-up measured left/right coefficient-bucket compatibility without full row-signature materialization:
  - representative bucket pairs drop from `17,656,740` to `79,398` under exact norm marginal compatibility
  - the heaviest pair drops from `17,643,894` to `80,322`
  - the row-shift marginal prunes no bucket pairs on either test
  - active side materialization is still hundreds of millions of combinations: about `302M` / `291M` on the representative pair and `266M` / `326M` on the heaviest pair
  - sampled exact joins inside the heaviest surviving bucket pair are extremely sparse:
    - representative top-three samples: `967,079,856,834` raw row-pair products collapse to `13,194`, `13,180`, and `13,180` exact norm-plus-shift joins
    - heaviest-pair top-three samples: `962,719,464,960` raw row-pair products collapse to `11,971`, `12,095`, and `12,095` exact joins
  - a follow-up active-frontier exact join now completes offline by materializing only those active buckets:
    - representative pair exact join count: `124,923,897`
    - heaviest pair exact join count: `124,940,502`
    - survival against the norm-only count is about `1.25%` on both pairs, a reduction factor just under `80x`
    - the active-frontier row-signature histograms are much smaller than the raw active materialization:
      - representative: `835,041` left signatures from `301,992,456` active rows, `791,879` right signatures from `290,879,718`
      - heaviest: `831,264` left signatures from `266,224,692` active rows, `793,767` right signatures from `326,268,000`
    - on the representative pair, shifts `1`, `2`, and `4` are exactly identical at this level:
      same active bucket counts, same active histogram sizes, same exact join count `124,923,897`
- carrying any two shifts together was also tested for `(1,2)`, `(1,4)`, and `(2,4)`:
  - on `[-15,5,11] | [-5,-1,7]`, every two-shift choice is injective at the coefficient-bucket level
  - left: `1,106,079` raw `UV` pairs and `1,106,079` coefficient buckets
  - right: `1,166,391` raw `UV` pairs and `1,166,391` coefficient buckets
  - max bucket mass is `1`, raw `W` materialization remains about `1.165e9` left and `1.249e9` right, and sampled local compression is `1.0000`
  - the heaviest pair `[-15,3,13] | [-5,3,3]` has the same singleton-bucket shape, with raw `W` materialization about `1.156e9` left and `1.259e9` right
- carrying all three shifts together was also tested:
  - on `[-15,5,11] | [-5,-1,7]`, all-shift coefficient buckets are injective on both sides
  - left: `1,106,079` raw `UV` pairs and `1,106,079` coefficient buckets
  - right: `1,166,391` raw `UV` pairs and `1,166,391` coefficient buckets
  - max bucket mass is `1`, so there is no downstream `W` batching left to exploit
  - sampled all-shift local buckets have local compression `1.0000`

Conclusion:
- the downstream `W` frontier is the right region to study, but a straightforward norm-plus-one-shift materialization is still too expensive as a default analyzer path
- coefficient-profile caching is worth keeping as a small implementation optimization, but it is not an algorithmic breakthrough
- norm marginal compatibility across `W` frontier buckets is a real staging filter, but by itself it only recovers another small constant factor in active materialization work
- exact local joins inside sampled survivor bucket pairs are very sparse, and the full active-frontier exact join now looks feasible as an offline analyzer path
- the exact one-shift count is still on the order of `1.25e8`, so the next plausible path is a more indexed active-bucket join or a sharper invariant inside those active buckets, not a default full materialization
- carrying two or three row shifts together is over-constrained for batching: it eliminates the useful bucket structure
- the next viable exact-lift prototype needs a sharper invariant or a different factorization, not another direct variant of the current `UV -> W` row-shift split

### Rejected in CRT component lift: shared-state reuse inside a representative `3`-node hub component under the current `UV -> W` model

Attempt:
- add a separate `hadamard analyze lp333-crt-component` diagnostic for one representative half-mass component
- test whether the two spoke families of the component share exact-lift state under:
  - full `UV` signatures
  - coefficient-only `UV` signatures
  - the coarse `W`-frontier projection

Representative component:
- hub `[-15, 5, 11]`
- spokes `[-5, -1, 7]` and `[-5, 7, -1]`

Why it looked promising:
- the CRT row-bundle residual had just collapsed to tiny connected components of sizes `3, 3, 3, 2`
- if a `3`-node hub component shared large parts of its lift state across the two spoke families, the first exact-lift prototype could reuse that shared state and beat pair-by-pair handling

What happened:
- hub `UV` signature count: `1,106,079`
- spoke `UV` signature counts: `1,166,391` and `1,153,467`
- spoke `UV` overlap: `0`
- spoke coefficient-only overlap: `0`
- spoke coarse `W`-frontier counts: `4,210` and `4,218`
- spoke coarse `W`-frontier overlap: `0`
- the same zero-overlap pattern reproduces on the slightly heaviest `3`-node component
  `[-9, -5, 15] | [-5, -3, 9] | [-5, 9, -3]`

Conclusion:
- the current `UV -> W` model does not create any cross-spoke reuse on a representative `3`-node hub component, even after collapsing to the coarsest tested `W`-frontier key
- this is already reproducible on a second top-mass hub component, so it does not look like a one-off representative artifact
- future exact-lift work should not assume that a component-scoped lift will be cheaper just because the residual orbit graph is small
- the next win, if there is one here, has to come from a different exact invariant or a different decomposition, not from shared-state reuse inside the current `UV -> W` model

### Rejected in default analyzer: naive exact row-shift lift above the CRT row-bundle sieve

Attempt:
- extend `hadamard analyze lp333-crt` to lift the heaviest surviving bundled pair orbit through exact row shifts `1, 2, 4`

Why it looked promising:
- the CRT row-bundle sieve had already reduced the residual problem to
  - `504` bundled exact pairs
  - `90` active bundled states
  - `168` active bundled pair orbits
- that looked small enough to try an exact lift directly

Why it was rejected:
- even the single heaviest surviving bundled pair orbit
  - `[-15, 3, 13] | [-5, 3, 3]`
  still sits on raw exact lift space about `1.4545e18`
- the naive distribution build was too slow to keep inside the default analysis path

What was learned instead:
- the natural `UV -> W` transition state for that same orbit is only about
  - `1,110,187` signatures on the left
  - `1,164,237` signatures on the right
- that is small enough to keep the orbit-level exact-lift idea alive
- however, the coefficient-only transition signature count is the same as the full transition count on that top orbit, so a trivial coefficient-only collapse is not the next win

Outcome:
- reverted from the main analyzer path
- kept the CRT analyzer fast
- treat optimized orbit-level row-shift lifting as the next serious implementation target

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

## Dated experiments

### EXP-2026-07-02-01: Linear-frontier compressed profiles for invariant shift `(3,1)`

Date:
- `2026-07-02`

Area:
- multiplier shift profile

Status:
- Adopted

Question:
- can the invariant shift-`(3,1)` frontier DP be represented densely enough to complete the nonfixed and fixed profile tables exactly?

Commands:
- `cargo run --release -p hadamard-cli -- analyze lp333-multiplier --invariant-shift31-compressed-profile`
- `cargo run --release -p hadamard-cli -- analyze lp333-multiplier --invariant-shift31-compressed-profile --invariant-shift31-profile-limit 30000000`

Finding:
- at the default `100,000` signature cap, the nonfixed profiles complete exactly with `31,020` and `26,598` signatures for column generators `10` and `26`
- with the `30,000,000` cap in release mode, the fixed profile also completes exactly
- the fixed profile peaks at `20,849,958` row signatures and `58,360,095` energy bits

Decision:
- keep the linear-frontier, row-signature, and bounded-energy-bitset representation
- use the measured profile to support an exact shift-`(3,1)` lift

Follow-up:
- run the exact value lift and record whether it removes any of the `1,296` surviving row pairs

### EXP-2026-07-02-02: Exact invariant shift-`(3,1)` lift

Date:
- `2026-07-02`

Area:
- multiplier shift lift

Status:
- Measured

Question:
- does exact feasibility for the first actual mixed shift `(3,1)` prune either surviving non-column-trivial order-`3` multiplier target?

Commands:
- `cargo run --release -p hadamard-cli -- analyze lp333-multiplier --invariant-shift31-exact`

Finding:
- the exact lift completes with all `42 / 42` fixed targets and all `6 / 6` nonfixed targets represented
- the row-value cache has maximum energy-set size `84`
- all `1,296` row pairs remain feasible, with aggregate pattern mass still `log10 ~= 60.818`

Decision:
- record single shift `(3,1)` as exact but non-pruning
- do not spend another full lift on an isolated mixed shift without evidence that it couples information absent from the current marginals

Follow-up:
- test coupled mixed shifts or a stronger invariant across the fixed and nonfixed row orbits

### EXP-2026-07-10-01: Audit and exact reduction of the Eliahou 64-modular fixed-`q` seed

Date:
- `2026-07-10`

Area:
- Eliahou 64-modular seed audit and exact pseudo-Boolean repair

Status:
- Measured/Unresolved

Question:
- can the fixed-`q` Eliahou seed be reproduced, audited exactly, and reduced to a bounded exact repair problem?

Commands:
- `uv run python py/search_modular_golay.py --audit-only`
- `uv run python py/search_modular_golay.py --timeout-seconds 300`
- separate exploratory OR-Tools and direct-CNF portfolio runs were made from the same independently checked equations

Finding:
- the 64-modular fixed-`q` seed and its exact reduction were reproduced
- the audited seed has `13` defective constraints
- fixing `q` reduces the remaining exact repair to a pseudo-Boolean model with:
  - `164` Boolean variables
  - `81` exact equations
- Z3 returned `unknown (timeout)` after `300` seconds
- a `12`-worker OR-Tools CP-SAT model also returned `UNKNOWN` after `300` seconds, after `1,567,071` branches and `97,717` conflicts
- direct CNF encodings with `64,116` variables and `227,384` clauses were interrupted without a result after exceeding the intended calibration window
- no satisfying assignment, unsatisfiability result, or Hadamard construction is claimed; all results remain unresolved

Decision:
- keep this reduction as the strongest exact continuation of the Eliahou seed
- treat the current result as a measured model reduction, not as evidence that the model is satisfiable or unsatisfiable

Follow-up:
- try an exact solver/proof configuration better matched to XOR cardinalities, or the staged modular lift described in the research note
- record the solver, complete invocation, version, resource use, and proof artifact for any decisive run
- independently verify any returned assignment against all `81` equations and the final Hadamard conditions

### EXP-2026-07-10-02: Naive local-repair and SDS warm-start calibration

Date:
- `2026-07-10`

Area:
- local repair and stochastic SDS calibration

Status:
- Measured/Rejected-as-naive

Question:
- do inexpensive single-coordinate, mirror-paired, or random-SDS warm-start moves repair the modular seed closely enough to justify using them as the primary search method?

Commands:
- quick exploratory local-search and Python calibration scripts were run in the active research session
- no stable repository CLI command or committed raw command transcript exists yet

Finding:
- variable-`q` single-coordinate and mirror-paired moves reached best observed scaled-L2 defect scores of roughly `288` and `308`, respectively, but did not reach zero
- fixed-`q` single-flip repair stalled around a scaled-L2 score of `708`
- a quick Python random-SDS warm-projection calibration did not show an advantage:
  - best warm-projected score was about `170`
  - best random score was `136`
- these are calibration observations, not exhaustive lower bounds

Decision:
- reject these naive move sets and the tested warm projection as the primary repair strategy
- do not interpret the negative calibration as rejecting structured local search or stochastic SDS more generally

Follow-up:
- prioritize the active fixed-`q` exact PB model
- only revisit stochastic repair with structure-aware neighborhoods, exact projection, reproducible commands, and comparison against the random baseline

### EXP-2026-07-24-01: Exact modulo-`4` quotient sieve for the fixed-`q` model

Date:
- `2026-07-24`

Area:
- Eliahou fixed-`q` exact quotient decomposition

Status:
- Adopted

Question:
- does projecting the exact norm identity into `Z[x] / (x^4 - 1)` split the
  `164`-sign fixed-`q` problem into a small, completeness-safe family that improves
  Boolean propagation?

Commands:
- `.venv/bin/python py/test_search_modular_golay.py`
- `.venv/bin/python py/search_modular_golay.py --timeout-seconds 5 --seed 20260724 --quotient-modulus 0`
- `.venv/bin/python py/search_modular_golay.py --timeout-seconds 5 --seed 20260724`
- `seq 0 41 | xargs -n1 -P4 .venv/bin/python py/search_modular_golay.py --timeout-seconds 5 --seed 20260724 --quotient-case-index`

Finding:
- if `U_r` and `V_r` are the sums in residue class `r mod 4`, the exact identity
  forces
  `PAF(U)_t + PAF(V)_t = (164, 0, -2, 0)_t`
- after the already-proved parity-class sums are imposed, exhaustive integer
  enumeration leaves exactly `42` ordered `(U mod 4, V mod 4)` sum pairs
- the table is an equivalence-level consequence of the exact model, so selecting one
  of its cases cannot discard a fixed-`q` solution
- with Z3 `4.16.0`'s direct Boolean SAT tactic and the same five-second seed:
  - the unsplit model processed `50,287` SAT conflicts and returned `unknown`
  - the complete `42`-case selector processed `108,654` SAT conflicts and returned
    `unknown`
- the first complete `42 x 5s` case portfolio also remained unresolved:
  - `42` timeouts / `unknown`
  - `1,580,757` aggregate SAT conflicts
  - per-shard range `27,505 .. 55,488`
- none of these timeouts is evidence for satisfiability or unsatisfiability

Decision:
- keep modulo `4` as the default exact quotient table
- keep `--quotient-case-index` as the completeness-safe campaign partition
- promote the direct Boolean tactic over the generic SMT path for bounded Z3 runs

Follow-up:
- export each case in a solver-neutral pseudo-Boolean format
- use a proof-producing native PB solver and independently check every decisive model
  or refutation

### EXP-2026-07-24-02: Restricted-OPB fixed-`q` proof campaign calibration

Date:
- `2026-07-24`

Area:
- Eliahou fixed-`q` proof-producing pseudo-Boolean campaign

Status:
- Measured/Unresolved

Question:
- can the `42` exact quotient cases be exported compactly enough for native
  cutting-planes/PB solvers and independently verifiable certificates?

Commands:
- `.venv/bin/python py/search_modular_golay.py --export-opb-dir outputs/fixed-q-opb-2026-07-24 --opb-all-cases`
- `/tmp/roundingsat-hadamard --time-limit=5 --lp=0 --print-sol=1 outputs/fixed-q-opb-2026-07-24/fixed-q-mod4-case-00.opb`
- `find outputs/fixed-q-opb-2026-07-24 -name 'fixed-q-mod4-case-*.opb' -print0 | sort -z | xargs -0 -n1 -P4 /tmp/roundingsat-hadamard --time-limit=5 --lp=0`

Solver identity:
- `RoundingSat 2`, branch `HEAD`, commit `d4edbf7`
- downloaded from the official GitLab `build-linux` artifact
- binary SHA-256:
  `cf115250c7539000b39b950d53ddf9d1dfd4ca0d004caf52038a116e7efe3ed5`

Finding:
- each restricted-OPB shard contains:
  - `164` base sign variables
  - `6,642` shared XOR-edge variables
  - `6,806` variables total
  - `26,568` exact XOR-link inequalities
  - `95` equalities: `81` correlations, `4` parity marginals, `2` endpoints, and
    `8` selected quotient residue sums
  - `26,663` declared constraints total
- the campaign writer emits a deterministic manifest with the quotient sums and
  SHA-256 digest of every shard
- an external solver model is accepted only after the repository reconstructs `U,V`,
  recomputes all aperiodic correlations, and reconstructs the full special quadruple
- the first `42 x 5s` RoundingSat campaign remained unresolved:
  - `42` `TIMELIMIT` results
  - `815,945` aggregate conflicts
  - per-shard range `14,977 .. 24,097`
- no model, refutation, or Hadamard matrix was produced

Decision:
- keep the OPB campaign as the strongest exact continuation of the published seed
- use proof logging plus VeriPB checking for any future UNSAT result
- retain the Z3 encoding as an independent model generator, not the only solver path

Follow-up:
- run longer, resource-recorded portfolios with RoundingSat and Exact
- preserve a complete signed model for SAT or one checked proof per quotient shard for
  exhaustive UNSAT coverage
- only after the fixed-`q` campaign is exhausted move the conditional multiplier
  subfamilies ahead of it

### EXP-2026-07-24-03: Reversal quotient of the fixed-`q` OPB campaign

Date:
- `2026-07-24`

Area:
- fixed-`q` exact campaign symmetry

Status:
- Adopted

Question:
- can an exact sequence equivalence reduce the `42` modulo-`4` proof obligations
  without adding a heuristic assumption or weakening eventual UNSAT coverage?

Commands:
- `.venv/bin/python py/test_search_modular_golay.py`
- `.venv/bin/python py/search_modular_golay.py --export-opb-dir outputs/fixed-q-opb-sym26-2026-07-24`
- `.venv/bin/python py/search_modular_golay.py --export-opb-dir outputs/fixed-q-opb-all-2026-07-24 --opb-all-cases`

Finding:
- independent reversal of the length-`81` sequence `V` preserves its aperiodic
  autocorrelations and its even/odd marginals, while leaving `U` and the endpoint
  normalization unchanged
- on the exact modulo-`4` table this involution has:
  - `10` fixed cases
  - `16` two-case orbits
  - `26` orbits covering all `42` raw cases
- the default campaign manifest records `covered_case_indices` for every selected
  representative and hashes every emitted shard
- the regression suite mechanically compares all `26` representative formulas with
  their `42` raw formulas under the explicit base/edge variable permutation
- the raw `42`-case export remains available with `--opb-all-cases`
- this is an exact quotient: a SAT witness transfers by reversing `V`, and a checked
  representative UNSAT proof establishes the paired case when combined with the
  separately tested OPB formula isomorphism; the proof file itself must be
  mechanically transported if a standalone certificate for the omitted raw formula
  is required

Decision:
- use the `26` representatives as the default proof campaign at this stage
- retain the `42`-case path only for reproduction, solver comparison, and independent
  checks of the symmetry argument
- this decision was superseded later the same day by `EXP-2026-07-24-06`, whose
  exact parity sieve reduces the current default to `14` representatives

Follow-up:
- run proof-logging PB solvers on the then-current `26` representatives
- preserve the v2 manifest beside all models, proofs, and resource logs

### EXP-2026-07-24-04: Multiplier-invariant weighted-PB shard geometry

Date:
- `2026-07-24`

Area:
- conditional multiplier-invariant `LP(333)` campaign design

Status:
- Measured calibration

Question:
- can the two surviving order-`3` multiplier subfamilies be represented as a
  finite, symmetry-reduced weighted-PB campaign without materializing the
  memory-heavy two-energy frontier?

Commands:
- `cargo test -p hadamard-cli multiplier_order3_pb_shard_geometry_matches_audit -- --nocapture`

Finding:
- both surviving subgroups reproduce the same `1,296` exact row-marginal pairs
- closure under subgroup-compatible translations, reversal, simultaneous unit
  multiplication, and sequence swap gives `19` safe row-pair orbits per subgroup:
  - `2` orbits of size `36`
  - `17` orbits of size `72`
- the planned campaign therefore has `38` shards
- for each subgroup, direct orbit enumeration gives:
  - `113` coordinate orbits per sequence (`3` fixed and `110` of size `3`)
  - `226` base sign variables
  - `56` independent nonzero shift equations
  - `12,656` shared XOR auxiliaries and `12,882` total variables
  - `50,624` four-inequality XOR links
  - `36,092` aggregated weighted-PB terms, with `222 .. 662` terms per equation
- the coupled graph scan's `(1,11)` label is only a representative of the cheapest
  observed multiplier-ratio class; it is not unique or proved strongest

Decision:
- prefer a streamed, independently checked `38`-shard weighted-PB export over a
  monolithic coupled two-energy DP
- keep the multiplier campaign explicitly conditional on the stabilizer assumption

Follow-up:
- implement the weighted-OPB exporter and a full-length `LP(333)` witness verifier
- test `(1,11)` only as a targeted propagation option before adding it globally

### EXP-2026-07-24-05: Tuned RoundingSat representative portfolio

Date:
- `2026-07-24`

Area:
- fixed-`q` native pseudo-Boolean solver calibration

Status:
- Measured/Unresolved

Question:
- can a parameter setting matched to the cardinality-heavy OPB encoding materially
  improve RoundingSat throughput or decide any of the `26` reversal representatives?

Commands:
- representative tuning command:
  `/tmp/roundingsat-hadamard --time-limit=5 --lp=0 --bits-learned=1 outputs/fixed-q-opb-2026-07-24/fixed-q-mod4-case-00.opb > outputs/exp-2026-07-24-rs-tune-case00-learned1.txt 2>&1`
- full campaign command:
  `find outputs/fixed-q-opb-sym26-2026-07-24 -name 'fixed-q-mod4-case-*.opb' -print0 | sort -z | xargs -0 -n1 -P4 bash -c 'opb_path="$1"; log_path="${opb_path%.opb}.rs30-learned1.txt"; /usr/bin/time -f "wall_seconds=%e\nmax_rss_kb=%M" /tmp/roundingsat-hadamard --time-limit=30 --lp=0 --bits-learned=1 --print-sol=1 "$opb_path" > "$log_path" 2>&1' _`

Solver and campaign identity:
- RoundingSat 2, commit `d4edbf7`; binary SHA-256
  `cf115250c7539000b39b950d53ddf9d1dfd4ca0d004caf52038a116e7efe3ed5`
- v2 campaign-manifest SHA-256
  `f5ebdb966c5b785851f5cb592739f9009f7b6f2949e13b618e0d4ebc8a5bf44f`

Finding:
- on cases `00`, `08`, `20`, and `41` at five seconds:
  - baseline `--lp=0`: `82,085` total conflicts
  - `--ca-slackdiv=1`: `104,343`
  - `--bits-learned=1`: `212,952`
- on a ten-second case-`17` check, `--bits-learned=1` reached `97,389`
  conflicts at `16,596 KiB` peak RSS, versus `41,195` conflicts at `28,884 KiB`
  for baseline `--lp=0`
- the full `26 x 30s` tuned campaign produced:
  - `26` `TIMELIMIT` results
  - `5,503,589` aggregate conflicts; mean `211,676.5`
  - per-shard conflict range `188,403 .. 245,957`
  - `7,622,002` aggregate decisions
  - mean measured wall time `32.00s` per process including startup/teardown
  - mean peak RSS `19,975 KiB`, range `17,856 .. 22,156 KiB`
- no model or refutation was produced; every shard remains unresolved
- conflict throughput is a calibration proxy, not evidence of better eventual proof
  time and not evidence for SAT or UNSAT

Decision:
- use `--lp=0 --bits-learned=1` as the primary RoundingSat configuration for this
  encoding; use `--ca-slackdiv=1` only as limited portfolio diversification
- do not extend blind timeout budgets indefinitely: the next long run should add
  proof logging, checkpointable resource accounting, or a stronger exact quotient

Follow-up:
- test whether a proof-logging run preserves the tuned behavior
- investigate exact quotient strengthening or safe within-shard symmetry breaking
  before committing cluster-scale time

### EXP-2026-07-24-06: Exact lag-parity sieve and reduced fixed-`q` campaign

Date:
- `2026-07-24`

Area:
- fixed-`q` exact quotient strengthening

Status:
- Adopted

Question:
- do the exact lag-disagreement equations have inexpensive modulo-`2`
  consequences that eliminate modulo-`4` quotient cases before native PB search?

Commands:
- `.venv/bin/python py/test_search_modular_golay.py`
- `.venv/bin/python py/search_modular_golay.py --export-opb-dir outputs/fixed-q-opb-parity14-2026-07-24`
- `find outputs/fixed-q-opb-parity14-2026-07-24 -name 'fixed-q-mod4-case-*.opb' -print0 | sort -z | xargs -0 -n1 -P4 bash -c 'opb_path="$1"; log_path="${opb_path%.opb}.rs30-learned1.txt"; /usr/bin/time -f "wall_seconds=%e\nmax_rss_kb=%M" /tmp/roundingsat-hadamard --time-limit=30 --lp=0 --bits-learned=1 --print-sol=1 "$opb_path" > "$log_path" 2>&1' _`

Finding:
- reducing each exact lag-`k` disagreement count modulo `2` gives a linear XOR
  equation
- XORing the lag rows with `k mod 8 in {3,4,5,6}` proves that the number of plus
  entries in `U` residue class `3 mod 4` is even; the fixed odd-index marginal then
  makes the residue-`1` plus count odd
- XORing the rows with `k mod 8 in {4,5,6,7}` similarly proves that the `V`
  residue-`2` plus count is even and the residue-`0` plus count is odd
- these explicit identities reject `20` of the `42` quotient cases:
  `6,7,8,10,12,13,16,17,18,19,21,22,25,26,27,28,30,32,40,41`
- the remaining `22` cases are closed under `V` reversal and form `14` orbits:
  six fixed cases and eight two-case orbits
- the v3 campaign contains `14` OPB shards and has manifest SHA-256
  `7e0a5bb1a9195932d7df23cc3f4fe67ef389c2e815150af9b2e6003b93744ad0`
- the `14 x 30s` tuned RoundingSat run produced:
  - `14` `TIMELIMIT` results
  - `2,657,005` aggregate conflicts; mean `189,786.1`
  - per-shard conflict range `177,064 .. 214,906`
  - `3,704,461` aggregate decisions
  - mean wall time `31.98s`
  - mean peak RSS `18,830 KiB`, range `17,640 .. 20,388 KiB`
- no model or refutation was produced; the surviving shards remain unresolved

Decision:
- replace the `26`-representative default campaign with the exact
  `42 -> 22 -> 14` parity/reversal reduction
- keep raw case numbering and the full `42`-case export for historical reproduction
  and standalone certificates
- treat the parity exclusions as a separate, mechanically tested mathematical lemma,
  not solver evidence

Follow-up:
- run only the `14` surviving representatives in future proof-oriented portfolios
- preserve either a validated SAT model or a checked UNSAT proof for every
  representative; timeouts remain non-results

### EXP-2026-07-24-07: Structure-aware fixed-`q` annealing calibration

Date:
- `2026-07-24`

Area:
- exploratory construction search inside fixed-`q` marginals

Status:
- Measured/Rejected-as-naive

Question:
- does simulated annealing with exact residue-preserving swaps approach zero
  autocorrelation energy more effectively inside a modulo-`4` quotient case than
  under only the coarser parity marginals?

Commands:
- `g++ -O3 -march=native -std=c++17 -DSTART_TEMP=800.0 -DEND_TEMP=4.0 /tmp/fixed_q_anneal.cpp -o /tmp/fixed_q_anneal_800_4`
- representative schema:
  `/tmp/fixed_q_anneal_800_4 SEED 10 MODULUS U0 U1 U2 U3 V0 V1 V2 V3 300000`
- `26` ten-second runs per arm were launched with `12` workers: `modulus=4` for
  the old reversal representatives and `modulus=2` for paired parity-only controls

Finding:
- every move swaps opposite signs within one residue class, preserving the selected
  marginals and fixed endpoints; incremental correlations were periodically checked
  against full recomputation
- modulo-`4` arm: best squared-correlation energy `568`, mean `723.077`, range
  `568 .. 776`, and `229,412,539` moves total
- parity-only control: best energy `584`, mean `686.769`, range `584 .. 744`, and
  `211,788,508` moves total
- neither arm reached energy zero, so no candidate pair was emitted
- the modulo-`4` arm had the best individual run but a worse mean; the sample shows
  no consistent advantage
- `12` of those `26` old representatives were subsequently proved parity-impossible,
  demonstrating why exact algebraic screening must precede heuristic search
- this exploratory source remained temporary:
  - source SHA-256
    `36ba8f73a5ac14a2d71a23551a964ba6251e4efc0ba219fead712104466b9b02`
  - binary SHA-256
    `792bfde62fcc2a87b279617251f4f48741af9b9d19b6ebd198401d596ed3ae2d`

Decision:
- do not promote this temperature schedule and single-swap neighborhood
- retain it only as exploratory calibration; it supplies neither a construction nor
  a lower bound

Follow-up:
- if annealing is revisited, restrict it to the `14` feasible representatives and
  test richer correlation-directed or multi-swap neighborhoods against the same
  parity-only control with repository-integrated source

### EXP-2026-07-24-08: Higher cyclic quotients and exact modulo-`6` propagation A/B

Date:
- `2026-07-24`

Area:
- fixed-`q` cyclic quotient strengthening

Status:
- Measured

Question:
- can a higher cyclic quotient eliminate modulo-`4` cases or improve native PB
  propagation enough to justify its additional state?

Commands:
- `.venv/bin/python /tmp/quotient_analysis.py`
- `rustc -O /tmp/hadamard_m12.rs -o /tmp/hadamard_m12`
- `.venv/bin/python /tmp/augment_m6_energy.py SOURCE_OPB AUGMENTED_OPB --projection both`
- paired shards used
  `/tmp/roundingsat-hadamard --time-limit=5 --lp=0 --bits-learned=1`

Finding:
- direct quotient enumeration grows rapidly:
  - modulo `3`: `144` cases / `80` reversal orbits
  - modulo `5`: `2,258` / `1,146`
  - modulo `6`: `2,560` / `1,312`
  - modulo `8`: `9,496` / `4,796`
  - modulo `12`: `889,024` compatible quotient pairs
- every modulo-`4` case still occurs modulo `8` and modulo `12`, so no current shard
  is eliminated
- the modulo-`6` table factors exactly through ordinary and alternating modulo-`3`
  compressions; a compact exact extended formulation avoids `2,560` selectors
- that formulation adds `340` indicators and `30` equalities per shard, giving
  `7,146` variables and `26,693` constraints
- paired `14 x 5s` results were:
  - baseline: `611,985` conflicts, `873,056` decisions, mean RSS `13,779 KiB`
  - modulo-`6`: `613,160` conflicts, `871,948` decisions, mean RSS `13,868 KiB`
  - all `28` runs reached `TIMELIMIT`
- the conflict change was only `+0.19%`, with no material propagation advantage
- temporary source SHA-256 values were:
  - `quotient_analysis.py`: `b98361442cfa7ce7f92441d5163ed915f7f2465484830c8dd16af1b5f3b6e7d7`
  - `hadamard_m12.rs`: `aec006861148da8d9b60ffc7a37290eb3cd46ba5b1b0a9ff3e7d73c681a33246`
  - `augment_m6_energy.py`: `0afafa73c15a1bdec31c0e6704480f0ecd60e02efc300a3fa164ee12d66ea55d`

Decision:
- do not add the modulo-`6` extension to the default campaign
- do not materialize the larger selector tables without a compatibility argument
  that removes current shards
- retain the compact energy formulation only as possible portfolio diversification

Follow-up:
- prioritize exact lag-system consequences and safe symmetry over larger flat
  quotients

### EXP-2026-07-24-09: RoundingSat proof logging and VeriPB validation

Date:
- `2026-07-24`

Area:
- independently checkable pseudo-Boolean proof pipeline

Status:
- Adopted

Question:
- can the tuned RoundingSat configuration emit proof transcripts for real fixed-`q`
  shards that a pinned independent VeriPB build accepts with the correct conclusion?

Commands:
- `/tmp/roundingsat-hadamard --time-limit=5 --lp=0 --bits-learned=1 --proof-log=outputs/exp-2026-07-24-rs-proof-case14-learned1-5s.pbp outputs/fixed-q-opb-parity14-2026-07-24/fixed-q-mod4-case-14.opb`
- `/tmp/veripb-src-3.0.2/target/release/veripb --stats outputs/fixed-q-opb-parity14-2026-07-24/fixed-q-mod4-case-14.opb outputs/exp-2026-07-24-rs-proof-case14-learned1-5s.pbp`
- the same pipeline was tested on an UNSAT `PHP(4,3)` OPB instance

Tool identity:
- RoundingSat 2, commit `d4edbf7`; binary SHA-256
  `cf115250c7539000b39b950d53ddf9d1dfd4ca0d004caf52038a116e7efe3ed5`
- VeriPB `3.0.2`; official archive SHA-256
  `aced5c32d18119da9ccde5dafdc9b8a831923187644c427b466a4945ea279138`
- locally built verifier SHA-256
  `2dc0ee657a2a27b61d8e8013df973e46e8cf64d328e209c6c8eb3d28056b3375`

Finding:
- the nontrivial `PHP(4,3)` smoke test learned six clauses and ended
  `VERIFIED UNSATISFIABLE`
- at a three-second case-`00` budget, proof logging reduced conflicts from `28,341`
  to `24,071`, about `15%` throughput overhead
- the five-second case-`14` run reached `TIMELIMIT` after `39,892` conflicts and
  `56,599` decisions; its `23,988,075`-byte proof has SHA-256
  `74ed2768403e07a0b94f7b90e22ce8f524d98c5c5feaf271554c636acf99aea8`
- VeriPB checked the partial transcript in `1.21s` and reported
  `VERIFIED NO CONCLUSION`, exactly the required status for a timeout
- raw proof growth was about `4.8 MB/s/shard`; `gzip -1` reduced a sample to `26%`
- VeriPB warned that it switched to unchecked deletion; forced checked deletion
  fails on these RoundingSat transcripts. The
  [official proof-format documentation](https://gitlab.com/MIAOresearch/software/VeriPB/blob/HEAD/proof_format_overview.md)
  states that equisatisfiability/equi-enumerability/equi-optimality guarantees are
  then lost. Deleting constraints still weakens the active formula, so a
  verifier-accepted terminal contradiction from the retained active database is a
  sound UNSAT refutation, but the lost transformation guarantees and this limitation
  must remain explicit
- the Exact Python wrapper was much heavier (`~321 MB` RSS) and did not emit the
  requested proof file, so it is not the preferred certified path

Decision:
- use `--lp=0 --bits-learned=1` with proof logging for decisive native-PB runs
- use the solver's internal time limit so partial transcripts close with conclusion
  `NONE`, and immediately verify/hash any terminal proof
- keep the unchecked-deletion limitation in the proof claim and continue seeking a
  proof-v3 or checked-deletion-safe path

Follow-up:
- retain and compress terminal proofs; keep only sampled timeout transcripts
- validate the first actual terminal fixed-`q` proof with the pinned verifier before
  scaling the remainder of the campaign

### EXP-2026-07-24-10: Normalized involution and reduced `T`-fixed construction probe

Date:
- `2026-07-24`

Area:
- exact fixed-`q` symmetry and restricted construction search

Status:
- Measured/Unresolved

Question:
- does the normalized fixed-`q` model have a within-shard involution that yields a
  useful exact orbit breaker or a much smaller symmetry-fixed construction family?

Commands:
- `.venv/bin/python py/search_modular_golay.py --export-t-fixed-opb-dir outputs/fixed-q-opb-tfixed14-2026-07-24`
- the reduced `14 x 30s` campaign used
  `/tmp/roundingsat-hadamard --time-limit=30 --lp=0 --bits-learned=1 --print-sol=1`
- a direct-Z3 cross-check imposed the same `T` equalities on all `14` surviving
  quotient representatives for ten seconds each

Finding:
- the normalized transformation
  `T(U)_i = -(-1)^i U_(82-i)`, `T(V)_i = (-1)^i V_(80-i)` preserves every exact
  correlation equation, marginal, endpoint, and raw modulo-`4` shard
- affine substitution through all `42` full OPBs mechanically confirmed formula
  invariance
- an exact lexicographic `T` orbit leader can remove essentially half the base
  assignments using `79` auxiliaries and `474` clauses, but a four-case five-second
  calibration changed conflicts by only `-0.9%`
- the six reversal-singleton shards admit an additional one-unit breaker, but its
  short-run conflict throughput fell about `7%`; neither breaker is promoted by
  this timing proxy
- fixing `T` algebraically gives a genuinely smaller construction subfamily:
  - `82` independent base signs
  - `1,560` XOR auxiliaries and `6,240` XOR-link inequalities
  - `40` even-lag equations; every odd-lag autocorrelation vanishes separately
  - `8` quotient equations
  - `1,642` variables and `6,288` constraints total
- the integrated v1 T-fixed manifest has SHA-256
  `6a90b858e84541b778d0f0cec5ca2752c115268464a1b670b554de575663dd8b`
- the integrated generator is normalized-constraint identical to the independently
  audited temporary generator on all `14` shards
- the `14 x 30s` reduced RoundingSat run produced:
  - `14` `TIMELIMIT` results
  - `3,386,209` conflicts; mean `241,872`, range `226,508 .. 254,037`
  - `4,166,053` decisions
  - mean wall time `32.17s`
  - mean peak RSS `13,147 KiB`, range `12,472 .. 13,624 KiB`
- direct Z3 also returned `14 unknown` at ten seconds, after `943,221` aggregate SAT
  conflicts and about `80,547 KiB` mean RSS
- modulo-`2` rank remains feasible in every reduced shard; no construction or
  refutation was found

Decision:
- keep the reduced T-fixed exporter as a reproducible construction probe because a
  SAT witness would be an unconditional fixed-`q` solution
- state its boundary prominently: UNSAT exhausts only the T-fixed subfamily, not the
  full fixed-`q` family
- do not add the full-family lex or singleton unit breaker to the default proof
  campaign until its symmetry/dominance introduction is itself certified

Follow-up:
- use the reduced campaign for bounded solver diversification, not blind long runs
- if an augmented full-shard breaker is revisited, certify the symmetry/dominance
  step rather than treating a refutation of the augmented formula as standalone

### EXP-2026-08-12-01: Primary-source reconstruction and exact verification of `HM(668)`

Date:
- `2026-08-12`

Area:
- order-`668` artifact provenance and independent verification

Status:
- Adopted

Question:
- does the public encoded result reconstruct to an exact Hadamard matrix of order
  `668`, independently of the public status label or construction explanation?

Commands:
- payload: <https://x.com/__alpoge__/status/2087504785952182273>
- decoder/generator: <https://x.com/__alpoge__/status/2087504788938510427>
- `sha256sum fixtures/known/hm-668/matrix.txt`
- `uv run python py/validate_matrix.py fixtures/known/hm-668/matrix.txt`
- the initial Python audit used scalar integer row dot products; a separate exact
  Hamming-distance audit checked rows and columns
- the current Python and native Rust verifiers use independently implemented exact
  bit-packed Hamming checks

Finding:
- decoding the posted wrapper as data and auditing the generated shell instructions
  produced `fixtures/known/hm-668/matrix.txt`; the opaque wrapper itself was not run
- the artifact is exactly `668 x 668`, contains only `+1` and `-1`, and has SHA-256
  `bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0`
- both independent verifiers checked all `222,778` unordered pairs of distinct rows
  exactly and confirmed `H H^T = 668 I`
- the separate row/column audit also checked all `222,778` unordered column pairs at
  exact Hamming distance `334`
- [Epoch AI's status page](https://epoch.ai/frontiermath/open-problems/hadamard)
  reports the order-`668` target as solved while describing its same-day AI/human
  attribution as provisional

Decision:
- adopt the matrix as the repository's verified order-`668` fixture and treat the
  former existence target as solved
- keep the claim boundary explicit: the finite artifact verification is exact; the
  public historical account may evolve, the particular seed's discovery provenance
  still needs explanation, and the general Hadamard conjecture remains open
- shift active priority from witness search to provenance, independent
  reconstruction, and structural analysis while preserving the search experiments
  as historical method work

Follow-up:
- completed in `EXP-2026-08-12-02` for the construction description, structural
  proof, compact seed, native byte-identical reconstruction, and independent
  standard-library Python reconstruction
- still determine how the particular cyclic `Z_166` family was discovered and
  compute deeper structural/equivalence invariants

### EXP-2026-08-12-02: Cyclic `H_4^*` seed and Wallis–Whiteman reconstruction

Date:
- `2026-08-12`

Area:
- order-`668` construction identification and exact reconstruction

Status:
- Adopted

Question:
- what mathematical object is encoded by the first order-`668` constructor, and can
  the stored matrix be rebuilt from a compact seed with a structural orthogonality
  proof?

Commands:
- primary payload: <https://x.com/__alpoge__/status/2087504785952182273>
- decoder/generator: <https://x.com/__alpoge__/status/2087504788938510427>
- `cargo test -p hadamard-construct`
- `cargo run -p hadamard-cli -- build spence --sequences-in fixtures/known/hm-668/spence-sequences.txt --output /tmp/hm668-spence.txt`
- `cmp /tmp/hm668-spence.txt fixtures/known/hm-668/matrix.txt`
- `uv run python py/reconstruct_hm668.py fixtures/known/hm-668/spence-sequences.txt /tmp/hm668-python.txt`
- `cmp /tmp/hm668-python.txt fixtures/known/hm-668/matrix.txt`
- `sha256sum /tmp/hm668-spence.txt`

Finding:
- the decoder's first input is the first `664` signs of the extracted primary sign
  stream, split consecutively into four rows of length `166`
- those `664` concatenated signs have SHA-256
  `6671d64e5118226a7b2b0e7f7be6fe52a371860984fd6152a925a0d73c5919a7`
- the sequence row sums are `[2,0,0,0]`, and the sum of their periodic
  autocorrelations is `-4` at every nonzero shift
- their negative supports form a cyclic type-`H_4^*` difference family
  `(166;82,83,83,83;164)`; complementing all four blocks gives the equivalent
  positive-support family `(166;84,83,83,83;166)`
- for the associated circulants, the exact PAF identity is
  `sum_i C_i C_i^T = 668 I_166 - 4 J_166`
- the decoded border and core are the standard Wallis–Whiteman bordered
  array; its border terms cancel the `-4J` contribution and prove
  `H H^T = 668 I_668`
- `build_spence_hadamard` validates the length, row sums, and PAF identity and then
  applies that exact block formula
- the compact seed reconstruction is byte-identical to `matrix.txt` and has SHA-256
  `bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0`
- the standard-library Python implementation applies the documented array without
  invoking the shell generator or Rust builder and independently produces the same
  bytes and digest

Decision:
- adopt the cyclic type-`H_4^*` / Wallis–Whiteman description as the exact
  structural identification of the fixture
- preserve `fixtures/known/hm-668/spence-sequences.txt` as the compact source seed
  and use the native builder as the direct reconstruction path
- do not infer that this particular seed comes from either of Spence's
  relative-difference-set families or another named seed family without provenance
  evidence; the generic array name does not establish the seed's origin

Follow-up:
- determine how this particular cyclic `(166;82,83,83,83;164)` family was
  discovered or searched for
- compute automorphism, rank, and equivalence fingerprints beyond the proved
  circulant/back-diagonal block structure

## Novel hypothesis queue

This section is a queue of untested or not-yet-completed hypotheses. Its items are not
recorded experiments and do not count toward the experiment total.

- **Seed-discovery provenance:** determine how the cyclic
  `(166;82,83,83,83;164)` family was found and whether it has a reproducible search
  certificate or a known construction beyond the generic bordered array.
- **Independent payload reconstruction:** preserve the now-complete standard-library
  Python reconstruction and its exact agreement with the native builder and stored
  fixture.
- **Structural classification:** compute block/circulant, automorphism, rank, and
  equivalence invariants that can identify the construction family and support
  comparison with independently distributed copies.

The remaining search hypotheses are retained as historical methods work rather than
as the primary route to an order-`668` witness:

- **Fixed-`q` PB completion:** finish the active `164`-variable, `81`-equation exact
  model; add justified symmetry breaking or equation-derived propagation only if it
  preserves the current feasible set.
- **Finite-field tail fingerprints:** evaluate exact compressed residual polynomials at
  roots of unity in machine-word finite fields and use the results as completeness-safe
  tail-join fingerprints before full residual evaluation.
- **Invariant LP(333) PB export:** export the two surviving order-`3`
  multiplier-invariant LP(333) sign-table subfamilies as independently checkable
  pseudo-Boolean models with exact autocorrelation equations.
- **Invariant column marginal plus realization:** enumerate the
  `col_units={1,10,26}` column-sum marginals, then join them to the surviving row
  marginals with exact signed-table realization checks such as Gale-Ryser or
  orbit-aware flow.
- **Constraint-graph separator factorization:** use frontier/min-fill structure of the
  autocorrelation constraint graph to choose exact-tail factors and separator
  signatures, instead of another contiguous or parity split.
- **Stochastic SDS with exact projection:** test stochastic SDS search only with an
  exact projection or repair phase, structure-aware neighborhoods, multiple seeded
  trials, and a recorded random-start baseline.
