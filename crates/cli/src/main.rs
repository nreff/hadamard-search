use hadamard_construct::build_two_circulant_hadamard;
use hadamard_core::{
    available_psd_backends, exact_row_sum_square_candidates_167, get_psd_backend, CheckpointState,
    LegendrePair, Sequence,
};
use hadamard_search::{
    decompress_bucket_artifact, direct_compressed_pair_probe, mitm_compressed_pair_probe,
    parse_bucket_artifact_text, run_legendre_search, run_sds_search, DecompressionConfig,
    DirectProbeOrdering, MitmSplitStrategy, SdsSearchConfig, SearchConfig,
};
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;
use std::collections::{BTreeMap, BTreeSet, HashSet};

const CLI_BANNER: &str = "\
██╗  ██╗  █████╗  ██████╗   █████╗  ███╗   ███╗  █████╗  ██████╗  ██████╗          ██████╗   ██████╗   █████╗
██║  ██║ ██╔══██╗ ██╔══██╗ ██╔══██╗ ████╗ ████║ ██╔══██╗ ██╔══██╗ ██╔══██╗        ██╔════╝  ██╔════╝  ██╔══██╗
███████║ ███████║ ██║  ██║ ███████║ ██╔████╔██║ ███████║ ██████╔╝ ██║  ██║ █████╗ ███████╗  ███████╗  ╚█████╔╝
██╔══██║ ██╔══██║ ██║  ██║ ██╔══██║ ██║╚██╔╝██║ ██╔══██║ ██╔══██╗ ██║  ██║ ╚════╝ ██╔═══██╗ ██╔═══██╗ ██╔══██╗
██║  ██║ ██║  ██║ ██████╔╝ ██║  ██║ ██║ ╚═╝ ██║ ██║  ██║ ██║  ██║ ██████╔╝        ╚██████╔╝ ╚██████╔╝ ╚█████╔╝
╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═════╝  ╚═╝  ╚═╝ ╚═╝     ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═════╝          ╚═════╝   ╚═════╝   ╚════╝";

const BANNER_BG: (u8, u8, u8) = (52, 0, 29);
const BANNER_START: (u8, u8, u8) = (165, 164, 245);
const BANNER_END: (u8, u8, u8) = (0, 211, 255);
const LABEL_COLOR: (u8, u8, u8) = (110, 194, 255);
const ERROR_COLOR: (u8, u8, u8) = (255, 133, 162);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LpSearchFileConfig {
    length: Option<usize>,
    compression: Option<usize>,
    max_attempts: Option<u64>,
    row_sum_target: Option<i32>,
    shard: Option<String>,
}

fn main() {
    if let Err(error) = run() {
        if error.contains('\n') {
            eprintln!("{}", colorize_label("error:", ERROR_COLOR));
            eprintln!("{error}");
        } else {
            eprintln!("{} {error}", colorize_label("error:", ERROR_COLOR));
        }
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| usage("missing command"))?;

    match command.as_str() {
        "analyze" => cmd_analyze(args.collect()),
        "search" => cmd_search(args.collect()),
        "decompress" => cmd_decompress(args.collect()),
        "verify" => cmd_verify(args.collect()),
        "build" => cmd_build(args.collect()),
        "enumerate" => cmd_enumerate(args.collect()),
        "benchmark" => cmd_benchmark(args.collect()),
        "test-known" => cmd_test_known(args.collect()),
        _ => Err(usage("unknown command")),
    }
}

fn cmd_analyze(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("lp333-crt") => cmd_analyze_lp333_crt(args),
        _ => Err("expected `analyze lp333-crt`".to_string()),
    }
}

fn cmd_analyze_lp333_crt(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("lp333-crt") {
        return Err("expected `analyze lp333-crt`".to_string());
    }
    print_crt_marginal_analysis("row", 9, 37, 594)?;
    print_crt_marginal_analysis("column", 37, 9, 650)?;
    Ok(())
}

