use hadamard_core::{
    default_psd_backend, ArtifactHeader, CheckpointState, CompressedSequence, LegendrePair,
    SearchArtifact, Sequence,
};
use std::collections::{BTreeMap, BTreeSet};

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
        format!("attempted_pairs={}", metrics.attempted_pairs),
        format!("accepted_pairs={}", metrics.accepted_pairs),
        format!("candidate_pool_a={}", metrics.candidate_pool_a),
        format!("candidate_pool_b={}", metrics.candidate_pool_b),
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
    let mut first = Vec::new();
    let mut second = Vec::new();
    generate_compressed_sequences(
        reduced_length,
        &alphabet,
        config.row_sum_target,
        &mut Vec::new(),
        &mut first,
    );
    generate_compressed_sequences(
        reduced_length,
        &alphabet,
        config.row_sum_target,
        &mut Vec::new(),
        &mut second,
    );
    metrics.candidate_pool_a = first.len() as u64;
    metrics.candidate_pool_b = second.len() as u64;
    let psd_backend = default_psd_backend();
    let first = build_candidate_stats(first, psd_backend);
    let second = build_candidate_stats(second, psd_backend);
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
    'outer: for a in &first {
        for b in matching_partners(a, &second_index, config.compression) {
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
        &bucket_data.a_candidates,
        &mut expansion_stats_a,
    )?;
    let exact_b = build_exact_candidate_stats(
        bucket_data.length,
        bucket_data.compression,
        &bucket_data.b_candidates,
        &mut expansion_stats_b,
    )?;
    let exact_b_index = build_exact_signature_index(&exact_b);

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
                let Some((canonical_a, canonical_b)) = pair.canonical_common_shift_pair() else {
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
        format!("length={}", bucket_data.length),
        format!("compression={}", bucket_data.compression),
        format!("pairs_checked={pairs_checked}"),
        format!("a_exact_candidates={}", exact_a.len()),
        format!("b_exact_candidates={}", exact_b.len()),
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

fn build_exact_candidate_stats(
    length: usize,
    compression: usize,
    compressed_candidates: &[CompressedSequence],
    stats: &mut ExpansionStats,
) -> Result<Vec<ExactCandidateStats>, String> {
    let mut unique = BTreeMap::new();
    for compressed in compressed_candidates {
        let exact = expand_compressed_sequence(length, compression, compressed, stats)?;
        for sequence in exact {
            if !sequence.is_normalized() || !sequence.is_canonical_normalized_rotation() {
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
    compression: usize,
) -> bool {
    if selected[0][0] != 1 {
        return false;
    }

    for shift in 1..assigned_columns {
        if selected[shift][0] != 1 {
            continue;
        }
        let mut decided = false;
        for layer in 0..compression {
            for pos in 0..(assigned_columns - shift) {
                let lhs = selected[pos][layer];
                let rhs = selected[pos + shift][layer];
                if lhs < rhs {
                    decided = true;
                    break;
                }
                if lhs > rhs {
                    return false;
                }
            }
            if decided {
                break;
            }
        }
    }
    true
}

fn build_bucket_artifact_body(
    config: &SearchConfig,
    first_index: &BTreeMap<(i32, Vec<i64>), Vec<&CompressedCandidateStats>>,
    second_index: &BTreeMap<(i32, Vec<i64>), Vec<&CompressedCandidateStats>>,
    raw_a: u64,
    raw_b: u64,
    signature_a: u64,
    signature_b: u64,
) -> Vec<String> {
    let mut body = vec![
        format!("length={}", config.length),
        format!("compression={}", config.compression),
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

fn generate_compressed_sequences(
    remaining: usize,
    alphabet: &[i16],
    target_sum: i32,
    prefix: &mut Vec<i16>,
    out: &mut Vec<CompressedSequence>,
) {
    if remaining == 0 {
        let candidate = CompressedSequence::new(derived_factor(alphabet), prefix.clone()).expect("candidate");
        if candidate.row_sum() == target_sum {
            out.push(candidate);
        }
        return;
    }

    for value in alphabet {
        prefix.push(*value);
        generate_compressed_sequences(remaining - 1, alphabet, target_sum, prefix, out);
        prefix.pop();
    }
}

fn derived_factor(alphabet: &[i16]) -> usize {
    alphabet
        .iter()
        .copied()
        .max()
        .unwrap_or(1)
        .unsigned_abs() as usize
}

#[cfg(test)]
mod tests {
    use super::{
        decompress_bucket_artifact, parse_bucket_artifact_text, run_legendre_search,
        DecompressionConfig, SearchConfig, SearchMatch,
    };
    use hadamard_core::{CheckpointState, LegendrePair};

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
        assert_eq!(decompressed.exact_matches.len(), 1);
        assert!(
            decompressed
                .artifact
                .to_text()
                .contains("canonical_exact_matches=")
        );
        assert!(decompressed.artifact.to_text().contains("a_branches_pruned="));
        for item in &decompressed.exact_matches {
            let pair = LegendrePair::new(item.a.clone(), item.b.clone()).expect("pair");
            assert!(pair.is_legendre_pair());
            assert!(item.a.is_canonical_normalized_rotation());
            assert!(item.b.is_canonical_normalized_rotation());
            assert!(item.a.to_line() <= item.b.to_line());
        }
    }
}
