# Open Hadamard orders and research problems

Sources checked on `2026-11-17`.

## Construction records

- The Hadamard Matrix Explorer, version 58 (October 2026), provides construction
  formulas and matrix images through order `10000`. Its unknown-order page lists
  `85` admissible orders with no construction in the program. The first four
  are `2396`, `2676`, `2876`, and `2956`. [Explorer](https://www.math74.com/Hadamard/) ·
  [Unknown orders](https://www.math74.com/Hadamard/unknown.html).
- Egan, Kotsireas, London and Ó Catháin's survey verifies the twelve August
  matrices, describes the construction families, and gives a dated comparison
  point. Table 5 lists `95` gaps in both versions checked. Comparing the sets,
  the Explorer covers these ten entries from Table 5: `2572`, `3356`, `3436`,
  `3820`, `4204`, `5912`, `6712`, `6796`, `8248`, and `8696`.
  [October preprint, Table 5](https://arxiv.org/html/2610.27175v1#S4.T5) ·
  [November revision, Table 5](https://arxiv.org/html/2610.27175v2#S4.T5).
- Kumar's pinned register tabulates the same `83` open orders of the form `4n`
  with `n` odd through `10000`. Its separate discussion of `8p` also identifies
  the Explorer's other two gaps, `4792` and `5752`. Use the methods note for
  estimates tied to specific search structures.
  [Kumar's frontier](https://raw.githubusercontent.com/vatskishan03/Hadamard/d885022/FRONTIER.md) ·
  [Methods and cost model](https://raw.githubusercontent.com/vatskishan03/Hadamard/d885022/METHOD.md).

## Search shortlist

The ranking below reflects the cost of getting an informative result with the
repository's tools. Published work estimates concern specific search families and
depend on candidate quality and join throughput.

| Order | Why it is worth testing | First experiment |
| --- | --- | --- |
| **4412 = 4 × 1103** | Best fit for the existing cyclic Goethals–Seidel tools. A common multiplier subgroup of order `19` gives `59` position orbits and `29` nonzero shift classes. The only sorted positive row-sum signature is `(1,37,39,39)`. Each raw row pool still has about `3 × 10^16` assignments. | Calibrate orbit-swap generation and partial-signature joins on the verified `2092` seed, then measure the near-target yield at `4412`. Kumar's pilot estimates about `10^-9` solutions per partial-sum target under its own setup. |
| **4748 = 4 × 1187** | Best bordered alternative. The core is `Z_1186 ≅ Z_2 × Z_593`. Kumar's work estimate for a bordered structure is slightly lower than the estimate for `4412`. Our parity-character screen leaves subgroup order `8` as the largest possible common cyclic multiplier, with absolute alternating sums `(0,2,30,62)` after ordering the last three rows. This specific subgroup still has `150` position orbits. | Measure whether separate parity rows and quotient correlations reduce the candidate pools far enough for matching. |
| **2956 = 4 × 739** | A smaller-order option with a clear cyclic search. The subgroup of order `9` gives `83` position orbits and `41` shift classes. Its only sorted positive row-sum signature is `(1,19,35,37)`. Kumar estimates much higher matching work for this family than for `4412`. | Use as a secondary target if the join can handle more shift coordinates. Kumar's witness for `5912` supplies the doubled order independently. |

The `4412` and `2956` row-sum checks enumerate positive odd quadruples of
square sum `N` and filter by `r ≡ ±1 (mod 2h)`, where `h` is the odd multiplier
order. The `4748` check uses the [parity-character identity](hm-2396-frontier.md#walliswhiteman-bordered-array-at-length-598)
with normalized row sums `(2,0,0,0)` and even-position orbit sizes
`1` and `d` repeated `592/d` times for each divisor `d` of `592`. The
existing `2396` campaign remains the route to the smallest open order.
[Published estimates and nearby constructions](https://raw.githubusercontent.com/vatskishan03/Hadamard/d885022/FRONTIER.md) ·
[Pilot details](https://raw.githubusercontent.com/vatskishan03/Hadamard/d885022/METHOD.md#limitations).

### Two exact obstructions for order 7572

Kumar's methods note suggests a two-row search at `7572 = 4 × 1893` with
four blocks of the form `A, B, 2B, 4B`, where the multipliers act on the
indices of `B`. The stated orbit counts are realized by `H = <8>` of order
`30` for `B` and `K = <2>` of order `90` for `A`, modulo `1893 = 3 × 631`.
[Kumar's block-symmetry model](https://raw.githubusercontent.com/vatskishan03/Hadamard/d885022/METHOD.md#block-symmetries).

Compress each row by summing its signs separately in the three residue classes
modulo `3`. Both groups contain a multiplier that swaps classes `1` and `2`,
so the compressed rows have the form `(a,b,b)`. At a nontrivial character
of `Z_3`, such a row has squared Fourier magnitude `(a-b)^2`. Multiplication
by `2` leaves that energy unchanged. For differences `u` and `v` from `A`
and `B`, complementarity thus requires `u² + 3v² = 7572`. The compressed
coordinates are odd, so `u` and `v` are even. The only absolute solution is
`(42,44)`.

The subgroup of `K` acting trivially modulo `3` has order `45`. Each residue
class contains one fixed position and fourteen orbits of size `45`. Its signed
sum is therefore `±1` modulo `90`, which forces `u ≡ 0, ±2 (mod 90)` and
excludes `|u| = 42`. This rules out the stated two-row model.

The frontier table's plain order-`42` multiplier model with `62` orbits and
`31` shift classes is `<62>` modulo `1893`. Its corresponding subgroup fixing
the residue classes has order `21`, with one fixed position and thirty
orbits of size `21` in each class. The differences are `0, ±2` modulo `42`.
The only possible sorted absolute differences with square sum `7572` are
`(42,44,44,44)`. But a normalized row with sum `1` has squared quotient
energy `4`, and a row with sum `43` has quotient energy in `{1600,7396}`.
The two possible sorted positive row-sum signatures `(1,1,1,87)` and
`(43,43,43,45)` each require three rows that cannot supply a difference
of `42` or `44`. To reproduce the row and energy values, start with the
zero vector in `Z³` and add or subtract each position orbit's vector of
three residue-class sizes. The full position orbit sizes are `1`, `2`,
thirty `21`s and thirty `42`s. Both exclusions concern the named search
families at `7572`.
[Kumar's frontier table](https://raw.githubusercontent.com/vatskishan03/Hadamard/d885022/FRONTIER.md).

## Open problems in the survey

Section 5 of the October preprint collects the following problems. Problems
`1`, `2`, and `10` map directly onto tooling already in this repository.
[Open problems, version 1](https://arxiv.org/html/2610.27175v1#S5) ·
[Open problems, version 2](https://arxiv.org/html/2610.27175v2#S5).

| Problems | Subject and connection to this repo |
| --- | --- |
| `1–2` | Legendre pairs at all odd lengths and Kotsireas's prescribed `q²`-compression lift. For `333 = 37 × 3²`, Problem 2 gives a specific target for a mod-9 compression experiment using the LP/CRT work. |
| `3–4` | Turyn-type sequences at even lengths and skew Hadamard existence at every admissible order. Turyn-type seeds supply one route to ingredients for product constructions. |
| `5–7` | Equality cases for the Barba and Ehlich determinant bounds and families approaching determinant bounds. Problem 5 singles out `85` and `145` as the smallest undecided orders in the survey. |
| `8–9` | Existence of fourth-root Butson matrices at every even order and sixth-root matrices at `2p`. The survey's symmetric `BH(86,4)` construction for `H(2076)` illustrates how a quaternary ingredient can close a real order. |
| `10–11` | Universal modular Hadamard existence at moduli `128` and `7` for orders divisible by four. Problem 10 connects to the [64-modular seed and lift study](modular-golay-167.md). |
| `12–13` | Infinite progressions of known Hadamard orders and a uniform bound on the exponent of two needed for all odd factors. |
| `14–16` | Skew Hadamard difference sets in abelian groups, Gauss-sum determinant bounds for explicit families, and cocyclic Hadamard existence. |
