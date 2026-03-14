use hadamard_core::{
    default_psd_backend, sds_target_lambda, ArtifactHeader, CheckpointState, CompressedSequence,
    CyclicDifferenceBlock, LegendrePair, SearchArtifact, Sequence, SupplementaryDifferenceSet,
    CURRENT_ARTIFACT_VERSION,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::f64::consts::PI;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchMode {
    Exact,
    Compressed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchConfig {
    pub length: usize,
    pub compression: usize,
    pub shard_index: usize,
    pub shard_count: usize,
    pub max_attempts: u64,
    pub row_sum_target: i32,
}

impl SearchConfig {
    pub fn mode(&self) -> SearchMode {
        if self.compression == 1 {
            SearchMode::Exact
        } else {
            SearchMode::Compressed
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchMetrics {
    pub attempted_pairs: u64,
    pub accepted_pairs: u64,
    pub candidate_pool_a: u64,
    pub candidate_pool_b: u64,
    pub generation_branches_a: u64,
    pub generation_branches_b: u64,
    pub generation_row_sum_pruned_a: u64,
    pub generation_row_sum_pruned_b: u64,
    pub generation_spectral_pruned_a: u64,
    pub generation_spectral_pruned_b: u64,
    pub compatible_pool_a: u64,
    pub compatible_pool_b: u64,
    pub signature_pool_a: u64,
    pub signature_pool_b: u64,
    pub residual_zero_pairs: u64,
    pub psd_consistent_pairs: u64,
    pub best_compressed_residual: Option<i64>,
    pub best_psd_residual: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactMatch {
    pub a: Sequence,
    pub b: Sequence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompressedMatch {
    pub a: CompressedSequence,
    pub b: CompressedSequence,
    pub compressed_residual: i64,
    pub psd_residual: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SearchMatch {
    Exact(ExactMatch),
    Compressed(CompressedMatch),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchOutcome {
    pub checkpoint: CheckpointState,
    pub matches: Vec<SearchMatch>,
    pub artifact: SearchArtifact,
    pub bucket_artifact: Option<SearchArtifact>,
    pub metrics: SearchMetrics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BucketArtifactData {
    pub length: usize,
    pub compression: usize,
    pub a_candidates: Vec<CompressedSequence>,
    pub b_candidates: Vec<CompressedSequence>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecompressionConfig {
    pub max_pairs: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecompressionOutcome {
    pub exact_matches: Vec<ExactMatch>,
    pub artifact: SearchArtifact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdsSearchConfig {
    pub order: usize,
    pub block_sizes: Vec<usize>,
    pub lambda: usize,
    pub shard_index: usize,
    pub shard_count: usize,
    pub max_matches: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdsMatch {
    pub blocks: Vec<CyclicDifferenceBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdsSearchOutcome {
    pub matches: Vec<SdsMatch>,
    pub attempted_pairs: usize,
    pub pair_bucket_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectCompressedPair {
    a: CompressedSequence,
    b: CompressedSequence,
}

impl DirectCompressedPair {
    pub fn a(&self) -> &CompressedSequence {
        &self.a
    }

    pub fn b(&self) -> &CompressedSequence {
        &self.b
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirectCompressedPairProbeStats {
    pub branches_considered: u64,
    pub row_sum_pruned: u64,
    pub norm_pruned: u64,
    pub autocorrelation_pruned: u64,
    pub spectral_pruned: u64,
    pub tail_spectral_pruned: u64,
    pub tail_residual_pruned: u64,
    pub tail_candidates_checked: u64,
    pub pairs_emitted: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectCompressedPairProbeOutcome {
    pub pairs: Vec<DirectCompressedPair>,
    pub stats: DirectCompressedPairProbeStats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectProbeOrdering {
    Natural,
    Generator2,
}

impl DirectProbeOrdering {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Natural => "natural",
            Self::Generator2 => "generator2",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MitmCompressedPairProbeStats {
    pub branches_considered: u64,
    pub row_sum_pruned: u64,
    pub norm_pruned: u64,
    pub autocorrelation_pruned: u64,
    pub left_states_emitted: u64,
    pub right_states_emitted: u64,
    pub join_candidates_checked: u64,
    pub pairs_emitted: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MitmCompressedPairProbeOutcome {
    pub pairs: Vec<DirectCompressedPair>,
    pub stats: MitmCompressedPairProbeStats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MitmSplitStrategy {
    Contiguous,
    Parity,
}

impl MitmSplitStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contiguous => "contiguous",
            Self::Parity => "parity",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ExpansionStats {
    branches_considered: usize,
    branches_pruned: usize,
    sequences_built: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactCandidateStats {
    sequence: Sequence,
    legendre_signature: Vec<i32>,
}

#[derive(Clone, Debug)]
struct CompressedCandidateStats {
    sequence: CompressedSequence,
    squared_norm: i32,
    max_nonzero_psd: f64,
    nonzero_psd_signature: Vec<i64>,
}

#[derive(Clone, Debug)]
struct ComplexAccumulator {
    real: f64,
    imag: f64,
}

impl ComplexAccumulator {
    fn zero() -> Self {
        Self { real: 0.0, imag: 0.0 }
    }

    fn add_scaled(&mut self, scale: i16, twiddle: (f64, f64)) {
        self.real += f64::from(scale) * twiddle.0;
        self.imag += f64::from(scale) * twiddle.1;
    }

    fn sub_scaled(&mut self, scale: i16, twiddle: (f64, f64)) {
        self.real -= f64::from(scale) * twiddle.0;
        self.imag -= f64::from(scale) * twiddle.1;
    }

    fn magnitude(self) -> f64 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }
}

#[derive(Clone, Debug)]
struct DirectPairSpectralContext {
    max_abs_symbol: f64,
    target_psd: f64,
    twiddles: Vec<Vec<(f64, f64)>>,
}

#[derive(Clone, Debug)]
struct DirectTailContext {
    max_remaining: usize,
    precomputed_max: usize,
    alphabet: Vec<i16>,
    base: u64,
    tails_by_len: Vec<BTreeMap<(i32, i32, i32, i32), Vec<(u64, u64)>>>,
}

#[derive(Clone, Debug, Default)]
struct TailSpectralCache {
    left: HashMap<u64, Vec<ComplexAccumulator>>,
    right: HashMap<u64, Vec<ComplexAccumulator>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TailBoundarySignature {
    first_a: i16,
    first_b: i16,
    last_a: i16,
    last_b: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TailShiftOneSignature {
    boundary: TailBoundarySignature,
    internal_sum: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartialCompressedPairState {
    positions: Vec<usize>,
    values_a: Vec<i16>,
    values_b: Vec<i16>,
    sum_a: i32,
    sum_b: i32,
    combined_squared_norm: i32,
}

#[derive(Clone, Debug)]
struct CompressedGenerationContext {
    order: usize,
    alphabet: Vec<i16>,
    reachable_sums: Vec<BTreeSet<i32>>,
    max_partner_squared_norm: i32,
    max_abs_symbol: f64,
    selected_frequencies: Vec<usize>,
    twiddles: Vec<Vec<(f64, f64)>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CompressedGenerationStats {
    branches_considered: u64,
    row_sum_pruned: u64,
    spectral_pruned: u64,
    candidates_emitted: u64,
}

pub fn run_legendre_search(
    config: &SearchConfig,
    checkpoint: Option<CheckpointState>,
) -> Result<SearchOutcome, String> {
    if config.length == 0 || config.length % 2 == 0 {
        return Err("LP search requires a positive odd length".to_string());
    }
    if config.compression == 0 || config.length % config.compression != 0 {
        return Err("compression factor must divide the target length".to_string());
    }
    if config.shard_count == 0 || config.shard_index >= config.shard_count {
        return Err("invalid shard specification".to_string());
    }
    if matches!(config.mode(), SearchMode::Exact) && config.length > 20 && config.max_attempts == 0 {
        return Err("exact exhaustive search above length 20 requires --max-attempts".to_string());
    }

    let mut state = checkpoint.unwrap_or_else(|| {
        CheckpointState::new("lp", config.length, config.compression, config.shard_index, config.shard_count)
    });
    let mut metrics = SearchMetrics {
        attempted_pairs: state.next_attempt,
        accepted_pairs: state.matches_found,
        candidate_pool_a: 0,
        candidate_pool_b: 0,
        generation_branches_a: 0,
        generation_branches_b: 0,
        generation_row_sum_pruned_a: 0,
        generation_row_sum_pruned_b: 0,
        generation_spectral_pruned_a: 0,
        generation_spectral_pruned_b: 0,
        compatible_pool_a: 0,
        compatible_pool_b: 0,
        signature_pool_a: 0,
        signature_pool_b: 0,
        residual_zero_pairs: 0,
        psd_consistent_pairs: 0,
        best_compressed_residual: None,
        best_psd_residual: None,
    };

    let (matches, bucket_artifact) = match config.mode() {
        SearchMode::Exact => (run_exact_search(config, &mut state, &mut metrics)?, None),
        SearchMode::Compressed => run_compressed_search(config, &mut state, &mut metrics)?,
    };

    let mut body = vec![
        "mode=lp-search".to_string(),
        format!("length={}", config.length),
        format!("compression={}", config.compression),
        format!("row_sum_target={}", config.row_sum_target),
        format!("max_attempts={}", config.max_attempts),
        format!("shard_index={}", config.shard_index),
        format!("shard_count={}", config.shard_count),
        format!("psd_backend={}", default_psd_backend().name()),
        format!("attempted_pairs={}", metrics.attempted_pairs),
        format!("accepted_pairs={}", metrics.accepted_pairs),
        format!("candidate_pool_a={}", metrics.candidate_pool_a),
        format!("candidate_pool_b={}", metrics.candidate_pool_b),
        format!("generation_branches_a={}", metrics.generation_branches_a),
        format!("generation_branches_b={}", metrics.generation_branches_b),
        format!("generation_row_sum_pruned_a={}", metrics.generation_row_sum_pruned_a),
        format!("generation_row_sum_pruned_b={}", metrics.generation_row_sum_pruned_b),
        format!(
            "generation_spectral_pruned_a={}",
            metrics.generation_spectral_pruned_a
        ),
        format!(
            "generation_spectral_pruned_b={}",
            metrics.generation_spectral_pruned_b
        ),
        format!("compatible_pool_a={}", metrics.compatible_pool_a),
        format!("compatible_pool_b={}", metrics.compatible_pool_b),
        format!("signature_pool_a={}", metrics.signature_pool_a),
        format!("signature_pool_b={}", metrics.signature_pool_b),
        format!("residual_zero_pairs={}", metrics.residual_zero_pairs),
        format!("psd_consistent_pairs={}", metrics.psd_consistent_pairs),
    ];
    if let Some(score) = metrics.best_compressed_residual {
        body.push(format!("best_compressed_residual={score}"));
    }
    if let Some(score) = metrics.best_psd_residual {
        body.push(format!("best_psd_residual={score:.6}"));
    }
    for found in &matches {
        match found {
            SearchMatch::Exact(item) => {
                body.push(format!("exact_a={}", item.a.to_line()));
                body.push(format!("exact_b={}", item.b.to_line()));
            }
            SearchMatch::Compressed(item) => {
                body.push(format!("compressed_a={}", item.a.to_line()));
                body.push(format!("compressed_b={}", item.b.to_line()));
                body.push(format!("compressed_residual={}", item.compressed_residual));
                body.push(format!("psd_residual={:.6}", item.psd_residual));
            }
        }
    }

    Ok(SearchOutcome {
        checkpoint: state,
        matches,
        artifact: SearchArtifact {
            header: ArtifactHeader::new("lp-search", "Legendre-pair search results"),
            body,
        },
        bucket_artifact,
        metrics,
    })
}

fn run_exact_search(
    config: &SearchConfig,
    state: &mut CheckpointState,
    metrics: &mut SearchMetrics,
) -> Result<Vec<SearchMatch>, String> {
    if config.length > 64 {
        return Err("exact search currently supports length <= 64".to_string());
    }

    let free_bits = config.length - 1;
    let sequence_count = 1_u64
        .checked_shl(free_bits as u32)
        .ok_or_else(|| "sequence space too large for exact enumeration".to_string())?;
    let total_pairs = sequence_count.saturating_mul(sequence_count);
    let mut matches = Vec::new();
    let mut attempt_index = state.next_attempt;

    while attempt_index < total_pairs && (attempt_index - state.next_attempt) < config.max_attempts {
        if attempt_index % config.shard_count as u64 != config.shard_index as u64 {
            attempt_index += 1;
            continue;
        }

        let a_bits = attempt_index / sequence_count;
        let b_bits = attempt_index % sequence_count;
        metrics.attempted_pairs += 1;

        let a = normalized_sequence_from_index(config.length, a_bits)?;
        let b = normalized_sequence_from_index(config.length, b_bits)?;
        if a.row_sum() == config.row_sum_target && b.row_sum() == config.row_sum_target {
            let pair = LegendrePair::new(a.clone(), b.clone())?;
            if pair.is_legendre_pair() {
                metrics.accepted_pairs += 1;
                matches.push(SearchMatch::Exact(ExactMatch { a, b }));
            }
        }
        attempt_index += 1;
    }

    state.next_attempt = attempt_index;
    state.matches_found = metrics.accepted_pairs;
    Ok(matches)
}

fn normalized_sequence_from_index(length: usize, index: u64) -> Result<Sequence, String> {
    let mut bits = 1_u64;
    for offset in 1..length {
        let source = (index >> (offset - 1)) & 1;
        bits |= source << offset;
    }
    Sequence::from_bits(length, bits)
}

fn run_compressed_search(
    config: &SearchConfig,
    state: &mut CheckpointState,
    metrics: &mut SearchMetrics,
) -> Result<(Vec<SearchMatch>, Option<SearchArtifact>), String> {
    let reduced_length = config.length / config.compression;
    let alphabet = CompressedSequence::alphabet_for_factor(config.compression);
    let generation_context =
        build_compressed_generation_context(reduced_length, &alphabet, config.row_sum_target)?;
    let mut generated = Vec::new();
    let mut generation_stats = CompressedGenerationStats::default();
    let mut accumulators = generation_context
        .selected_frequencies
        .iter()
        .map(|_| ComplexAccumulator::zero())
        .collect::<Vec<_>>();
    generate_compressed_sequences(
        &generation_context,
        0,
        0,
        0,
        &mut Vec::new(),
        &mut accumulators,
        &mut generation_stats,
        &mut generated,
    );
    metrics.candidate_pool_a = generated.len() as u64;
    metrics.candidate_pool_b = generated.len() as u64;
    metrics.generation_branches_a = generation_stats.branches_considered;
    metrics.generation_branches_b = generation_stats.branches_considered;
    metrics.generation_row_sum_pruned_a = generation_stats.row_sum_pruned;
    metrics.generation_row_sum_pruned_b = generation_stats.row_sum_pruned;
    metrics.generation_spectral_pruned_a = generation_stats.spectral_pruned;
    metrics.generation_spectral_pruned_b = generation_stats.spectral_pruned;
    let psd_backend = default_psd_backend();
    let candidate_stats = build_candidate_stats(generated, psd_backend);
    let first = candidate_stats.clone();
    let second = candidate_stats;
    let first = prune_candidates_by_pair_compatibility(&first, &second, config.compression);
    let second = prune_candidates_by_pair_compatibility(&second, &first, config.compression);
    metrics.compatible_pool_a = first.len() as u64;
    metrics.compatible_pool_b = second.len() as u64;
    let first = prune_candidates_by_signature(&first, &second, config.compression);
    let second = prune_candidates_by_signature(&second, &first, config.compression);
    metrics.signature_pool_a = first.len() as u64;
    metrics.signature_pool_b = second.len() as u64;

    let mut matches = Vec::new();
    let mut local_seen = 0_u64;
    let mut attempt_index = state.next_attempt;
    let second_index = build_signature_index(&second);
    let first_index = build_signature_index(&first);
    let mut seen_compressed_pairs = BTreeSet::new();
    'outer: for a in &first {
        for b in matching_partners(a, &second_index, config.compression) {
            let a_line = a.sequence.to_line();
            let b_line = b.sequence.to_line();
            let compressed_pair_key = if a_line <= b_line {
                format!("{a_line}|{b_line}")
            } else {
                format!("{b_line}|{a_line}")
            };
            if !seen_compressed_pairs.insert(compressed_pair_key) {
                continue;
            }
            if attempt_index % config.shard_count as u64 != config.shard_index as u64 {
                attempt_index += 1;
                continue;
            }
            if local_seen >= config.max_attempts {
                break 'outer;
            }

            let score = a.sequence.compressed_legendre_residual_against(&b.sequence);
            metrics.attempted_pairs += 1;
            if metrics.best_compressed_residual.map_or(true, |best| score < best) {
                metrics.best_compressed_residual = Some(score);
            }
            if score == 0 {
                metrics.residual_zero_pairs += 1;
                let psd_residual =
                    a.sequence
                        .compressed_psd_residual_against(&b.sequence, psd_backend);
                if metrics
                    .best_psd_residual
                    .map_or(true, |best| psd_residual < best)
                {
                    metrics.best_psd_residual = Some(psd_residual);
                }
                if psd_residual.abs() < 1.0e-6 {
                    metrics.psd_consistent_pairs += 1;
                    metrics.accepted_pairs += 1;
                    matches.push(SearchMatch::Compressed(CompressedMatch {
                        a: a.sequence.clone(),
                        b: b.sequence.clone(),
                        compressed_residual: score,
                        psd_residual,
                    }));
                }
            }

            local_seen += 1;
            attempt_index += 1;
        }
    }

    state.next_attempt = attempt_index;
    state.matches_found = metrics.accepted_pairs;
    let bucket_artifact = SearchArtifact {
        header: ArtifactHeader::new("lp-buckets", "Compressed LP signature buckets"),
        body: build_bucket_artifact_body(
            config,
            psd_backend.name(),
            &first_index,
            &second_index,
            metrics.candidate_pool_a,
            metrics.candidate_pool_b,
            metrics.signature_pool_a,
            metrics.signature_pool_b,
        ),
    };
    Ok((matches, Some(bucket_artifact)))
}

fn build_candidate_stats(
    sequences: Vec<CompressedSequence>,
    backend: &dyn hadamard_core::PsdBackend,
) -> Vec<CompressedCandidateStats> {
    sequences
        .into_iter()
        .map(|sequence| {
            let psd = sequence.psd_with_backend(backend);
            let max_nonzero_psd = psd
                .iter()
                .skip(1)
                .copied()
                .fold(0.0_f64, f64::max);
            let squared_norm = sequence.squared_norm();
            let nonzero_psd_signature = psd
                .iter()
                .skip(1)
                .map(|value| rounded_psd_bin(*value))
                .collect::<Vec<_>>();
            CompressedCandidateStats {
                sequence,
                squared_norm,
                max_nonzero_psd,
                nonzero_psd_signature,
            }
        })
        .collect()
}

fn prune_candidates_by_pair_compatibility(
    primary: &[CompressedCandidateStats],
    partner_pool: &[CompressedCandidateStats],
    factor: usize,
) -> Vec<CompressedCandidateStats> {
    primary
        .iter()
        .filter(|candidate| {
            partner_pool.iter().any(|partner| {
                let target =
                    f64::from(candidate.squared_norm + partner.squared_norm) + 2.0 * factor as f64;
                candidate.max_nonzero_psd <= target && partner.max_nonzero_psd <= target
            })
        })
        .cloned()
        .collect()
}

fn build_signature_index<'a>(
    candidates: &'a [CompressedCandidateStats],
) -> BTreeMap<(i32, Vec<i64>), Vec<&'a CompressedCandidateStats>> {
    let mut index: BTreeMap<(i32, Vec<i64>), Vec<&'a CompressedCandidateStats>> = BTreeMap::new();
    for candidate in candidates {
        index
            .entry((candidate.squared_norm, candidate.nonzero_psd_signature.clone()))
            .or_default()
            .push(candidate);
    }
    index
}

fn signature_complements(
    candidate: &CompressedCandidateStats,
    partner_norms: &BTreeSet<i32>,
    factor: usize,
) -> Vec<(i32, Vec<i64>)> {
    partner_norms
        .iter()
        .map(|partner_norm| {
            let target = i64::from(candidate.squared_norm + partner_norm) + 2 * factor as i64;
            let complement = candidate
                .nonzero_psd_signature
                .iter()
                .map(|value| target - value)
                .collect::<Vec<_>>();
            (*partner_norm, complement)
        })
        .filter(|(_, complement)| complement.iter().all(|value| *value >= 0))
        .collect()
}

fn prune_candidates_by_signature(
    primary: &[CompressedCandidateStats],
    partner_pool: &[CompressedCandidateStats],
    factor: usize,
) -> Vec<CompressedCandidateStats> {
    let partner_norms = partner_pool
        .iter()
        .map(|candidate| candidate.squared_norm)
        .collect::<BTreeSet<_>>();
    let partner_keys = partner_pool
        .iter()
        .map(|candidate| (candidate.squared_norm, candidate.nonzero_psd_signature.clone()))
        .collect::<BTreeSet<_>>();

    primary
        .iter()
        .filter(|candidate| {
            signature_complements(candidate, &partner_norms, factor)
                .into_iter()
                .any(|key| partner_keys.contains(&key))
        })
        .cloned()
        .collect()
}

fn matching_partners<'a>(
    candidate: &CompressedCandidateStats,
    partner_index: &'a BTreeMap<(i32, Vec<i64>), Vec<&'a CompressedCandidateStats>>,
    factor: usize,
) -> Vec<&'a CompressedCandidateStats> {
    let partner_norms = partner_index
        .keys()
        .map(|(norm, _)| *norm)
        .collect::<BTreeSet<_>>();
    let mut matches = Vec::new();
    for key in signature_complements(candidate, &partner_norms, factor) {
        if let Some(bucket) = partner_index.get(&key) {
            matches.extend(bucket.iter().copied());
        }
    }
    matches
}

fn rounded_psd_bin(value: f64) -> i64 {
    value.round() as i64
}

pub fn parse_bucket_artifact_text(text: &str) -> Result<BucketArtifactData, String> {
    let mut version = None;
    let mut family = None;
    let mut length = None;
    let mut compression = None;
    let mut a_candidates = Vec::new();
    let mut b_candidates = Vec::new();

    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "version" => version = Some(value.trim().parse::<u32>().map_err(|e| e.to_string())?),
            "family" => family = Some(value.trim().to_string()),
            "length" => length = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?),
            "compression" => {
                compression = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?)
            }
            "a_candidate" => a_candidates.push(parse_compressed_sequence(value.trim(), compression)?),
            "b_candidate" => b_candidates.push(parse_compressed_sequence(value.trim(), compression)?),
            _ => {}
        }
    }

    if family.as_deref() != Some("lp-buckets") {
        return Err("bucket artifact family must be lp-buckets".to_string());
    }
    let version = version.ok_or_else(|| "missing version".to_string())?;
    if version != CURRENT_ARTIFACT_VERSION {
        return Err(format!(
            "unsupported bucket artifact version {version}; expected {}",
            CURRENT_ARTIFACT_VERSION
        ));
    }

    Ok(BucketArtifactData {
        length: length.ok_or_else(|| "missing length".to_string())?,
        compression: compression.ok_or_else(|| "missing compression".to_string())?,
        a_candidates,
        b_candidates,
    })
}

pub fn decompress_bucket_artifact(
    bucket_data: &BucketArtifactData,
    config: &DecompressionConfig,
) -> Result<DecompressionOutcome, String> {
    let mut exact_matches = Vec::new();
    let mut seen_pair_keys = BTreeSet::new();
    let mut pairs_checked = 0_usize;
    let mut canonical_sequence_pairs = 0_usize;
    let mut canonical_exact_matches = 0_usize;
    let mut expansion_stats_a = ExpansionStats::default();
    let mut expansion_stats_b = ExpansionStats::default();

    let exact_a = build_exact_candidate_stats(
        bucket_data.length,
        bucket_data.compression,
        &build_candidate_stats(bucket_data.a_candidates.clone(), default_psd_backend()),
        &mut expansion_stats_a,
    )?;
    let exact_b = build_exact_candidate_stats(
        bucket_data.length,
        bucket_data.compression,
        &build_candidate_stats(bucket_data.b_candidates.clone(), default_psd_backend()),
        &mut expansion_stats_b,
    )?;
    let (exact_a, exact_b) = prune_exact_candidates_by_complementary_signature(exact_a, exact_b);
    let exact_a_signature_buckets = build_exact_signature_index(&exact_a).len();
    let exact_b_index = build_exact_signature_index(&exact_b);
    let exact_b_signature_buckets = exact_b_index.len();

    'outer: for a in &exact_a {
        let target = complement_legendre_signature(&a.legendre_signature);
        let Some(partners) = exact_b_index.get(&target) else {
            continue;
        };
        for b in partners {
            if pairs_checked >= config.max_pairs {
                break 'outer;
            }
            pairs_checked += 1;
            canonical_sequence_pairs += 1;
            let pair = LegendrePair::new(a.sequence.clone(), b.sequence.clone())?;
            if pair.has_two_circulant_row_sums() && pair.is_legendre_pair() {
                let Some((canonical_a, canonical_b)) = pair.canonical_common_dihedral_pair() else {
                    continue;
                };
                let key = format!("{}|{}", canonical_a.to_line(), canonical_b.to_line());
                if seen_pair_keys.insert(key) {
                    let exact = ExactMatch {
                        a: canonical_a,
                        b: canonical_b,
                    };
                    exact_matches.push(exact);
                    canonical_exact_matches += 1;
                }
            }
        }
    }

    let mut body = vec![
        "mode=lp-decompress".to_string(),
        format!("length={}", bucket_data.length),
        format!("compression={}", bucket_data.compression),
        format!("max_pairs={}", config.max_pairs),
        format!("psd_backend={}", default_psd_backend().name()),
        format!("pairs_checked={pairs_checked}"),
        format!("a_exact_candidates={}", exact_a.len()),
        format!("b_exact_candidates={}", exact_b.len()),
        format!("a_exact_signature_buckets={exact_a_signature_buckets}"),
        format!("b_exact_signature_buckets={exact_b_signature_buckets}"),
        format!("a_branches_considered={}", expansion_stats_a.branches_considered),
        format!("a_branches_pruned={}", expansion_stats_a.branches_pruned),
        format!("a_sequences_built={}", expansion_stats_a.sequences_built),
        format!("b_branches_considered={}", expansion_stats_b.branches_considered),
        format!("b_branches_pruned={}", expansion_stats_b.branches_pruned),
        format!("b_sequences_built={}", expansion_stats_b.sequences_built),
        format!("canonical_sequence_pairs={canonical_sequence_pairs}"),
        format!("exact_matches={}", exact_matches.len()),
        format!("canonical_exact_matches={canonical_exact_matches}"),
    ];
    for item in &exact_matches {
        body.push(format!("exact_a={}", item.a.to_line()));
        body.push(format!("exact_b={}", item.b.to_line()));
    }

    Ok(DecompressionOutcome {
        exact_matches,
        artifact: SearchArtifact {
            header: ArtifactHeader::new("lp-decompress", "Exact decompression results"),
            body,
        },
    })
}

fn prune_exact_candidates_by_complementary_signature(
    exact_a: Vec<ExactCandidateStats>,
    exact_b: Vec<ExactCandidateStats>,
) -> (Vec<ExactCandidateStats>, Vec<ExactCandidateStats>) {
    let a_signatures = exact_a
        .iter()
        .map(|candidate| candidate.legendre_signature.clone())
        .collect::<BTreeSet<_>>();
    let b_signatures = exact_b
        .iter()
        .map(|candidate| candidate.legendre_signature.clone())
        .collect::<BTreeSet<_>>();

    let exact_a = exact_a
        .into_iter()
        .filter(|candidate| {
            b_signatures.contains(&complement_legendre_signature(&candidate.legendre_signature))
        })
        .collect();
    let exact_b = exact_b
        .into_iter()
        .filter(|candidate| {
            a_signatures.contains(&complement_legendre_signature(&candidate.legendre_signature))
        })
        .collect();
    (exact_a, exact_b)
}

fn build_exact_candidate_stats(
    length: usize,
    compression: usize,
    compressed_candidates: &[CompressedCandidateStats],
    stats: &mut ExpansionStats,
) -> Result<Vec<ExactCandidateStats>, String> {
    let mut unique = BTreeMap::new();
    for compressed in compressed_candidates {
        let exact = expand_compressed_sequence(length, compression, &compressed.sequence, stats)?;
        for sequence in exact {
            if !sequence.is_normalized() || !sequence.is_canonical_normalized_dihedral() {
                continue;
            }
            unique.entry(sequence.to_line()).or_insert_with(|| ExactCandidateStats {
                legendre_signature: exact_legendre_signature(&sequence),
                sequence,
            });
        }
    }
    Ok(unique.into_values().collect())
}

fn exact_legendre_signature(sequence: &Sequence) -> Vec<i32> {
    (1..sequence.len())
        .map(|shift| sequence.periodic_autocorrelation(shift))
        .collect()
}

fn complement_legendre_signature(signature: &[i32]) -> Vec<i32> {
    signature.iter().map(|value| -2 - value).collect()
}

fn build_exact_signature_index<'a>(
    candidates: &'a [ExactCandidateStats],
) -> BTreeMap<Vec<i32>, Vec<&'a ExactCandidateStats>> {
    let mut index = BTreeMap::new();
    for candidate in candidates {
        index
            .entry(candidate.legendre_signature.clone())
            .or_insert_with(Vec::new)
            .push(candidate);
    }
    index
}

fn parse_compressed_sequence(
    input: &str,
    compression: Option<usize>,
) -> Result<CompressedSequence, String> {
    let factor = compression.ok_or_else(|| "compression must be parsed before candidates".to_string())?;
    let values = input
        .split(',')
        .map(|value| value.trim().parse::<i16>().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    CompressedSequence::new(factor, values)
}

fn expand_compressed_sequence(
    length: usize,
    compression: usize,
    compressed: &CompressedSequence,
    stats: &mut ExpansionStats,
) -> Result<Vec<Sequence>, String> {
    let reduced = length / compression;
    if compressed.len() != reduced {
        return Err("compressed sequence length does not match length/compression".to_string());
    }

    let mut columns = Vec::new();
    for value in compressed.values() {
        columns.push(expand_column(*value, compression)?);
    }

    let mut exact = Vec::new();
    let mut column_choice = vec![vec![0_i8; compression]; reduced];
    expand_columns_recursive(
        &columns,
        0,
        &mut column_choice,
        reduced,
        compression,
        &mut exact,
        stats,
    )?;
    Ok(exact)
}

fn expand_column(sum: i16, compression: usize) -> Result<Vec<Vec<i8>>, String> {
    let mut out = Vec::new();
    let mut current = vec![-1_i8; compression];
    expand_column_recursive(sum, 0, &mut current, &mut out);
    if out.is_empty() {
        return Err(format!("no expansions for compressed value {sum}"));
    }
    Ok(out)
}

fn expand_column_recursive(
    target_sum: i16,
    index: usize,
    current: &mut Vec<i8>,
    out: &mut Vec<Vec<i8>>,
) {
    if index == current.len() {
        let total: i16 = current.iter().map(|value| i16::from(*value)).sum();
        if total == target_sum {
            out.push(current.clone());
        }
        return;
    }

    current[index] = -1;
    expand_column_recursive(target_sum, index + 1, current, out);
    current[index] = 1;
    expand_column_recursive(target_sum, index + 1, current, out);
}

fn expand_columns_recursive(
    choices: &[Vec<Vec<i8>>],
    index: usize,
    selected: &mut Vec<Vec<i8>>,
    reduced: usize,
    compression: usize,
    out: &mut Vec<Sequence>,
    stats: &mut ExpansionStats,
) -> Result<(), String> {
    if index == choices.len() {
        let mut values = vec![0_i8; reduced * compression];
        for reduced_index in 0..reduced {
            for layer in 0..compression {
                values[reduced_index + layer * reduced] = selected[reduced_index][layer];
            }
        }
        out.push(Sequence::new(values)?);
        stats.sequences_built += 1;
        return Ok(());
    }

    for option in &choices[index] {
        stats.branches_considered += 1;
        if index == 0 && option[0] != 1 {
            stats.branches_pruned += 1;
            continue;
        }
        selected[index] = option.clone();
        if !partial_canonical_prefix_ok(selected, index + 1, reduced, compression) {
            stats.branches_pruned += 1;
            continue;
        }
        expand_columns_recursive(
            choices,
            index + 1,
            selected,
            reduced,
            compression,
            out,
            stats,
        )?;
    }
    Ok(())
}

fn partial_canonical_prefix_ok(
    selected: &[Vec<i8>],
    assigned_columns: usize,
    _reduced: usize,
    _compression: usize,
) -> bool {
    if selected[0][0] != 1 {
        return false;
    }

    // Columns are assigned in reduced-index order, and the exact sequence layout
    // stores all layer-0 entries before any later layer entries. Until every
    // reduced column is assigned, only the layer-0 slice forms a contiguous
    // lexicographic prefix of the eventual exact sequence, so pruning must stay
    // within that safe prefix.
    for shift in 1..assigned_columns {
        if selected[shift][0] != 1 {
            continue;
        }
        for pos in 0..(assigned_columns - shift) {
            let lhs = selected[pos][0];
            let rhs = selected[pos + shift][0];
            if lhs > rhs {
                break;
            }
            if lhs < rhs {
                return false;
            }
        }
    }
    true
}

fn build_bucket_artifact_body(
    config: &SearchConfig,
    psd_backend_name: &str,
    first_index: &BTreeMap<(i32, Vec<i64>), Vec<&CompressedCandidateStats>>,
    second_index: &BTreeMap<(i32, Vec<i64>), Vec<&CompressedCandidateStats>>,
    raw_a: u64,
    raw_b: u64,
    signature_a: u64,
    signature_b: u64,
) -> Vec<String> {
    let mut body = vec![
        "mode=lp-buckets".to_string(),
        format!("length={}", config.length),
        format!("compression={}", config.compression),
        format!("row_sum_target={}", config.row_sum_target),
        format!("max_attempts={}", config.max_attempts),
        format!("shard_index={}", config.shard_index),
        format!("shard_count={}", config.shard_count),
        format!("psd_backend={psd_backend_name}"),
        format!("raw_candidate_pool_a={raw_a}"),
        format!("raw_candidate_pool_b={raw_b}"),
        format!("signature_pool_a={signature_a}"),
        format!("signature_pool_b={signature_b}"),
        format!("bucket_count_a={}", first_index.len()),
        format!("bucket_count_b={}", second_index.len()),
    ];
    append_bucket_lines("a", first_index, &mut body);
    append_bucket_lines("b", second_index, &mut body);
    body
}

fn append_bucket_lines(
    label: &str,
    index: &BTreeMap<(i32, Vec<i64>), Vec<&CompressedCandidateStats>>,
    body: &mut Vec<String>,
) {
    for ((norm, signature), bucket) in index {
        body.push(format!(
            "{label}_bucket norm={} signature={} count={}",
            norm,
            signature
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(","),
            bucket.len()
        ));
        for candidate in bucket {
            body.push(format!("{label}_candidate={}", candidate.sequence.to_line()));
        }
    }
}

fn build_compressed_generation_context(
    order: usize,
    alphabet: &[i16],
    row_sum_target: i32,
) -> Result<CompressedGenerationContext, String> {
    let max_partner_squared_norm = max_row_sum_constrained_squared_norm(order, alphabet, row_sum_target)?
        .ok_or_else(|| "row sum target is not achievable for compressed search".to_string())?;
    let max_abs_symbol = alphabet
        .iter()
        .map(|value| f64::from(i16::unsigned_abs(*value)))
        .fold(0.0_f64, f64::max);
    let selected_frequencies = choose_generation_frequencies(order);
    let twiddles = selected_frequencies
        .iter()
        .map(|frequency| {
            (0..order)
                .map(|index| {
                    let angle = -2.0 * PI * (*frequency * index) as f64 / order as f64;
                    (angle.cos(), angle.sin())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let reachable_sums = build_row_sum_reachability_table(order, alphabet);

    Ok(CompressedGenerationContext {
        order,
        alphabet: alphabet.to_vec(),
        reachable_sums,
        max_partner_squared_norm,
        max_abs_symbol,
        selected_frequencies,
        twiddles,
    })
}

fn choose_generation_frequencies(order: usize) -> Vec<usize> {
    let max_checked = if order <= 15 { order.saturating_sub(1) } else { order.min(12) };
    (1..=max_checked).collect()
}

fn generate_compressed_sequences(
    context: &CompressedGenerationContext,
    index: usize,
    partial_sum: i32,
    partial_squared_norm: i32,
    prefix: &mut Vec<i16>,
    spectral_accumulators: &mut [ComplexAccumulator],
    stats: &mut CompressedGenerationStats,
    out: &mut Vec<CompressedSequence>,
) {
    let remaining = context.order - index;
    if exact_row_sum_is_impossible(remaining, partial_sum, 1, &context.reachable_sums) {
        stats.row_sum_pruned += 1;
        return;
    }
    if spectral_bound_is_impossible(
        context,
        remaining,
        partial_squared_norm,
        spectral_accumulators,
    ) {
        stats.spectral_pruned += 1;
        return;
    }
    if index == context.order {
        let candidate =
            CompressedSequence::new(derived_factor(&context.alphabet), prefix.clone()).expect("candidate");
        out.push(candidate);
        stats.candidates_emitted += 1;
        return;
    }

    for value in &context.alphabet {
        stats.branches_considered += 1;
        prefix.push(*value);
        for (slot, accumulator) in spectral_accumulators.iter_mut().enumerate() {
            accumulator.add_scaled(*value, context.twiddles[slot][index]);
        }
        generate_compressed_sequences(
            context,
            index + 1,
            partial_sum + i32::from(*value),
            partial_squared_norm + i32::from(*value) * i32::from(*value),
            prefix,
            spectral_accumulators,
            stats,
            out,
        );
        for (slot, accumulator) in spectral_accumulators.iter_mut().enumerate() {
            accumulator.sub_scaled(*value, context.twiddles[slot][index]);
        }
        prefix.pop();
    }
}

fn build_row_sum_reachability_table(order: usize, alphabet: &[i16]) -> Vec<BTreeSet<i32>> {
    let mut tables = Vec::with_capacity(order + 1);
    tables.push(BTreeSet::from([0_i32]));
    for used in 0..order {
        let mut next = BTreeSet::new();
        for sum in &tables[used] {
            for value in alphabet {
                next.insert(*sum + i32::from(*value));
            }
        }
        tables.push(next);
    }
    tables
}

fn exact_row_sum_is_impossible(
    remaining: usize,
    partial_sum: i32,
    row_sum_target: i32,
    reachable_sums: &[BTreeSet<i32>],
) -> bool {
    !reachable_sums[remaining].contains(&(row_sum_target - partial_sum))
}

fn exact_row_sum_is_impossible_from_norms(
    remaining: usize,
    partial_sum: i32,
    row_sum_target: i32,
    remaining_norms: &[BTreeMap<i32, BTreeSet<i32>>],
) -> bool {
    !remaining_norms[remaining].contains_key(&(row_sum_target - partial_sum))
}

fn spectral_bound_is_impossible(
    context: &CompressedGenerationContext,
    remaining: usize,
    partial_squared_norm: i32,
    spectral_accumulators: &[ComplexAccumulator],
) -> bool {
    let max_final_squared_norm = partial_squared_norm + remaining as i32 * 9;
    let max_allowed_psd =
        f64::from(max_final_squared_norm + context.max_partner_squared_norm + 2 * derived_factor(&context.alphabet) as i32);
    let remaining_budget = context.max_abs_symbol * remaining as f64;
    spectral_accumulators.iter().any(|accumulator| {
        let lower_bound = (accumulator.clone().magnitude() - remaining_budget).max(0.0);
        lower_bound * lower_bound > max_allowed_psd + 1.0e-9
    })
}

fn max_row_sum_constrained_squared_norm(
    order: usize,
    alphabet: &[i16],
    row_sum_target: i32,
) -> Result<Option<i32>, String> {
    let mut current = BTreeMap::new();
    current.insert(0_i32, 0_i32);
    for _ in 0..order {
        let mut next = BTreeMap::new();
        for (sum, squared_norm) in &current {
            for value in alphabet {
                let candidate_sum = *sum + i32::from(*value);
                let candidate_squared_norm = *squared_norm + i32::from(*value) * i32::from(*value);
                next.entry(candidate_sum)
                    .and_modify(|best: &mut i32| *best = (*best).max(candidate_squared_norm))
                    .or_insert(candidate_squared_norm);
            }
        }
        current = next;
    }
    Ok(current.get(&row_sum_target).copied())
}

fn derived_factor(alphabet: &[i16]) -> usize {
    alphabet
        .iter()
        .copied()
        .max()
        .unwrap_or(1)
        .unsigned_abs() as usize
}

pub fn direct_compressed_pair_probe(
    order: usize,
    factor: usize,
    row_sum_target: i32,
    ordering: DirectProbeOrdering,
    spectral_frequency_count: usize,
    tail_depth: usize,
    max_pairs: usize,
) -> Result<DirectCompressedPairProbeOutcome, String> {
    let alphabet = CompressedSequence::alphabet_for_factor(factor);
    let remaining_norms = build_row_sum_squared_norm_table(order, &alphabet);
    let positions = direct_probe_positions(order, ordering)?;
    let total_squared_norm =
        2 * row_sum_target * row_sum_target + 2 * factor as i32 * (order as i32 - 1);
    let spectral_context = build_direct_pair_spectral_context(
        order,
        factor,
        total_squared_norm,
        spectral_frequency_count,
    );
    let tail_context = build_direct_tail_context(&alphabet, tail_depth.min(order));
    let mut stats = DirectCompressedPairProbeStats::default();
    let mut out = Vec::new();
    let mut a_assignments = vec![None; order];
    let mut b_assignments = vec![None; order];
    let mut spectral_a = vec![ComplexAccumulator::zero(); spectral_context.twiddles.len()];
    let mut spectral_b = vec![ComplexAccumulator::zero(); spectral_context.twiddles.len()];
    direct_compressed_pair_probe_recursive(
        order,
        factor,
        row_sum_target,
        &alphabet,
        0,
        &positions,
        0,
        0,
        0,
        0,
        &mut a_assignments,
        &mut b_assignments,
        &remaining_norms,
        total_squared_norm,
        &spectral_context,
        &tail_context,
        &mut spectral_a,
        &mut spectral_b,
        &mut stats,
        &mut out,
        max_pairs,
    )?;
    Ok(DirectCompressedPairProbeOutcome { pairs: out, stats })
}

pub fn mitm_compressed_pair_probe(
    order: usize,
    factor: usize,
    row_sum_target: i32,
    split_strategy: MitmSplitStrategy,
    max_pairs: usize,
) -> Result<MitmCompressedPairProbeOutcome, String> {
    let alphabet = CompressedSequence::alphabet_for_factor(factor);
    let remaining_norms = build_row_sum_squared_norm_table(order, &alphabet);
    let total_squared_norm =
        2 * row_sum_target * row_sum_target + 2 * factor as i32 * (order as i32 - 1);
    let (left_positions, right_positions) = mitm_split_positions(order, split_strategy);
    let mut stats = MitmCompressedPairProbeStats::default();

    let mut left_a = vec![None; order];
    let mut left_b = vec![None; order];
    let mut left_states = Vec::new();
    enumerate_partial_pair_states(
        order,
        factor,
        row_sum_target,
        &alphabet,
        0,
        &left_positions,
        0,
        0,
        &remaining_norms,
        total_squared_norm,
        &mut left_a,
        &mut left_b,
        &mut stats,
        &mut left_states,
        true,
    );

    let mut right_a = vec![None; order];
    let mut right_b = vec![None; order];
    let mut right_buckets = BTreeMap::<(i32, i32, i32), Vec<PartialCompressedPairState>>::new();
    let mut right_states = Vec::new();
    enumerate_partial_pair_states(
        order,
        factor,
        row_sum_target,
        &alphabet,
        0,
        &right_positions,
        0,
        0,
        &remaining_norms,
        total_squared_norm,
        &mut right_a,
        &mut right_b,
        &mut stats,
        &mut right_states,
        false,
    );
    for state in right_states {
        right_buckets
            .entry((state.sum_a, state.sum_b, state.combined_squared_norm))
            .or_default()
            .push(state);
    }

    let mut pairs = Vec::new();
    for left in left_states {
        if pairs.len() >= max_pairs {
            break;
        }
        let key = (
            row_sum_target - left.sum_a,
            row_sum_target - left.sum_b,
            total_squared_norm - left.combined_squared_norm,
        );
        let Some(candidates) = right_buckets.get(&key) else {
            continue;
        };
        for right in candidates {
            stats.join_candidates_checked += 1;
            let (a, b) = assemble_full_pair(order, factor, &left, right)?;
            if a.compressed_legendre_residual_against(&b) == 0 {
                pairs.push(DirectCompressedPair { a, b });
                stats.pairs_emitted += 1;
                if pairs.len() >= max_pairs {
                    break;
                }
            }
        }
    }

    Ok(MitmCompressedPairProbeOutcome { pairs, stats })
}

fn direct_compressed_pair_probe_recursive(
    order: usize,
    factor: usize,
    row_sum_target: i32,
    alphabet: &[i16],
    index: usize,
    positions: &[usize],
    sum_a: i32,
    sum_b: i32,
    squared_norm_a: i32,
    squared_norm_b: i32,
    a_assignments: &mut [Option<i16>],
    b_assignments: &mut [Option<i16>],
    remaining_norms: &[BTreeMap<i32, BTreeSet<i32>>],
    total_squared_norm: i32,
    spectral_context: &DirectPairSpectralContext,
    tail_context: &DirectTailContext,
    spectral_a: &mut [ComplexAccumulator],
    spectral_b: &mut [ComplexAccumulator],
    stats: &mut DirectCompressedPairProbeStats,
    out: &mut Vec<DirectCompressedPair>,
    max_pairs: usize,
) -> Result<(), String> {
    if out.len() >= max_pairs {
        return Ok(());
    }
    let assigned = index;
    let remaining = order - assigned;
    if exact_row_sum_is_impossible_from_norms(remaining, sum_a, row_sum_target, remaining_norms)
        || exact_row_sum_is_impossible_from_norms(remaining, sum_b, row_sum_target, remaining_norms)
    {
        stats.row_sum_pruned += 1;
        return Ok(());
    }
    if combined_squared_norm_is_impossible(
        remaining,
        row_sum_target - sum_a,
        row_sum_target - sum_b,
        squared_norm_a + squared_norm_b,
        total_squared_norm,
        remaining_norms,
    ) {
        stats.norm_pruned += 1;
        return Ok(());
    }
    if !partial_compressed_pair_feasible_sparse(a_assignments, b_assignments, order, factor) {
        stats.autocorrelation_pruned += 1;
        return Ok(());
    }
    if pair_spectral_bound_is_impossible(remaining, spectral_context, spectral_a, spectral_b) {
        stats.spectral_pruned += 1;
        return Ok(());
    }
    if remaining > 0 && remaining <= tail_context.max_remaining {
        exact_tail_complete(
            order,
            factor,
            row_sum_target,
            positions,
            index,
            sum_a,
            sum_b,
            squared_norm_a,
            squared_norm_b,
            a_assignments,
            b_assignments,
            remaining_norms,
            total_squared_norm,
            spectral_context,
            spectral_a,
            spectral_b,
            tail_context,
            stats,
            out,
            max_pairs,
        )?;
        return Ok(());
    }
    if index == order {
        let a = CompressedSequence::new(
            factor,
            a_assignments
                .iter()
                .map(|value| value.expect("full assignment"))
                .collect(),
        )?;
        let b = CompressedSequence::new(
            factor,
            b_assignments
                .iter()
                .map(|value| value.expect("full assignment"))
                .collect(),
        )?;
        if a.row_sum() == row_sum_target
            && b.row_sum() == row_sum_target
            && a.compressed_legendre_residual_against(&b) == 0
        {
            out.push(DirectCompressedPair { a, b });
            stats.pairs_emitted += 1;
        }
        return Ok(());
    }

    for a in alphabet {
        for b in alphabet {
            stats.branches_considered += 1;
            let position = positions[index];
            a_assignments[position] = Some(*a);
            b_assignments[position] = Some(*b);
            for (slot, accumulator) in spectral_a.iter_mut().enumerate() {
                accumulator.add_scaled(*a, spectral_context.twiddles[slot][position]);
            }
            for (slot, accumulator) in spectral_b.iter_mut().enumerate() {
                accumulator.add_scaled(*b, spectral_context.twiddles[slot][position]);
            }
            direct_compressed_pair_probe_recursive(
                order,
                factor,
                row_sum_target,
                alphabet,
                index + 1,
                positions,
                sum_a + i32::from(*a),
                sum_b + i32::from(*b),
                squared_norm_a + i32::from(*a) * i32::from(*a),
                squared_norm_b + i32::from(*b) * i32::from(*b),
                a_assignments,
                b_assignments,
                remaining_norms,
                total_squared_norm,
                spectral_context,
                tail_context,
                spectral_a,
                spectral_b,
                stats,
                out,
                max_pairs,
            )?;
            for (slot, accumulator) in spectral_a.iter_mut().enumerate() {
                accumulator.sub_scaled(*a, spectral_context.twiddles[slot][position]);
            }
            for (slot, accumulator) in spectral_b.iter_mut().enumerate() {
                accumulator.sub_scaled(*b, spectral_context.twiddles[slot][position]);
            }
            a_assignments[position] = None;
            b_assignments[position] = None;
        }
    }
    Ok(())
}

fn direct_probe_positions(
    order: usize,
    ordering: DirectProbeOrdering,
) -> Result<Vec<usize>, String> {
    match ordering {
        DirectProbeOrdering::Natural => Ok((0..order).collect()),
        DirectProbeOrdering::Generator2 => {
            if order == 0 || order % 2 == 0 {
                return Err("generator2 ordering requires positive odd order".to_string());
            }
            let mut positions = Vec::with_capacity(order);
            let mut current = 0usize;
            for _ in 0..order {
                positions.push(current);
                current = (current + 2) % order;
            }
            Ok(positions)
        }
    }
}

fn build_direct_pair_spectral_context(
    order: usize,
    factor: usize,
    total_squared_norm: i32,
    spectral_frequency_count: usize,
) -> DirectPairSpectralContext {
    let selected_frequencies = if order <= 1 {
        Vec::new()
    } else {
        let count = spectral_frequency_count.min(order.saturating_sub(1));
        (1..order).take(count).collect::<Vec<_>>()
    };
    let twiddles = selected_frequencies
        .iter()
        .map(|frequency| {
            (0..order)
                .map(|index| {
                    let angle = -2.0 * PI * (*frequency as f64) * (index as f64) / order as f64;
                    (angle.cos(), angle.sin())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    DirectPairSpectralContext {
        max_abs_symbol: factor as f64,
        target_psd: f64::from(total_squared_norm + 2 * factor as i32),
        twiddles,
    }
}

fn pair_spectral_bound_is_impossible(
    remaining: usize,
    context: &DirectPairSpectralContext,
    spectral_a: &[ComplexAccumulator],
    spectral_b: &[ComplexAccumulator],
) -> bool {
    let remaining_budget = remaining as f64 * context.max_abs_symbol;
    spectral_a
        .iter()
        .zip(spectral_b)
        .any(|(acc_a, acc_b)| {
            let lower_a = (acc_a.clone().magnitude() - remaining_budget).max(0.0);
            let lower_b = (acc_b.clone().magnitude() - remaining_budget).max(0.0);
            lower_a * lower_a + lower_b * lower_b > context.target_psd + 1.0e-9
        })
}

fn build_direct_tail_context(alphabet: &[i16], max_remaining: usize) -> DirectTailContext {
    let base = alphabet.len() as u64;
    let precomputed_max = max_remaining.min(6);
    let mut tails_by_len = Vec::with_capacity(precomputed_max + 1);
    tails_by_len.push(BTreeMap::from([((0, 0, 0, 0), vec![(0_u64, 0_u64)])]));
    for length in 1..=precomputed_max {
        let mut map = BTreeMap::<(i32, i32, i32, i32), Vec<(u64, u64)>>::new();
        enumerate_tail_pairs(length, alphabet, base, 0, 0, 0, 0, 0, 0, &mut map);
        tails_by_len.push(map);
    }
    DirectTailContext {
        max_remaining,
        precomputed_max,
        alphabet: alphabet.to_vec(),
        base,
        tails_by_len,
    }
}

fn enumerate_tail_pairs(
    remaining: usize,
    alphabet: &[i16],
    base: u64,
    a_code: u64,
    b_code: u64,
    sum_a: i32,
    sum_b: i32,
    norm_a: i32,
    norm_b: i32,
    out: &mut BTreeMap<(i32, i32, i32, i32), Vec<(u64, u64)>>,
) {
    if remaining == 0 {
        out.entry((sum_a, sum_b, norm_a, norm_b))
            .or_default()
            .push((a_code, b_code));
        return;
    }
    for (a_index, a) in alphabet.iter().enumerate() {
        for (b_index, b) in alphabet.iter().enumerate() {
            enumerate_tail_pairs(
                remaining - 1,
                alphabet,
                base,
                a_code * base + a_index as u64,
                b_code * base + b_index as u64,
                sum_a + i32::from(*a),
                sum_b + i32::from(*b),
                norm_a + i32::from(*a) * i32::from(*a),
                norm_b + i32::from(*b) * i32::from(*b),
                out,
            );
        }
    }
}

fn exact_tail_complete(
    order: usize,
    factor: usize,
    row_sum_target: i32,
    positions: &[usize],
    index: usize,
    sum_a: i32,
    sum_b: i32,
    squared_norm_a: i32,
    squared_norm_b: i32,
    a_assignments: &mut [Option<i16>],
    b_assignments: &mut [Option<i16>],
    remaining_norms: &[BTreeMap<i32, BTreeSet<i32>>],
    total_squared_norm: i32,
    spectral_context: &DirectPairSpectralContext,
    spectral_a: &[ComplexAccumulator],
    spectral_b: &[ComplexAccumulator],
    tail_context: &DirectTailContext,
    stats: &mut DirectCompressedPairProbeStats,
    out: &mut Vec<DirectCompressedPair>,
    max_pairs: usize,
) -> Result<(), String> {
    let remaining = order - index;
    let required_sum_a = row_sum_target - sum_a;
    let required_sum_b = row_sum_target - sum_b;
    let required_norm = total_squared_norm - (squared_norm_a + squared_norm_b);
    let feasible_splits = feasible_tail_norm_splits(
        remaining,
        required_sum_a,
        required_sum_b,
        required_norm,
        remaining_norms,
    );
    if remaining <= tail_context.precomputed_max {
        for (required_norm_a, required_norm_b) in feasible_splits {
            let Some(candidates) = tail_context.tails_by_len[remaining]
                .get(&(required_sum_a, required_sum_b, required_norm_a, required_norm_b))
            else {
                continue;
            };
            apply_tail_candidates(
                remaining,
                factor,
                positions,
                index,
                a_assignments,
                b_assignments,
                spectral_context,
                spectral_a,
                spectral_b,
                tail_context,
                candidates,
                stats,
                out,
                max_pairs,
            )?;
            if out.len() >= max_pairs {
                break;
            }
        }
        return Ok(());
    }

    exact_factorized_tail_complete(
        remaining,
        factor,
        positions,
        index,
        required_sum_a,
        required_sum_b,
        &feasible_splits,
        a_assignments,
        b_assignments,
        spectral_context,
        spectral_a,
        spectral_b,
        tail_context,
        stats,
        out,
        max_pairs,
    )
}

fn apply_tail_candidates(
    remaining: usize,
    factor: usize,
    positions: &[usize],
    index: usize,
    a_assignments: &mut [Option<i16>],
    b_assignments: &mut [Option<i16>],
    spectral_context: &DirectPairSpectralContext,
    spectral_a: &[ComplexAccumulator],
    spectral_b: &[ComplexAccumulator],
    tail_context: &DirectTailContext,
    candidates: &[(u64, u64)],
    stats: &mut DirectCompressedPairProbeStats,
    out: &mut Vec<DirectCompressedPair>,
    max_pairs: usize,
) -> Result<(), String> {
    for (a_code, b_code) in candidates {
        if out.len() >= max_pairs {
            break;
        }
        stats.tail_candidates_checked += 1;
        let a_tail = decode_tail_code(*a_code, remaining, tail_context.base, &tail_context.alphabet);
        let b_tail = decode_tail_code(*b_code, remaining, tail_context.base, &tail_context.alphabet);
        if !tail_exact_spectral_consistent(
            positions,
            index,
            &a_tail,
            &b_tail,
            spectral_context,
            spectral_a,
            spectral_b,
        ) {
            stats.tail_spectral_pruned += 1;
            continue;
        }
        for offset in 0..remaining {
            let position = positions[index + offset];
            a_assignments[position] = Some(a_tail[offset]);
            b_assignments[position] = Some(b_tail[offset]);
        }
        if !tail_exact_shift_consistent(a_assignments, b_assignments, factor) {
            stats.tail_residual_pruned += 1;
            for offset in 0..remaining {
                let position = positions[index + offset];
                a_assignments[position] = None;
                b_assignments[position] = None;
            }
            continue;
        }
        if exact_compressed_legendre_residual_from_assignments(a_assignments, b_assignments, factor)
            == 0
        {
            let a = CompressedSequence::new(
                factor,
                a_assignments
                    .iter()
                    .map(|value| value.expect("full assignment"))
                    .collect(),
            )?;
            let b = CompressedSequence::new(
                factor,
                b_assignments
                    .iter()
                    .map(|value| value.expect("full assignment"))
                    .collect(),
            )?;
            out.push(DirectCompressedPair { a, b });
            stats.pairs_emitted += 1;
        } else {
            stats.tail_residual_pruned += 1;
        }
        for offset in 0..remaining {
            let position = positions[index + offset];
            a_assignments[position] = None;
            b_assignments[position] = None;
        }
    }
    Ok(())
}

fn exact_factorized_tail_complete(
    remaining: usize,
    factor: usize,
    positions: &[usize],
    index: usize,
    required_sum_a: i32,
    required_sum_b: i32,
    feasible_splits: &[(i32, i32)],
    a_assignments: &mut [Option<i16>],
    b_assignments: &mut [Option<i16>],
    spectral_context: &DirectPairSpectralContext,
    spectral_a: &[ComplexAccumulator],
    spectral_b: &[ComplexAccumulator],
    tail_context: &DirectTailContext,
    stats: &mut DirectCompressedPairProbeStats,
    out: &mut Vec<DirectCompressedPair>,
    max_pairs: usize,
) -> Result<(), String> {
    let left_len = remaining / 2;
    let right_len = remaining - left_len;
    if left_len == 0 || right_len == 0 || right_len > tail_context.precomputed_max {
        return Ok(());
    }
    let mut spectral_cache = TailSpectralCache::default();
    let mut left_shift_one_cache = HashMap::<(u64, u64), TailShiftOneSignature>::new();
    let mut right_shift_one_cache = HashMap::<(u64, u64), TailShiftOneSignature>::new();
    let left_map = &tail_context.tails_by_len[left_len];
    let right_map = &tail_context.tails_by_len[right_len];
    let natural_positions = positions.iter().enumerate().all(|(slot, position)| *position == slot);
    let shift_one_context = if natural_positions {
        Some(build_factorized_shift_one_context(
            a_assignments,
            b_assignments,
            index,
            factor,
        ))
    } else {
        None
    };
    for (required_norm_a, required_norm_b) in feasible_splits {
        if out.len() >= max_pairs {
            break;
        }
        for ((sum_a_left, sum_b_left, norm_a_left, norm_b_left), left_candidates) in left_map {
            if out.len() >= max_pairs {
                break;
            }
            let key = (
                required_sum_a - *sum_a_left,
                required_sum_b - *sum_b_left,
                required_norm_a - *norm_a_left,
                required_norm_b - *norm_b_left,
            );
            let Some(right_candidates) = right_map.get(&key) else {
                continue;
            };
            let right_shift_one_buckets = shift_one_context.as_ref().map(|_| {
                build_right_shift_one_buckets(
                    right_candidates,
                    right_len,
                    tail_context,
                    &mut right_shift_one_cache,
                )
            });
            for (left_a, left_b) in left_candidates {
                let left_shift_one_sig = shift_one_context.as_ref().map(|_| {
                    cached_tail_shift_one_signature(
                        &mut left_shift_one_cache,
                        *left_a,
                        *left_b,
                        left_len,
                        tail_context,
                    )
                });
                let matching_right_candidates: Vec<(u64, u64)> = if let (Some(context), Some(sig), Some(buckets)) =
                    (shift_one_context.as_ref(), left_shift_one_sig, right_shift_one_buckets.as_ref())
                {
                    right_candidates_for_shift_one(context, sig, buckets)
                } else {
                    right_candidates.to_vec()
                };
                for (right_a, right_b) in matching_right_candidates.iter() {
                    if out.len() >= max_pairs {
                        break;
                    }
                    stats.tail_candidates_checked += 1;
                    if !factorized_tail_spectral_consistent(
                        positions,
                        index,
                        left_len,
                        right_len,
                        *left_a,
                        *left_b,
                        *right_a,
                        *right_b,
                        spectral_context,
                        spectral_a,
                        spectral_b,
                        tail_context,
                        &mut spectral_cache,
                    ) {
                        stats.tail_spectral_pruned += 1;
                        continue;
                    }
                    let combined = vec![(*left_a, *left_b), (*right_a, *right_b)];
                    let mut stitched = Vec::with_capacity(remaining);
                    for (a_code, b_code, len) in [
                        (combined[0].0, combined[0].1, left_len),
                        (combined[1].0, combined[1].1, right_len),
                    ] {
                        let a_tail = decode_tail_code(
                            a_code,
                            len,
                            tail_context.base,
                            &tail_context.alphabet,
                        );
                        let b_tail = decode_tail_code(
                            b_code,
                            len,
                            tail_context.base,
                            &tail_context.alphabet,
                        );
                        stitched.extend(a_tail.into_iter().zip(b_tail.into_iter()));
                    }
                    for (offset, (a_value, b_value)) in stitched.iter().enumerate() {
                        let position = positions[index + offset];
                        a_assignments[position] = Some(*a_value);
                        b_assignments[position] = Some(*b_value);
                    }
                    if !tail_exact_shift_consistent(a_assignments, b_assignments, factor) {
                        stats.tail_residual_pruned += 1;
                        for offset in 0..remaining {
                            let position = positions[index + offset];
                            a_assignments[position] = None;
                            b_assignments[position] = None;
                        }
                        continue;
                    }
                    if exact_compressed_legendre_residual_from_assignments(
                        a_assignments,
                        b_assignments,
                        factor,
                    ) == 0
                    {
                        let a = CompressedSequence::new(
                            factor,
                            a_assignments
                                .iter()
                                .map(|value| value.expect("full assignment"))
                                .collect(),
                        )?;
                        let b = CompressedSequence::new(
                            factor,
                            b_assignments
                                .iter()
                                .map(|value| value.expect("full assignment"))
                                .collect(),
                        )?;
                        out.push(DirectCompressedPair { a, b });
                        stats.pairs_emitted += 1;
                    } else {
                        stats.tail_residual_pruned += 1;
                    }
                    for offset in 0..remaining {
                        let position = positions[index + offset];
                        a_assignments[position] = None;
                        b_assignments[position] = None;
                    }
                }
            }
        }
    }
    Ok(())
}

fn feasible_tail_norm_splits(
    remaining: usize,
    required_sum_a: i32,
    required_sum_b: i32,
    required_norm: i32,
    remaining_norms: &[BTreeMap<i32, BTreeSet<i32>>],
) -> Vec<(i32, i32)> {
    let Some(a_norms) = remaining_norms[remaining].get(&required_sum_a) else {
        return Vec::new();
    };
    let Some(b_norms) = remaining_norms[remaining].get(&required_sum_b) else {
        return Vec::new();
    };
    let mut splits = Vec::new();
    let (iterate, lookup, a_first) = if a_norms.len() <= b_norms.len() {
        (a_norms, b_norms, true)
    } else {
        (b_norms, a_norms, false)
    };
    for norm in iterate {
        let other = required_norm - *norm;
        if lookup.contains(&other) {
            if a_first {
                splits.push((*norm, other));
            } else {
                splits.push((other, *norm));
            }
        }
    }
    splits
}

#[derive(Clone, Copy, Debug)]
struct FactorizedShiftOneContext {
    target: i32,
    prefix_internal: i32,
    prefix_first_a: i16,
    prefix_first_b: i16,
    prefix_last_a: i16,
    prefix_last_b: i16,
    has_prefix: bool,
}

fn build_factorized_shift_one_context(
    a_assignments: &[Option<i16>],
    b_assignments: &[Option<i16>],
    index: usize,
    factor: usize,
) -> FactorizedShiftOneContext {
    let has_prefix = index > 0;
    let mut prefix_internal = 0_i32;
    if has_prefix {
        for slot in 0..index.saturating_sub(1) {
            let a_i = i32::from(a_assignments[slot].expect("prefix assignment"));
            let a_j = i32::from(a_assignments[slot + 1].expect("prefix assignment"));
            let b_i = i32::from(b_assignments[slot].expect("prefix assignment"));
            let b_j = i32::from(b_assignments[slot + 1].expect("prefix assignment"));
            prefix_internal += a_i * a_j + b_i * b_j;
        }
    }
    FactorizedShiftOneContext {
        target: -2 * factor as i32,
        prefix_internal,
        prefix_first_a: a_assignments[0].unwrap_or(0),
        prefix_first_b: b_assignments[0].unwrap_or(0),
        prefix_last_a: if has_prefix {
            a_assignments[index - 1].expect("prefix assignment")
        } else {
            0
        },
        prefix_last_b: if has_prefix {
            b_assignments[index - 1].expect("prefix assignment")
        } else {
            0
        },
        has_prefix,
    }
}

fn cached_tail_shift_one_signature(
    cache: &mut HashMap<(u64, u64), TailShiftOneSignature>,
    a_code: u64,
    b_code: u64,
    length: usize,
    tail_context: &DirectTailContext,
) -> TailShiftOneSignature {
    *cache.entry((a_code, b_code)).or_insert_with(|| {
        let a_values = decode_tail_code(a_code, length, tail_context.base, &tail_context.alphabet);
        let b_values = decode_tail_code(b_code, length, tail_context.base, &tail_context.alphabet);
        let mut internal_sum = 0_i32;
        for slot in 0..length.saturating_sub(1) {
            internal_sum +=
                i32::from(a_values[slot]) * i32::from(a_values[slot + 1])
                    + i32::from(b_values[slot]) * i32::from(b_values[slot + 1]);
        }
        TailShiftOneSignature {
            boundary: TailBoundarySignature {
                first_a: a_values[0],
                first_b: b_values[0],
                last_a: a_values[length - 1],
                last_b: b_values[length - 1],
            },
            internal_sum,
        }
    })
}

fn build_right_shift_one_buckets(
    right_candidates: &[(u64, u64)],
    right_len: usize,
    tail_context: &DirectTailContext,
    cache: &mut HashMap<(u64, u64), TailShiftOneSignature>,
) -> HashMap<TailBoundarySignature, HashMap<i32, Vec<(u64, u64)>>> {
    let mut buckets = HashMap::<TailBoundarySignature, HashMap<i32, Vec<(u64, u64)>>>::new();
    for (right_a, right_b) in right_candidates {
        let sig =
            cached_tail_shift_one_signature(cache, *right_a, *right_b, right_len, tail_context);
        buckets
            .entry(sig.boundary)
            .or_default()
            .entry(sig.internal_sum)
            .or_default()
            .push((*right_a, *right_b));
    }
    buckets
}

fn right_candidates_for_shift_one(
    context: &FactorizedShiftOneContext,
    left_sig: TailShiftOneSignature,
    right_buckets: &HashMap<TailBoundarySignature, HashMap<i32, Vec<(u64, u64)>>>,
) -> Vec<(u64, u64)> {
    let mut out = Vec::new();
    for (boundary, internal_map) in right_buckets {
        let required_internal = if context.has_prefix {
            context.target
                - context.prefix_internal
                - (i32::from(context.prefix_last_a) * i32::from(left_sig.boundary.first_a)
                    + i32::from(context.prefix_last_b) * i32::from(left_sig.boundary.first_b))
                - left_sig.internal_sum
                - (i32::from(left_sig.boundary.last_a) * i32::from(boundary.first_a)
                    + i32::from(left_sig.boundary.last_b) * i32::from(boundary.first_b))
                - (i32::from(boundary.last_a) * i32::from(context.prefix_first_a)
                    + i32::from(boundary.last_b) * i32::from(context.prefix_first_b))
        } else {
            context.target
                - left_sig.internal_sum
                - (i32::from(left_sig.boundary.last_a) * i32::from(boundary.first_a)
                    + i32::from(left_sig.boundary.last_b) * i32::from(boundary.first_b))
                - (i32::from(boundary.last_a) * i32::from(left_sig.boundary.first_a)
                    + i32::from(boundary.last_b) * i32::from(left_sig.boundary.first_b))
        };
        if let Some(candidates) = internal_map.get(&required_internal) {
            out.extend(candidates.iter().copied());
        }
    }
    out
}

fn factorized_tail_spectral_consistent(
    positions: &[usize],
    index: usize,
    left_len: usize,
    right_len: usize,
    left_a: u64,
    left_b: u64,
    right_a: u64,
    right_b: u64,
    spectral_context: &DirectPairSpectralContext,
    spectral_a: &[ComplexAccumulator],
    spectral_b: &[ComplexAccumulator],
    tail_context: &DirectTailContext,
    cache: &mut TailSpectralCache,
) -> bool {
    let left_a_contrib = cached_tail_segment_spectral(
        &mut cache.left,
        left_a,
        left_len,
        positions,
        index,
        spectral_context,
        tail_context,
    );
    let left_b_contrib = cached_tail_segment_spectral(
        &mut cache.left,
        left_b,
        left_len,
        positions,
        index,
        spectral_context,
        tail_context,
    );
    let right_a_contrib = cached_tail_segment_spectral(
        &mut cache.right,
        right_a,
        right_len,
        positions,
        index + left_len,
        spectral_context,
        tail_context,
    );
    let right_b_contrib = cached_tail_segment_spectral(
        &mut cache.right,
        right_b,
        right_len,
        positions,
        index + left_len,
        spectral_context,
        tail_context,
    );

    for slot in 0..spectral_context.twiddles.len() {
        let real_a =
            spectral_a[slot].real + left_a_contrib[slot].real + right_a_contrib[slot].real;
        let imag_a =
            spectral_a[slot].imag + left_a_contrib[slot].imag + right_a_contrib[slot].imag;
        let real_b =
            spectral_b[slot].real + left_b_contrib[slot].real + right_b_contrib[slot].real;
        let imag_b =
            spectral_b[slot].imag + left_b_contrib[slot].imag + right_b_contrib[slot].imag;
        let psd_a = real_a * real_a + imag_a * imag_a;
        let psd_b = real_b * real_b + imag_b * imag_b;
        if psd_a + psd_b > spectral_context.target_psd + 1.0e-9 {
            return false;
        }
    }

    true
}

fn cached_tail_segment_spectral(
    cache: &mut HashMap<u64, Vec<ComplexAccumulator>>,
    code: u64,
    length: usize,
    positions: &[usize],
    start_index: usize,
    spectral_context: &DirectPairSpectralContext,
    tail_context: &DirectTailContext,
) -> Vec<ComplexAccumulator> {
    cache
        .entry(code)
        .or_insert_with(|| {
            let values = decode_tail_code(code, length, tail_context.base, &tail_context.alphabet);
            let mut accumulators = vec![ComplexAccumulator::zero(); spectral_context.twiddles.len()];
            for (offset, value) in values.iter().enumerate() {
                let position = positions[start_index + offset];
                for (slot, accumulator) in accumulators.iter_mut().enumerate() {
                    accumulator.add_scaled(*value, spectral_context.twiddles[slot][position]);
                }
            }
            accumulators
        })
        .clone()
}

fn decode_tail_code(code: u64, length: usize, base: u64, alphabet: &[i16]) -> Vec<i16> {
    let mut values = vec![0_i16; length];
    let mut current = code;
    for index in (0..length).rev() {
        let digit = (current % base) as usize;
        values[index] = alphabet[digit];
        current /= base;
    }
    values
}

fn tail_exact_shift_consistent(
    a_assignments: &[Option<i16>],
    b_assignments: &[Option<i16>],
    factor: usize,
) -> bool {
    let order = a_assignments.len();
    if order <= 1 {
        return true;
    }
    let target = -2 * factor as i32;
    let selected_shift_count = (order - 1).min(4);
    for shift in 1..=selected_shift_count {
        let mut total = 0_i32;
        for index in 0..order {
            let next = (index + shift) % order;
            let a_i = i32::from(a_assignments[index].expect("full assignment"));
            let a_j = i32::from(a_assignments[next].expect("full assignment"));
            let b_i = i32::from(b_assignments[index].expect("full assignment"));
            let b_j = i32::from(b_assignments[next].expect("full assignment"));
            total += a_i * a_j + b_i * b_j;
        }
        if total != target {
            return false;
        }
    }
    true
}

fn exact_compressed_legendre_residual_from_assignments(
    a_assignments: &[Option<i16>],
    b_assignments: &[Option<i16>],
    factor: usize,
) -> i64 {
    let order = a_assignments.len();
    if order <= 1 {
        return 0;
    }
    let target = -2_i64 * factor as i64;
    let mut total = 0_i64;
    for shift in 1..order {
        let mut shift_total = 0_i64;
        for index in 0..order {
            let next = (index + shift) % order;
            let a_i = i64::from(a_assignments[index].expect("full assignment"));
            let a_j = i64::from(a_assignments[next].expect("full assignment"));
            let b_i = i64::from(b_assignments[index].expect("full assignment"));
            let b_j = i64::from(b_assignments[next].expect("full assignment"));
            shift_total += a_i * a_j + b_i * b_j;
        }
        total += (shift_total - target).abs();
    }
    total
}

fn tail_exact_spectral_consistent(
    positions: &[usize],
    index: usize,
    a_tail: &[i16],
    b_tail: &[i16],
    spectral_context: &DirectPairSpectralContext,
    spectral_a: &[ComplexAccumulator],
    spectral_b: &[ComplexAccumulator],
) -> bool {
    for slot in 0..spectral_context.twiddles.len() {
        let mut real_a = spectral_a[slot].real;
        let mut imag_a = spectral_a[slot].imag;
        let mut real_b = spectral_b[slot].real;
        let mut imag_b = spectral_b[slot].imag;
        for offset in 0..a_tail.len() {
            let position = positions[index + offset];
            let twiddle = spectral_context.twiddles[slot][position];
            real_a += f64::from(a_tail[offset]) * twiddle.0;
            imag_a += f64::from(a_tail[offset]) * twiddle.1;
            real_b += f64::from(b_tail[offset]) * twiddle.0;
            imag_b += f64::from(b_tail[offset]) * twiddle.1;
        }
        let psd_a = real_a * real_a + imag_a * imag_a;
        let psd_b = real_b * real_b + imag_b * imag_b;
        if psd_a + psd_b > spectral_context.target_psd + 1.0e-9 {
            return false;
        }
    }
    true
}

fn build_row_sum_squared_norm_table(
    order: usize,
    alphabet: &[i16],
) -> Vec<BTreeMap<i32, BTreeSet<i32>>> {
    let mut tables = Vec::with_capacity(order + 1);
    let mut zero = BTreeMap::new();
    zero.insert(0, BTreeSet::from([0]));
    tables.push(zero);
    for used in 0..order {
        let mut next = BTreeMap::new();
        for (sum, norms) in &tables[used] {
            for value in alphabet {
                let entry = next
                    .entry(*sum + i32::from(*value))
                    .or_insert_with(BTreeSet::new);
                for norm in norms {
                    entry.insert(*norm + i32::from(*value) * i32::from(*value));
                }
            }
        }
        tables.push(next);
    }
    tables
}

fn combined_squared_norm_is_impossible(
    remaining: usize,
    remaining_sum_a: i32,
    remaining_sum_b: i32,
    partial_total_squared_norm: i32,
    total_squared_norm: i32,
    remaining_norms: &[BTreeMap<i32, BTreeSet<i32>>],
) -> bool {
    let target_remaining_norm = total_squared_norm - partial_total_squared_norm;
    let Some(a_norms) = remaining_norms[remaining].get(&remaining_sum_a) else {
        return true;
    };
    let Some(b_norms) = remaining_norms[remaining].get(&remaining_sum_b) else {
        return true;
    };
    let (smaller, larger) = if a_norms.len() <= b_norms.len() {
        (a_norms, b_norms)
    } else {
        (b_norms, a_norms)
    };
    !smaller
        .iter()
        .any(|norm| larger.contains(&(target_remaining_norm - *norm)))
}

fn enumerate_partial_pair_states(
    order: usize,
    factor: usize,
    row_sum_target: i32,
    alphabet: &[i16],
    index: usize,
    positions: &[usize],
    sum_a: i32,
    sum_b: i32,
    remaining_norms: &[BTreeMap<i32, BTreeSet<i32>>],
    total_squared_norm: i32,
    a_assignments: &mut [Option<i16>],
    b_assignments: &mut [Option<i16>],
    stats: &mut MitmCompressedPairProbeStats,
    out: &mut Vec<PartialCompressedPairState>,
    is_left: bool,
) {
    let assigned_count = a_assignments.iter().filter(|value| value.is_some()).count();
    let remaining = order - assigned_count;
    if exact_row_sum_is_impossible_from_norms(remaining, sum_a, row_sum_target, remaining_norms)
        || exact_row_sum_is_impossible_from_norms(remaining, sum_b, row_sum_target, remaining_norms)
    {
        stats.row_sum_pruned += 1;
        return;
    }
    let partial_total_squared_norm = assigned_combined_squared_norm(a_assignments, b_assignments);
    if combined_squared_norm_is_impossible(
        remaining,
        row_sum_target - sum_a,
        row_sum_target - sum_b,
        partial_total_squared_norm,
        total_squared_norm,
        remaining_norms,
    ) {
        stats.norm_pruned += 1;
        return;
    }
    if !partial_compressed_pair_feasible_sparse(a_assignments, b_assignments, order, factor) {
        stats.autocorrelation_pruned += 1;
        return;
    }
    if index == positions.len() {
        let values_a = positions
            .iter()
            .map(|position| a_assignments[*position].expect("assigned half entry"))
            .collect::<Vec<_>>();
        let values_b = positions
            .iter()
            .map(|position| b_assignments[*position].expect("assigned half entry"))
            .collect::<Vec<_>>();
        out.push(PartialCompressedPairState {
            positions: positions.to_vec(),
            values_a,
            values_b,
            sum_a,
            sum_b,
            combined_squared_norm: partial_total_squared_norm,
        });
        if is_left {
            stats.left_states_emitted += 1;
        } else {
            stats.right_states_emitted += 1;
        }
        return;
    }

    for a in alphabet {
        for b in alphabet {
            stats.branches_considered += 1;
            let position = positions[index];
            a_assignments[position] = Some(*a);
            b_assignments[position] = Some(*b);
            enumerate_partial_pair_states(
                order,
                factor,
                row_sum_target,
                alphabet,
                index + 1,
                positions,
                sum_a + i32::from(*a),
                sum_b + i32::from(*b),
                remaining_norms,
                total_squared_norm,
                a_assignments,
                b_assignments,
                stats,
                out,
                is_left,
            );
            a_assignments[position] = None;
            b_assignments[position] = None;
        }
    }
}

fn assigned_combined_squared_norm(a_assignments: &[Option<i16>], b_assignments: &[Option<i16>]) -> i32 {
    let norm_a = a_assignments
        .iter()
        .flatten()
        .map(|value| i32::from(*value) * i32::from(*value))
        .sum::<i32>();
    let norm_b = b_assignments
        .iter()
        .flatten()
        .map(|value| i32::from(*value) * i32::from(*value))
        .sum::<i32>();
    norm_a + norm_b
}

fn mitm_split_positions(order: usize, split_strategy: MitmSplitStrategy) -> (Vec<usize>, Vec<usize>) {
    match split_strategy {
        MitmSplitStrategy::Contiguous => {
            let split = order / 2;
            ((0..split).collect(), (split..order).collect())
        }
        MitmSplitStrategy::Parity => {
            let left = (0..order).filter(|index| index % 2 == 0).collect::<Vec<_>>();
            let right = (0..order).filter(|index| index % 2 == 1).collect::<Vec<_>>();
            (left, right)
        }
    }
}

fn assemble_full_pair(
    order: usize,
    factor: usize,
    left: &PartialCompressedPairState,
    right: &PartialCompressedPairState,
) -> Result<(CompressedSequence, CompressedSequence), String> {
    let mut values_a = vec![0_i16; order];
    let mut values_b = vec![0_i16; order];
    for (position, value) in left.positions.iter().zip(&left.values_a) {
        values_a[*position] = *value;
    }
    for (position, value) in right.positions.iter().zip(&right.values_a) {
        values_a[*position] = *value;
    }
    for (position, value) in left.positions.iter().zip(&left.values_b) {
        values_b[*position] = *value;
    }
    for (position, value) in right.positions.iter().zip(&right.values_b) {
        values_b[*position] = *value;
    }
    Ok((
        CompressedSequence::new(factor, values_a)?,
        CompressedSequence::new(factor, values_b)?,
    ))
}

fn partial_compressed_pair_feasible_sparse(
    a_assignments: &[Option<i16>],
    b_assignments: &[Option<i16>],
    order: usize,
    factor: usize,
) -> bool {
    let target = -2_i32 * factor as i32;
    for shift in 1..order {
        let mut min_possible = 0_i32;
        let mut max_possible = 0_i32;
        for index in 0..order {
            let partner = (index + shift) % order;
            let (min_a, max_a) =
                partial_product_interval_sparse(a_assignments, index, partner, factor as i32);
            let (min_b, max_b) =
                partial_product_interval_sparse(b_assignments, index, partner, factor as i32);
            min_possible += min_a + min_b;
            max_possible += max_a + max_b;
        }
        if target < min_possible || target > max_possible {
            return false;
        }
    }
    true
}

fn partial_product_interval_sparse(
    values: &[Option<i16>],
    index: usize,
    partner: usize,
    max_abs_symbol: i32,
) -> (i32, i32) {
    match (values[index], values[partner]) {
        (Some(lhs), Some(rhs)) => {
            let exact = i32::from(lhs) * i32::from(rhs);
            (exact, exact)
        }
        (Some(lhs), None) => {
            let bound = i32::from(lhs).abs() * max_abs_symbol;
            (-bound, bound)
        }
        (None, Some(rhs)) => {
            let bound = i32::from(rhs).abs() * max_abs_symbol;
            (-bound, bound)
        }
        (None, None) => {
            let bound = max_abs_symbol * max_abs_symbol;
            (-bound, bound)
        }
    }
}

pub fn run_sds_search(config: &SdsSearchConfig) -> Result<SdsSearchOutcome, String> {
    if config.order == 0 {
        return Err("SDS search requires a positive order".to_string());
    }
    if config.block_sizes.len() != 4 {
        return Err("SDS search currently expects exactly four block sizes".to_string());
    }
    if config.shard_count == 0 || config.shard_index >= config.shard_count {
        return Err("invalid shard specification".to_string());
    }
    if sds_target_lambda(config.order, &config.block_sizes) != Some(config.lambda) {
        return Err("block sizes do not match the requested lambda".to_string());
    }

    let block_pools = config
        .block_sizes
        .iter()
        .map(|size| enumerate_normalized_cyclic_blocks(config.order, *size))
        .collect::<Result<Vec<_>, _>>()?;

    let left_pairs = build_sds_pair_candidates(&block_pools[0], &block_pools[1], config.order);
    let right_pairs = build_sds_pair_candidates(&block_pools[2], &block_pools[3], config.order);
    let right_index = build_sds_pair_index(&right_pairs);

    let mut matches = Vec::new();
    let mut attempted_pairs = 0_usize;
    let mut seen = BTreeSet::new();

    'outer: for (pair_index, (left_a, left_b, left_profile)) in left_pairs.iter().enumerate() {
        if pair_index % config.shard_count != config.shard_index {
            continue;
        }
        let Some(target) = complement_sds_profile(left_profile, config.lambda) else {
            continue;
        };
        let Some(partners) = right_index.get(&target) else {
            continue;
        };
        for (right_a, right_b, _) in partners {
            attempted_pairs += 1;
            let blocks = vec![
                left_a.clone(),
                left_b.clone(),
                right_a.clone(),
                right_b.clone(),
            ];
            let sds = SupplementaryDifferenceSet::new(config.order, blocks.clone())?;
            if !sds.is_supplementary_difference_set() {
                continue;
            }
            let key = blocks
                .iter()
                .map(CyclicDifferenceBlock::to_line)
                .collect::<Vec<_>>()
                .join("|");
            if seen.insert(key) {
                matches.push(SdsMatch { blocks });
                if matches.len() >= config.max_matches {
                    break 'outer;
                }
            }
        }
    }

    Ok(SdsSearchOutcome {
        matches,
        attempted_pairs,
        pair_bucket_count: right_index.len(),
    })
}

fn enumerate_normalized_cyclic_blocks(
    order: usize,
    size: usize,
) -> Result<Vec<CyclicDifferenceBlock>, String> {
    if size > order {
        return Err("block size cannot exceed the group order".to_string());
    }
    if size == 0 {
        return Ok(vec![CyclicDifferenceBlock::new(order, Vec::new())?]);
    }

    let mut out = Vec::new();
    let mut current = vec![0_usize];
    enumerate_block_combinations(order, size - 1, 1, &mut current, &mut out)?;
    Ok(out)
}

fn enumerate_block_combinations(
    order: usize,
    remaining: usize,
    next: usize,
    current: &mut Vec<usize>,
    out: &mut Vec<CyclicDifferenceBlock>,
) -> Result<(), String> {
    if remaining == 0 {
        out.push(CyclicDifferenceBlock::new(order, current.clone())?);
        return Ok(());
    }

    for value in next..=order - remaining {
        current.push(value);
        enumerate_block_combinations(order, remaining - 1, value + 1, current, out)?;
        current.pop();
    }
    Ok(())
}

type SdsPairCandidate = (CyclicDifferenceBlock, CyclicDifferenceBlock, Vec<usize>);

fn build_sds_pair_candidates(
    first: &[CyclicDifferenceBlock],
    second: &[CyclicDifferenceBlock],
    order: usize,
) -> Vec<SdsPairCandidate> {
    let mut out = Vec::new();
    for left in first {
        let left_profile = left.difference_profile();
        for right in second {
            let right_profile = right.difference_profile();
            let mut combined = vec![0_usize; order];
            for shift in 1..order {
                combined[shift] = left_profile[shift] + right_profile[shift];
            }
            out.push((left.clone(), right.clone(), combined));
        }
    }
    out
}

fn build_sds_pair_index(
    pairs: &[SdsPairCandidate],
) -> BTreeMap<Vec<usize>, Vec<&SdsPairCandidate>> {
    let mut index = BTreeMap::new();
    for pair in pairs {
        index.entry(pair.2.clone()).or_insert_with(Vec::new).push(pair);
    }
    index
}

fn complement_sds_profile(profile: &[usize], lambda: usize) -> Option<Vec<usize>> {
    let mut complement = vec![0_usize; profile.len()];
    for shift in 1..profile.len() {
        if profile[shift] > lambda {
            return None;
        }
        complement[shift] = lambda - profile[shift];
    }
    Some(complement)
}

#[cfg(test)]
mod tests {
    use super::{
        decompress_bucket_artifact, direct_compressed_pair_probe, mitm_compressed_pair_probe,
        parse_bucket_artifact_text, run_legendre_search, run_sds_search, DecompressionConfig,
        DirectProbeOrdering, MitmSplitStrategy, SdsSearchConfig, SearchConfig, SearchMatch,
    };
    use hadamard_core::{CheckpointState, LegendrePair, CURRENT_ARTIFACT_VERSION};

    #[test]
    fn exact_search_recovers_known_length_five_pair() {
        let outcome = run_legendre_search(
            &SearchConfig {
                length: 5,
                compression: 1,
                shard_index: 0,
                shard_count: 1,
                max_attempts: 1024,
                row_sum_target: 1,
            },
            None,
        )
        .expect("search");
        assert!(!outcome.matches.is_empty());
        assert!(matches!(outcome.matches[0], SearchMatch::Exact(_)));
    }

    #[test]
    fn exact_search_recovers_known_length_eleven_pair() {
        let outcome = run_legendre_search(
            &SearchConfig {
                length: 11,
                compression: 1,
                shard_index: 0,
                shard_count: 1,
                max_attempts: 1_048_576,
                row_sum_target: 1,
            },
            None,
        )
        .expect("search");
        let found = outcome.matches.iter().any(|item| {
            matches!(
                item,
                SearchMatch::Exact(match_item)
                    if match_item.a.to_line() == "+++-++-+---"
                        && match_item.b.to_line() == "+++-++-+---"
            )
        });
        assert!(found, "expected known exact length-11 match");
    }

    #[test]
    fn exact_search_recovers_known_length_thirteen_pair() {
        let outcome = run_legendre_search(
            &SearchConfig {
                length: 13,
                compression: 1,
                shard_index: 0,
                shard_count: 1,
                max_attempts: 1_000_000,
                row_sum_target: 1,
            },
            None,
        )
        .expect("search");
        let found = outcome.matches.iter().any(|item| {
            matches!(
                item,
                SearchMatch::Exact(match_item)
                    if match_item.a.to_line() == "+++-+++-+----"
                        && match_item.b.to_line() == "+-+++--++-+--"
            )
        });
        assert!(found, "expected known exact length-13 match");
    }

    #[test]
    fn compressed_search_checkpoint_roundtrip_works() {
        let checkpoint = CheckpointState {
            version: 1,
            mode: "lp".into(),
            length: 9,
            compression: 3,
            shard_index: 0,
            shard_count: 1,
            next_attempt: 0,
            matches_found: 0,
        };
        let outcome = run_legendre_search(
            &SearchConfig {
                length: 9,
                compression: 3,
                shard_index: 0,
                shard_count: 1,
                max_attempts: 8,
                row_sum_target: 1,
            },
            Some(checkpoint),
        )
        .expect("search");
        assert!(outcome.checkpoint.next_attempt > 0);
    }

    #[test]
    fn compressed_search_recovers_known_length_nine_projection() {
        let outcome = run_legendre_search(
            &SearchConfig {
                length: 9,
                compression: 3,
                shard_index: 0,
                shard_count: 1,
                max_attempts: 256,
                row_sum_target: 1,
            },
            None,
        )
        .expect("search");
        let found = outcome.matches.iter().any(|item| {
            matches!(
                item,
                SearchMatch::Compressed(match_item)
                    if match_item.a.values() == [1, 1, -1] && match_item.b.values() == [3, -1, -1]
            )
        });
        assert!(found, "expected known compressed length-9 match");
        assert!(outcome.metrics.compatible_pool_a <= outcome.metrics.candidate_pool_a);
        assert!(outcome.metrics.compatible_pool_b <= outcome.metrics.candidate_pool_b);
        assert!(outcome.metrics.signature_pool_a <= outcome.metrics.compatible_pool_a);
        assert!(outcome.metrics.signature_pool_b <= outcome.metrics.compatible_pool_b);
        assert!(outcome.metrics.compatible_pool_a > 0);
        assert!(outcome.metrics.compatible_pool_b > 0);
        assert!(outcome.metrics.residual_zero_pairs >= outcome.metrics.accepted_pairs);
        assert!(outcome.metrics.psd_consistent_pairs == outcome.metrics.accepted_pairs);
        let bucket_artifact = outcome.bucket_artifact.expect("bucket artifact");
        let bucket_text = bucket_artifact.to_text();
        assert!(bucket_text.contains("bucket_count_a="));
        assert!(bucket_text.contains("a_candidate=1,1,-1"));
    }

    #[test]
    fn compressed_search_recovers_known_length_fifteen_projection() {
        let outcome = run_legendre_search(
            &SearchConfig {
                length: 15,
                compression: 3,
                shard_index: 0,
                shard_count: 1,
                max_attempts: 32_768,
                row_sum_target: 1,
            },
            None,
        )
        .expect("search");
        let found = outcome.matches.iter().any(|item| {
            matches!(
                item,
                SearchMatch::Compressed(match_item)
                    if match_item.a.values() == [1, 1, -1, 1, -1]
                        && match_item.b.values() == [3, 1, 1, -1, -3]
            )
        });
        assert!(found, "expected known compressed length-15 match");
        assert_eq!(outcome.metrics.candidate_pool_a, 135);
        assert_eq!(outcome.metrics.candidate_pool_b, 135);
        assert_eq!(outcome.metrics.compatible_pool_a, 135);
        assert_eq!(outcome.metrics.compatible_pool_b, 135);
        assert_eq!(outcome.metrics.signature_pool_a, 55);
        assert_eq!(outcome.metrics.signature_pool_b, 55);
        assert_eq!(outcome.metrics.residual_zero_pairs, 215);
        assert_eq!(outcome.metrics.psd_consistent_pairs, 215);
        let bucket_text = outcome.bucket_artifact.expect("bucket artifact").to_text();
        assert!(bucket_text.contains("length=15"));
        assert!(bucket_text.contains("a_candidate=1,1,-1,1,-1"));
        assert!(bucket_text.contains("b_candidate=3,1,1,-1,-3"));
    }

    #[test]
    fn bucket_artifact_parses_and_decompresses_known_length_nine_case() {
        let outcome = run_legendre_search(
            &SearchConfig {
                length: 9,
                compression: 3,
                shard_index: 0,
                shard_count: 1,
                max_attempts: 256,
                row_sum_target: 1,
            },
            None,
        )
        .expect("search");
        let bucket_text = outcome.bucket_artifact.expect("bucket artifact").to_text();
        let parsed = parse_bucket_artifact_text(&bucket_text).expect("parse");
        assert_eq!(parsed.length, 9);
        assert_eq!(parsed.compression, 3);
        let decompressed = decompress_bucket_artifact(
            &parsed,
            &DecompressionConfig { max_pairs: 64 },
        )
        .expect("decompress");
        assert!(!decompressed.exact_matches.is_empty());
        assert!(
            decompressed
                .artifact
                .to_text()
                .contains("canonical_exact_matches=")
        );
        assert!(decompressed.artifact.to_text().contains("a_branches_pruned="));
        assert!(decompressed.artifact.to_text().contains("a_exact_candidates=6"));
        assert!(decompressed.artifact.to_text().contains("b_exact_candidates=6"));
        assert!(decompressed.artifact.to_text().contains("a_exact_signature_buckets=6"));
        assert!(decompressed.artifact.to_text().contains("b_exact_signature_buckets=6"));
        assert!(decompressed.artifact.to_text().contains("pairs_checked=6"));
        assert!(decompressed.artifact.to_text().contains("canonical_exact_matches=3"));
        for item in &decompressed.exact_matches {
            let pair = LegendrePair::new(item.a.clone(), item.b.clone()).expect("pair");
            assert!(pair.is_legendre_pair());
            assert_eq!(
                pair.canonical_common_dihedral_pair()
                    .map(|(x, y)| (x.to_line(), y.to_line())),
                Some((item.a.to_line(), item.b.to_line()))
            );
        }
    }

    #[test]
    fn length_fifteen_bucket_artifact_decompresses_to_known_exact_pairs() {
        let outcome = run_legendre_search(
            &SearchConfig {
                length: 15,
                compression: 3,
                shard_index: 0,
                shard_count: 1,
                max_attempts: 32_768,
                row_sum_target: 1,
            },
            None,
        )
        .expect("search");
        let bucket_text = outcome.bucket_artifact.expect("bucket artifact").to_text();
        let parsed = parse_bucket_artifact_text(&bucket_text).expect("parse");
        let decompressed = decompress_bucket_artifact(
            &parsed,
            &DecompressionConfig { max_pairs: 4_096 },
        )
        .expect("decompress");
        assert!(!decompressed.exact_matches.is_empty());
        let found_known = decompressed.exact_matches.iter().any(|item| {
            item.a.to_line() == "++++-++-++-----"
                && item.b.to_line() == "+++--++-+-+-+--"
        });
        assert!(found_known, "expected known exact length-15 match");
        assert!(decompressed.artifact.to_text().contains("a_exact_candidates=43"));
        assert!(decompressed.artifact.to_text().contains("b_exact_candidates=43"));
        assert!(decompressed.artifact.to_text().contains("a_exact_signature_buckets=39"));
        assert!(decompressed.artifact.to_text().contains("b_exact_signature_buckets=39"));
        assert!(decompressed.artifact.to_text().contains("pairs_checked=47"));
        assert!(decompressed.artifact.to_text().contains("canonical_exact_matches=24"));
    }

    #[test]
    fn sds_search_recovers_small_known_z5_instance() {
        let outcome = run_sds_search(&SdsSearchConfig {
            order: 5,
            block_sizes: vec![2, 2, 0, 0],
            lambda: 1,
            shard_index: 0,
            shard_count: 1,
            max_matches: 8,
        })
        .expect("sds search");
        assert!(!outcome.matches.is_empty());
        let found_known = outcome.matches.iter().any(|item| {
            item.blocks[0].to_line() == "0,1"
                && item.blocks[1].to_line() == "0,2"
                && item.blocks[2].to_line().is_empty()
                && item.blocks[3].to_line().is_empty()
        });
        assert!(found_known, "expected known small SDS instance");
    }

    #[test]
    fn bucket_artifact_rejects_unknown_version() {
        let text = format!(
            "version={}\nfamily=lp-buckets\ndescription=test\nlength=9\ncompression=3\na_candidate=1,1,-1\nb_candidate=3,-1,-1\n",
            CURRENT_ARTIFACT_VERSION + 1
        );
        let error = parse_bucket_artifact_text(&text).expect_err("expected version error");
        assert!(error.contains("unsupported bucket artifact version"));
    }

    #[test]
    fn direct_compressed_pair_probe_recovers_known_length_fifteen_projection() {
        let outcome = direct_compressed_pair_probe(
            5,
            3,
            1,
            DirectProbeOrdering::Natural,
            4,
            3,
            512,
        )
        .expect("probe");
        assert!(!outcome.pairs.is_empty());
        let found_known = outcome.pairs.iter().any(|pair| {
            pair.a().values() == [1, 1, -1, 1, -1] && pair.b().values() == [3, 1, 1, -1, -3]
        });
        assert!(found_known, "expected known compressed pair");
        assert!(outcome.stats.branches_considered < 1_048_576);
        assert!(
            outcome.stats.row_sum_pruned > 0
                || outcome.stats.norm_pruned > 0
                || outcome.stats.autocorrelation_pruned > 0
                || outcome.stats.spectral_pruned > 0
                || outcome.stats.tail_candidates_checked > 0
        );
    }

    #[test]
    fn generator_order_direct_probe_recovers_known_length_fifteen_projection() {
        let outcome = direct_compressed_pair_probe(
            5,
            3,
            1,
            DirectProbeOrdering::Generator2,
            4,
            3,
            512,
        )
        .expect("probe");
        assert!(!outcome.pairs.is_empty());
        let found_known = outcome.pairs.iter().any(|pair| {
            pair.a().values() == [1, 1, -1, 1, -1] && pair.b().values() == [3, 1, 1, -1, -3]
        });
        assert!(found_known, "expected known compressed pair");
    }

    #[test]
    fn mitm_compressed_pair_probe_recovers_known_length_fifteen_projection() {
        let outcome =
            mitm_compressed_pair_probe(5, 3, 1, MitmSplitStrategy::Contiguous, 128).expect("probe");
        assert!(!outcome.pairs.is_empty());
        let found_known = outcome.pairs.iter().any(|pair| {
            common_dihedral_pair_equivalent(
                pair.a().values(),
                pair.b().values(),
                &[1, 1, -1, 1, -1],
                &[3, 1, 1, -1, -3],
            )
        });
        assert!(found_known, "expected known compressed pair");
        assert!(outcome.stats.left_states_emitted > 0);
        assert!(outcome.stats.right_states_emitted > 0);
        assert!(outcome.stats.join_candidates_checked > 0);
    }

    fn common_dihedral_pair_equivalent(
        lhs_a: &[i16],
        lhs_b: &[i16],
        rhs_a: &[i16],
        rhs_b: &[i16],
    ) -> bool {
        let n = lhs_a.len();
        if lhs_b.len() != n || rhs_a.len() != n || rhs_b.len() != n {
            return false;
        }
        for shift in 0..n {
            if rotate_i16(lhs_a, shift) == rhs_a && rotate_i16(lhs_b, shift) == rhs_b {
                return true;
            }
        }
        let reversed_a = reverse_i16(lhs_a);
        let reversed_b = reverse_i16(lhs_b);
        for shift in 0..n {
            if rotate_i16(&reversed_a, shift) == rhs_a && rotate_i16(&reversed_b, shift) == rhs_b {
                return true;
            }
        }
        false
    }

    fn rotate_i16(values: &[i16], shift: usize) -> Vec<i16> {
        let n = values.len();
        (0..n).map(|index| values[(index + shift) % n]).collect()
    }

    fn reverse_i16(values: &[i16]) -> Vec<i16> {
        let mut out = values.to_vec();
        out.reverse();
        out
    }
}