fn print_crt_marginal_analysis(
    label: &str,
    length: usize,
    block_size: i32,
    combined_norm_target: i32,
) -> Result<(), String> {
    let alphabet = odd_alphabet(block_size);
    let reachable = reachable_norms_by_sum(length, &alphabet);
    let sum_counts = if length <= 9 {
        Some(sequence_counts_by_sum_and_norm(length, &alphabet))
    } else {
        None
    };
    let norms = reachable
        .get(&1)
        .ok_or_else(|| format!("no reachable sum-1 states for {label} marginal"))?;
    let sum_one_sequence_count = if length <= 9 {
        Some(count_sum_one_sequences(length, &alphabet))
    } else {
        None
    };
    let feasible_norms = norms
        .iter()
        .copied()
        .filter(|norm| norms.contains(&(combined_norm_target - norm)))
        .collect::<Vec<_>>();
    println!("marginal={label}");
    println!("length={length}");
    println!("entry_bound={block_size}");
    println!("combined_norm_target={combined_norm_target}");
    println!("reachable_sum1_norms={}", norms.len());
    if let Some(count) = sum_one_sequence_count {
        println!("sum1_sequence_count={count}");
    }
    println!("feasible_norm_split_count={}", feasible_norms.len());
    println!(
        "feasible_norm_splits={}",
        feasible_norms
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    if let Some(sum_counts) = sum_counts.as_ref().and_then(|counts| counts.get(&1)) {
        let sample_counts = feasible_norms
            .iter()
            .step_by((feasible_norms.len().max(6) / 6).max(1))
            .map(|norm| format!("{norm}:{}", sum_counts.get(norm).copied().unwrap_or(0)))
            .collect::<Vec<_>>();
        println!("feasible_norm_sample_counts={}", sample_counts.join(","));
    }
    if label == "row" {
        let (raw_blocks, internal_signatures, endpoint_signatures) =
            length_three_block_signature_counts(&alphabet);
        println!("length3_raw_blocks={raw_blocks}");
        println!("length3_internal_signature_count={internal_signatures}");
        println!("length3_endpoint_signature_count={endpoint_signatures}");
        print_row_mod3_bundle_analysis()?;
    }
    println!();
    Ok(())
}

fn print_row_mod3_bundle_analysis() -> Result<(), String> {
    let alphabet = odd_alphabet(111);
    let reachable = reachable_norms_by_sum(3, &alphabet);
    let norms = reachable
        .get(&1)
        .ok_or_else(|| "no reachable sum-1 states for row mod-3 bundle marginal".to_string())?;
    let sum_one_sequence_count = count_sum_one_sequences(3, &alphabet);
    let feasible_norms = norms
        .iter()
        .copied()
        .filter(|norm| norms.contains(&(446 - norm)))
        .collect::<Vec<_>>();
    let (
        distinct_paf_values,
        ordered_pair_count,
        lifted_pair_upper_bound,
        norm_compatible_pair_count,
        norm_refined_lifted_pair_upper_bound,
        active_bundle_count,
        active_bundle_cyclic_orbit_count,
        active_pair_cyclic_orbit_count,
        active_pair_swap_orbit_count,
        active_pair_dihedral_swap_orbit_count,
        top_pair_dihedral_swap_mass_share,
        top_pair_dihedral_swap_orbits_by_mass,
        top_pair_swap_orbits_by_mass,
        top_pair_component_sizes,
        top_pair_naive_raw_lift_space,
        top_pair_uv_raw_pair_counts,
        top_pair_uv_signature_sizes,
        top_pair_uv_coefficient_signature_sizes,
        top_pair_cyclic_split_signature_sizes,
        top_pair_orbits_by_mass,
        top_active_bundle_orbits_by_mass,
        top_active_bundles_by_mass,
        top_norm_refined_pairs,
    ) = row_mod3_bundle_pair_counts(&alphabet);
    println!("row_mod3_bundle_length=3");
    println!("row_mod3_bundle_entry_bound=111");
    println!("row_mod3_bundle_combined_norm_target=446");
    println!("row_mod3_bundle_reachable_sum1_norms={}", norms.len());
    println!("row_mod3_bundle_sum1_sequence_count={sum_one_sequence_count}");
    println!(
        "row_mod3_bundle_feasible_norm_split_count={}",
        feasible_norms.len()
    );
    println!(
        "row_mod3_bundle_feasible_norm_splits={}",
        feasible_norms
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("row_mod3_bundle_distinct_paf1_values={distinct_paf_values}");
    println!("row_mod3_bundle_ordered_pair_count={ordered_pair_count}");
    println!("row_mod3_bundle_lifted_pair_upper_bound={lifted_pair_upper_bound}");
    println!("row_mod3_bundle_norm_compatible_pair_count={norm_compatible_pair_count}");
    println!(
        "row_mod3_bundle_norm_refined_lifted_pair_upper_bound={norm_refined_lifted_pair_upper_bound}"
    );
    println!("row_mod3_bundle_active_bundle_count={active_bundle_count}");
    println!("row_mod3_bundle_active_bundle_cyclic_orbit_count={active_bundle_cyclic_orbit_count}");
    println!("row_mod3_bundle_active_pair_cyclic_orbit_count={active_pair_cyclic_orbit_count}");
    println!("row_mod3_bundle_active_pair_swap_orbit_count={active_pair_swap_orbit_count}");
    println!(
        "row_mod3_bundle_active_pair_dihedral_swap_orbit_count={active_pair_dihedral_swap_orbit_count}"
    );
    println!("row_mod3_bundle_top_pair_dihedral_swap_mass_share={top_pair_dihedral_swap_mass_share}");
    println!(
        "row_mod3_bundle_top_pair_dihedral_swap_orbits_by_mass={}",
        top_pair_dihedral_swap_orbits_by_mass.join(";")
    );
    println!(
        "row_mod3_bundle_top_pair_swap_orbits_by_mass={}",
        top_pair_swap_orbits_by_mass.join(";")
    );
    println!("row_mod3_bundle_top_pair_component_sizes={top_pair_component_sizes}");
    println!("row_mod3_bundle_top_pair_naive_raw_lift_space={top_pair_naive_raw_lift_space}");
    println!("row_mod3_bundle_top_pair_uv_raw_pair_counts={top_pair_uv_raw_pair_counts}");
    println!("row_mod3_bundle_top_pair_uv_signature_sizes={top_pair_uv_signature_sizes}");
    println!(
        "row_mod3_bundle_top_pair_uv_coefficient_signature_sizes={top_pair_uv_coefficient_signature_sizes}"
    );
    println!(
        "row_mod3_bundle_top_pair_cyclic_split_signature_sizes={top_pair_cyclic_split_signature_sizes}"
    );
    println!(
        "row_mod3_bundle_top_pair_orbits_by_mass={}",
        top_pair_orbits_by_mass.join(";")
    );
    println!(
        "row_mod3_bundle_top_active_bundle_orbits_by_mass={}",
        top_active_bundle_orbits_by_mass.join(";")
    );
    println!(
        "row_mod3_bundle_top_active_bundles_by_mass={}",
        top_active_bundles_by_mass.join(";")
    );
    println!(
        "row_mod3_bundle_top_norm_refined_pairs={}",
        top_norm_refined_pairs.join(";")
    );
    Ok(())
}

fn odd_alphabet(bound: i32) -> Vec<i32> {
    (-bound..=bound).filter(|value| value % 2 != 0).collect()
}

fn reachable_norms_by_sum(length: usize, alphabet: &[i32]) -> BTreeMap<i32, BTreeSet<i32>> {
    let mut reachable = BTreeMap::<i32, BTreeSet<i32>>::new();
    reachable.insert(0, BTreeSet::from([0]));
    for _ in 0..length {
        let mut next = BTreeMap::<i32, BTreeSet<i32>>::new();
        for (sum, norms) in &reachable {
            for value in alphabet {
                let entry = next.entry(*sum + *value).or_default();
                let square = value * value;
                for norm in norms {
                    entry.insert(*norm + square);
                }
            }
        }
        reachable = next;
    }
    reachable
}

fn count_sum_one_sequences(length: usize, alphabet: &[i32]) -> u128 {
    let counts = sequence_counts_by_sum_and_norm(length, alphabet);
    counts
        .get(&1)
        .map(|by_norm| by_norm.values().copied().sum())
        .unwrap_or(0)
}

fn sequence_counts_by_sum_and_norm(
    length: usize,
    alphabet: &[i32],
) -> BTreeMap<i32, BTreeMap<i32, u128>> {
    let mut counts = BTreeMap::<i32, u128>::from([(0, 1)]);
    let mut by_sum_norm = BTreeMap::<i32, BTreeMap<i32, u128>>::from([(0, BTreeMap::from([(0, 1)]))]);
    for _ in 0..length {
        let mut next = BTreeMap::<i32, u128>::new();
        let mut next_by_sum_norm = BTreeMap::<i32, BTreeMap<i32, u128>>::new();
        for (sum, count) in &counts {
            for value in alphabet {
                *next.entry(*sum + *value).or_default() += *count;
            }
        }
        for (sum, by_norm) in &by_sum_norm {
            for value in alphabet {
                let square = value * value;
                let entry = next_by_sum_norm.entry(*sum + *value).or_default();
                for (norm, count) in by_norm {
                    *entry.entry(*norm + square).or_default() += *count;
                }
            }
        }
        counts = next;
        by_sum_norm = next_by_sum_norm;
    }
    by_sum_norm
}

fn length_three_block_signature_counts(alphabet: &[i32]) -> (usize, usize, usize) {
    let mut internal = BTreeSet::new();
    let mut with_endpoints = BTreeSet::new();
    for a in alphabet {
        for b in alphabet {
            for c in alphabet {
                let sum = a + b + c;
                let norm = a * a + b * b + c * c;
                let paf1 = a * b + b * c;
                let paf2 = a * c;
                internal.insert((sum, norm, paf1, paf2));
                with_endpoints.insert((sum, norm, paf1, paf2, a, c));
            }
        }
    }
    (alphabet.len().pow(3), internal.len(), with_endpoints.len())
}

#[derive(Clone, Debug)]
struct BundleSequenceAnalysis {
    bundle: [i32; 3],
    row_norm_counts: BTreeMap<i32, u128>,
}

fn row_mod3_bundle_pair_counts(
    alphabet: &[i32],
) -> (
    usize,
    u128,
    u128,
    u128,
    u128,
    usize,
    usize,
    usize,
    usize,
    usize,
    String,
    Vec<String>,
    Vec<String>,
    String,
    u128,
    String,
    String,
    String,
    String,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
) {
    let membership = alphabet.iter().copied().collect::<BTreeSet<_>>();
    let triple_lift_counts = sequence_counts_by_sum_and_norm(3, &odd_alphabet(37));
    let triple_sum_counts = triple_lift_counts
        .iter()
        .map(|(sum, by_norm)| (*sum, by_norm.values().copied().sum::<u128>()))
        .collect::<BTreeMap<_, _>>();
    let mut paf_counts = BTreeMap::<i32, u128>::new();
    let mut paf_lift_counts = BTreeMap::<i32, u128>::new();
    let mut bundles_by_paf = BTreeMap::<i32, Vec<[i32; 3]>>::new();
    for a in alphabet {
        for b in alphabet {
            let c = 1 - a - b;
            if !membership.contains(&c) {
                continue;
            }
            let paf1 = a * b + b * c + c * a;
            let lift = triple_sum_counts.get(a).copied().unwrap_or(0)
                * triple_sum_counts.get(b).copied().unwrap_or(0)
                * triple_sum_counts.get(&c).copied().unwrap_or(0);
            *paf_counts.entry(paf1).or_default() += 1;
            *paf_lift_counts.entry(paf1).or_default() += lift;
            bundles_by_paf.entry(paf1).or_default().push([*a, *b, c]);
        }
    }
    let ordered_pair_count = paf_counts
        .iter()
        .map(|(paf, count)| count * paf_counts.get(&(-222 - *paf)).copied().unwrap_or(0))
        .sum();
    let lifted_pair_upper_bound = paf_lift_counts
        .iter()
        .map(|(paf, count)| count * paf_lift_counts.get(&(-222 - *paf)).copied().unwrap_or(0))
        .sum();
    let active_pafs = paf_counts
        .keys()
        .copied()
        .filter(|paf| paf_counts.contains_key(&(-222 - *paf)))
        .collect::<BTreeSet<_>>();
    let active_bundle_count = active_pafs
        .iter()
        .map(|paf| bundles_by_paf.get(paf).map(Vec::len).unwrap_or(0))
        .sum();
    let active_bundles = active_pafs
        .iter()
        .flat_map(|paf| bundles_by_paf.get(paf).into_iter().flat_map(|bundles| bundles.iter().copied()))
        .collect::<Vec<_>>();
    let active_bundle_cyclic_orbit_count = active_bundles
        .iter()
        .map(|bundle| canonical_bundle_rotation(*bundle))
        .collect::<BTreeSet<_>>()
        .len();
    let triples = triples_by_sum(37);
    let mut sequences_by_paf = BTreeMap::<i32, Vec<BundleSequenceAnalysis>>::new();
    for paf in active_pafs {
        let analyses = bundles_by_paf
            .get(&paf)
            .into_iter()
            .flat_map(|bundles| bundles.iter())
            .map(|bundle| BundleSequenceAnalysis {
                bundle: *bundle,
                row_norm_counts: convolve_bundle_component_norms(*bundle, &triple_lift_counts),
            })
            .collect::<Vec<_>>();
        sequences_by_paf.insert(paf, analyses);
    }
    let mut norm_compatible_pair_count = 0u128;
    let mut norm_refined_lifted_pair_upper_bound = 0u128;
    let mut bundle_mass = BTreeMap::<[i32; 3], u128>::new();
    let mut orbit_mass = BTreeMap::<[i32; 3], u128>::new();
    let mut pair_orbit_mass = BTreeMap::<([i32; 3], [i32; 3]), u128>::new();
    let mut pair_swap_orbit_mass = BTreeMap::<([i32; 3], [i32; 3]), u128>::new();
    let mut pair_dihedral_swap_orbit_mass = BTreeMap::<([i32; 3], [i32; 3]), u128>::new();
    let mut top_pairs = Vec::<(u128, [i32; 3], [i32; 3])>::new();
    for (left_paf, left_sequences) in &sequences_by_paf {
        if let Some(right_sequences) = sequences_by_paf.get(&(-222 - *left_paf)) {
            for left in left_sequences {
                for right in right_sequences {
                    let compatible_lifts =
                        compatible_row_norm_pair_count(&left.row_norm_counts, &right.row_norm_counts, 594);
                    if compatible_lifts == 0 {
                        continue;
                    }
                    norm_compatible_pair_count += 1;
                    norm_refined_lifted_pair_upper_bound += compatible_lifts;
                    *bundle_mass.entry(left.bundle).or_default() += compatible_lifts;
                    *bundle_mass.entry(right.bundle).or_default() += compatible_lifts;
                    *orbit_mass
                        .entry(canonical_bundle_rotation(left.bundle))
                        .or_default() += compatible_lifts;
                    *orbit_mass
                        .entry(canonical_bundle_rotation(right.bundle))
                        .or_default() += compatible_lifts;
                    *pair_orbit_mass
                        .entry(canonical_pair_rotation(left.bundle, right.bundle))
                        .or_default() += compatible_lifts;
                    *pair_swap_orbit_mass
                        .entry(canonical_pair_swap_rotation(left.bundle, right.bundle))
                        .or_default() += compatible_lifts;
                    *pair_dihedral_swap_orbit_mass
                        .entry(canonical_pair_dihedral_swap(left.bundle, right.bundle))
                        .or_default() += compatible_lifts;
                    top_pairs.push((
                        compatible_lifts,
                        left.bundle,
                        right.bundle,
                    ));
                }
            }
        }
    }
    let active_pair_cyclic_orbit_count = pair_orbit_mass.len();
    let active_pair_swap_orbit_count = pair_swap_orbit_mass.len();
    let active_pair_dihedral_swap_orbit_count = pair_dihedral_swap_orbit_mass.len();
    top_pairs.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)).then_with(|| a.2.cmp(&b.2)));
    top_pairs.truncate(5);
    let mut top_bundles = bundle_mass.into_iter().collect::<Vec<_>>();
    top_bundles.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_bundles.truncate(5);
    let mut top_orbits = orbit_mass.into_iter().collect::<Vec<_>>();
    top_orbits.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_orbits.truncate(5);
    let mut top_pair_orbits = pair_orbit_mass.into_iter().collect::<Vec<_>>();
    top_pair_orbits.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut top_pair_swap_orbits = pair_swap_orbit_mass.into_iter().collect::<Vec<_>>();
    top_pair_swap_orbits.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_pair_swap_orbits.truncate(5);
    let mut top_pair_dihedral_swap_orbits = pair_dihedral_swap_orbit_mass.into_iter().collect::<Vec<_>>();
    top_pair_dihedral_swap_orbits.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let total_dihedral_swap_mass: u128 = top_pair_dihedral_swap_orbits.iter().map(|(_, mass)| *mass).sum();
    let full_dihedral_swap_mass: u128 = pair_dihedral_swap_orbit_mass.values().copied().sum();
    let top_pair_dihedral_swap_mass_share = if full_dihedral_swap_mass == 0 {
        "none".to_string()
    } else {
        format!(
            "top5={}/{} ({:.4})",
            total_dihedral_swap_mass,
            full_dihedral_swap_mass,
            total_dihedral_swap_mass as f64 / full_dihedral_swap_mass as f64
        )
    };
    top_pair_dihedral_swap_orbits.truncate(5);
    let top_pair_component_sizes = top_pair_orbits
        .first()
        .map(|((left, right), _)| {
            format!(
                "left=[{}:{}|{}:{}|{}:{}],right=[{}:{}|{}:{}|{}:{}]",
                left[0],
                triple_sum_counts.get(&left[0]).copied().unwrap_or(0),
                left[1],
                triple_sum_counts.get(&left[1]).copied().unwrap_or(0),
                left[2],
                triple_sum_counts.get(&left[2]).copied().unwrap_or(0),
                right[0],
                triple_sum_counts.get(&right[0]).copied().unwrap_or(0),
                right[1],
                triple_sum_counts.get(&right[1]).copied().unwrap_or(0),
                right[2],
                triple_sum_counts.get(&right[2]).copied().unwrap_or(0),
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let top_pair_naive_raw_lift_space = top_pair_orbits
        .first()
        .map(|((left, right), _)| {
            triple_sum_counts.get(&left[0]).copied().unwrap_or(0)
                * triple_sum_counts.get(&left[1]).copied().unwrap_or(0)
                * triple_sum_counts.get(&left[2]).copied().unwrap_or(0)
                * triple_sum_counts.get(&right[0]).copied().unwrap_or(0)
                * triple_sum_counts.get(&right[1]).copied().unwrap_or(0)
                * triple_sum_counts.get(&right[2]).copied().unwrap_or(0)
        })
        .unwrap_or(0);
    let top_pair_uv_raw_pair_counts = top_pair_orbits
        .first()
        .map(|((left, right), _)| {
            let left_raw = triple_sum_counts.get(&left[0]).copied().unwrap_or(0)
                * triple_sum_counts.get(&left[1]).copied().unwrap_or(0);
            let right_raw = triple_sum_counts.get(&right[0]).copied().unwrap_or(0)
                * triple_sum_counts.get(&right[1]).copied().unwrap_or(0);
            format!("left={left_raw},right={right_raw}")
        })
        .unwrap_or_else(|| "none".to_string());
    let top_pair_uv_signature_sizes = top_pair_orbits
        .first()
        .map(|((left, right), _)| {
            let left_uv = uv_transition_signature_count(*left, &triples);
            let right_uv = uv_transition_signature_count(*right, &triples);
            format!("left={left_uv},right={right_uv}")
        })
        .unwrap_or_else(|| "none".to_string());
    let top_pair_uv_coefficient_signature_sizes = top_pair_orbits
        .first()
        .map(|((left, right), _)| {
            let left_uv = uv_transition_coefficient_signature_count(*left, &triples);
            let right_uv = uv_transition_coefficient_signature_count(*right, &triples);
            format!("left={left_uv},right={right_uv}")
        })
        .unwrap_or_else(|| "none".to_string());
    let top_pair_cyclic_split_signature_sizes = top_pair_orbits
        .first()
        .map(|((left, right), _)| {
            format!(
                "left=[{},{},{}],right=[{},{},{}]",
                uv_transition_signature_count(*left, &triples),
                uv_transition_signature_count(cycle_bundle(*left, 1), &triples),
                uv_transition_signature_count(cycle_bundle(*left, 2), &triples),
                uv_transition_signature_count(*right, &triples),
                uv_transition_signature_count(cycle_bundle(*right, 1), &triples),
                uv_transition_signature_count(cycle_bundle(*right, 2), &triples),
            )
        })
        .unwrap_or_else(|| "none".to_string());
    top_pair_orbits.truncate(5);
    (
        paf_counts.len(),
        ordered_pair_count,
        lifted_pair_upper_bound,
        norm_compatible_pair_count,
        norm_refined_lifted_pair_upper_bound,
        active_bundle_count,
        active_bundle_cyclic_orbit_count,
        active_pair_cyclic_orbit_count,
        active_pair_swap_orbit_count,
        active_pair_dihedral_swap_orbit_count,
        top_pair_dihedral_swap_mass_share,
        top_pair_dihedral_swap_orbits
            .into_iter()
            .map(|((left, right), mass)| {
                format!(
                    "{mass}:[{},{},{}]|[{},{},{}]",
                    left[0], left[1], left[2], right[0], right[1], right[2]
                )
            })
            .collect(),
        top_pair_swap_orbits
            .into_iter()
            .map(|((left, right), mass)| {
                format!(
                    "{mass}:[{},{},{}]|[{},{},{}]",
                    left[0], left[1], left[2], right[0], right[1], right[2]
                )
            })
            .collect(),
        top_pair_component_sizes,
        top_pair_naive_raw_lift_space,
        top_pair_uv_raw_pair_counts,
        top_pair_uv_signature_sizes,
        top_pair_uv_coefficient_signature_sizes,
        top_pair_cyclic_split_signature_sizes,
        top_pair_orbits
            .into_iter()
            .map(|((left, right), mass)| {
                format!(
                    "{mass}:[{},{},{}]|[{},{},{}]",
                    left[0], left[1], left[2], right[0], right[1], right[2]
                )
            })
            .collect(),
        top_orbits
            .into_iter()
            .map(|(bundle, mass)| format!("{mass}:[{},{},{}]", bundle[0], bundle[1], bundle[2]))
            .collect(),
        top_bundles
            .into_iter()
            .map(|(bundle, mass)| format!("{mass}:[{},{},{}]", bundle[0], bundle[1], bundle[2]))
            .collect(),
        top_pairs
            .into_iter()
            .map(|(count, left, right)| {
                format!(
                    "{count}:[{},{},{}]|[{},{},{}]",
                    left[0], left[1], left[2], right[0], right[1], right[2]
                )
            })
            .collect(),
    )
}

fn canonical_bundle_rotation(bundle: [i32; 3]) -> [i32; 3] {
    let rotations = [
        bundle,
        cycle_bundle(bundle, 1),
        cycle_bundle(bundle, 2),
    ];
    *rotations.iter().min().unwrap()
}

fn canonical_pair_rotation(left: [i32; 3], right: [i32; 3]) -> ([i32; 3], [i32; 3]) {
    let rotations = [
        (left, right),
        (
            [left[1], left[2], left[0]],
            [right[1], right[2], right[0]],
        ),
        (
            [left[2], left[0], left[1]],
            [right[2], right[0], right[1]],
        ),
    ];
    *rotations.iter().min().unwrap()
}

fn canonical_pair_swap_rotation(left: [i32; 3], right: [i32; 3]) -> ([i32; 3], [i32; 3]) {
    let direct = canonical_pair_rotation(left, right);
    let swapped = canonical_pair_rotation(right, left);
    direct.min(swapped)
}

fn canonical_pair_dihedral_swap(left: [i32; 3], right: [i32; 3]) -> ([i32; 3], [i32; 3]) {
    let direct = canonical_pair_swap_rotation(left, right);
    let reversed = canonical_pair_swap_rotation(reverse3(left), reverse3(right));
    direct.min(reversed)
}

fn triples_by_sum(bound: i32) -> BTreeMap<i32, Vec<[i32; 3]>> {
    let alphabet = odd_alphabet(bound);
    let mut by_sum = BTreeMap::<i32, Vec<[i32; 3]>>::new();
    for a in &alphabet {
        for b in &alphabet {
            for c in &alphabet {
                by_sum.entry(a + b + c).or_default().push([*a, *b, *c]);
            }
        }
    }
    by_sum
}

fn uv_transition_signature_count(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> usize {
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let mut signatures = HashSet::new();
    for u in us {
        let ru1 = rotate3(*u, 1);
        let norm_u = dot3(*u, *u);
        for v in vs {
            let rv1 = rotate3(*v, 1);
            signatures.insert((
                norm_u + dot3(*v, *v),
                dot3(*u, *v),
                dot3(*v, ru1),
                dot3(*u, rv1),
                add3(*v, ru1),
                add3(*u, rv1),
                add3(rotate3(*u, 2), rotate3(*v, 2)),
            ));
        }
    }
    signatures.len()
}

fn uv_transition_coefficient_signature_count(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> usize {
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let mut signatures = HashSet::new();
    for u in us {
        let ru1 = rotate3(*u, 1);
        for v in vs {
            let rv1 = rotate3(*v, 1);
            signatures.insert((
                add3(*v, ru1),
                add3(*u, rv1),
                add3(rotate3(*u, 2), rotate3(*v, 2)),
            ));
        }
    }
    signatures.len()
}

fn dot3(a: [i32; 3], b: [i32; 3]) -> i32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add3(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn rotate3(a: [i32; 3], shift: usize) -> [i32; 3] {
    match shift % 3 {
        0 => a,
        1 => [a[1], a[2], a[0]],
        _ => [a[2], a[0], a[1]],
    }
}

fn cycle_bundle(bundle: [i32; 3], shift: usize) -> [i32; 3] {
    rotate3(bundle, shift)
}

fn reverse3(a: [i32; 3]) -> [i32; 3] {
    [a[0], a[2], a[1]]
}

fn convolve_bundle_component_norms(
    bundle: [i32; 3],
    triple_lift_counts: &BTreeMap<i32, BTreeMap<i32, u128>>,
) -> BTreeMap<i32, u128> {
    let mut counts = BTreeMap::from([(0, 1u128)]);
    for sum in bundle {
        let component = triple_lift_counts
            .get(&sum)
            .unwrap_or_else(|| panic!("missing triple lift counts for sum {sum}"));
        let mut next = BTreeMap::<i32, u128>::new();
        for (prefix_norm, prefix_count) in &counts {
            for (component_norm, component_count) in component {
                *next.entry(*prefix_norm + *component_norm).or_default() += prefix_count * component_count;
            }
        }
        counts = next;
    }
    counts
}

fn compatible_row_norm_pair_count(
    left_norm_counts: &BTreeMap<i32, u128>,
    right_norm_counts: &BTreeMap<i32, u128>,
    combined_target: i32,
) -> u128 {
    left_norm_counts
        .iter()
        .map(|(norm, count)| count * right_norm_counts.get(&(combined_target - *norm)).copied().unwrap_or(0))
        .sum()
}

fn cmd_search(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("lp") => cmd_search_lp(args),
        Some("sds") => cmd_search_sds(args),
        _ => Err("expected `search lp` or `search sds`".to_string()),
    }
}

fn cmd_search_lp(args: Vec<String>) -> Result<(), String> {
    let file_config = if let Some(path) = find_flag_value(&args, "--config") {
        Some(parse_lp_search_file_config(&path)?)
    } else {
        None
    };
    let length = parse_usize_flag_or_config(&args, "--length", file_config.as_ref().and_then(|cfg| cfg.length))?;
    let compression =
        parse_usize_flag_or_default(&args, "--compression", file_config.as_ref().and_then(|cfg| cfg.compression), 1)?;
    let max_attempts = parse_u64_flag_or_default(
        &args,
        "--max-attempts",
        file_config.as_ref().and_then(|cfg| cfg.max_attempts),
        1000,
    )?;
    let row_sum_target =
        parse_i32_flag_or_default(&args, "--row-sum", file_config.as_ref().and_then(|cfg| cfg.row_sum_target), 1)?;
    let shard_spec =
        find_flag_value(&args, "--shard")
            .or_else(|| file_config.as_ref().and_then(|cfg| cfg.shard.clone()))
            .unwrap_or_else(|| "0/1".to_string());
    let (shard_index, shard_count) = parse_shard_spec(&shard_spec)?;

    let checkpoint = if let Some(path) = find_flag_value(&args, "--resume") {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Some(CheckpointState::from_text(&text)?)
    } else {
        None
    };

    let outcome = run_legendre_search(
        &SearchConfig {
            length,
            compression,
            shard_index,
            shard_count,
            max_attempts,
            row_sum_target,
        },
        checkpoint,
    )?;

    println!("{}", outcome.artifact.to_text());
    if let Some(path) = find_flag_value(&args, "--checkpoint-out") {
        write_output_text(&path, &outcome.checkpoint.to_text())?;
        println!("checkpoint_written={path}");
    }
    if let Some(path) = find_flag_value(&args, "--artifact-out") {
        write_output_text(&path, &outcome.artifact.to_text())?;
        println!("artifact_written={path}");
    }
    if let Some(path) = find_flag_value(&args, "--bucket-out") {
        let bucket_artifact = outcome
            .bucket_artifact
            .as_ref()
            .ok_or_else(|| "bucket output is only available for compressed search".to_string())?;
        write_output_text(&path, &bucket_artifact.to_text())?;
        println!("bucket_written={path}");
    }
    Ok(())
}

fn cmd_search_sds(args: Vec<String>) -> Result<(), String> {
    let order = parse_flag_value(&args, "--order")?.parse::<usize>().map_err(|e| e.to_string())?;
    let block_sizes = parse_flag_value(&args, "--block-sizes")?
        .split(',')
        .map(|value| value.trim().parse::<usize>().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let lambda = parse_flag_value(&args, "--lambda")?.parse::<usize>().map_err(|e| e.to_string())?;
    let max_matches = parse_flag_value_or(&args, "--max-matches", "16")
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    let shard_spec = parse_flag_value_or(&args, "--shard", "0/1");
    let (shard_index, shard_count) = parse_shard_spec(&shard_spec)?;

    let outcome = run_sds_search(&SdsSearchConfig {
        order,
        block_sizes,
        lambda,
        shard_index,
        shard_count,
        max_matches,
    })?;

    println!("order={order}");
    println!("lambda={lambda}");
    println!("attempted_pairs={}", outcome.attempted_pairs);
    println!("pair_bucket_count={}", outcome.pair_bucket_count);
    println!("matches={}", outcome.matches.len());
    for found in outcome.matches {
        for (index, block) in found.blocks.iter().enumerate() {
            println!("block_{}={}", index + 1, block.to_line());
        }
        println!();
    }
    Ok(())
}

fn cmd_verify(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("lp") {
        return Err("expected `verify lp`".to_string());
    }
    let a = parse_sequence(&parse_flag_value(&args, "--a")?)?;
    let b = parse_sequence(&parse_flag_value(&args, "--b")?)?;
    let pair = LegendrePair::new(a, b)?;
    println!("is_legendre_pair={}", pair.is_legendre_pair());
    println!("row_sum_a={}", pair.a.row_sum());
    println!("row_sum_b={}", pair.b.row_sum());
    println!("two_circulant_ready={}", pair.has_two_circulant_row_sums());
    Ok(())
}

fn cmd_decompress(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("lp") {
        return Err("expected `decompress lp`".to_string());
    }
    let bucket_path = parse_flag_value(&args, "--bucket-in")?;
    let bucket_text = fs::read_to_string(&bucket_path).map_err(|e| e.to_string())?;
    let bucket_data = parse_bucket_artifact_text(&bucket_text)?;
    let max_pairs = parse_flag_value_or(&args, "--max-pairs", "256")
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    let outcome = decompress_bucket_artifact(&bucket_data, &DecompressionConfig { max_pairs })?;
    println!("{}", outcome.artifact.to_text());
    if let Some(path) = find_flag_value(&args, "--artifact-out") {
        write_output_text(&path, &outcome.artifact.to_text())?;
        println!("artifact_written={path}");
    }
    Ok(())
}

fn cmd_build(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("2cc") {
        return Err("expected `build 2cc`".to_string());
    }
    let a = parse_sequence(&parse_flag_value(&args, "--a")?)?;
    let b = parse_sequence(&parse_flag_value(&args, "--b")?)?;
    let pair = LegendrePair::new(a, b)?;
    let matrix = build_two_circulant_hadamard(&pair)?;
    println!("order={}", matrix.rows());
    println!("is_hadamard={}", matrix.is_hadamard());
    if let Some(path) = find_flag_value(&args, "--output") {
        let mut out = String::new();
        for row in 0..matrix.rows() {
            let mut line = String::new();
            for col in 0..matrix.cols() {
                let symbol = if matrix.get(row, col) == 1 { '+' } else { '-' };
                line.push(symbol);
            }
            out.push_str(&line);
            out.push('\n');
        }
        write_output_text(&path, &out)?;
        println!("matrix_written={path}");
    }
    Ok(())
}

fn cmd_enumerate(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("sds-167") {
        return Err("expected `enumerate sds-167`".to_string());
    }
    for (row_sums, k_values, lambda) in exact_row_sum_square_candidates_167() {
        println!(
            "row_sums={:?} k_values={:?} lambda={}",
            row_sums, k_values, lambda
        );
    }
    Ok(())
}

fn cmd_benchmark(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("psd") => cmd_benchmark_psd(args),
        Some("compressed-pairs") => cmd_benchmark_compressed_pairs(args),
        Some("compressed-pairs-mitm") => cmd_benchmark_compressed_pairs_mitm(args),
        _ => Err(
            "expected `benchmark psd`, `benchmark compressed-pairs`, or `benchmark compressed-pairs-mitm`"
                .to_string(),
        ),
    }
}

fn cmd_benchmark_psd(args: Vec<String>) -> Result<(), String> {
    let sequence = parse_sequence(&parse_flag_value_or(&args, "--sequence", "+--++"))?;
    let backend_name = parse_flag_value_or(&args, "--backend", "direct");
    let backend = get_psd_backend(&backend_name).ok_or_else(|| {
        format!(
            "unknown PSD backend `{backend_name}`; available: {}",
            available_psd_backends().join(", ")
        )
    })?;
    let psd = sequence.psd_with_backend(backend);
    let sum: f64 = psd.iter().sum();
    println!("backend={}", backend.name());
    println!("length={}", sequence.len());
    println!("psd_bins={}", psd.len());
    println!("psd_energy_sum={sum:.3}");
    Ok(())
}

fn cmd_benchmark_compressed_pairs(args: Vec<String>) -> Result<(), String> {
    let length = parse_flag_value(&args, "--length")?.parse::<usize>().map_err(|e| e.to_string())?;
    let compression = parse_flag_value_or(&args, "--compression", "3")
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    let ordering = parse_flag_value_or(&args, "--ordering", "natural");
    let spectral_frequency_count = parse_flag_value_or(&args, "--spectral-frequencies", "4")
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    let tail_depth = parse_flag_value_or(&args, "--tail-depth", "3")
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    let row_sum = parse_flag_value_or(&args, "--row-sum", "1")
        .parse::<i32>()
        .map_err(|e| e.to_string())?;
    let max_pairs = parse_flag_value_or(&args, "--max-pairs", "32")
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    if compression == 0 || length % compression != 0 {
        return Err("compression factor must divide the target length".to_string());
    }
    let ordering = match ordering.as_str() {
        "natural" => DirectProbeOrdering::Natural,
        "generator2" => DirectProbeOrdering::Generator2,
        _ => return Err("ordering must be `natural` or `generator2`".to_string()),
    };
    let reduced_length = length / compression;
    let outcome = direct_compressed_pair_probe(
        reduced_length,
        compression,
        row_sum,
        ordering,
        spectral_frequency_count,
        tail_depth,
        max_pairs,
    )?;
    println!("mode=compressed-pair-probe");
    println!("length={length}");
    println!("compression={compression}");
    println!("reduced_length={reduced_length}");
    println!("ordering={}", ordering.as_str());
    println!("spectral_frequency_count={spectral_frequency_count}");
    println!("tail_depth={tail_depth}");
    println!("effective_tail_depth={}", tail_depth.min(reduced_length).min(12));
    println!("row_sum_target={row_sum}");
    println!("max_pairs={max_pairs}");
    println!("branches_considered={}", outcome.stats.branches_considered);
    println!("row_sum_pruned={}", outcome.stats.row_sum_pruned);
    println!("norm_pruned={}", outcome.stats.norm_pruned);
    println!(
        "autocorrelation_pruned={}",
        outcome.stats.autocorrelation_pruned
    );
    println!("spectral_pruned={}", outcome.stats.spectral_pruned);
    println!("tail_shift_pruned={}", outcome.stats.tail_shift_pruned);
    println!(
        "tail_spectral_pruned={}",
        outcome.stats.tail_spectral_pruned
    );
    println!(
        "tail_residual_pruned={}",
        outcome.stats.tail_residual_pruned
    );
    println!(
        "tail_candidates_checked={}",
        outcome.stats.tail_candidates_checked
    );
    println!("pairs_emitted={}", outcome.stats.pairs_emitted);
    for pair in outcome.pairs {
        println!("compressed_a={}", pair.a().to_line());
        println!("compressed_b={}", pair.b().to_line());
    }
    Ok(())
}

fn cmd_benchmark_compressed_pairs_mitm(args: Vec<String>) -> Result<(), String> {
    let length = parse_flag_value(&args, "--length")?.parse::<usize>().map_err(|e| e.to_string())?;
    let compression = parse_flag_value_or(&args, "--compression", "3")
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    let split = parse_flag_value_or(&args, "--split", "contiguous");
    let row_sum = parse_flag_value_or(&args, "--row-sum", "1")
        .parse::<i32>()
        .map_err(|e| e.to_string())?;
    let max_pairs = parse_flag_value_or(&args, "--max-pairs", "32")
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    if compression == 0 || length % compression != 0 {
        return Err("compression factor must divide the target length".to_string());
    }
    let split_strategy = match split.as_str() {
        "contiguous" => MitmSplitStrategy::Contiguous,
        "parity" => MitmSplitStrategy::Parity,
        _ => return Err("split must be `contiguous` or `parity`".to_string()),
    };
    let reduced_length = length / compression;
    let outcome = mitm_compressed_pair_probe(
        reduced_length,
        compression,
        row_sum,
        split_strategy,
        max_pairs,
    )?;
    let (left_width, right_width) = match split_strategy {
        MitmSplitStrategy::Contiguous => (reduced_length / 2, reduced_length - (reduced_length / 2)),
        MitmSplitStrategy::Parity => (reduced_length - (reduced_length / 2), reduced_length / 2),
    };
    let estimated_left_bytes =
        outcome.stats.left_states_emitted as usize * left_width * 2 * std::mem::size_of::<i16>();
    let estimated_right_bytes =
        outcome.stats.right_states_emitted as usize * right_width * 2 * std::mem::size_of::<i16>();
    println!("mode=compressed-pair-probe-mitm");
    println!("length={length}");
    println!("compression={compression}");
    println!("reduced_length={reduced_length}");
    println!("split={}", split_strategy.as_str());
    println!("row_sum_target={row_sum}");
    println!("max_pairs={max_pairs}");
    println!("branches_considered={}", outcome.stats.branches_considered);
    println!("row_sum_pruned={}", outcome.stats.row_sum_pruned);
    println!("norm_pruned={}", outcome.stats.norm_pruned);
    println!(
        "autocorrelation_pruned={}",
        outcome.stats.autocorrelation_pruned
    );
    println!("left_states_emitted={}", outcome.stats.left_states_emitted);
    println!("right_states_emitted={}", outcome.stats.right_states_emitted);
    println!(
        "join_candidates_checked={}",
        outcome.stats.join_candidates_checked
    );
    println!("estimated_left_state_bytes={estimated_left_bytes}");
    println!("estimated_right_state_bytes={estimated_right_bytes}");
    println!("pairs_emitted={}", outcome.stats.pairs_emitted);
    for pair in outcome.pairs {
        println!("compressed_a={}", pair.a().to_line());
        println!("compressed_b={}", pair.b().to_line());
    }
    Ok(())
}

fn cmd_test_known(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("lp-small") => test_known_pair("+--++", "+-+-+"),
        Some("lp-seven") => test_known_pair("+--+-++", "+--+-++"),
        Some("lp-nine") => test_known_pair("+---+-+++", "+--++-+-+"),
        Some("lp-eleven") => test_known_pair("+++-++-+---", "+++-++-+---"),
        Some("lp-thirteen") => test_known_pair("+++-+++-+----", "+-+++--++-+--"),
        _ => Err(
            "expected `test-known lp-small`, `test-known lp-seven`, `test-known lp-nine`, `test-known lp-eleven`, or `test-known lp-thirteen`"
                .to_string(),
        ),
    }
}

fn test_known_pair(a_text: &str, b_text: &str) -> Result<(), String> {
    let a = parse_sequence(a_text)?;
    let b = parse_sequence(b_text)?;
    let pair = LegendrePair::new(a, b)?;
    let matrix = build_two_circulant_hadamard(&pair)?;
    println!("is_legendre_pair={}", pair.is_legendre_pair());
    println!("order={}", matrix.rows());
    println!("is_hadamard={}", matrix.is_hadamard());
    Ok(())
}

fn parse_sequence(input: &str) -> Result<Sequence, String> {
    let mut values = Vec::new();
    for ch in input.chars() {
        match ch {
            '+' => values.push(1),
            '-' => values.push(-1),
            _ => return Err(format!("invalid sequence symbol: {ch}")),
        }
    }
    Sequence::new(values)
}

fn parse_flag_value(args: &[String], flag: &str) -> Result<String, String> {
    find_flag_value(args, flag).ok_or_else(|| format!("missing required flag `{flag}`"))
}

fn parse_flag_value_or(args: &[String], flag: &str, default: &str) -> String {
    find_flag_value(args, flag).unwrap_or_else(|| default.to_string())
}

fn parse_usize_flag_or_config(
    args: &[String],
    flag: &str,
    config_value: Option<usize>,
) -> Result<usize, String> {
    if let Some(value) = find_flag_value(args, flag) {
        return value.parse::<usize>().map_err(|e| e.to_string());
    }
    config_value.ok_or_else(|| format!("missing required flag `{flag}`"))
}

fn parse_usize_flag_or_default(
    args: &[String],
    flag: &str,
    config_value: Option<usize>,
    default: usize,
) -> Result<usize, String> {
    if let Some(value) = find_flag_value(args, flag) {
        return value.parse::<usize>().map_err(|e| e.to_string());
    }
    Ok(config_value.unwrap_or(default))
}

fn parse_u64_flag_or_default(
    args: &[String],
    flag: &str,
    config_value: Option<u64>,
    default: u64,
) -> Result<u64, String> {
    if let Some(value) = find_flag_value(args, flag) {
        return value.parse::<u64>().map_err(|e| e.to_string());
    }
    Ok(config_value.unwrap_or(default))
}

fn parse_i32_flag_or_default(
    args: &[String],
    flag: &str,
    config_value: Option<i32>,
    default: i32,
) -> Result<i32, String> {
    if let Some(value) = find_flag_value(args, flag) {
        return value.parse::<i32>().map_err(|e| e.to_string());
    }
    Ok(config_value.unwrap_or(default))
}

fn parse_lp_search_file_config(path: &str) -> Result<LpSearchFileConfig, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut config = LpSearchFileConfig::default();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("invalid config line `{line}`"));
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "length" => config.length = Some(value.parse::<usize>().map_err(|e| e.to_string())?),
            "compression" => {
                config.compression = Some(value.parse::<usize>().map_err(|e| e.to_string())?)
            }
            "max_attempts" => {
                config.max_attempts = Some(value.parse::<u64>().map_err(|e| e.to_string())?)
            }
            "row_sum" => {
                config.row_sum_target = Some(value.parse::<i32>().map_err(|e| e.to_string())?)
            }
            "shard" => config.shard = Some(value.to_string()),
            _ => return Err(format!("unknown LP search config key `{key}`")),
        }
    }
    Ok(config)
}

