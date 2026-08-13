# The Verified Order-668 Construction

This note identifies the exact structure of the matrix in
[`fixtures/known/hm-668/matrix.txt`](../../fixtures/known/hm-668/matrix.txt).
It separates what follows mechanically from the public payload from what remains
unknown about how its seed was discovered.

## Result in one paragraph

The first `664` signs of the stream extracted from Levent Alpöge's primary X
[`+/-` payload](https://x.com/__alpoge__/status/2087504785952182273) split into four
length-`166` sequences. Their negative supports form a cyclic difference family of
type `H_4^*` with parameters

```text
(166; 82,83,83,83; 164).
```

Equivalently, their positive supports form
`(166; 84,83,83,83; 166)`. Substitution into the standard Wallis–Whiteman
bordered array (also presented as a Spence SDS construction) gives a matrix of order
`4(166+1) = 668`. The repository's native implementation reconstructs the stored
matrix byte for byte. This identifies the array and the exact seed family; it does
not yet identify how this particular cyclic family was found.

## Primary data and compact seed

The two primary public inputs are:

- payload: <https://x.com/__alpoge__/status/2087504785952182273>
- decoder/generator: <https://x.com/__alpoge__/status/2087504788938510427>

The extracted primary `+/-` sign stream contains `23,828` characters and has
SHA-256 `5b5fe8fa42f0d6a8b4e4c9926726d82a6aab8e1070c1ae4d1b430c1277e58db4`.
The decoder's first constructor consumes its first `664` signs; concatenated, those
signs have SHA-256
`6671d64e5118226a7b2b0e7f7be6fe52a371860984fd6152a925a0d73c5919a7`.
Splitting them consecutively into `S_0,S_1,S_2,S_3`, each of length `166`, gives the compact
repository seed
[`spence-sequences.txt`](../../fixtures/known/hm-668/spence-sequences.txt). Its
SHA-256 is
`bff1d35f7b8ebbbeada18e6d765bf53ded9574edd5b2bc55fa200da069d05e8a`.

For exact comparison, the SHA-256 values of the `166` ASCII signs in each row,
excluding the terminating line feed, are:

| Sequence | SHA-256 |
| --- | --- |
| `S_0` | `ab43e555e591a019a21a6752c4683f9f1c99b9b8e58a27d00a8ae22559d419b5` |
| `S_1` | `57ff062e013e3c7fee2f17670b77a28dd6c673410f63e03d876736fcab2222d8` |
| `S_2` | `c374bf559769e365cacf056199f6cfcfe1753d7227d0523ae0d1e91efa8447ed` |
| `S_3` | `e54613dac2d3828a90aff373d5dfb872fce258c59731b6d62a6cf53ee1e16900` |

The plus counts, minus counts, and row sums are:

| Sequence | `+` | `-` | Row sum |
| --- | ---: | ---: | ---: |
| `S_0` | 84 | 82 | 2 |
| `S_1` | 83 | 83 | 0 |
| `S_2` | 83 | 83 | 0 |
| `S_3` | 83 | 83 | 0 |

Thus the row-sum vector is exactly `[2,0,0,0]`.

## Difference-family and autocorrelation identity

Let `G = Z_166`, and let

```text
D_i = {x in G : S_i(x) = -1}.
```

The block sizes are `[82,83,83,83]`. For every nonzero `g in G`, exact counting
gives

```text
sum_i |D_i intersection (D_i + g)| = 164.
```

Therefore the negative supports have parameters
`(166;82,83,83,83;164)`. They are of type `H_4^*` because

```text
82 + 83 + 83 + 83 - (166 + 1) = 164.
```

Taking complements gives the equivalent positive-support parameters
`(166;84,83,83,83;166)`.

For a sign sequence obtained from a block of size `k`, with difference count
`N(g)`, the periodic autocorrelation at a nonzero shift is

```text
PAF(g) = 166 - 4(k - N(g)).
```

Summing this identity over the four blocks proves

```text
sum_i PAF(S_i, g) = -4                 for every nonzero g,
sum_i PAF(S_i, 0) = 4 * 166 = 664.
```

Leung, Momihara, and Xiang define type `H_4^*` difference families and state the
Wallis–Whiteman consequence in
[*A new family of Hadamard matrices of order
4(2q^2+1)*](https://doi.org/10.1016/j.disc.2020.112163), also available as
[arXiv:1907.02623](https://arxiv.org/abs/1907.02623). In their terminology, an
`H_4^*` family in a group `G` yields a Hadamard matrix of order `4(|G|+1)` through
the Wallis–Whiteman array.

## Exact bordered block array

Let `C_i` be the `166 x 166` circulant with

```text
C_i(r,c) = S_i((c-r) mod 166),
```

and let `P` be the back-diagonal permutation matrix, so right multiplication by
`P` reverses the columns. The `664 x 664` core implemented by
`build_spence_hadamard` is

```text
Q = [ C0    C1 P       C2 P       C3 P      ]
    [-C1 P  C0         C3^T P    -C2^T P    ]
    [-C2 P -C3^T P     C0          C1^T P   ]
    [-C3 P  C2^T P    -C1^T P      C0       ].
```

Write `1` for the all-one column of length `166`, and define

```text
T = [- + + -]      R = [- - - +]      L = [+ + + -]
    [+ - + -]          [- - + -]          [- - + -]
    [+ + - -]          [- + - -]          [- + - -]
    [- - - -]          [+ - - -]          [+ - - -].
```

Here the displayed signs mean `+1` and `-1`. The full matrix is

```text
H = [       T          R tensor 1^T ]
    [ L tensor 1             Q     ].
```

This is exactly the border and core emitted by the public decoder's first
constructor and by the repository's native implementation.

## Why the array is Hadamard

This section gives a direct check for the exact sign convention above. It is the
specialized `Z_166` instance of the standard Wallis–Whiteman theorem cited earlier.

The autocorrelation identity is equivalent to

```text
sum_i C_i C_i^T = 668 I_166 - 4 J_166.
```

The circulants commute, and the back-diagonal matrix satisfies
`P^2=I` and `P C_i P=C_i^T`. Direct block multiplication of the displayed core
therefore gives

```text
Q Q^T = I_4 tensor (668 I_166 - 4 J_166).
```

The three border sign arrays satisfy

```text
T T^T = R R^T = L L^T = 4 I_4,
T L^T = -2 R.
```

Because the four sequence row sums are `[2,0,0,0]`, every block row of `Q` has
row sum `2` only in its diagonal `C_0` block and row sum `0` in its other three
blocks. The consequences are exact:

- top against top:
  `T T^T + 166 R R^T = 4 I_4 + 664 I_4 = 668 I_4`;
- bottom against bottom: the left border contributes `4 J_166` to each diagonal
  block, cancelling the `-4 J_166` term in `Q Q^T`;
- top against bottom: `T L^T + 2R = 0`.

Hence `H H^T = 668 I_668` without an exhaustive row-pair calculation. The two
independent all-pairs verifiers remain valuable implementation-level cross-checks
of the stored artifact.

## Native reconstruction

The native CLI now exposes the construction directly:

```bash
cargo run -p hadamard-cli -- build spence \
  --sequences-in fixtures/known/hm-668/spence-sequences.txt \
  --output /tmp/hm668-spence.txt
cmp /tmp/hm668-spence.txt fixtures/known/hm-668/matrix.txt
sha256sum /tmp/hm668-spence.txt
```

Expected build output begins:

```text
order=668
is_hadamard=true
matrix_written=/tmp/hm668-spence.txt
```

`cmp` produces no output, and the reconstructed matrix SHA-256 is
`bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0`.
The `hadamard-construct` regression also constructs from the compact seed and
compares every output byte with `matrix.txt` without using an exhaustive pairwise
orthogonality check to establish equality.

An independent standard-library Python implementation applies the mathematical
array directly, without invoking either the public shell generator or the Rust
builder:

```bash
uv run python py/reconstruct_hm668.py \
  fixtures/known/hm-668/spence-sequences.txt /tmp/hm668-python.txt
cmp /tmp/hm668-python.txt fixtures/known/hm-668/matrix.txt
```

It reports `construction=wallis-whiteman`, seed order `166`, negative
support sizes `82,83,83,83`, `lambda=164`, order `668`, and the same final SHA-256.
The `cmp` is empty.

The stored fixture can be independently rechecked with exact native labels:

```bash
cargo run -p hadamard-cli -- verify matrix \
  --input fixtures/known/hm-668/matrix.txt
```

```text
rows=668
columns=668
is_hadamard=true
off_diagonal_pairs_checked=222778
```

## Claim boundary and remaining question

The exact identification is:

- a cyclic type-`H_4^*` difference family on `Z_166`;
- substituted into the standard Wallis–Whiteman bordered array;
- reconstructed byte for byte by `build_spence_hadamard`.

It is **not** currently justified to claim that this particular `Z_166` seed was
obtained from either of Spence's relative-difference-set families, from the
finite-field construction in the cited paper, or from any other specific published
seed construction. The parameters and array shape alone do not establish that
provenance.

The main remaining construction question is narrower than before: how was this
specific cyclic `(166;82,83,83,83;164)` family discovered or searched for? That
search provenance remains open pending a fuller account from the authors.
