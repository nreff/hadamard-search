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