fn find_flag_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}

fn parse_shard_spec(spec: &str) -> Result<(usize, usize), String> {
    let (left, right) = spec
        .split_once('/')
        .ok_or_else(|| "shard must be in index/count format".to_string())?;
    let index = left.parse::<usize>().map_err(|e| e.to_string())?;
    let count = right.parse::<usize>().map_err(|e| e.to_string())?;
    if index >= count {
        return Err("shard index must be smaller than shard count".to_string());
    }
    Ok((index, count))
}

fn usage(message: &str) -> String {
    let banner = render_banner();
    let message = if use_ansi() {
        colorize_label(message, ERROR_COLOR)
    } else {
        message.to_string()
    };
    let usage_label = colorize_label("usage:", LABEL_COLOR);
    format!(
        "{banner}\n\n{message}\n{usage_label}\n  hadamard analyze lp333-crt\n  hadamard search lp [--config PATH] --length N [--compression D] [--max-attempts M] [--shard i/n]\n  hadamard search sds --order N --block-sizes k1,k2,k3,k4 --lambda L [--max-matches M] [--shard i/n]\n  hadamard decompress lp --bucket-in PATH [--max-pairs N] [--artifact-out PATH]\n  hadamard verify lp --a +--++ --b +-+-+\n  hadamard build 2cc --a +--++ --b +-+-+\n  hadamard enumerate sds-167\n  hadamard benchmark psd [--sequence +--++] [--backend direct|fft|autocorrelation]\n  hadamard benchmark compressed-pairs --length N [--compression D] [--ordering natural|generator2] [--spectral-frequencies K] [--tail-depth T] [--row-sum R] [--max-pairs M]\n  hadamard benchmark compressed-pairs-mitm --length N [--compression D] [--split contiguous|parity] [--row-sum R] [--max-pairs M]\n  hadamard test-known lp-small|lp-seven|lp-nine|lp-eleven|lp-thirteen"
    )
}

