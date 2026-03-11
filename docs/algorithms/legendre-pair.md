# Legendre-Pair Track

Primary target: `LP(333)`.

## Current implementation

- exact `±1` sequences
- periodic autocorrelation
- Legendre-pair verification
- two-circulant-core construction for row-sum-1 pairs
- exact small-order search
- compressed candidate generation for factors dividing the target length
- candidate-level compatibility pruning before pair enumeration
- exact PSD-signature bucket matching before pair enumeration
- compressed residual checking against the factor-scaled Legendre target
- PSD-consistency checking for compressed candidate pairs
- reusable bucket artifact emission for future decompression/search stages
- exact small-case decompression from bucket artifacts

## Intended production flow

1. choose compression factor in `{3, 9, 37, 111}`
2. generate compressed candidate sequences
3. filter with spectral and autocorrelation constraints
4. retain top candidates or exact compressed hits
5. decompress with a future dedicated solver
6. verify exact Legendre-pair conditions
7. build the 2cc Hadamard matrix and verify `H H^T = n I`

## Notes

- The current compressed mode is still pre-decompression, but it now uses an exact compressed residual target rather than a placeholder score.
- The compressed runner now records stage metrics such as candidate-pool sizes, signature-compatible pools, residual-zero pairs, and PSD-consistent pairs.
- The current decompressor is intentionally small-scope and exhaustive. It is suitable for known-case recovery and interface validation, not yet for production LP(333) work.
- The known length-5 case is the validation anchor for the exact path.
