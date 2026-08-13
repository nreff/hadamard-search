# Verified Hadamard Matrix Of Order 668

`matrix.txt` is a `668 x 668` matrix in the repository's text format: one row
per line, with `+` for `+1` and `-` for `-1`.

Exact artifact digest:

```text
SHA-256  bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0
bytes    446892
rows     668
columns  668
```

## Provenance

The artifact was reconstructed on `2026-08-12` from Levent Alpöge's public X
thread:

- primary `+/-` payload: <https://x.com/__alpoge__/status/2087504785952182273>
- decoder/generator: <https://x.com/__alpoge__/status/2087504788938510427>
- attribution reply: <https://x.com/__alpoge__/status/2087504790435840207>

The extracted primary `+/-` sign stream contains exactly `23,828` characters and
has SHA-256
`5b5fe8fa42f0d6a8b4e4c9926726d82a6aab8e1070c1ae4d1b430c1277e58db4`.
The first `664` signs concatenated, as consumed by constructor `h`, have SHA-256
`6671d64e5118226a7b2b0e7f7be6fe52a371860984fd6152a925a0d73c5919a7`.
The decoder status text has SHA-256
`30d552fa26eab955cf73d7c762b34e23bea5a3ba70e384518cf718b040fd9f0a`.

The posted wrapper was not executed. Its leading `sed` substitution was decoded
as data, producing a shell generator whose commands were audited. The generator
uses only `sed`, `cut`, `fold`, `printf`, `cat`, and `echo`, plus shell arithmetic
and temporary scratch files. Only its first instruction, constructor `h` with a
`664`-sign input, was run; it emits this order-`668` matrix. The remaining eleven
instructions were not needed to produce this fixture.

## Construction

The first `664` signs split consecutively into the four rows of
`spence-sequences.txt`, each of length `166`. Their row sums are `[2,0,0,0]`, and
their combined periodic autocorrelation is `-4` at every nonzero shift. Their
negative supports therefore form a cyclic type-`H_4^*` difference family
`(166;82,83,83,83;164)`; equivalently, the positive supports form
`(166;84,83,83,83;166)`.

The decoder uses the standard Wallis–Whiteman bordered array for this family, a
form also presented in the literature as a Spence SDS construction.
The repository's `build_spence_hadamard` implements the exact circulant,
back-diagonal, and border formula and reconstructs `matrix.txt` byte for byte:

```bash
cargo run -p hadamard-cli -- build spence \
  --sequences-in fixtures/known/hm-668/spence-sequences.txt \
  --output /tmp/hm668-spence.txt
cmp /tmp/hm668-spence.txt fixtures/known/hm-668/matrix.txt
uv run python py/reconstruct_hm668.py \
  fixtures/known/hm-668/spence-sequences.txt /tmp/hm668-python.txt
cmp /tmp/hm668-python.txt fixtures/known/hm-668/matrix.txt
```

The Python reconstructor uses the standard library and the documented mathematical
array directly; it neither invokes the public shell generator nor calls the Rust
implementation. Both `cmp` commands are empty, and both outputs have the matrix
SHA-256 recorded above.

See [`docs/research/hm-668-construction.md`](../../../docs/research/hm-668-construction.md)
for the exact block array and the structural proof of orthogonality. This identifies
the generic array and the seed's difference-family parameters. It does not establish
that the seed came from one of Spence's relative-difference-set families; how this
particular cyclic `Z_166` family was discovered remains open.

## Independent Verification

The initial audit and current repository checks are distinct:

1. the initial Python reference verifier computed scalar integer row dot products
   directly before the validation path was optimized;
2. the current Python and native Rust verifiers independently implement exact
   bit-packed Hamming-distance checks and each checks all `222,778` unordered row
   pairs;
3. a separate audit verifier checked all `222,778`
   unordered row pairs and all `222,778` unordered column pairs, each at the
   required distance `334`.

All checks confirmed that every entry is `+1` or `-1` and

```text
H H^T = 668 I.
```

Re-run the repository verifier with:

```bash
uv run python py/validate_matrix.py fixtures/known/hm-668/matrix.txt
```

Expected identifying output:

```text
order=668
is_hadamard=True
off_diagonal_pairs_checked=222778
sha256=bdeb5059d77e2703211082627b60441b8c888c928a55cc6f295e011941a387b0
```

The native verifier reports:

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

This fixture establishes existence at order `668`, and the compact seed plus
bordered-array theorem now explain its construction. The seed's discovery provenance
and the general Hadamard conjecture remain open.
