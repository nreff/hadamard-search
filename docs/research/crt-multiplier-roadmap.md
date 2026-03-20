# CRT And Multiplier Roadmap For LP(333)

## Why This Is The Right Next Bet

Recent search work has mostly exhausted local engineering wins on the current exact-tail path:

- richer local seam keys lost on wall-clock time
- cross-branch caching lost
- adaptive handling of the current `6+6` join lost
- the best kept result is still a constant-factor improvement, not a search-geometry change

The seam-key result is still the strongest clue in the codebase: one exact boundary equation cut tail multiplicity by orders of magnitude. That strongly suggests the next breakthrough, if there is one, is more likely to come from a larger exact algebraic sieve than from another refinement of the current search implementation.

For LP(333), the most natural unexploited structure is:

- `333 = 9 * 37`
- `Z_333 ~= Z_9 x Z_37`
- `U(333)` has order `phi(333) = phi(9) * phi(37) = 6 * 36 = 216`

That points to two primary mathematical lines:

1. exact CRT-derived necessary conditions
2. multiplier / automorphism constraints

## Main Questions

### 1. CRT decomposition of the LP identity

Write the LP identity in the group ring / polynomial form:

- `A(x) A(x^-1) + B(x) B(x^-1) = 333 - 2 * (x^333 - 1) / (x - 1)`

Then rewrite it under the CRT identification:

- `Z[x] / (x^333 - 1)`
- into components corresponding to the factorization through `Z_9 x Z_37`

The immediate goal is not a construction. It is to answer:

- what exact necessary conditions do valid LP(333) pairs induce on the `9` and `37` components?
- are there componentwise invariants that can be checked before any DFS branching?
- do some frequency families in the `9 x 37` character grid force stronger conditions than the current PSD treatment expresses?

### 2. Multiplier / automorphism sieve

Determine whether LP(333) admits any nontrivial multiplier action that can be used as a hard normalization or sieve.

The practical questions are:

- which units `t in U(333)` could plausibly act as multipliers?
- what equivalence should be allowed: translation, negation, swapping the pair, or more general sequence symmetries?
- if a subgroup survives, what orbit constraints does it impose on coordinates of a valid pair?

The best outcome would be a true orbit reduction. Even a weaker outcome, such as a small exact congruence obstruction or a forced normalization on one CRT component, would still be valuable.

## Concrete Near-Term Tasks

### Paper-first tasks

1. Derive the LP identity explicitly in the product-group basis for `Z_9 x Z_37`.
2. Separate the Fourier characters into:
   - pure `Z_9` characters
   - pure `Z_37` characters
   - mixed characters
3. Identify which of those classes yield exact equalities versus only PSD-style inequalities.
4. Work out what a multiplier action would mean for LP pairs rather than just difference sets.
5. Record which normalizations remain valid under the current search conventions.

### Code-support tasks

1. Add a small analysis utility that can inspect compressed or exact candidates by CRT component rather than only by cyclic order.
2. Add a way to summarize frequency families by `(freq mod 9, freq mod 37)` instead of a flat frequency index.
3. Build a dedicated solver for the exact row-sum and column-sum marginal problems rather than relying on the current generic `compression 37` / `compression 9` benchmark path.
4. If the paper derivation yields a candidate sieve, implement it first as a cheap verifier on full assignments before trying to push it into the recursive search.
5. Only move a CRT or multiplier condition into the hot search path after it is proven exact and benchmarked on reduced anchors.

## What Not To Do Next

Until the algebraic questions above are better understood, avoid spending time on:

- another cache layer on the current `6+6` exact-tail join
- another local shift-summary variation of the seam key
- another generic DFS ordering heuristic
- another naive MITM split

Those lines have already been explored enough to show diminishing returns.

## Fallback If The Algebraic Sieve Is Weak

If the CRT and multiplier work produces no strong exact filter, the next major algorithmic experiment should be a genuinely different exact-tail factorization rather than another tweak of the present one.

The most plausible candidate at that point is:

- a `4+4+4` factorized exact-tail path for the `12`-deep regime

That should only be attempted after the algebraic analysis, because a useful CRT-derived invariant might fit naturally into such a factorization.

## Current Recommendation

Proceed in this order:

1. derive the CRT form of the LP identity
2. investigate plausible multiplier actions on LP(333)
3. turn any exact resulting condition into a standalone verifier
4. benchmark it on the reduced-length anchors
5. only then decide whether to invest in a new search architecture such as `4+4+4`

Starting point:

- [docs/research/crt-fourier-notes.md](/home/nate/projects/hadamard/docs/research/crt-fourier-notes.md)