fn use_ansi() -> bool {
    env::var_os("NO_COLOR").is_none()
        && env::var("TERM").map(|term| term != "dumb").unwrap_or(true)
        && std::io::stderr().is_terminal()
}

fn render_banner() -> String {
    if !use_ansi() {
        return CLI_BANNER.to_string();
    }

    CLI_BANNER
        .lines()
        .map(render_banner_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_banner_line(line: &str) -> String {
    let visible_count = line.chars().filter(|ch| *ch != ' ').count().max(1);
    let mut visible_index = 0usize;
    let mut styled = String::new();

    for ch in line.chars() {
        if ch == ' ' {
            styled.push_str(&bg_color(BANNER_BG));
            styled.push(' ');
            continue;
        }

        let rgb = interpolate_color(BANNER_START, BANNER_END, visible_index, visible_count - 1);
        styled.push_str(&format!(
            "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m{}",
            BANNER_BG.0, BANNER_BG.1, BANNER_BG.2, rgb.0, rgb.1, rgb.2, ch
        ));
        visible_index += 1;
    }

    styled.push_str("\x1b[0m");
    styled
}

fn interpolate_color(
    start: (u8, u8, u8),
    end: (u8, u8, u8),
    index: usize,
    max_index: usize,
) -> (u8, u8, u8) {
    if max_index == 0 {
        return start;
    }

    let mix = index as f32 / max_index as f32;
    (
        lerp_channel(start.0, end.0, mix),
        lerp_channel(start.1, end.1, mix),
        lerp_channel(start.2, end.2, mix),
    )
}

fn lerp_channel(start: u8, end: u8, mix: f32) -> u8 {
    let start = start as f32;
    let end = end as f32;
    (start + (end - start) * mix).round() as u8
}

fn colorize_label(text: &str, color: (u8, u8, u8)) -> String {
    if !use_ansi() {
        return text.to_string();
    }

    format!(
        "\x1b[38;2;{};{};{}m{}\x1b[0m",
        color.0, color.1, color.2, text
    )
}

fn bg_color(color: (u8, u8, u8)) -> String {
    format!("\x1b[48;2;{};{};{}m", color.0, color.1, color.2)
}

fn write_output_text(path: &str, contents: &str) -> Result<(), String> {
    let output_path = Path::new(path);
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    fs::write(output_path, contents).map_err(|e| e.to_string())
}

#[allow(dead_code)]
fn _assert_path_is_relative(path: &str) -> bool {
    !Path::new(path).is_absolute()
}

#[cfg(test)]
mod tests {
    use super::parse_lp_search_file_config;

    #[test]
    fn lp_search_config_parser_reads_key_value_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("hadamard-lp-search.cfg");
        std::fs::write(
            &path,
            "length=333\ncompression=3\nmax_attempts=4096\nrow_sum=1\nshard=0/8\n",
        )
        .expect("write config");
        let parsed = parse_lp_search_file_config(path.to_str().expect("utf8 path")).expect("config");
        assert_eq!(parsed.length, Some(333));
        assert_eq!(parsed.compression, Some(3));
        assert_eq!(parsed.max_attempts, Some(4096));
        assert_eq!(parsed.row_sum_target, Some(1));
        assert_eq!(parsed.shard.as_deref(), Some("0/8"));
        std::fs::remove_file(path).expect("remove config");
    }
}
