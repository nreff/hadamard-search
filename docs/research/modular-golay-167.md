# Exact Lift Of The 64-Modular Length-167 Golay Quadruple

Last updated: 2026-07-24

## Goal And Claim Boundary

Eliahou's 2025 construction gives a `64`-modular Hadamard matrix of order `668`,
not a real Hadamard matrix. Its four length-`167` sequences nevertheless provide
the closest explicit starting point currently known for a Goethals-Seidel route.

Primary source:

- [S. Eliahou, *A 64-modular Hadamard matrix of order 668*](https://ajc.maths.uq.edu.au/pdf/93/ajc_v93_p422.pdf)

The source uses the special quadruple

```text
(s, s', s*q, (s*q)')
```

where `'` preserves the first `84` signs and negates the last `83`. The published
run-length encodings, both starting with `+1`, are

```text
q = (83, 2, 81, 1)
s = (4)^5 (2,1,1)^5 (1,5) (4)^4 (2,1,1)^6
    (4)^4 (3) (1,2,1)^5 (3) (4)^4 (3) (1,2,1)^5
```

The independent reproduction in this repository recovers the source's `13`
nonzero aggregate aperiodic correlations:

```text
k:   4   8   12  16  26  30  34  38  42  46  50  54  58
c: -512 384 -256 128 -64 128 -192 256 -320 256 -192 128 -64
```

## Exact Fixed-`q` Reduction

Let `e_i=+1` for `0 <= i < 84` and `e_i=-1` otherwise, so `s'=e*s`.
At lag `k`, an edge `(i,i+k)` contributes

```text
s_i s_(i+k) (1 + e_i e_(i+k)) (1 + q_i q_(i+k)).
```

It is therefore nonzero exactly when both endpoints lie in the same half and
have the same `q` sign. For the published `q`, define

```text
U = s[0..82]    length 83
V = s[85..165] length 81
```

Then `s[83]` is invisible to every correlation equation. The two isolated
negative-`q` coordinates `s[84]` and `s[166]` contribute only at lag `82`.
The full exact problem for this fixed `q` is consequently

```text
C_U(k) + C_V(k) = 0,  k = 1..81
u_0 u_82 + s_84 s_166 = 0.
```

This is an equivalence inside the fixed-`q` special family, not merely a
necessary filter.

## Forced Marginals

The norm identity is

```text
N(U) + N(V) = 164 + epsilon * (z^82 + z^-82),
epsilon = u_0 u_82.
```

Evaluation at `z=1` rules out `epsilon=+1`, because it would express
`166 = 6 mod 8` as a sum of two odd squares. Hence

```text
u_0 u_82 = -1
|sum(U)| = |sum(V)| = 9
s_84 s_166 = +1.
```

Evaluation at `z=-1` forces both alternating sums to have absolute value `9`.
After independent global sign normalization of `U` and `V`, the exact Boolean
cardinalities are

```text
U even indices: 21 plus of 42
U odd indices:  25 plus of 41
V even indices: 25 plus of 41
V odd indices:  20 plus of 40
u_0 = +1, u_82 = -1.
```

The resulting Goethals-Seidel row-sum magnitudes can only be

```text
(21, 15, 1, 1)
(19, 17, 3, 3)
```

which are two of the ten exact `Z_167` SDS parameter families already encoded
by the repository.

## Exact Pseudo-Boolean Model

For each lag, let a Boolean edge variable mean that its endpoint signs differ.
The equation `C_U(k)+C_V(k)=0` is exactly the cardinality constraint

```text
number of disagreeing lag-k edges = 82 - k.
```

The complete model has

```text
164 sign variables
6,642 XOR edge incidences
81 correlation cardinality equalities
4 parity-class cardinality equalities
2 fixed endpoint literals
```

## Exact Modulo-`4` Quotient

Let

```text
U_r = sum of U_i with i = r mod 4
V_r = sum of V_i with i = r mod 4.
```

Reducing the exact norm identity modulo `x^4 - 1` gives the periodic
autocorrelation identity

```text
PAF(U)_t + PAF(V)_t = (164, 0, -2, 0)_t.
```

The residue classes have fixed original parity, so the forced marginals also give

```text
U_0 + U_2 = 0,  U_1 + U_3 = 9,
V_0 + V_2 = 9,  V_1 + V_3 = 0.
```

Enumerating the bounded integer residue sums and enforcing the four quotient PAF
equations leaves exactly `42` ordered `(U,V)` residue-sum pairs. This is not a
heuristic filter: every exact fixed-`q` solution belongs to exactly one table case.
The script can assert the complete table or select one case as an exhaustive shard.

There is a further exact parity sieve that is easy to certify without trusting the
enumerator. Write a sign as a bit that is `1` exactly for `+1`. Reducing the lag-`k`
disagreement equation modulo `2` gives

```text
XOR_i (x_i XOR x_(i+k)) = (82-k) mod 2.
```

XORing the lag rows with `k mod 8` in `{3,4,5,6}` leaves precisely the bits
`U_i` with `i = 3 mod 4` and right-hand side zero. Thus the `U_3` plus count is
even; the odd-index marginal of `25` then forces the `U_1` plus count to be odd.
Similarly, the lag rows with `k mod 8` in `{4,5,6,7}` force the `V_2` plus count
to be even, so the even-index marginal of `25` forces the `V_0` plus count to be
odd. These two consequences reject raw cases

```text
6, 7, 8, 10, 12, 13, 16, 17, 18, 19,
21, 22, 25, 26, 27, 28, 30, 32, 40, 41.
```

The exact proof obligation therefore falls from `42` raw cases to `22`
parity-compatible cases before a PB solver is invoked.

Independent reversal of the length-`81` sequence `V` preserves every aperiodic
autocorrelation, every forced marginal, and the fixed-`q` endpoint condition. On
the quotient table it fixes `10` cases and pairs the other `32` cases into `16`
two-case orbits. The `22` parity-compatible cases are closed under this action and
form `14` orbits: six fixed cases and eight two-case orbits. Therefore only `14`
representative OPB shards remain. The v3 manifest records the parity exclusions and
all reversal-covered raw case IDs; exporting all `42` remains available for
calibration and standalone per-case proofs.

A five-second calibration with Z3's direct Boolean SAT tactic processed roughly
twice as many conflicts with the table as without it, but a complete `42 x 5s`
portfolio left every shard `unknown`. The result justifies the partition; it does
not decide any shard.

## Proof-Oriented OPB Campaign

The repository can linearize each selected quotient shard into the restricted OPB
format used by current pseudo-Boolean solvers. An edge variable is shared for every
XOR occurrence. Four linear inequalities enforce each exact relation
`e = x XOR y`.

Each shard has

```text
164 base sign variables
6,642 XOR edge variables
6,806 variables total
26,568 XOR-link inequalities
95 equalities
26,663 constraints total
```

The `95` equalities comprise the `81` correlation equations, four parity-class
cardinalities, two endpoint literals, and eight selected residue-sum equations.
The campaign manifest records the SHA-256 digest of every representative shard,
the raw case indices covered by that shard, and the explicit residue/base/edge
permutation induced by reversal.

This representation is aimed at native cutting-planes solvers such as
[RoundingSat](https://gitlab.com/MIAOresearch/software/roundingsat) or
[Exact](https://gitlab.com/nonfiction-software/exact). Any UNSAT result must carry a
proof accepted by [VeriPB](https://veripb.org/); any SAT model must pass the repository's
independent reconstruction before it is treated as a construction.

Any satisfying assignment must be reconstructed into the four length-`167`
sequences, checked against all `166` aperiodic correlation equations, built with
the Goethals-Seidel array, and finally checked over the integers. A solver timeout
is `unknown`; it is never evidence of unsatisfiability.

## Normalized Involution Construction Probe

The normalized model has an exact within-shard involution

```text
T(U)_i = -(-1)^i U_(82-i),
T(V)_i =   (-1)^i V_(80-i).
```

It sends each individual autocorrelation to `(-1)^k C(k)`, preserves the forced
marginals and endpoints, and fixes every raw modulo-`4` quotient case. Restricting
to `T(U,V)=(U,V)` is therefore a sound construction subfamily. Odd-lag
autocorrelations vanish individually, reflection pairs the remaining even-lag
edges, and direct affine substitution gives each reduced shard

```text
82 independent sign variables
1,560 XOR edge variables
40 even-lag equations
8 quotient equations
1,642 variables and 6,288 constraints total.
```

The separate manifest and verifier label the claim boundary: a reconstructed SAT
model is an unconditional fixed-`q` solution, but UNSAT excludes only the `T`-fixed
subfamily. Fourteen 30-second RoundingSat probes and fourteen ten-second Z3 probes
all timed out, so this is a reproducible construction target rather than a result.

Repository commands:

```bash
uv sync
uv run python py/search_modular_golay.py --audit-only
uv run python py/search_modular_golay.py --timeout-seconds 300 --smt2-out outputs/modular-golay-fixed-q.smt2 --output outputs/modular-golay-fixed-q.txt
uv run python py/search_modular_golay.py --quotient-case-index 0 --timeout-seconds 300
uv run python py/search_modular_golay.py --export-opb-dir outputs/fixed-q-opb
uv run python py/search_modular_golay.py --export-t-fixed-opb-dir outputs/fixed-q-opb-t-fixed
uv run python py/search_modular_golay.py --export-opb-dir outputs/fixed-q-opb-all --opb-all-cases
roundingsat --lp=0 --bits-learned=1 --print-sol=1 --proof-log=outputs/fixed-q-case-00.pbp outputs/fixed-q-opb/fixed-q-mod4-case-00.opb > outputs/fixed-q-case-00.model
veripb outputs/fixed-q-opb/fixed-q-mod4-case-00.opb outputs/fixed-q-case-00.pbp
uv run python py/search_modular_golay.py --verify-opb-model outputs/fixed-q-case-00.model --output outputs/modular-golay-fixed-q.txt
uv run python py/search_modular_golay.py --verify-t-fixed-opb-model outputs/fixed-q-t-fixed-case.model --output outputs/modular-golay-fixed-q.txt
uv run python py/test_search_modular_golay.py
```

The script writes a candidate only after recomputing all aperiodic correlations.
The Rust `build gs` command then provides an independent periodic-correlation and
full-matrix construction path for the four emitted sequences.

## Runs On 2026-07-10

Exact solver calibration:

| Encoding | Limit | Result |
| --- | ---: | --- |
| Z3 4.16 native pseudo-Boolean | 300 seconds | `unknown (timeout)` |
| OR-Tools CP-SAT with explicit XOR variables | 300 seconds, 12 workers | `UNKNOWN` |

The CP-SAT run explored `1,567,071` branches and reported `97,717` conflicts.
Neither run found a model or proved unsatisfiability. Direct CNF encodings with
`64,116` variables and `227,384` clauses were also tried with CaDiCaL and Kissat;
both were manually interrupted without a result after exceeding the intended
five-minute calibration window. Those interrupted runs establish neither
satisfiability nor unsatisfiability.

Naive local repair was also calibrated and did not reach zero. Those timings and
scores are recorded in `docs/EXPERIMENT_LOG.md`; they are not lower bounds.

## Runs On 2026-07-24

Exact quotient and proof-campaign calibration:

| Encoding | Limit | Result |
| --- | ---: | --- |
| Z3 4.16 direct SAT, complete modulo-`4` selector | 5 seconds | `unknown`; `108,654` conflicts |
| Z3 4.16 direct SAT, 42 selected cases | 5 seconds each, 4 workers | `42 unknown`; `1,580,757` aggregate conflicts |
| RoundingSat 2 `d4edbf7`, restricted OPB, 42 cases | 5 seconds each, 4 workers | `42 TIMELIMIT`; `815,945` aggregate conflicts |
| RoundingSat 2 `d4edbf7`, `--bits-learned=1`, 26 reversal reps | 30 seconds each, 4 workers | `26 TIMELIMIT`; `5,503,589` aggregate conflicts |
| RoundingSat 2 `d4edbf7`, `--bits-learned=1`, 14 parity/reversal reps | 30 seconds each, 4 workers | `14 TIMELIMIT`; `2,657,005` aggregate conflicts |

These portfolios are bounded solver calibrations. They found no model and proved
no shard unsatisfiable. The durable result is the exact quotient decomposition,
solver-neutral campaign export, exact `42 -> 22 -> 14` parity/reversal reduction,
and independent witness-verification path.

## Ranked Hypotheses

### H1. Exact fixed-`q` completion

Run the `14` parity-compatible symmetry-representative modulo-`4` OPB shards with a native
proof-logging PB portfolio.
This is the most direct route because a satisfying assignment immediately supplies
a Goethals-Seidel `HM(668)`, while checked UNSAT proofs for every shard would
exhaust the `22` parity-compatible cases when combined with the independently
tested reversal-isomorphism lemma; the exact lag-parity proof excludes the other
`20`. Together these exhaust the fixed-published-`q` family. Retaining all `42`
raw shards is the fallback if standalone per-instance certificates are required.

### H2. Staged modular lift

Use the published `64`-modular solution as the base level and impose the same
edge equations successively modulo `128`, `256`, `512`, and `1024`. At modulus
`1024`, the bounded integer correlations make modular zero equivalent to exact
zero. The hypothesis is that staged feasible sets give better propagation than
the monolithic equalities.

### H3. Finite-field tail fingerprints for `LP(333)`

Evaluate the exact residual polynomial at roots of unity in machine-word finite
fields before decoding full factorized tails. For reduced length `111`, modulus
`223` supports an order-`111` root; powers representing character orders `111`,
`37`, and `3` cover the CRT spectral families. This is exact modulo the chosen
prime and has no false negatives, but its wall-clock value is unmeasured.

### H4. Pseudo-Boolean export of invariant `LP(333)` families

The two surviving order-`3` multiplier hypotheses each leave `113` coordinate
orbits per sequence. Export their `226` sign variables and exact weighted-XOR
autocorrelation equations rather than extending the large in-process frontier
DP. This remains conditional on multiplier invariance.

### H5. Invariant column marginals plus table realization

Enumerate the column marginals fixed by `col_units={1,10,26}`, then join them to
the `1,296` surviving row marginals with an exact signed-table realization test.
This couples the currently separate pure-row and pure-column CRT information.

### H6. Constraint-graph separator factorization

Choose the proposed `4+4+4` tail factors from autocorrelation graph separators
and carry only exact separator bilinear signatures. This differs from the tested
contiguous and parity splits and should be accepted only if it beats the current
reduced-length-`17` wall-clock anchor.

### H7. Stochastic `Z_167` SDS with exact projection

The repository has not run a real `Z_167` SDS campaign. A stochastic search is
worth testing only with fixed block cardinalities, incremental difference counts,
an exact repair/projection phase, seeded repetitions, and random-start controls.
The naive modular warm projection did not outperform its random baseline.

## Acceptance Rules

- Never report `unknown` as `unsat`.
- Independently recompute every correlation from a returned assignment.
- Build the full `668 x 668` matrix and verify `H H^T = 668 I` over integers.
- Preserve solver inputs, versions, seeds, limits, raw output, and proof artifacts.
- Treat multiplier-invariant results as conditional subfamily results only.
