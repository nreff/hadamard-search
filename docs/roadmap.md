# Roadmap

Use [`docs/MILESTONES.md`](docs/MILESTONES.md) as the execution checklist. This file stays high-level; milestones define the required test gates.

## Phase 1

- stabilize exact math and artifact formats
- keep the known length-5 Legendre pair green in tests and CLI
- make checkpoint/resume deterministic

## Phase 2

- add a genuinely faster PSD/DFT backend behind the current abstraction
- improve compressed LP filtering beyond the current surrogate score
- benchmark shard throughput and artifact size

## Phase 3

- add decompressor interfaces for SAT/CAS and meet-in-the-middle methods
- move from candidate ranking to full compressed LP workflows
- add more known structured cases

## Phase 4

- production LP(333) search campaigns
- SDS-167 matching implementation
- publication-quality artifact bundles and validation reports
