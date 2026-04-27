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
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;
use std::sync::OnceLock;

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
const LP333_ROW_NORM_TARGET: i32 = 594;
const LP333_ROW_SHIFT_TARGET: i32 = -74;
const LP333_ACTUAL_SHIFT_TARGET: i32 = -2;
const LP333_ROW_SHIFT_BOUND: i32 = 12321;

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
        Some("lp333-crt-bundle") => cmd_analyze_lp333_crt_bundle(args),
        Some("lp333-crt-pair") => cmd_analyze_lp333_crt_pair(args),
        Some("lp333-crt-component") => cmd_analyze_lp333_crt_component(args),
        Some("lp333-multiplier") => cmd_analyze_lp333_multiplier(args),
        _ => Err(
            "expected `analyze lp333-crt`, `analyze lp333-crt-bundle`, `analyze lp333-crt-pair`, `analyze lp333-crt-component`, or `analyze lp333-multiplier`"
                .to_string(),
        ),
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

fn cmd_analyze_lp333_crt_bundle(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("lp333-crt-bundle") {
        return Err("expected `analyze lp333-crt-bundle`".to_string());
    }
    let bundle = find_flag_value(&args, "--bundle")
        .map(|value| parse_i32_triple(&value))
        .transpose()?
        .map(canonical_bundle_rotation)
        .unwrap_or([-15, 5, 11]);
    print_lp333_crt_bundle_analysis(bundle)?;
    Ok(())
}

fn cmd_analyze_lp333_crt_pair(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("lp333-crt-pair") {
        return Err("expected `analyze lp333-crt-pair`".to_string());
    }
    let left = find_flag_value(&args, "--left")
        .map(|value| parse_i32_triple(&value))
        .transpose()?
        .map(canonical_bundle_rotation)
        .unwrap_or([-15, 5, 11]);
    let right = find_flag_value(&args, "--right")
        .map(|value| parse_i32_triple(&value))
        .transpose()?
        .map(canonical_bundle_rotation)
        .unwrap_or([-5, -1, 7]);
    let left_shift = find_flag_value(&args, "--left-shift")
        .map(|value| parse_usize_flag(&value, "--left-shift"))
        .transpose()?
        .unwrap_or(0);
    let right_shift = find_flag_value(&args, "--right-shift")
        .map(|value| parse_usize_flag(&value, "--right-shift"))
        .transpose()?
        .unwrap_or(0);
    let shift = find_flag_value(&args, "--shift")
        .map(|value| parse_row_shift_flag(&value))
        .transpose()?
        .unwrap_or(4);
    let exact_join = args.iter().any(|arg| arg == "--exact-join");
    let sample_buckets = find_flag_value(&args, "--sample-buckets")
        .map(|value| parse_usize_flag(&value, "--sample-buckets"))
        .transpose()?
        .unwrap_or(3);
    let frontier_join = args.iter().any(|arg| arg == "--frontier-join");
    let frontier_exact_join = args.iter().any(|arg| arg == "--frontier-exact-join");
    let two_shifts = args.iter().any(|arg| arg == "--two-shifts");
    let all_shifts = args.iter().any(|arg| arg == "--all-shifts");
    if left_shift > 2 || right_shift > 2 {
        return Err("bundle split shifts must be in {0,1,2}".to_string());
    }
    print_lp333_crt_pair_analysis(
        left,
        right,
        left_shift,
        right_shift,
        shift,
        exact_join,
        sample_buckets,
        frontier_join,
        frontier_exact_join,
        two_shifts,
        all_shifts,
    )?;
    Ok(())
}

fn cmd_analyze_lp333_crt_component(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("lp333-crt-component") {
        return Err("expected `analyze lp333-crt-component`".to_string());
    }
    let hub = find_flag_value(&args, "--hub")
        .map(|value| parse_i32_triple(&value))
        .transpose()?
        .map(canonical_bundle_rotation)
        .unwrap_or([-15, 5, 11]);
    print_lp333_crt_component_analysis(hub)?;
    Ok(())
}

fn cmd_analyze_lp333_multiplier(args: Vec<String>) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("lp333-multiplier") {
        return Err("expected `analyze lp333-multiplier`".to_string());
    }
    let include_col10_shift1 = args.iter().any(|arg| arg == "--col10-shift1");
    print_lp333_multiplier_analysis(include_col10_shift1)
}

fn print_lp333_multiplier_analysis(include_col10_shift1: bool) -> Result<(), String> {
    let n = 333u32;
    let (p, q) = (9u32, 37u32);

    // 1. Group structure
    let units = units_mod(n);
    let units9 = units_mod(p);
    let units37 = units_mod(q);
    println!("multiplier_group_order={}", units.len());
    println!("u9_elements={}", format_u32_list(&units9));
    println!("u9_order={}", units9.len());
    println!("u37_order={}", units37.len());

    let gen9 = primitive_root_mod_prime_power(3, 2);
    let gen37 = primitive_root_mod_prime(37);
    println!("u9_generator={gen9}");
    println!("u37_generator={gen37}");
    debug_assert_eq!(mult_order(gen9, p), units9.len() as u32);
    debug_assert_eq!(mult_order(gen37, q), units37.len() as u32);

    // 2. Coordinate orbits on Z_333
    let coord_orbits = multiplier_orbits(n, &units);
    let total_elements: usize = coord_orbits.iter().map(|orbit| orbit.len()).sum();
    debug_assert_eq!(total_elements, (n - 1) as usize);
    println!("coordinate_orbit_count={}", coord_orbits.len());
    let orbit_size_dist = size_distribution(&coord_orbits);
    println!(
        "coordinate_orbit_size_distribution={}",
        format_size_dist(&orbit_size_dist)
    );
    let min_orbit = coord_orbits.iter().map(|o| o.len()).min().unwrap_or(0);
    let max_orbit = coord_orbits.iter().map(|o| o.len()).max().unwrap_or(0);
    println!("min_coordinate_orbit_size={min_orbit}");
    println!("max_coordinate_orbit_size={max_orbit}");

    // 3. CRT-factored orbit analysis
    // Orbits of U(9) on Z_9 \ {0}
    let row_orbits = multiplier_orbits(p, &units9);
    println!("row_index_orbit_count={}", row_orbits.len());
    for (i, orbit) in row_orbits.iter().enumerate() {
        let mut sorted = orbit.clone();
        sorted.sort_unstable();
        println!(
            "row_index_orbit_{i}={}",
            sorted
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }

    // Orbits of U(37) on Z_37 \ {0}
    let col_orbits = multiplier_orbits(q, &units37);
    println!("column_index_orbit_count={}", col_orbits.len());
    let col_orbit_size_dist = size_distribution(&col_orbits);
    println!(
        "column_index_orbit_size_distribution={}",
        format_size_dist(&col_orbit_size_dist)
    );

    // Orbits of U(9) on Z_9 mod 3 (the three residue classes {0,3,6}, {1,4,7}, {2,5,8})
    println!();
    println!("# Row-index orbits under U(9) and mod-3 bundling");
    for orbit in &row_orbits {
        let mod3_classes = orbit.iter().map(|u| u % 3).collect::<BTreeSet<_>>();
        let mut sorted = orbit.clone();
        sorted.sort_unstable();
        println!(
            "row_orbit={} mod3_classes={}",
            sorted
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(","),
            mod3_classes
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }

    // 4. Bundle-preserving multiplier subgroup
    // t mod 3 ≡ 1 means t preserves mod-3 residue classes in Z_9
    let bundle_preserving: Vec<u32> = units.iter().copied().filter(|t| t % 3 == 1).collect();
    println!();
    println!(
        "bundle_preserving_multiplier_count={}",
        bundle_preserving.len()
    );

    // Also check t mod 3 ≡ 2 (swaps classes 1↔2, fixes class 0)
    let bundle_swapping: Vec<u32> = units.iter().copied().filter(|t| t % 3 == 2).collect();
    println!("bundle_swapping_multiplier_count={}", bundle_swapping.len());

    // 5. Row-preserving subgroup (t ≡ 1 mod 9)
    let row_preserving: Vec<u32> = units.iter().copied().filter(|t| t % p == 1).collect();
    println!();
    println!("row_preserving_multiplier_count={}", row_preserving.len());
    // These act only on columns: v → t*v mod 37
    let col_orbits_under_row_preserving =
        multiplier_orbits(q, &row_preserving.iter().map(|t| t % q).collect::<Vec<_>>());
    println!(
        "row_preserving_column_orbit_count={}",
        col_orbits_under_row_preserving.len()
    );
    let rp_col_size_dist = size_distribution(&col_orbits_under_row_preserving);
    println!(
        "row_preserving_column_orbit_size_distribution={}",
        format_size_dist(&rp_col_size_dist)
    );

    // 6. Column-preserving subgroup (t ≡ 1 mod 37)
    let col_preserving: Vec<u32> = units.iter().copied().filter(|t| t % q == 1).collect();
    println!();
    println!(
        "column_preserving_multiplier_count={}",
        col_preserving.len()
    );
    println!(
        "column_preserving_multipliers={}",
        col_preserving
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    // These act only on rows: u → t*u mod 9
    let row_orbits_under_col_preserving =
        multiplier_orbits(p, &col_preserving.iter().map(|t| t % p).collect::<Vec<_>>());
    println!(
        "column_preserving_row_orbit_count={}",
        row_orbits_under_col_preserving.len()
    );
    for (i, orbit) in row_orbits_under_col_preserving.iter().enumerate() {
        let mut sorted = orbit.clone();
        sorted.sort_unstable();
        println!(
            "column_preserving_row_orbit_{i}={}",
            sorted
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }

    // 7. Normalization potential: orbit of index 1
    let orbit_of_1 = {
        let mut orbit = BTreeSet::new();
        for &t in &units {
            orbit.insert((t % n) as u32);
        }
        orbit
    };
    let stabilizer_of_1: Vec<u32> = units.iter().copied().filter(|t| (t * 1) % n == 1).collect();
    println!();
    println!("orbit_of_1_size={}", orbit_of_1.len());
    println!("stabilizer_of_1_order={}", stabilizer_of_1.len());
    println!("normalization_factor={}", orbit_of_1.len());

    // 8. Conditional row-sum constraints from column-preserving stabilizers.
    // Multiplication by a unit is always an equivalence action on valid pairs.
    // The equalities below are only forced under the extra hypothesis that the
    // pair is invariant under the named subgroup.
    println!();
    println!("# Conditional row-sum constraints from a column-preserving stabilizer");
    println!("# If a pair is invariant under all t with t ≡ 1 mod 37, then");
    println!("# R_A(u) = R_A(t*u mod 9), and similarly for B.");
    println!("# This is a stabilizer hypothesis, not an unconditional LP(333) theorem.");
    for orbit in &row_orbits_under_col_preserving {
        let mut sorted = orbit.clone();
        sorted.sort_unstable();
        println!(
            "conditional_row_sum_equivalence_class={}",
            sorted
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    let free_row_sum_parameters = row_orbits_under_col_preserving.len();
    println!("conditional_free_row_sum_parameters={free_row_sum_parameters}");
    println!("# (down from 9 independent row sums under this stabilizer hypothesis)");

    // 9. Conditional column-sum constraints from row-preserving stabilizers.
    println!();
    println!("# Conditional column-sum constraints from a row-preserving stabilizer");
    println!("# If a pair is invariant under all t with t ≡ 1 mod 9, then");
    println!("# C_A(v) = C_A(t*v mod 37), and similarly for B.");
    let free_col_sum_parameters = col_orbits_under_row_preserving.len();
    println!("conditional_free_column_sum_parameters={free_col_sum_parameters}");
    println!("# (down from 37 independent column sums under this stabilizer hypothesis)");

    // 10. Bundle-orbit interaction with multiplier stabilizer hypotheses.
    // The column-preserving multipliers (order 6) permute rows of the 9x37 grid.
    // The mod-3 bundle sums T_A(j) = R_A(j) + R_A(j+3) + R_A(j+6) are determined
    // by the row sums. Under an invariance hypothesis, the multiplier action on
    // rows also constrains the bundle sums.
    println!();
    println!("# Bundle-level stabilizer-hypothesis constraints");
    // Column-preserving multipliers act on Z_9 by multiplication.
    // Bundle class j = {j, j+3, j+6} for j in {0,1,2}.
    // Under t ∈ U(9): j mod 3 → (t*j) mod 3.
    // If t ≡ 1 mod 3, bundle classes are preserved.
    // If t ≡ 2 mod 3, bundle classes 1↔2 are swapped.
    let col_pres_mod3: Vec<u32> = col_preserving.iter().map(|t| t % 3).collect();
    let col_pres_preserving = col_preserving.iter().filter(|t| *t % 3 == 1).count();
    let col_pres_swapping = col_preserving.iter().filter(|t| *t % 3 == 2).count();
    println!("column_preserving_bundle_preserving_count={col_pres_preserving}");
    println!("column_preserving_bundle_swapping_count={col_pres_swapping}");
    println!(
        "column_preserving_mod3_actions={}",
        col_pres_mod3
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );

    let row_bundle_pair_masses = row_mod3_bundle_pair_masses();
    let col_preserving_row_units = sorted_residues(&col_preserving, p);
    let col_preserving_allowed_bundles =
        row_bundle_triples_for_row_units(&col_preserving_row_units);
    let (col_preserving_pairs, col_preserving_mass) =
        row_bundle_pair_survival(&row_bundle_pair_masses, &col_preserving_allowed_bundles);

    // If a stabilizer contains a multiplier that swaps bundle classes 1 and 2,
    // then the invariant bundle sums satisfy T_A(1) = T_A(2).
    if col_pres_swapping > 0 {
        println!("conditional_bundle_sum_constraint=T_A(1)=T_A(2)");
        println!(
            "# Under the full column-preserving stabilizer hypothesis, the length-3 bundle vector must be invariant under the induced row action."
        );
        println!(
            "full_column_preserving_allowed_bundle_count={}",
            col_preserving_allowed_bundles.len()
        );
        println!("full_column_preserving_ordered_bundle_pairs={col_preserving_pairs}");
        println!("full_column_preserving_norm_refined_mass={col_preserving_mass}");
        println!(
            "full_column_preserving_survives_row_bundle_sieve={}",
            col_preserving_pairs > 0
        );
        println!("# (compare to 504 ordered pairs with total mass 5035801219344)");
    } else {
        println!("conditional_bundle_sum_constraint=none for this subgroup");
    }

    // 11. Cyclic subgroup stabilizer hypotheses.
    println!();
    println!("# Cyclic multiplier stabilizer hypotheses");
    println!("# Counts below assume A and B are invariant under the generated cyclic subgroup.");
    println!("# They are a way to prioritize or reject symmetry assumptions, not a proof that every LP(333) has such symmetry.");
    let cyclic_hypotheses = cyclic_multiplier_hypotheses(&units, n, &row_bundle_pair_masses);
    let nontrivial_row_action_hypotheses = cyclic_hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.row_units.as_slice() != [1])
        .cloned()
        .collect::<Vec<_>>();
    println!(
        "cyclic_multiplier_subgroup_count={}",
        cyclic_hypotheses.len()
    );
    println!(
        "cyclic_multiplier_subgroup_order_distribution={}",
        format_size_dist(&cyclic_hypothesis_order_distribution(&cyclic_hypotheses))
    );
    println!(
        "cyclic_invariant_hypotheses_with_row_bundle_pairs={}",
        cyclic_hypotheses
            .iter()
            .filter(|hypothesis| hypothesis.surviving_pair_count > 0)
            .count()
    );
    println!(
        "cyclic_invariant_row_bundle_survival_by_order={}",
        cyclic_hypothesis_survival_by_order(&cyclic_hypotheses).join(";")
    );
    println!(
        "top_cyclic_invariant_row_bundle_hypotheses={}",
        top_cyclic_hypothesis_summaries(&cyclic_hypotheses, 8).join(";")
    );
    println!(
        "cyclic_nontrivial_row_action_subgroup_count={}",
        nontrivial_row_action_hypotheses.len()
    );
    println!(
        "cyclic_nontrivial_row_action_hypotheses_with_row_bundle_pairs={}",
        nontrivial_row_action_hypotheses
            .iter()
            .filter(|hypothesis| hypothesis.surviving_pair_count > 0)
            .count()
    );
    println!(
        "cyclic_nontrivial_row_action_survival_by_order={}",
        cyclic_hypothesis_survival_by_order(&nontrivial_row_action_hypotheses).join(";")
    );
    println!(
        "top_cyclic_nontrivial_row_action_hypotheses={}",
        top_cyclic_hypothesis_summaries(&nontrivial_row_action_hypotheses, 8).join(";")
    );
    println!(
        "top_cyclic_nontrivial_row_action_pair_samples={}",
        top_cyclic_hypothesis_pair_sample_summaries(&nontrivial_row_action_hypotheses, 5).join(";")
    );
    let row_units_147_lift =
        row_units_147_full_row_lift_analysis(&row_bundle_pair_masses, include_col10_shift1);
    println!(
        "row_units_147_full_row_lift_stats={}",
        row_units_147_full_row_lift_summary(&row_units_147_lift)
    );
    if !row_units_147_lift.top_exact_samples.is_empty() {
        println!(
            "row_units_147_full_row_lift_top_exact_samples={}",
            row_units_147_lift.top_exact_samples.join(";")
        );
    }
    if !row_units_147_lift
        .top_shift0_dot_feasible_samples
        .is_empty()
    {
        println!(
            "row_units_147_shift0_dot_marginal_top_samples={}",
            row_units_147_lift.top_shift0_dot_feasible_samples.join(";")
        );
    }
    if !row_units_147_lift.top_col10_fixed_rows_samples.is_empty() {
        println!(
            "row_units_147_col10_fixed_rows_top_samples={}",
            row_units_147_lift.top_col10_fixed_rows_samples.join(";")
        );
    }
    if !row_units_147_lift.top_col10_shift0_e3_samples.is_empty() {
        println!(
            "row_units_147_col10_shift0_e3_top_samples={}",
            row_units_147_lift.top_col10_shift0_e3_samples.join(";")
        );
    }
    if !row_units_147_lift.top_col10_shift1_samples.is_empty() {
        println!(
            "row_units_147_col10_shift1_top_samples={}",
            row_units_147_lift.top_col10_shift1_samples.join(";")
        );
    }

    // 12. Full multiplier orbit on the 42 dihedral-swap pair orbits
    // The multiplier action on bundles [a,b,c] at the mod-3 level:
    // - column-preserving with t mod 3 = 1: [a,b,c] → [a,b,c] (identity on bundles)
    //   but permutes entries WITHIN each bundle class
    // - column-preserving with t mod 3 = 2: [a,b,c] → [a,c,b] (swaps positions 1↔2)
    // This is exactly the reverse3 operation already used in the dihedral-swap canonicalization.
    println!();
    println!("# Multiplier action on bundle-level pair orbits");
    println!("# Column-preserving multipliers with t mod 3 = 2 act as reverse3 on bundles");
    println!("# This is already captured by the dihedral-swap canonicalization");
    if col_pres_swapping > 0 {
        println!("bundle_reversal_action_already_dihedral_quotiented=true");
        println!("# The 42 dihedral-swap pair orbits already account for this multiplier action");
        println!("# This is an equivalence quotient, separate from any stabilizer assumption");
    } else {
        println!("bundle_reversal_action_already_dihedral_quotiented=false");
    }

    // 13. Row-preserving multiplier action on within-row structure.
    // For t ≡ 1 mod 9, t acts on columns v → t*v mod 37.
    // Under a stabilizer hypothesis, each row is constant on column orbits.
    // The column orbits under the row-preserving subgroup tell us which
    // within-row coordinates would be coupled.
    println!();
    println!("# Within-row column orbit structure (row-preserving stabilizer hypothesis)");
    println!("# Each row of the 9x37 grid has 37 entries");
    println!("# Column 0 is always fixed (trivial orbit)");
    println!(
        "# The remaining 36 columns form {} orbits under v -> t*v mod 37",
        col_orbits_under_row_preserving.len()
    );
    for (i, orbit) in col_orbits_under_row_preserving.iter().enumerate() {
        let mut sorted = orbit.clone();
        sorted.sort_unstable();
        println!(
            "column_orbit_{i}_size={} elements={}",
            sorted.len(),
            sorted
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }

    // 13. Summary: effective search space reduction
    println!();
    println!("# Summary");
    println!("full_multiplier_group_order={}", units.len());
    println!("coordinate_orbits={}", coord_orbits.len());
    println!(
        "available_multiplier_equivalence_orbit_size={}x",
        orbit_of_1.len()
    );
    println!("conditional_row_sum_free_parameters={free_row_sum_parameters}");
    println!("conditional_column_sum_free_parameters={free_col_sum_parameters}");

    Ok(())
}

fn units_mod(n: u32) -> Vec<u32> {
    (1..n).filter(|t| gcd(*t, n) == 1).collect()
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn mult_order(g: u32, n: u32) -> u32 {
    let mut current = g % n;
    let mut order = 1u32;
    while current != 1 {
        current = (current as u64 * g as u64 % n as u64) as u32;
        order += 1;
    }
    order
}

fn primitive_root_mod_prime(p: u32) -> u32 {
    let phi = p - 1;
    let factors = prime_factors(phi);
    'outer: for g in 2..p {
        for &f in &factors {
            let exp = phi / f;
            if mod_pow(g, exp, p) == 1 {
                continue 'outer;
            }
        }
        return g;
    }
    panic!("no primitive root found for prime {p}");
}

fn primitive_root_mod_prime_power(p: u32, k: u32) -> u32 {
    // For p odd prime, p^k: a primitive root mod p lifts to a primitive root mod p^k
    // if g^(p-1) ≢ 1 mod p^2. If it does, use g+p instead.
    let pk = p.pow(k);
    let g = primitive_root_mod_prime(p);
    if mod_pow(g, p - 1, p * p) != 1 {
        debug_assert_eq!(mult_order(g, pk), euler_phi(pk));
        g
    } else {
        let g2 = g + p;
        debug_assert_eq!(mult_order(g2, pk), euler_phi(pk));
        g2
    }
}

fn euler_phi(n: u32) -> u32 {
    let mut result = n;
    let mut m = n;
    let mut p = 2u32;
    while p * p <= m {
        if m % p == 0 {
            while m % p == 0 {
                m /= p;
            }
            result -= result / p;
        }
        p += 1;
    }
    if m > 1 {
        result -= result / m;
    }
    result
}

fn prime_factors(mut n: u32) -> Vec<u32> {
    let mut factors = Vec::new();
    let mut d = 2u32;
    while d * d <= n {
        if n % d == 0 {
            factors.push(d);
            while n % d == 0 {
                n /= d;
            }
        }
        d += 1;
    }
    if n > 1 {
        factors.push(n);
    }
    factors
}

fn mod_pow(base: u32, mut exp: u32, modulus: u32) -> u32 {
    let mut result = 1u64;
    let mut b = base as u64;
    let m = modulus as u64;
    while exp > 0 {
        if exp & 1 == 1 {
            result = result * b % m;
        }
        b = b * b % m;
        exp >>= 1;
    }
    result as u32
}

fn multiplier_orbits(n: u32, units: &[u32]) -> Vec<Vec<u32>> {
    let mut seen = vec![false; n as usize];
    seen[0] = true; // 0 is always in its own trivial orbit, exclude it
    let mut orbits = Vec::new();
    for start in 1..n {
        if seen[start as usize] {
            continue;
        }
        let mut orbit = Vec::new();
        let current = start;
        loop {
            if seen[current as usize] {
                break;
            }
            seen[current as usize] = true;
            orbit.push(current);
            // Apply all multipliers to current and add to queue
            for &t in units {
                let next = (t as u64 * current as u64 % n as u64) as u32;
                if !seen[next as usize] {
                    // BFS-style: we need the full orbit closure
                }
            }
            // Actually, just compute the orbit of `start` directly
            break;
        }
        // Recompute properly: orbit of `start` under the group action
        seen[start as usize] = false; // reset
        for &s in &orbit {
            seen[s as usize] = false;
        }
        let mut orbit = BTreeSet::new();
        orbit.insert(start);
        let mut frontier = vec![start];
        while let Some(elem) = frontier.pop() {
            for &t in units {
                let next = (t as u64 * elem as u64 % n as u64) as u32;
                if orbit.insert(next) {
                    frontier.push(next);
                }
            }
        }
        for &elem in &orbit {
            seen[elem as usize] = true;
        }
        orbits.push(orbit.into_iter().collect());
    }
    orbits
}

fn size_distribution(orbits: &[Vec<u32>]) -> BTreeMap<usize, usize> {
    let mut dist = BTreeMap::new();
    for orbit in orbits {
        *dist.entry(orbit.len()).or_default() += 1;
    }
    dist
}

fn format_size_dist(dist: &BTreeMap<usize, usize>) -> String {
    dist.iter()
        .map(|(size, count)| format!("{size}:{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn format_u32_list(items: &[u32]) -> String {
    items
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Clone, Debug)]
struct RowBundlePairMass {
    left: [i32; 3],
    right: [i32; 3],
    mass: u128,
}

#[derive(Clone, Debug)]
struct CyclicMultiplierHypothesis {
    subgroup: Vec<u32>,
    order: usize,
    row_units: Vec<u32>,
    column_units: Vec<u32>,
    mod3_actions: Vec<u32>,
    row_orbit_signature: String,
    allowed_bundle_count: usize,
    surviving_pair_count: u128,
    surviving_mass: u128,
    top_pair_samples: Vec<String>,
}

#[derive(Clone, Debug)]
struct RowUnits147Marginal {
    rows: [i32; 9],
    norm: i32,
    paf: [i32; 9],
    shift0_e1_dot_sums: BTreeSet<i32>,
    shift0_e3_dot_sums: BTreeSet<i32>,
    column_trivial_pattern_log10: f64,
    col10_fixed_rows_pattern_log10: Option<f64>,
    col10_shift0_e3_dot_sums: Option<BTreeSet<i32>>,
    col10_shift1_dot_sums: Option<BTreeSet<i32>>,
}

#[derive(Clone, Debug)]
struct RowUnits147FullRowLiftAnalysis {
    bundle_pair_count: usize,
    active_bundle_count: usize,
    active_row_marginal_count: usize,
    row_pair_candidate_count: u128,
    norm_compatible_row_pair_count: u128,
    exact_row_pair_count: u128,
    shift0_dot_marginal_feasible_row_pair_count: u128,
    col10_fixed_rows_exact_row_pair_count: u128,
    col10_shift0_e3_feasible_row_pair_count: u128,
    col10_shift1_enabled: bool,
    col10_shift1_feasible_row_pair_count: u128,
    exact_column_trivial_pattern_log10: Option<f64>,
    col10_fixed_rows_pattern_log10: Option<f64>,
    col10_shift0_e3_pattern_log10: Option<f64>,
    col10_shift1_pattern_log10: Option<f64>,
    top_exact_samples: Vec<String>,
    top_shift0_dot_feasible_samples: Vec<String>,
    top_col10_fixed_rows_samples: Vec<String>,
    top_col10_shift0_e3_samples: Vec<String>,
    top_col10_shift1_samples: Vec<String>,
}

fn cyclic_multiplier_hypotheses(
    units: &[u32],
    modulus: u32,
    pair_masses: &[RowBundlePairMass],
) -> Vec<CyclicMultiplierHypothesis> {
    let mut allowed_bundle_cache = BTreeMap::<Vec<u32>, BTreeSet<[i32; 3]>>::new();
    let mut hypotheses = Vec::new();
    for subgroup in cyclic_subgroups_mod(modulus, units) {
        let row_units = sorted_residues(&subgroup, 9);
        let column_units = sorted_residues(&subgroup, 37);
        let mod3_actions = sorted_residues(&subgroup, 3);
        let row_orbits = action_orbits_mod(9, &row_units, true);
        let row_orbit_signature = format_orbits(&row_orbits);
        let allowed_bundles = allowed_bundle_cache
            .entry(row_units.clone())
            .or_insert_with(|| row_bundle_triples_for_row_units(&row_units));
        let (surviving_pair_count, surviving_mass, top_pair_samples) =
            row_bundle_pair_survival_with_samples(pair_masses, allowed_bundles, 3);
        hypotheses.push(CyclicMultiplierHypothesis {
            order: subgroup.len(),
            subgroup,
            row_units,
            column_units,
            mod3_actions,
            row_orbit_signature,
            allowed_bundle_count: allowed_bundles.len(),
            surviving_pair_count,
            surviving_mass,
            top_pair_samples,
        });
    }
    hypotheses.sort_unstable_by(|a, b| {
        a.order
            .cmp(&b.order)
            .then_with(|| a.subgroup.cmp(&b.subgroup))
    });
    hypotheses
}

fn cyclic_subgroups_mod(modulus: u32, units: &[u32]) -> Vec<Vec<u32>> {
    let mut subgroups = BTreeSet::<Vec<u32>>::new();
    for generator in units {
        let subgroup = generated_subgroup_mod(*generator, modulus);
        if subgroup.len() > 1 {
            subgroups.insert(subgroup);
        }
    }
    subgroups.into_iter().collect()
}

fn generated_subgroup_mod(generator: u32, modulus: u32) -> Vec<u32> {
    let mut subgroup = BTreeSet::new();
    let mut current = 1u32;
    loop {
        if !subgroup.insert(current) {
            break;
        }
        current = (current as u64 * generator as u64 % modulus as u64) as u32;
    }
    subgroup.into_iter().collect()
}

fn sorted_residues(values: &[u32], modulus: u32) -> Vec<u32> {
    values
        .iter()
        .map(|value| value % modulus)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn action_orbits_mod(modulus: u32, multipliers: &[u32], include_zero: bool) -> Vec<Vec<u32>> {
    let mut seen = vec![false; modulus as usize];
    let starts = if include_zero { 0 } else { 1 };
    let mut orbits = Vec::new();
    for start in starts..modulus {
        if seen[start as usize] {
            continue;
        }
        let mut orbit = BTreeSet::new();
        orbit.insert(start);
        let mut frontier = vec![start];
        while let Some(elem) = frontier.pop() {
            for multiplier in multipliers {
                let next = (*multiplier as u64 * elem as u64 % modulus as u64) as u32;
                if orbit.insert(next) {
                    frontier.push(next);
                }
            }
        }
        for elem in &orbit {
            seen[*elem as usize] = true;
        }
        orbits.push(orbit.into_iter().collect());
    }
    orbits.sort_unstable();
    orbits
}

fn format_orbits(orbits: &[Vec<u32>]) -> String {
    orbits
        .iter()
        .map(|orbit| {
            orbit
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn row_bundle_triples_for_row_units(row_units: &[u32]) -> BTreeSet<[i32; 3]> {
    if row_units == [1] {
        return all_row_bundle_sum_one_triples();
    }
    let row_orbits = action_orbits_mod(9, row_units, true);
    let alphabet = odd_alphabet(37);
    let mut states = BTreeSet::from([[0i32; 3]]);
    for orbit in row_orbits {
        let mut residue_counts = [0i32; 3];
        for row in orbit {
            residue_counts[(row % 3) as usize] += 1;
        }
        let mut next = BTreeSet::new();
        for state in &states {
            for value in &alphabet {
                let candidate = [
                    state[0] + residue_counts[0] * value,
                    state[1] + residue_counts[1] * value,
                    state[2] + residue_counts[2] * value,
                ];
                if candidate.iter().all(|entry| (-111..=111).contains(entry)) {
                    next.insert(candidate);
                }
            }
        }
        states = next;
    }
    states
        .into_iter()
        .filter(|bundle| bundle.iter().sum::<i32>() == 1)
        .collect()
}

fn all_row_bundle_sum_one_triples() -> BTreeSet<[i32; 3]> {
    let alphabet = odd_alphabet(111);
    let membership = alphabet.iter().copied().collect::<BTreeSet<_>>();
    let mut triples = BTreeSet::new();
    for left in &alphabet {
        for middle in &alphabet {
            let right = 1 - left - middle;
            if membership.contains(&right) {
                triples.insert([*left, *middle, right]);
            }
        }
    }
    triples
}

fn row_mod3_bundle_pair_masses() -> Vec<RowBundlePairMass> {
    let alphabet = odd_alphabet(111);
    let membership = alphabet.iter().copied().collect::<BTreeSet<_>>();
    let triple_lift_counts = sequence_counts_by_sum_and_norm(3, &odd_alphabet(37));
    let mut bundles_by_paf = BTreeMap::<i32, Vec<[i32; 3]>>::new();
    for left in &alphabet {
        for middle in &alphabet {
            let right = 1 - left - middle;
            if !membership.contains(&right) {
                continue;
            }
            let bundle = [*left, *middle, right];
            let paf = left * middle + middle * right + right * left;
            bundles_by_paf.entry(paf).or_default().push(bundle);
        }
    }
    let mut norm_cache = HashMap::<[i32; 3], BTreeMap<i32, u128>>::new();
    let mut pairs = Vec::new();
    for (left_paf, left_bundles) in &bundles_by_paf {
        let Some(right_bundles) = bundles_by_paf.get(&(-222 - *left_paf)) else {
            continue;
        };
        for left in left_bundles {
            let left_norm_counts = norm_cache
                .entry(*left)
                .or_insert_with(|| convolve_bundle_component_norms(*left, &triple_lift_counts))
                .clone();
            for right in right_bundles {
                let right_norm_counts = norm_cache
                    .entry(*right)
                    .or_insert_with(|| convolve_bundle_component_norms(*right, &triple_lift_counts))
                    .clone();
                let mass =
                    compatible_row_norm_pair_count(&left_norm_counts, &right_norm_counts, 594);
                if mass > 0 {
                    pairs.push(RowBundlePairMass {
                        left: *left,
                        right: *right,
                        mass,
                    });
                }
            }
        }
    }
    pairs
}

fn row_bundle_pair_survival(
    pairs: &[RowBundlePairMass],
    allowed_bundles: &BTreeSet<[i32; 3]>,
) -> (u128, u128) {
    let (pair_count, mass, _) = row_bundle_pair_survival_with_samples(pairs, allowed_bundles, 0);
    (pair_count, mass)
}

fn row_bundle_pair_survival_with_samples(
    pairs: &[RowBundlePairMass],
    allowed_bundles: &BTreeSet<[i32; 3]>,
    sample_count: usize,
) -> (u128, u128, Vec<String>) {
    let mut pair_count = 0u128;
    let mut mass = 0u128;
    let mut samples = Vec::<(u128, [i32; 3], [i32; 3])>::new();
    for pair in pairs {
        if allowed_bundles.contains(&pair.left) && allowed_bundles.contains(&pair.right) {
            pair_count += 1;
            mass += pair.mass;
            if sample_count > 0 {
                samples.push((pair.mass, pair.left, pair.right));
            }
        }
    }
    samples.sort_unstable_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    samples.truncate(sample_count);
    let samples = samples
        .into_iter()
        .map(|(pair_mass, left, right)| {
            format!(
                "{}:{}/{}",
                pair_mass,
                format_coefficient(left),
                format_coefficient(right)
            )
        })
        .collect();
    (pair_count, mass, samples)
}

fn cyclic_hypothesis_order_distribution(
    hypotheses: &[CyclicMultiplierHypothesis],
) -> BTreeMap<usize, usize> {
    let mut distribution = BTreeMap::new();
    for hypothesis in hypotheses {
        *distribution.entry(hypothesis.order).or_default() += 1;
    }
    distribution
}

fn cyclic_hypothesis_survival_by_order(hypotheses: &[CyclicMultiplierHypothesis]) -> Vec<String> {
    let mut by_order = BTreeMap::<usize, (usize, usize, u128, u128)>::new();
    for hypothesis in hypotheses {
        let entry = by_order.entry(hypothesis.order).or_default();
        entry.0 += 1;
        if hypothesis.surviving_pair_count > 0 {
            entry.1 += 1;
        }
        entry.2 = entry.2.max(hypothesis.surviving_pair_count);
        entry.3 = entry.3.max(hypothesis.surviving_mass);
    }
    by_order
        .into_iter()
        .map(|(order, (total, nonzero, max_pairs, max_mass))| {
            format!(
                "order{order}:subgroups={total},nonzero={nonzero},max_pairs={max_pairs},max_mass={max_mass}"
            )
        })
        .collect()
}

fn top_cyclic_hypothesis_summaries(
    hypotheses: &[CyclicMultiplierHypothesis],
    limit: usize,
) -> Vec<String> {
    let mut ranked = hypotheses.iter().collect::<Vec<_>>();
    ranked.sort_unstable_by(|a, b| {
        b.surviving_mass
            .cmp(&a.surviving_mass)
            .then_with(|| b.surviving_pair_count.cmp(&a.surviving_pair_count))
            .then_with(|| a.order.cmp(&b.order))
            .then_with(|| a.subgroup.cmp(&b.subgroup))
    });
    ranked
        .into_iter()
        .take(limit)
        .map(|hypothesis| {
            format!(
                "order={},subgroup={},row_units={},col_units={},mod3={},row_orbits={},allowed_bundles={},pairs={},mass={}",
                hypothesis.order,
                format_u32_list(&hypothesis.subgroup),
                format_u32_list(&hypothesis.row_units),
                format_u32_list(&hypothesis.column_units),
                format_u32_list(&hypothesis.mod3_actions),
                hypothesis.row_orbit_signature,
                hypothesis.allowed_bundle_count,
                hypothesis.surviving_pair_count,
                hypothesis.surviving_mass
            )
        })
        .collect()
}

fn top_cyclic_hypothesis_pair_sample_summaries(
    hypotheses: &[CyclicMultiplierHypothesis],
    limit: usize,
) -> Vec<String> {
    let mut ranked = hypotheses
        .iter()
        .filter(|hypothesis| hypothesis.surviving_pair_count > 0)
        .collect::<Vec<_>>();
    ranked.sort_unstable_by(|a, b| {
        b.surviving_mass
            .cmp(&a.surviving_mass)
            .then_with(|| b.surviving_pair_count.cmp(&a.surviving_pair_count))
            .then_with(|| a.order.cmp(&b.order))
            .then_with(|| a.subgroup.cmp(&b.subgroup))
    });
    ranked
        .into_iter()
        .take(limit)
        .map(|hypothesis| {
            format!(
                "order={},subgroup={},row_units={},pairs={}",
                hypothesis.order,
                format_u32_list(&hypothesis.subgroup),
                format_u32_list(&hypothesis.row_units),
                hypothesis.top_pair_samples.join(",")
            )
        })
        .collect()
}

fn row_units_147_full_row_lift_analysis(
    pair_masses: &[RowBundlePairMass],
    include_col10_shift1: bool,
) -> RowUnits147FullRowLiftAnalysis {
    let allowed_bundles = row_bundle_triples_for_row_units(&[1, 4, 7]);
    let bundle_pairs = pair_masses
        .iter()
        .filter(|pair| {
            allowed_bundles.contains(&pair.left) && allowed_bundles.contains(&pair.right)
        })
        .collect::<Vec<_>>();
    let active_bundles = bundle_pairs
        .iter()
        .flat_map(|pair| [pair.left, pair.right])
        .collect::<BTreeSet<_>>();
    let marginal_cache = active_bundles
        .iter()
        .map(|bundle| {
            (
                *bundle,
                row_units_147_marginals_for_bundle(*bundle, include_col10_shift1),
            )
        })
        .collect::<HashMap<_, _>>();
    let active_row_marginal_count = marginal_cache.values().map(Vec::len).sum();

    let mut row_pair_candidate_count = 0u128;
    let mut norm_compatible_row_pair_count = 0u128;
    let mut exact_row_pair_count = 0u128;
    let mut shift0_dot_marginal_feasible_row_pair_count = 0u128;
    let mut col10_fixed_rows_exact_row_pair_count = 0u128;
    let mut col10_shift0_e3_feasible_row_pair_count = 0u128;
    let mut col10_shift1_feasible_row_pair_count = 0u128;
    let mut exact_column_trivial_pattern_log10 = None;
    let mut col10_fixed_rows_pattern_log10 = None;
    let mut col10_shift0_e3_pattern_log10 = None;
    let mut col10_shift1_pattern_log10 = None;
    let mut top_samples = Vec::<(f64, [i32; 9], [i32; 9])>::new();
    let mut top_shift0_dot_feasible_samples = Vec::<(f64, [i32; 9], [i32; 9])>::new();
    let mut top_col10_fixed_rows_samples = Vec::<(f64, [i32; 9], [i32; 9])>::new();
    let mut top_col10_shift0_e3_samples = Vec::<(f64, [i32; 9], [i32; 9])>::new();
    let mut top_col10_shift1_samples = Vec::<(f64, [i32; 9], [i32; 9])>::new();
    for pair in &bundle_pairs {
        let left_marginals = marginal_cache
            .get(&pair.left)
            .expect("active left bundle must have marginals");
        let right_marginals = marginal_cache
            .get(&pair.right)
            .expect("active right bundle must have marginals");
        row_pair_candidate_count += left_marginals.len() as u128 * right_marginals.len() as u128;
        for left in left_marginals {
            for right in right_marginals {
                if left.norm + right.norm != LP333_ROW_NORM_TARGET {
                    continue;
                }
                norm_compatible_row_pair_count += 1;
                if !row_paf_pair_is_exact(&left.paf, &right.paf) {
                    continue;
                }
                exact_row_pair_count += 1;
                let pattern_log10 =
                    left.column_trivial_pattern_log10 + right.column_trivial_pattern_log10;
                exact_column_trivial_pattern_log10 = Some(log10_sum_optional(
                    exact_column_trivial_pattern_log10,
                    pattern_log10,
                ));
                if row_units_147_shift0_dot_marginal_feasible(left, right) {
                    shift0_dot_marginal_feasible_row_pair_count += 1;
                    push_top_row_units_147_sample(
                        &mut top_shift0_dot_feasible_samples,
                        pattern_log10,
                        left.rows,
                        right.rows,
                        5,
                    );
                }
                if let (Some(left_col10_log10), Some(right_col10_log10)) = (
                    left.col10_fixed_rows_pattern_log10,
                    right.col10_fixed_rows_pattern_log10,
                ) {
                    col10_fixed_rows_exact_row_pair_count += 1;
                    let col10_pattern_log10 = left_col10_log10 + right_col10_log10;
                    col10_fixed_rows_pattern_log10 = Some(log10_sum_optional(
                        col10_fixed_rows_pattern_log10,
                        col10_pattern_log10,
                    ));
                    push_top_row_units_147_sample(
                        &mut top_col10_fixed_rows_samples,
                        col10_pattern_log10,
                        left.rows,
                        right.rows,
                        5,
                    );
                    if let (Some(left_e3_sums), Some(right_e3_sums)) = (
                        &left.col10_shift0_e3_dot_sums,
                        &right.col10_shift0_e3_dot_sums,
                    ) {
                        if sumset_has_target(left_e3_sums, right_e3_sums, LP333_ACTUAL_SHIFT_TARGET)
                        {
                            col10_shift0_e3_feasible_row_pair_count += 1;
                            col10_shift0_e3_pattern_log10 = Some(log10_sum_optional(
                                col10_shift0_e3_pattern_log10,
                                col10_pattern_log10,
                            ));
                            push_top_row_units_147_sample(
                                &mut top_col10_shift0_e3_samples,
                                col10_pattern_log10,
                                left.rows,
                                right.rows,
                                5,
                            );
                        }
                    }
                    if let (Some(left_shift1_sums), Some(right_shift1_sums)) =
                        (&left.col10_shift1_dot_sums, &right.col10_shift1_dot_sums)
                    {
                        if sumset_has_target(
                            left_shift1_sums,
                            right_shift1_sums,
                            LP333_ACTUAL_SHIFT_TARGET,
                        ) {
                            col10_shift1_feasible_row_pair_count += 1;
                            col10_shift1_pattern_log10 = Some(log10_sum_optional(
                                col10_shift1_pattern_log10,
                                col10_pattern_log10,
                            ));
                            push_top_row_units_147_sample(
                                &mut top_col10_shift1_samples,
                                col10_pattern_log10,
                                left.rows,
                                right.rows,
                                5,
                            );
                        }
                    }
                }
                push_top_row_units_147_sample(
                    &mut top_samples,
                    pattern_log10,
                    left.rows,
                    right.rows,
                    5,
                );
            }
        }
    }

    RowUnits147FullRowLiftAnalysis {
        bundle_pair_count: bundle_pairs.len(),
        active_bundle_count: active_bundles.len(),
        active_row_marginal_count,
        row_pair_candidate_count,
        norm_compatible_row_pair_count,
        exact_row_pair_count,
        shift0_dot_marginal_feasible_row_pair_count,
        col10_fixed_rows_exact_row_pair_count,
        col10_shift0_e3_feasible_row_pair_count,
        col10_shift1_enabled: include_col10_shift1,
        col10_shift1_feasible_row_pair_count,
        exact_column_trivial_pattern_log10,
        col10_fixed_rows_pattern_log10,
        col10_shift0_e3_pattern_log10,
        col10_shift1_pattern_log10,
        top_exact_samples: top_samples
            .into_iter()
            .map(|(log10_count, left, right)| {
                format!(
                    "log10_count={:.3}:{}|{}",
                    log10_count,
                    format_i32_list(&left),
                    format_i32_list(&right)
                )
            })
            .collect(),
        top_shift0_dot_feasible_samples: top_shift0_dot_feasible_samples
            .into_iter()
            .map(|(log10_count, left, right)| {
                format!(
                    "log10_count={:.3}:{}|{}",
                    log10_count,
                    format_i32_list(&left),
                    format_i32_list(&right)
                )
            })
            .collect(),
        top_col10_fixed_rows_samples: top_col10_fixed_rows_samples
            .into_iter()
            .map(|(log10_count, left, right)| {
                format!(
                    "log10_count={:.3}:{}|{}",
                    log10_count,
                    format_i32_list(&left),
                    format_i32_list(&right)
                )
            })
            .collect(),
        top_col10_shift0_e3_samples: top_col10_shift0_e3_samples
            .into_iter()
            .map(|(log10_count, left, right)| {
                format!(
                    "log10_count={:.3}:{}|{}",
                    log10_count,
                    format_i32_list(&left),
                    format_i32_list(&right)
                )
            })
            .collect(),
        top_col10_shift1_samples: top_col10_shift1_samples
            .into_iter()
            .map(|(log10_count, left, right)| {
                format!(
                    "log10_count={:.3}:{}|{}",
                    log10_count,
                    format_i32_list(&left),
                    format_i32_list(&right)
                )
            })
            .collect(),
    }
}

fn row_units_147_marginals_for_bundle(
    bundle: [i32; 3],
    include_col10_shift1: bool,
) -> Vec<RowUnits147Marginal> {
    if bundle[1] % 3 != 0 || bundle[2] % 3 != 0 {
        return Vec::new();
    }
    let row_147 = bundle[1] / 3;
    let row_258 = bundle[2] / 3;
    if !is_valid_row_sum(row_147) || !is_valid_row_sum(row_258) {
        return Vec::new();
    }

    let alphabet = odd_alphabet(37);
    let repeated_pattern_log10 = row_pattern_log10(row_147) + row_pattern_log10(row_258);
    let mut marginals = Vec::new();
    for row_0 in &alphabet {
        for row_3 in &alphabet {
            let row_6 = bundle[0] - row_0 - row_3;
            if !is_valid_row_sum(row_6) {
                continue;
            }
            let col10_fixed_rows_pattern_log10 = col10_fixed_row_pattern_log10(*row_0)
                .and_then(|row_0_log10| {
                    col10_fixed_row_pattern_log10(*row_3)
                        .map(|row_3_log10| row_0_log10 + row_3_log10)
                })
                .and_then(|prefix_log10| {
                    col10_fixed_row_pattern_log10(row_6)
                        .map(|row_6_log10| prefix_log10 + row_6_log10 + repeated_pattern_log10)
                });
            let rows = [
                *row_0, row_147, row_258, *row_3, row_147, row_258, row_6, row_147, row_258,
            ];
            let col10_shift0_e3_dot_sums = row_units_147_col10_shift0_e3_dot_sums(rows);
            let col10_shift1_dot_sums = include_col10_shift1
                .then(|| row_units_147_col10_shift1_dot_sums(rows))
                .flatten();
            let column_trivial_pattern_log10 = row_pattern_log10(*row_0)
                + repeated_pattern_log10
                + row_pattern_log10(*row_3)
                + row_pattern_log10(row_6);
            marginals.push(RowUnits147Marginal {
                rows,
                norm: dot9(rows, rows),
                paf: paf9(rows),
                shift0_e1_dot_sums: row_units_147_e1_dot_sums(rows),
                shift0_e3_dot_sums: row_units_147_e3_dot_sums(rows),
                column_trivial_pattern_log10,
                col10_fixed_rows_pattern_log10,
                col10_shift0_e3_dot_sums,
                col10_shift1_dot_sums,
            });
        }
    }
    marginals
}

fn is_valid_row_sum(value: i32) -> bool {
    (-37..=37).contains(&value) && value % 2 != 0
}

fn row_pattern_count(row_sum: i32) -> u128 {
    debug_assert!(is_valid_row_sum(row_sum));
    let plus_count = ((37 + row_sum) / 2) as u32;
    binomial_u128(37, plus_count)
}

fn row_pattern_log10(row_sum: i32) -> f64 {
    (row_pattern_count(row_sum) as f64).log10()
}

fn col10_fixed_row_pattern_count(row_sum: i32) -> u128 {
    // A row fixed by column multiplication by 10 has column 0 free and 12
    // length-3 orbits among the nonzero columns of Z_37.
    [-1, 1]
        .into_iter()
        .filter_map(|fixed_value| {
            let residual = row_sum - fixed_value;
            if residual % 6 != 0 {
                return None;
            }
            let plus_orbit_count = (residual + 36) / 6;
            if !(0..=12).contains(&plus_orbit_count) {
                return None;
            }
            Some(binomial_u128(12, plus_orbit_count as u32))
        })
        .sum()
}

fn col10_fixed_row_pattern_log10(row_sum: i32) -> Option<f64> {
    let count = col10_fixed_row_pattern_count(row_sum);
    (count > 0).then(|| (count as f64).log10())
}

fn row_units_147_col10_shift0_e3_dot_sums(rows: [i32; 9]) -> Option<BTreeSet<i32>> {
    let fixed_row_dot_sums = col10_fixed_triple_dot_sum_values(rows[0], rows[3], rows[6]);
    let x_self_dot_sums = col10_self_dot_values(rows[1]);
    let y_self_dot_sums = col10_self_dot_values(rows[2]);
    if fixed_row_dot_sums.is_empty() || x_self_dot_sums.is_empty() || y_self_dot_sums.is_empty() {
        return None;
    }

    let mut sums = BTreeSet::new();
    for fixed_sum in &fixed_row_dot_sums {
        for x_sum in &x_self_dot_sums {
            for y_sum in &y_self_dot_sums {
                sums.insert(fixed_sum + 3 * x_sum + 3 * y_sum);
            }
        }
    }
    Some(sums)
}

fn row_units_147_col10_shift1_dot_sums(rows: [i32; 9]) -> Option<BTreeSet<i32>> {
    let fixed_0_shift1 = col10_fixed_shift1_values(rows[0]);
    let fixed_3_shift1 = col10_fixed_shift1_values(rows[3]);
    let fixed_6_shift1 = col10_fixed_shift1_values(rows[6]);
    let x_orbit_shift1 = col10_orbit_autocorr_values(rows[1]);
    let y_orbit_shift1 = col10_orbit_autocorr_values(rows[2]);
    if fixed_0_shift1.is_empty()
        || fixed_3_shift1.is_empty()
        || fixed_6_shift1.is_empty()
        || x_orbit_shift1.is_empty()
        || y_orbit_shift1.is_empty()
    {
        return None;
    }

    let mut sums = BTreeSet::new();
    for fixed_0 in &fixed_0_shift1 {
        for fixed_3 in &fixed_3_shift1 {
            for fixed_6 in &fixed_6_shift1 {
                for x_sum in &x_orbit_shift1 {
                    for y_sum in &y_orbit_shift1 {
                        sums.insert(fixed_0 + fixed_3 + fixed_6 + x_sum + y_sum);
                    }
                }
            }
        }
    }
    Some(sums)
}

fn col10_fixed_shift1_values(row_sum: i32) -> BTreeSet<i32> {
    static TABLE: OnceLock<HashMap<i32, BTreeSet<i32>>> = OnceLock::new();
    TABLE
        .get_or_init(build_col10_fixed_shift1_table)
        .get(&row_sum)
        .cloned()
        .unwrap_or_default()
}

fn build_col10_fixed_shift1_table() -> HashMap<i32, BTreeSet<i32>> {
    let orbits = col10_column_orbits();
    let mut table = HashMap::<i32, BTreeSet<i32>>::new();
    for mask in 0..(1usize << orbits.len()) {
        let mut values = [0i32; 37];
        for (orbit_index, orbit) in orbits.iter().enumerate() {
            let sign = if (mask >> orbit_index) & 1 == 1 {
                1
            } else {
                -1
            };
            for column in orbit {
                values[*column] = sign;
            }
        }
        let row_sum = values.iter().sum();
        let shift1_dot = (0..37)
            .map(|column| values[column] * values[(column + 1) % 37])
            .sum();
        table.entry(row_sum).or_default().insert(shift1_dot);
    }
    table
}

fn col10_orbit_autocorr_values(row_sum: i32) -> BTreeSet<i32> {
    static TABLE: OnceLock<HashMap<i32, BTreeSet<i32>>> = OnceLock::new();
    TABLE
        .get_or_init(build_col10_orbit_autocorr_table)
        .get(&row_sum)
        .cloned()
        .unwrap_or_default()
}

fn build_col10_orbit_autocorr_table() -> HashMap<i32, BTreeSet<i32>> {
    let orbits = col10_column_orbits();
    let orbit_lookup = col10_orbit_lookup(&orbits);
    let domains = col10_orbit_domains(&orbits);

    let order = [5usize, 12, 0, 6, 10, 4, 11, 7, 8, 1, 2, 3, 9];
    let order_index = order
        .iter()
        .enumerate()
        .map(|(index, variable)| (*variable, index))
        .collect::<HashMap<_, _>>();
    let mut active_variables = Vec::<usize>::new();
    let mut states =
        HashMap::<Vec<usize>, BTreeSet<(i32, i32)>>::from([(Vec::new(), BTreeSet::from([(0, 0)]))]);

    for (step, variable) in order.iter().copied().enumerate() {
        let future = order[step + 1..].iter().copied().collect::<BTreeSet<_>>();
        let next_active_variables = order[..=step]
            .iter()
            .copied()
            .filter(|processed| {
                col10_orbit_has_neighbor_in_future(&orbit_lookup, *processed, &future)
            })
            .collect::<Vec<_>>();
        let variable_loop_energy = (0..domains[variable].len())
            .map(|state_index| {
                col10_orbit_edge_energy(
                    &orbits,
                    &orbit_lookup,
                    &domains,
                    variable,
                    state_index,
                    variable,
                    state_index,
                )
            })
            .collect::<Vec<_>>();
        let mut next_states = HashMap::<Vec<usize>, BTreeSet<(i32, i32)>>::new();

        for (assignment, values) in &states {
            for state_index in 0..domains[variable].len() {
                let mut added_energy = variable_loop_energy[state_index];
                for (active_position, active_variable) in active_variables.iter().enumerate() {
                    if !col10_orbit_pair_has_edges(&orbit_lookup, *active_variable, variable) {
                        continue;
                    }
                    added_energy += col10_orbit_edge_energy(
                        &orbits,
                        &orbit_lookup,
                        &domains,
                        *active_variable,
                        assignment[active_position],
                        variable,
                        state_index,
                    );
                }
                let added_sum: i32 = domains[variable][state_index].iter().sum();
                let projected_assignment = next_active_variables
                    .iter()
                    .map(|active_variable| {
                        if *active_variable == variable {
                            state_index
                        } else {
                            let old_position = active_variables
                                .iter()
                                .position(|old_variable| old_variable == active_variable)
                                .expect("retained variable must have an old assignment");
                            assignment[old_position]
                        }
                    })
                    .collect::<Vec<_>>();
                let projected_values = next_states.entry(projected_assignment).or_default();
                for (row_sum, energy) in values {
                    projected_values.insert((row_sum + added_sum, energy + added_energy));
                }
            }
        }
        active_variables = next_active_variables;
        states = next_states;
    }

    debug_assert!(active_variables.is_empty());
    let mut table = HashMap::<i32, BTreeSet<i32>>::new();
    for values in states.values() {
        for (row_sum, energy) in values {
            table.entry(*row_sum).or_default().insert(*energy);
        }
    }
    debug_assert_eq!(table.get(&37), Some(&BTreeSet::from([111])));
    debug_assert_eq!(table.get(&-37), Some(&BTreeSet::from([111])));
    debug_assert_eq!(order_index.len(), 13);
    table
}

fn col10_column_orbits() -> Vec<Vec<usize>> {
    let mut seen = [false; 37];
    let mut orbits = Vec::new();
    for start in 0..37 {
        if seen[start] {
            continue;
        }
        let mut orbit = Vec::new();
        let mut column = start;
        while !seen[column] {
            seen[column] = true;
            orbit.push(column);
            column = (10 * column) % 37;
        }
        orbits.push(orbit);
    }
    orbits
}

fn col10_orbit_lookup(orbits: &[Vec<usize>]) -> [(usize, usize); 37] {
    let mut lookup = [(0usize, 0usize); 37];
    for (orbit_index, orbit) in orbits.iter().enumerate() {
        for (position, column) in orbit.iter().enumerate() {
            lookup[*column] = (orbit_index, position);
        }
    }
    lookup
}

fn col10_orbit_pair_has_edges(
    orbit_lookup: &[(usize, usize); 37],
    left_orbit: usize,
    right_orbit: usize,
) -> bool {
    [1usize, 10, 26].into_iter().any(|shift| {
        (0..37).any(|column| {
            let source_orbit = orbit_lookup[column].0;
            let target_orbit = orbit_lookup[(column + shift) % 37].0;
            (source_orbit == left_orbit && target_orbit == right_orbit)
                || (left_orbit != right_orbit
                    && source_orbit == right_orbit
                    && target_orbit == left_orbit)
        })
    })
}

fn col10_orbit_has_neighbor_in_future(
    orbit_lookup: &[(usize, usize); 37],
    orbit: usize,
    future: &BTreeSet<usize>,
) -> bool {
    future
        .iter()
        .any(|future_orbit| col10_orbit_pair_has_edges(orbit_lookup, orbit, *future_orbit))
}

fn col10_orbit_domains(orbits: &[Vec<usize>]) -> Vec<Vec<Vec<i32>>> {
    orbits
        .iter()
        .map(|orbit| {
            (0..(1usize << orbit.len()))
                .map(|mask| {
                    (0..orbit.len())
                        .map(|position| if (mask >> position) & 1 == 1 { 1 } else { -1 })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn col10_orbit_edge_energy(
    orbits: &[Vec<usize>],
    orbit_lookup: &[(usize, usize); 37],
    domains: &[Vec<Vec<i32>>],
    left_orbit: usize,
    left_state_index: usize,
    right_orbit: usize,
    right_state_index: usize,
) -> i32 {
    let mut energy = 0;
    for shift in [1usize, 10, 26] {
        for column in 0..37 {
            let (source_orbit, source_position) = orbit_lookup[column];
            let (target_orbit, target_position) = orbit_lookup[(column + shift) % 37];
            if source_orbit == left_orbit && target_orbit == right_orbit {
                energy += domains[left_orbit][left_state_index][source_position]
                    * domains[right_orbit][right_state_index][target_position];
            } else if left_orbit != right_orbit
                && source_orbit == right_orbit
                && target_orbit == left_orbit
            {
                energy += domains[right_orbit][right_state_index][source_position]
                    * domains[left_orbit][left_state_index][target_position];
            }
        }
    }
    debug_assert_eq!(energy % 1, 0);
    debug_assert!(left_orbit < orbits.len());
    debug_assert!(right_orbit < orbits.len());
    energy
}

fn col10_fixed_triple_dot_sum_values(
    row_0_sum: i32,
    row_3_sum: i32,
    row_6_sum: i32,
) -> BTreeSet<i32> {
    let mut values = BTreeSet::new();
    for fixed_0 in [-1, 1] {
        let Some(plus_0) = col10_fixed_nonzero_orbit_plus_count(row_0_sum, fixed_0) else {
            continue;
        };
        for fixed_3 in [-1, 1] {
            let Some(plus_3) = col10_fixed_nonzero_orbit_plus_count(row_3_sum, fixed_3) else {
                continue;
            };
            for fixed_6 in [-1, 1] {
                let Some(plus_6) = col10_fixed_nonzero_orbit_plus_count(row_6_sum, fixed_6) else {
                    continue;
                };
                let fixed_column_dot = fixed_0 * fixed_3 + fixed_3 * fixed_6 + fixed_6 * fixed_0;
                for orbit_dot in col10_fixed_triple_orbit_dot_values(plus_0, plus_3, plus_6) {
                    values.insert(fixed_column_dot + orbit_dot);
                }
            }
        }
    }
    values
}

fn col10_fixed_nonzero_orbit_plus_count(row_sum: i32, fixed_value: i32) -> Option<i32> {
    let residual = row_sum - fixed_value;
    if residual % 6 != 0 {
        return None;
    }
    let plus_orbit_count = (residual + 36) / 6;
    (0..=12)
        .contains(&plus_orbit_count)
        .then_some(plus_orbit_count)
}

fn col10_fixed_triple_orbit_dot_values(plus_0: i32, plus_3: i32, plus_6: i32) -> BTreeSet<i32> {
    static TABLE: OnceLock<HashMap<(i32, i32, i32), BTreeSet<i32>>> = OnceLock::new();
    TABLE
        .get_or_init(build_col10_fixed_triple_orbit_dot_table)
        .get(&(plus_0, plus_3, plus_6))
        .cloned()
        .unwrap_or_default()
}

fn build_col10_fixed_triple_orbit_dot_table() -> HashMap<(i32, i32, i32), BTreeSet<i32>> {
    let mut states = HashSet::from([(0i32, 0i32, 0i32, 0i32)]);
    for _ in 0..12 {
        let mut next = HashSet::new();
        for (plus_0, plus_3, plus_6, all_equal_count) in &states {
            for sign_0 in [-1, 1] {
                for sign_3 in [-1, 1] {
                    for sign_6 in [-1, 1] {
                        next.insert((
                            plus_0 + i32::from(sign_0 == 1),
                            plus_3 + i32::from(sign_3 == 1),
                            plus_6 + i32::from(sign_6 == 1),
                            all_equal_count + i32::from(sign_0 == sign_3 && sign_3 == sign_6),
                        ));
                    }
                }
            }
        }
        states = next;
    }

    let mut table = HashMap::<(i32, i32, i32), BTreeSet<i32>>::new();
    for (plus_0, plus_3, plus_6, all_equal_count) in states {
        let orbit_dot = 3 * (4 * all_equal_count - 12);
        table
            .entry((plus_0, plus_3, plus_6))
            .or_default()
            .insert(orbit_dot);
    }
    table
}

fn col10_self_dot_values(row_sum: i32) -> BTreeSet<i32> {
    static TABLE: OnceLock<HashMap<i32, BTreeSet<i32>>> = OnceLock::new();
    TABLE
        .get_or_init(build_col10_self_dot_table)
        .get(&row_sum)
        .cloned()
        .unwrap_or_default()
}

fn build_col10_self_dot_table() -> HashMap<i32, BTreeSet<i32>> {
    let mut states = HashSet::from([(-1i32, 1i32), (1, 1)]);
    for _ in 0..12 {
        let mut next = HashSet::new();
        for (row_sum, self_dot) in &states {
            for sign_a in [-1, 1] {
                for sign_b in [-1, 1] {
                    for sign_c in [-1, 1] {
                        next.insert((
                            row_sum + sign_a + sign_b + sign_c,
                            self_dot + sign_a * sign_b + sign_b * sign_c + sign_c * sign_a,
                        ));
                    }
                }
            }
        }
        states = next;
    }

    let mut table = HashMap::<i32, BTreeSet<i32>>::new();
    for (row_sum, self_dot) in states {
        table.entry(row_sum).or_default().insert(self_dot);
    }
    table
}

fn binomial_u128(n: u32, k: u32) -> u128 {
    let k = k.min(n - k);
    let mut result = 1u128;
    for i in 1..=k {
        result = result * u128::from(n + 1 - i) / u128::from(i);
    }
    result
}

fn dot9(left: [i32; 9], right: [i32; 9]) -> i32 {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

fn paf9(rows: [i32; 9]) -> [i32; 9] {
    let mut paf = [0i32; 9];
    for shift in 0..9 {
        let mut total = 0i32;
        for index in 0..9 {
            total += rows[index] * rows[(index + shift) % 9];
        }
        paf[shift] = total;
    }
    paf
}

fn row_paf_pair_is_exact(left: &[i32; 9], right: &[i32; 9]) -> bool {
    (1..9).all(|shift| left[shift] + right[shift] == LP333_ROW_SHIFT_TARGET)
}

fn row_units_147_shift0_dot_marginal_feasible(
    left: &RowUnits147Marginal,
    right: &RowUnits147Marginal,
) -> bool {
    row_units_147_shift0_dot_e1_feasible(left, right)
        && row_units_147_shift0_dot_e3_feasible(left, right)
}

fn row_units_147_shift0_dot_e1_feasible(
    left: &RowUnits147Marginal,
    right: &RowUnits147Marginal,
) -> bool {
    sumset_has_target(
        &left.shift0_e1_dot_sums,
        &right.shift0_e1_dot_sums,
        LP333_ACTUAL_SHIFT_TARGET,
    )
}

fn row_units_147_shift0_dot_e3_feasible(
    left: &RowUnits147Marginal,
    right: &RowUnits147Marginal,
) -> bool {
    let target_without_repeated_diagonals = LP333_ACTUAL_SHIFT_TARGET - 12 * 37;
    sumset_has_target(
        &left.shift0_e3_dot_sums,
        &right.shift0_e3_dot_sums,
        target_without_repeated_diagonals,
    )
}

fn row_units_147_e1_dot_sums(rows: [i32; 9]) -> BTreeSet<i32> {
    let representatives = row_units_147_representative_sums(rows);
    let terms = [
        (0usize, 3usize),
        (0, 4),
        (1, 3),
        (1, 4),
        (2, 3),
        (2, 4),
        (3, 4),
        (3, 4),
        (3, 4),
    ];
    row_dot_sumset(&representatives, &terms)
}

fn row_units_147_e3_dot_sums(rows: [i32; 9]) -> BTreeSet<i32> {
    let representatives = row_units_147_representative_sums(rows);
    row_dot_sumset(&representatives, &[(0usize, 1usize), (0, 2), (1, 2)])
}

fn row_units_147_representative_sums(rows: [i32; 9]) -> [i32; 5] {
    [rows[0], rows[3], rows[6], rows[1], rows[2]]
}

fn row_dot_sumset(row_sums: &[i32; 5], terms: &[(usize, usize)]) -> BTreeSet<i32> {
    let mut sums = BTreeSet::from([0i32]);
    for (left, right) in terms {
        let values = possible_row_dot_values(row_sums[*left], row_sums[*right]);
        let mut next = BTreeSet::new();
        for prefix in &sums {
            for value in &values {
                next.insert(prefix + value);
            }
        }
        sums = next;
    }
    sums
}

fn possible_row_dot_values(left_sum: i32, right_sum: i32) -> Vec<i32> {
    let left_plus = (37 + left_sum) / 2;
    let right_plus = (37 + right_sum) / 2;
    let intersection_min = 0.max(left_plus + right_plus - 37);
    let intersection_max = left_plus.min(right_plus);
    (intersection_min..=intersection_max)
        .map(|intersection| 37 - 2 * (left_plus + right_plus - 2 * intersection))
        .collect()
}

fn sumset_has_target(left: &BTreeSet<i32>, right: &BTreeSet<i32>, target: i32) -> bool {
    if left.len() <= right.len() {
        left.iter().any(|value| right.contains(&(target - value)))
    } else {
        right.iter().any(|value| left.contains(&(target - value)))
    }
}

fn push_top_row_units_147_sample(
    samples: &mut Vec<(f64, [i32; 9], [i32; 9])>,
    log10_count: f64,
    left: [i32; 9],
    right: [i32; 9],
    limit: usize,
) {
    samples.push((log10_count, left, right));
    samples.sort_unstable_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    samples.truncate(limit);
}

fn log10_sum_optional(current: Option<f64>, next: f64) -> f64 {
    let Some(current) = current else {
        return next;
    };
    let max = current.max(next);
    max + (10f64.powf(current - max) + 10f64.powf(next - max)).log10()
}

fn row_units_147_full_row_lift_summary(analysis: &RowUnits147FullRowLiftAnalysis) -> String {
    let pattern_log10 = analysis
        .exact_column_trivial_pattern_log10
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "none".to_string());
    let col10_pattern_log10 = analysis
        .col10_fixed_rows_pattern_log10
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "none".to_string());
    let col10_shift0_e3_pattern_log10 = analysis
        .col10_shift0_e3_pattern_log10
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "none".to_string());
    let col10_shift1_pattern_log10 = analysis
        .col10_shift1_pattern_log10
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "none".to_string());
    let col10_shift1_count = if analysis.col10_shift1_enabled {
        analysis.col10_shift1_feasible_row_pair_count.to_string()
    } else {
        "skipped".to_string()
    };
    let col10_shift1_pattern = if analysis.col10_shift1_enabled {
        col10_shift1_pattern_log10
    } else {
        "skipped".to_string()
    };
    format!(
        "bundle_pairs={},active_bundles={},active_row_marginals={},row_pair_candidates={},norm_compatible_row_pairs={},exact_row_pairs={},shift0_dot_marginal_feasible_row_pairs={},col10_fixed_rows_exact_row_pairs={},col10_shift0_e3_feasible_row_pairs={},col10_shift1_feasible_row_pairs={},exact_column_trivial_pattern_log10={},col10_fixed_rows_pattern_log10={},col10_shift0_e3_pattern_log10={},col10_shift1_pattern_log10={}",
        analysis.bundle_pair_count,
        analysis.active_bundle_count,
        analysis.active_row_marginal_count,
        analysis.row_pair_candidate_count,
        analysis.norm_compatible_row_pair_count,
        analysis.exact_row_pair_count,
        analysis.shift0_dot_marginal_feasible_row_pair_count,
        analysis.col10_fixed_rows_exact_row_pair_count,
        analysis.col10_shift0_e3_feasible_row_pair_count,
        col10_shift1_count,
        pattern_log10,
        col10_pattern_log10,
        col10_shift0_e3_pattern_log10,
        col10_shift1_pattern
    )
}

fn format_i32_list(values: &[i32]) -> String {
    values
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",")
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
        top_pair_dihedral_swap_mass_shares,
        pair_dihedral_swap_half_mass_prefix_count,
        pair_dihedral_swap_half_mass_bundle_orbit_count,
        pair_dihedral_swap_half_mass_bundle_orbits,
        pair_dihedral_swap_half_mass_bundle_orbit_frequencies,
        pair_dihedral_swap_half_mass_pair_frequencies,
        pair_dihedral_swap_half_mass_graph_components,
        pair_dihedral_swap_half_mass_graph_component_sizes,
        pair_dihedral_swap_half_mass_graph_degrees,
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
    println!(
        "row_mod3_bundle_top_pair_dihedral_swap_mass_shares={top_pair_dihedral_swap_mass_shares}"
    );
    println!(
        "row_mod3_bundle_pair_dihedral_swap_half_mass_prefix_count={pair_dihedral_swap_half_mass_prefix_count}"
    );
    println!(
        "row_mod3_bundle_pair_dihedral_swap_half_mass_bundle_orbit_count={pair_dihedral_swap_half_mass_bundle_orbit_count}"
    );
    println!(
        "row_mod3_bundle_pair_dihedral_swap_half_mass_bundle_orbits={}",
        pair_dihedral_swap_half_mass_bundle_orbits.join(";")
    );
    println!(
        "row_mod3_bundle_pair_dihedral_swap_half_mass_bundle_orbit_frequencies={}",
        pair_dihedral_swap_half_mass_bundle_orbit_frequencies.join(";")
    );
    println!(
        "row_mod3_bundle_pair_dihedral_swap_half_mass_pair_frequencies={}",
        pair_dihedral_swap_half_mass_pair_frequencies.join(";")
    );
    println!(
        "row_mod3_bundle_pair_dihedral_swap_half_mass_graph_components={}",
        pair_dihedral_swap_half_mass_graph_components.join(";")
    );
    println!(
        "row_mod3_bundle_pair_dihedral_swap_half_mass_graph_component_sizes={}",
        pair_dihedral_swap_half_mass_graph_component_sizes.join(",")
    );
    println!(
        "row_mod3_bundle_pair_dihedral_swap_half_mass_graph_degrees={}",
        pair_dihedral_swap_half_mass_graph_degrees.join(";")
    );
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

fn print_lp333_crt_component_analysis(hub: [i32; 3]) -> Result<(), String> {
    let triples = triples_by_sum(37);
    let (hub, spoke_a, spoke_b) = crt_component_by_hub(hub)?;
    let hub_uv = uv_transition_signature_keys(hub, &triples);
    let spoke_a_uv = uv_transition_signature_keys(spoke_a, &triples);
    let spoke_b_uv = uv_transition_signature_keys(spoke_b, &triples);
    let hub_coefficient = uv_transition_coefficient_signature_keys(hub, &triples);
    let spoke_a_coefficient = uv_transition_coefficient_signature_keys(spoke_a, &triples);
    let spoke_b_coefficient = uv_transition_coefficient_signature_keys(spoke_b, &triples);
    let hub_w_frontier = uv_transition_w_frontier_keys(hub, &triples);
    let spoke_a_w_frontier = uv_transition_w_frontier_keys(spoke_a, &triples);
    let spoke_b_w_frontier = uv_transition_w_frontier_keys(spoke_b, &triples);

    let uv_overlap = signature_overlap_summary(&spoke_a_uv, &spoke_b_uv);
    let coefficient_overlap = signature_overlap_summary(&spoke_a_coefficient, &spoke_b_coefficient);
    let w_frontier_overlap = signature_overlap_summary(&spoke_a_w_frontier, &spoke_b_w_frontier);

    println!(
        "representative_row_component_hub=[{},{},{}]",
        hub[0], hub[1], hub[2]
    );
    println!(
        "representative_row_component_spokes=[{},{},{}];[{},{},{}]",
        spoke_a[0], spoke_a[1], spoke_a[2], spoke_b[0], spoke_b[1], spoke_b[2]
    );
    println!(
        "representative_row_component_hub_uv_signature_count={}",
        hub_uv.len()
    );
    println!(
        "representative_row_component_spoke_uv_signature_counts=left={},right={}",
        spoke_a_uv.len(),
        spoke_b_uv.len()
    );
    println!(
        "representative_row_component_spoke_uv_overlap={}",
        uv_overlap
    );
    println!(
        "representative_row_component_hub_coefficient_signature_count={}",
        hub_coefficient.len()
    );
    println!(
        "representative_row_component_spoke_coefficient_signature_counts=left={},right={}",
        spoke_a_coefficient.len(),
        spoke_b_coefficient.len()
    );
    println!(
        "representative_row_component_spoke_coefficient_overlap={}",
        coefficient_overlap
    );
    println!(
        "representative_row_component_hub_w_frontier_count={}",
        hub_w_frontier.len()
    );
    println!(
        "representative_row_component_spoke_w_frontier_counts=left={},right={}",
        spoke_a_w_frontier.len(),
        spoke_b_w_frontier.len()
    );
    println!(
        "representative_row_component_spoke_w_frontier_overlap={}",
        w_frontier_overlap
    );
    Ok(())
}

fn print_lp333_crt_bundle_analysis(bundle: [i32; 3]) -> Result<(), String> {
    let triples = triples_by_sum(37);
    validate_crt_row_bundle(bundle, &triples)?;
    let canonical_bundle = canonical_bundle_rotation(bundle);
    let triple_sum_counts = triples
        .iter()
        .map(|(sum, triples)| (*sum, triples.len() as u64))
        .collect::<BTreeMap<_, _>>();

    println!(
        "crt_row_bundle_canonical=[{},{},{}]",
        canonical_bundle[0], canonical_bundle[1], canonical_bundle[2]
    );
    for shift in 0..3 {
        let rotated = cycle_bundle(canonical_bundle, shift);
        let component_sizes = rotated
            .iter()
            .map(|sum| triple_sum_counts.get(sum).copied().unwrap_or(0))
            .collect::<Vec<_>>();
        let uv_histogram = uv_transition_signature_histogram(rotated, &triples);
        let coefficient_histogram =
            uv_transition_coefficient_signature_histogram(rotated, &triples);
        let w_histogram = uv_transition_w_frontier_histogram(rotated, &triples);
        println!(
            "crt_row_bundle_split_{shift}=[{},{},{}]",
            rotated[0], rotated[1], rotated[2]
        );
        println!(
            "crt_row_bundle_split_{shift}_component_sizes={},{},{}",
            component_sizes[0], component_sizes[1], component_sizes[2]
        );
        println!(
            "crt_row_bundle_split_{shift}_uv_signature_stats={}",
            histogram_collision_summary(&uv_histogram)
        );
        println!(
            "crt_row_bundle_split_{shift}_coefficient_signature_stats={}",
            histogram_collision_summary(&coefficient_histogram)
        );
        println!(
            "crt_row_bundle_split_{shift}_w_frontier_stats={}",
            histogram_collision_summary(&w_histogram)
        );
        println!(
            "crt_row_bundle_split_{shift}_uv_to_w_unique_ratio={:.4}",
            ratio(uv_histogram.len(), w_histogram.len())
        );
    }
    Ok(())
}

fn print_lp333_crt_pair_analysis(
    left: [i32; 3],
    right: [i32; 3],
    left_shift: usize,
    right_shift: usize,
    shift: i32,
    exact_join: bool,
    sample_buckets: usize,
    frontier_join: bool,
    frontier_exact_join: bool,
    two_shifts: bool,
    all_shifts: bool,
) -> Result<(), String> {
    let triples = triples_by_sum(37);
    validate_crt_row_bundle(left, &triples)?;
    validate_crt_row_bundle(right, &triples)?;
    let left_canonical = canonical_bundle_rotation(left);
    let right_canonical = canonical_bundle_rotation(right);
    let left_split = cycle_bundle(left_canonical, left_shift);
    let right_split = cycle_bundle(right_canonical, right_shift);
    let triple_sum_counts = triples
        .iter()
        .map(|(sum, triples)| (*sum, triples.len() as u64))
        .collect::<BTreeMap<_, _>>();
    let triple_lift_counts = sequence_counts_by_sum_and_norm(3, &odd_alphabet(37));
    let left_norm_counts = convolve_bundle_component_norms(left_split, &triple_lift_counts);
    let right_norm_counts = convolve_bundle_component_norms(right_split, &triple_lift_counts);
    let norm_only_pair_count =
        compatible_row_norm_pair_count(&left_norm_counts, &right_norm_counts, 594);

    println!(
        "crt_row_pair_left_canonical=[{},{},{}]",
        left_canonical[0], left_canonical[1], left_canonical[2]
    );
    println!(
        "crt_row_pair_right_canonical=[{},{},{}]",
        right_canonical[0], right_canonical[1], right_canonical[2]
    );
    println!(
        "crt_row_pair_left_split={left_shift}:[{},{},{}]",
        left_split[0], left_split[1], left_split[2]
    );
    println!(
        "crt_row_pair_right_split={right_shift}:[{},{},{}]",
        right_split[0], right_split[1], right_split[2]
    );
    println!(
        "crt_row_pair_left_component_sizes={},{},{}",
        triple_sum_counts.get(&left_split[0]).copied().unwrap_or(0),
        triple_sum_counts.get(&left_split[1]).copied().unwrap_or(0),
        triple_sum_counts.get(&left_split[2]).copied().unwrap_or(0)
    );
    println!(
        "crt_row_pair_right_component_sizes={},{},{}",
        triple_sum_counts.get(&right_split[0]).copied().unwrap_or(0),
        triple_sum_counts.get(&right_split[1]).copied().unwrap_or(0),
        triple_sum_counts.get(&right_split[2]).copied().unwrap_or(0)
    );
    println!("crt_row_pair_norm_only_count={norm_only_pair_count}");
    let form = row_shift_linear_form(shift)
        .ok_or_else(|| format!("unsupported row shift {shift}; expected one of 1, 2, 4"))?;
    let frontier_join_requested = frontier_join || frontier_exact_join;
    println!("crt_row_pair_shift={shift}");
    println!("crt_row_pair_exact_join_requested={exact_join}");
    println!("crt_row_pair_sample_buckets={sample_buckets}");
    println!("crt_row_pair_frontier_join_requested={frontier_join_requested}");
    println!("crt_row_pair_frontier_exact_join_requested={frontier_exact_join}");
    println!("crt_row_pair_two_shifts_requested={two_shifts}");
    println!("crt_row_pair_all_shifts_requested={all_shifts}");
    let left_analysis =
        bundle_row_shift_analysis(left_split, &triples, form, exact_join, sample_buckets);
    let right_analysis =
        bundle_row_shift_analysis(right_split, &triples, form, exact_join, sample_buckets);
    println!(
        "crt_row_pair_{}_left_uv_stats={}",
        form.label,
        reduced_bucket_summary(
            left_analysis.raw_uv_pairs,
            left_analysis.reduced_uv_state_count,
            left_analysis.coefficient_bucket_count,
            left_analysis.coefficient_permutation_orbit_count,
            left_analysis.max_coefficient_permutation_orbit_size,
            left_analysis.max_coefficient_bucket_mass,
        )
    );
    println!(
        "crt_row_pair_{}_right_uv_stats={}",
        form.label,
        reduced_bucket_summary(
            right_analysis.raw_uv_pairs,
            right_analysis.reduced_uv_state_count,
            right_analysis.coefficient_bucket_count,
            right_analysis.coefficient_permutation_orbit_count,
            right_analysis.max_coefficient_permutation_orbit_size,
            right_analysis.max_coefficient_bucket_mass,
        )
    );
    println!(
        "crt_row_pair_{}_left_w_stats={}",
        form.label,
        w_signature_summary(
            left_analysis.coefficient_bucket_count,
            left_analysis.total_w_signature_count,
            left_analysis.max_w_signature_count,
            left_analysis.materialization_work_estimate,
        )
    );
    println!(
        "crt_row_pair_{}_right_w_stats={}",
        form.label,
        w_signature_summary(
            right_analysis.coefficient_bucket_count,
            right_analysis.total_w_signature_count,
            right_analysis.max_w_signature_count,
            right_analysis.materialization_work_estimate,
        )
    );
    if !left_analysis.bucket_convolution_samples.is_empty() {
        println!(
            "crt_row_pair_{}_left_bucket_samples={}",
            form.label,
            left_analysis.bucket_convolution_samples.join(";")
        );
    }
    if !right_analysis.bucket_convolution_samples.is_empty() {
        println!(
            "crt_row_pair_{}_right_bucket_samples={}",
            form.label,
            right_analysis.bucket_convolution_samples.join(";")
        );
    }
    if frontier_join_requested {
        let left_frontier = bundle_row_shift_frontier_buckets(left_split, &triples, form);
        let right_frontier = bundle_row_shift_frontier_buckets(right_split, &triples, form);
        let frontier_analysis = row_shift_frontier_join_analysis(
            &left_frontier,
            &right_frontier,
            LP333_ROW_NORM_TARGET,
            LP333_ROW_SHIFT_TARGET,
            sample_buckets,
        );
        println!(
            "crt_row_pair_{}_frontier_join_stats={}",
            form.label,
            frontier_join_summary(&frontier_analysis)
        );
        if !frontier_analysis.bucket_pair_samples.is_empty() {
            let sample_summaries = frontier_join_sample_summaries(
                &frontier_analysis.bucket_pair_samples,
                left_split,
                right_split,
                &triples,
                form,
                LP333_ROW_NORM_TARGET,
                LP333_ROW_SHIFT_TARGET,
            );
            println!(
                "crt_row_pair_{}_frontier_join_samples={}",
                form.label,
                sample_summaries.join(";")
            );
        }
        if frontier_exact_join {
            let left_active_histogram = row_shift_active_bucket_row_signature_histogram(
                left_split,
                &triples,
                form,
                &left_frontier,
                &frontier_analysis.active_left_indices,
            );
            let right_active_histogram = row_shift_active_bucket_row_signature_histogram(
                right_split,
                &triples,
                form,
                &right_frontier,
                &frontier_analysis.active_right_indices,
            );
            let join_count = row_shift_join_count(
                &left_active_histogram,
                &right_active_histogram,
                LP333_ROW_NORM_TARGET,
                LP333_ROW_SHIFT_TARGET,
            );
            println!(
                "crt_row_pair_{}_frontier_exact_left_row_signature_stats={}",
                form.label,
                aggregate_count_summary(
                    frontier_analysis.active_left_materialization_work,
                    left_active_histogram.len(),
                )
            );
            println!(
                "crt_row_pair_{}_frontier_exact_right_row_signature_stats={}",
                form.label,
                aggregate_count_summary(
                    frontier_analysis.active_right_materialization_work,
                    right_active_histogram.len(),
                )
            );
            println!(
                "crt_row_pair_{}_frontier_exact_join_stats=join_count={},survival={:.8},reduction_factor={:.4}",
                form.label,
                join_count,
                fraction_u128(join_count, norm_only_pair_count),
                reduction_factor(norm_only_pair_count, join_count),
            );
        }
    }
    if two_shifts {
        for (left_index, right_index) in [(0usize, 1usize), (0, 2), (1, 2)] {
            let left_two_shift_analysis = bundle_two_row_shifts_analysis(
                left_split,
                &triples,
                left_index,
                right_index,
                sample_buckets,
            );
            let right_two_shift_analysis = bundle_two_row_shifts_analysis(
                right_split,
                &triples,
                left_index,
                right_index,
                sample_buckets,
            );
            println!(
                "crt_row_pair_{}_left_uv_stats={}",
                left_two_shift_analysis.label,
                two_shift_bucket_summary(&left_two_shift_analysis)
            );
            println!(
                "crt_row_pair_{}_right_uv_stats={}",
                right_two_shift_analysis.label,
                two_shift_bucket_summary(&right_two_shift_analysis)
            );
            if !left_two_shift_analysis.bucket_samples.is_empty() {
                println!(
                    "crt_row_pair_{}_left_bucket_samples={}",
                    left_two_shift_analysis.label,
                    left_two_shift_analysis.bucket_samples.join(";")
                );
            }
            if !right_two_shift_analysis.bucket_samples.is_empty() {
                println!(
                    "crt_row_pair_{}_right_bucket_samples={}",
                    right_two_shift_analysis.label,
                    right_two_shift_analysis.bucket_samples.join(";")
                );
            }
        }
    }
    if all_shifts {
        let left_all_shift_analysis =
            bundle_all_row_shifts_analysis(left_split, &triples, sample_buckets);
        let right_all_shift_analysis =
            bundle_all_row_shifts_analysis(right_split, &triples, sample_buckets);
        println!(
            "crt_row_pair_all_shifts_left_uv_stats={}",
            all_shift_bucket_summary(&left_all_shift_analysis)
        );
        println!(
            "crt_row_pair_all_shifts_right_uv_stats={}",
            all_shift_bucket_summary(&right_all_shift_analysis)
        );
        if !left_all_shift_analysis.bucket_samples.is_empty() {
            println!(
                "crt_row_pair_all_shifts_left_bucket_samples={}",
                left_all_shift_analysis.bucket_samples.join(";")
            );
        }
        if !right_all_shift_analysis.bucket_samples.is_empty() {
            println!(
                "crt_row_pair_all_shifts_right_bucket_samples={}",
                right_all_shift_analysis.bucket_samples.join(";")
            );
        }
    }
    if let (Some(left_hist), Some(right_hist)) = (
        left_analysis.row_signature_histogram.as_ref(),
        right_analysis.row_signature_histogram.as_ref(),
    ) {
        let join_count = row_shift_join_count(
            left_hist,
            right_hist,
            LP333_ROW_NORM_TARGET,
            LP333_ROW_SHIFT_TARGET,
        );
        println!(
            "crt_row_pair_{}_left_row_signature_stats={}",
            form.label,
            aggregate_count_summary(left_analysis.raw_rows, left_hist.len())
        );
        println!(
            "crt_row_pair_{}_right_row_signature_stats={}",
            form.label,
            aggregate_count_summary(right_analysis.raw_rows, right_hist.len())
        );
        println!(
            "crt_row_pair_{}_join_stats=join_count={},survival={:.8},reduction_factor={:.4}",
            form.label,
            join_count,
            fraction_u128(join_count, norm_only_pair_count),
            reduction_factor(norm_only_pair_count, join_count),
        );
    } else {
        println!(
            "crt_row_pair_{}_join_stats=skipped (pass --exact-join to materialize full norm-plus-shift row signatures)",
            form.label
        );
    }
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
    let mut by_sum_norm =
        BTreeMap::<i32, BTreeMap<i32, u128>>::from([(0, BTreeMap::from([(0, 1)]))]);
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
    usize,
    usize,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
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
        .flat_map(|paf| {
            bundles_by_paf
                .get(paf)
                .into_iter()
                .flat_map(|bundles| bundles.iter().copied())
        })
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
                    let compatible_lifts = compatible_row_norm_pair_count(
                        &left.row_norm_counts,
                        &right.row_norm_counts,
                        594,
                    );
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
                    top_pairs.push((compatible_lifts, left.bundle, right.bundle));
                }
            }
        }
    }
    let active_pair_cyclic_orbit_count = pair_orbit_mass.len();
    let active_pair_swap_orbit_count = pair_swap_orbit_mass.len();
    let active_pair_dihedral_swap_orbit_count = pair_dihedral_swap_orbit_mass.len();
    let full_dihedral_swap_mass: u128 = pair_dihedral_swap_orbit_mass.values().copied().sum();
    top_pairs.sort_unstable_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
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
    let mut top_pair_dihedral_swap_orbits = pair_dihedral_swap_orbit_mass
        .into_iter()
        .collect::<Vec<_>>();
    top_pair_dihedral_swap_orbits
        .sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top1_dihedral_swap_mass: u128 = top_pair_dihedral_swap_orbits
        .iter()
        .take(1)
        .map(|(_, mass)| *mass)
        .sum();
    let top5_dihedral_swap_mass: u128 = top_pair_dihedral_swap_orbits
        .iter()
        .take(5)
        .map(|(_, mass)| *mass)
        .sum();
    let top10_dihedral_swap_mass: u128 = top_pair_dihedral_swap_orbits
        .iter()
        .take(10)
        .map(|(_, mass)| *mass)
        .sum();
    let half_mass_target = (full_dihedral_swap_mass + 1) / 2;
    let mut running_mass = 0u128;
    let mut pair_dihedral_swap_half_mass_prefix_count = 0usize;
    for (_, mass) in &top_pair_dihedral_swap_orbits {
        running_mass += *mass;
        pair_dihedral_swap_half_mass_prefix_count += 1;
        if running_mass >= half_mass_target {
            break;
        }
    }
    let pair_dihedral_swap_half_mass_bundle_orbit_count = top_pair_dihedral_swap_orbits
        .iter()
        .take(pair_dihedral_swap_half_mass_prefix_count)
        .flat_map(|((left, right), _)| {
            [
                canonical_bundle_rotation(*left),
                canonical_bundle_rotation(*right),
            ]
        })
        .collect::<BTreeSet<_>>()
        .len();
    let pair_dihedral_swap_half_mass_bundle_orbits = top_pair_dihedral_swap_orbits
        .iter()
        .take(pair_dihedral_swap_half_mass_prefix_count)
        .flat_map(|((left, right), _)| {
            [
                canonical_bundle_rotation(*left),
                canonical_bundle_rotation(*right),
            ]
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|bundle| format!("[{},{},{}]", bundle[0], bundle[1], bundle[2]))
        .collect::<Vec<_>>();
    let mut half_mass_bundle_orbit_frequency = BTreeMap::<[i32; 3], usize>::new();
    for ((left, right), _) in top_pair_dihedral_swap_orbits
        .iter()
        .take(pair_dihedral_swap_half_mass_prefix_count)
    {
        *half_mass_bundle_orbit_frequency
            .entry(canonical_bundle_rotation(*left))
            .or_default() += 1;
        *half_mass_bundle_orbit_frequency
            .entry(canonical_bundle_rotation(*right))
            .or_default() += 1;
    }
    let mut pair_dihedral_swap_half_mass_bundle_orbit_frequencies =
        half_mass_bundle_orbit_frequency
            .into_iter()
            .collect::<Vec<_>>();
    pair_dihedral_swap_half_mass_bundle_orbit_frequencies
        .sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut half_mass_pair_frequency = BTreeMap::<([i32; 3], [i32; 3]), usize>::new();
    for ((left, right), _) in top_pair_dihedral_swap_orbits
        .iter()
        .take(pair_dihedral_swap_half_mass_prefix_count)
    {
        let left = canonical_bundle_rotation(*left);
        let right = canonical_bundle_rotation(*right);
        let pair = if left <= right {
            (left, right)
        } else {
            (right, left)
        };
        *half_mass_pair_frequency.entry(pair).or_default() += 1;
    }
    let mut pair_dihedral_swap_half_mass_pair_frequencies =
        half_mass_pair_frequency.into_iter().collect::<Vec<_>>();
    pair_dihedral_swap_half_mass_pair_frequencies
        .sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut half_mass_pair_mass = BTreeMap::<([i32; 3], [i32; 3]), u128>::new();
    for ((left, right), mass) in top_pair_dihedral_swap_orbits
        .iter()
        .take(pair_dihedral_swap_half_mass_prefix_count)
    {
        let left = canonical_bundle_rotation(*left);
        let right = canonical_bundle_rotation(*right);
        let pair = if left <= right {
            (left, right)
        } else {
            (right, left)
        };
        *half_mass_pair_mass.entry(pair).or_default() += *mass;
    }
    let half_mass_pairs = pair_dihedral_swap_half_mass_pair_frequencies
        .iter()
        .map(|((left, right), _)| (*left, *right))
        .collect::<Vec<_>>();
    let pair_dihedral_swap_half_mass_graph_components =
        half_mass_graph_components(&half_mass_pair_mass)
            .into_iter()
            .map(|(mass, edge_count, nodes)| {
                let node_count = nodes.len();
                let labels = nodes
                    .into_iter()
                    .map(|bundle| format!("[{},{},{}]", bundle[0], bundle[1], bundle[2]))
                    .collect::<Vec<_>>()
                    .join("|");
                format!("{mass}:{}nodes/{}edges:{labels}", node_count, edge_count)
            })
            .collect::<Vec<_>>();
    let pair_dihedral_swap_half_mass_graph_component_sizes =
        half_mass_graph_component_sizes(&half_mass_pairs)
            .into_iter()
            .map(|size| size.to_string())
            .collect::<Vec<_>>();
    let pair_dihedral_swap_half_mass_graph_degrees = half_mass_graph_degrees(&half_mass_pairs)
        .into_iter()
        .map(|(bundle, degree)| format!("{degree}:[{},{},{}]", bundle[0], bundle[1], bundle[2]))
        .collect::<Vec<_>>();
    let top_pair_dihedral_swap_mass_shares = if full_dihedral_swap_mass == 0 {
        "none".to_string()
    } else {
        let top20_dihedral_swap_mass: u128 = top_pair_dihedral_swap_orbits
            .iter()
            .take(20)
            .map(|(_, mass)| *mass)
            .sum();
        format!(
            "top1={}/{} ({:.4}),top5={}/{} ({:.4}),top10={}/{} ({:.4}),top20={}/{} ({:.4})",
            top1_dihedral_swap_mass,
            full_dihedral_swap_mass,
            top1_dihedral_swap_mass as f64 / full_dihedral_swap_mass as f64,
            top5_dihedral_swap_mass,
            full_dihedral_swap_mass,
            top5_dihedral_swap_mass as f64 / full_dihedral_swap_mass as f64,
            top10_dihedral_swap_mass,
            full_dihedral_swap_mass,
            top10_dihedral_swap_mass as f64 / full_dihedral_swap_mass as f64,
            top20_dihedral_swap_mass,
            full_dihedral_swap_mass,
            top20_dihedral_swap_mass as f64 / full_dihedral_swap_mass as f64
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
        top_pair_dihedral_swap_mass_shares,
        pair_dihedral_swap_half_mass_prefix_count,
        pair_dihedral_swap_half_mass_bundle_orbit_count,
        pair_dihedral_swap_half_mass_bundle_orbits,
        pair_dihedral_swap_half_mass_bundle_orbit_frequencies
            .into_iter()
            .map(|(bundle, count)| format!("{count}:[{},{},{}]", bundle[0], bundle[1], bundle[2]))
            .collect(),
        pair_dihedral_swap_half_mass_pair_frequencies
            .into_iter()
            .map(|((left, right), count)| {
                format!(
                    "{count}:[{},{},{}]|[{},{},{}]",
                    left[0], left[1], left[2], right[0], right[1], right[2]
                )
            })
            .collect(),
        pair_dihedral_swap_half_mass_graph_components,
        pair_dihedral_swap_half_mass_graph_component_sizes,
        pair_dihedral_swap_half_mass_graph_degrees,
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
    let rotations = [bundle, cycle_bundle(bundle, 1), cycle_bundle(bundle, 2)];
    *rotations.iter().min().unwrap()
}

fn canonical_pair_rotation(left: [i32; 3], right: [i32; 3]) -> ([i32; 3], [i32; 3]) {
    let rotations = [
        (left, right),
        ([left[1], left[2], left[0]], [right[1], right[2], right[0]]),
        ([left[2], left[0], left[1]], [right[2], right[0], right[1]]),
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

fn half_mass_graph_component_sizes(pairs: &[([i32; 3], [i32; 3])]) -> Vec<usize> {
    let mut adjacency = BTreeMap::<[i32; 3], BTreeSet<[i32; 3]>>::new();
    for (left, right) in pairs {
        adjacency.entry(*left).or_default().insert(*right);
        adjacency.entry(*right).or_default().insert(*left);
    }
    let mut seen = BTreeSet::new();
    let mut sizes = Vec::new();
    for start in adjacency.keys() {
        if !seen.insert(*start) {
            continue;
        }
        let mut stack = vec![*start];
        let mut size = 0usize;
        while let Some(node) = stack.pop() {
            size += 1;
            if let Some(neighbors) = adjacency.get(&node) {
                for neighbor in neighbors {
                    if seen.insert(*neighbor) {
                        stack.push(*neighbor);
                    }
                }
            }
        }
        sizes.push(size);
    }
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    sizes
}

fn half_mass_graph_components(
    pair_masses: &BTreeMap<([i32; 3], [i32; 3]), u128>,
) -> Vec<(u128, usize, Vec<[i32; 3]>)> {
    let mut adjacency = BTreeMap::<[i32; 3], BTreeSet<[i32; 3]>>::new();
    for ((left, right), _) in pair_masses {
        adjacency.entry(*left).or_default().insert(*right);
        adjacency.entry(*right).or_default().insert(*left);
    }
    let mut seen = BTreeSet::new();
    let mut components = Vec::new();
    for start in adjacency.keys() {
        if !seen.insert(*start) {
            continue;
        }
        let mut stack = vec![*start];
        let mut nodes = Vec::new();
        while let Some(node) = stack.pop() {
            nodes.push(node);
            if let Some(neighbors) = adjacency.get(&node) {
                for neighbor in neighbors {
                    if seen.insert(*neighbor) {
                        stack.push(*neighbor);
                    }
                }
            }
        }
        nodes.sort_unstable();
        let node_set = nodes.iter().copied().collect::<BTreeSet<_>>();
        let mut component_mass = 0u128;
        let mut edge_count = 0usize;
        for ((left, right), mass) in pair_masses {
            if node_set.contains(left) && node_set.contains(right) {
                component_mass += *mass;
                edge_count += 1;
            }
        }
        components.push((component_mass, edge_count, nodes));
    }
    components.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.2.cmp(&b.2)));
    components
}

fn half_mass_graph_degrees(pairs: &[([i32; 3], [i32; 3])]) -> Vec<([i32; 3], usize)> {
    let mut adjacency = BTreeMap::<[i32; 3], BTreeSet<[i32; 3]>>::new();
    for (left, right) in pairs {
        adjacency.entry(*left).or_default().insert(*right);
        adjacency.entry(*right).or_default().insert(*left);
    }
    let mut degrees = adjacency
        .into_iter()
        .map(|(bundle, neighbors)| (bundle, neighbors.len()))
        .collect::<Vec<_>>();
    degrees.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    degrees
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
    uv_transition_signature_histogram(bundle, triples_by_sum).len()
}

fn uv_transition_signature_keys(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> HashSet<u128> {
    uv_transition_signature_histogram(bundle, triples_by_sum)
        .into_iter()
        .map(|(signature, _)| signature)
        .collect()
}

fn uv_transition_signature_histogram(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> HashMap<u128, u64> {
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let mut signatures = HashMap::new();
    for u in us {
        let ru1 = rotate3(*u, 1);
        let norm_u = dot3(*u, *u);
        for v in vs {
            let rv1 = rotate3(*v, 1);
            *signatures
                .entry(pack_uv_transition_signature(
                    norm_u + dot3(*v, *v),
                    dot3(*u, *v),
                    dot3(*v, ru1),
                    dot3(*u, rv1),
                    add3(*v, ru1),
                    add3(*u, rv1),
                    add3(rotate3(*u, 2), rotate3(*v, 2)),
                ))
                .or_default() += 1;
        }
    }
    signatures
}

fn uv_transition_coefficient_signature_count(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> usize {
    uv_transition_coefficient_signature_histogram(bundle, triples_by_sum).len()
}

fn uv_transition_coefficient_signature_keys(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> HashSet<u128> {
    uv_transition_coefficient_signature_histogram(bundle, triples_by_sum)
        .into_iter()
        .map(|(signature, _)| signature)
        .collect()
}

fn uv_transition_coefficient_signature_histogram(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> HashMap<u128, u64> {
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let mut signatures = HashMap::new();
    for u in us {
        let ru1 = rotate3(*u, 1);
        for v in vs {
            let rv1 = rotate3(*v, 1);
            *signatures
                .entry(pack_uv_transition_coefficient_signature(
                    add3(*v, ru1),
                    add3(*u, rv1),
                    add3(rotate3(*u, 2), rotate3(*v, 2)),
                ))
                .or_default() += 1;
        }
    }
    signatures
}

fn uv_transition_w_frontier_keys(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> HashSet<u128> {
    uv_transition_w_frontier_histogram(bundle, triples_by_sum)
        .into_iter()
        .map(|(signature, _)| signature)
        .collect()
}

fn uv_transition_w_frontier_histogram(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> HashMap<u128, u64> {
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let mut signatures = HashMap::new();
    for u in us {
        for v in vs {
            *signatures
                .entry(pack_uv_transition_w_frontier(add3(
                    rotate3(*u, 2),
                    rotate3(*v, 2),
                )))
                .or_default() += 1;
        }
    }
    signatures
}

fn pack_uv_transition_signature(
    norm: i32,
    dot_uv: i32,
    dot_v_ru1: i32,
    dot_u_rv1: i32,
    seam_a: [i32; 3],
    seam_b: [i32; 3],
    seam_c: [i32; 3],
) -> u128 {
    let mut packed = 0u128;
    push_bits(&mut packed, pack_nonnegative(norm, 14), 14);
    push_bits(&mut packed, pack_signed(dot_uv, 4107, 14), 14);
    push_bits(&mut packed, pack_signed(dot_v_ru1, 4107, 14), 14);
    push_bits(&mut packed, pack_signed(dot_u_rv1, 4107, 14), 14);
    for value in seam_a.into_iter().chain(seam_b).chain(seam_c) {
        push_bits(&mut packed, pack_signed(value, 74, 8), 8);
    }
    packed
}

fn pack_uv_transition_coefficient_signature(
    seam_a: [i32; 3],
    seam_b: [i32; 3],
    seam_c: [i32; 3],
) -> u128 {
    let mut packed = 0u128;
    for value in seam_a.into_iter().chain(seam_b).chain(seam_c) {
        push_bits(&mut packed, pack_signed(value, 74, 8), 8);
    }
    packed
}

fn pack_uv_transition_w_frontier(seam_c: [i32; 3]) -> u128 {
    let mut packed = 0u128;
    for value in seam_c {
        push_bits(&mut packed, pack_signed(value, 74, 8), 8);
    }
    packed
}

fn push_bits(packed: &mut u128, value: u128, width: u32) {
    *packed = (*packed << width) | value;
}

fn pack_nonnegative(value: i32, width: u32) -> u128 {
    debug_assert!(value >= 0);
    let value = value as u128;
    debug_assert!(value < (1u128 << width));
    value
}

fn pack_signed(value: i32, offset: i32, width: u32) -> u128 {
    let shifted = value + offset;
    debug_assert!(shifted >= 0);
    let shifted = shifted as u128;
    debug_assert!(shifted < (1u128 << width));
    shifted
}

fn signature_overlap_summary(left: &HashSet<u128>, right: &HashSet<u128>) -> String {
    let intersection = if left.len() <= right.len() {
        left.iter()
            .filter(|signature| right.contains(signature))
            .count()
    } else {
        right
            .iter()
            .filter(|signature| left.contains(signature))
            .count()
    };
    let union = left.len() + right.len() - intersection;
    let reuse_ratio = if union == 0 {
        0.0
    } else {
        (left.len() + right.len()) as f64 / union as f64
    };
    format!(
        "intersection={},union={},reuse_factor={:.4}",
        intersection, union, reuse_ratio
    )
}

#[derive(Clone, Copy)]
struct RowShiftLinearForm {
    label: &'static str,
    base_term: fn([i32; 3], [i32; 3]) -> i32,
    coefficient: fn([i32; 3], [i32; 3]) -> [i32; 3],
}

struct BundleRowShiftAnalysis {
    raw_uv_pairs: u64,
    reduced_uv_state_count: usize,
    coefficient_bucket_count: usize,
    coefficient_permutation_orbit_count: usize,
    max_coefficient_permutation_orbit_size: usize,
    max_coefficient_bucket_mass: u64,
    total_w_signature_count: usize,
    max_w_signature_count: usize,
    materialization_work_estimate: u128,
    raw_rows: u128,
    row_signature_histogram: Option<HashMap<(i32, i32), u128>>,
    bucket_convolution_samples: Vec<String>,
}

struct BundleAllRowShiftsAnalysis {
    raw_uv_pairs: u64,
    reduced_uv_state_count: usize,
    coefficient_bucket_count: usize,
    coefficient_permutation_orbit_count: usize,
    max_coefficient_permutation_orbit_size: usize,
    max_coefficient_bucket_mass: u64,
    estimated_materialization_work: u128,
    bucket_samples: Vec<String>,
}

struct BundleTwoRowShiftsAnalysis {
    label: String,
    raw_uv_pairs: u64,
    reduced_uv_state_count: usize,
    coefficient_bucket_count: usize,
    coefficient_permutation_orbit_count: usize,
    max_coefficient_permutation_orbit_size: usize,
    max_coefficient_bucket_mass: u64,
    raw_w_materialization_work: u128,
    bucket_samples: Vec<String>,
}

struct RowShiftFrontierBucket {
    coefficient: [i32; 3],
    uv_state_count: usize,
    w_signature_count: usize,
    materialization_work: u128,
    norm_value_count: usize,
    shift_value_count: usize,
    norm_min: i32,
    norm_max: i32,
    shift_min: i32,
    shift_max: i32,
    norm_bits: Vec<u64>,
    shift_bits: Vec<u64>,
}

struct RowShiftFrontierJoinAnalysis {
    bucket_pair_count: u128,
    interval_survivor_count: u128,
    norm_survivor_count: u128,
    shift_survivor_count: u128,
    norm_shift_survivor_count: u128,
    active_left_bucket_count: usize,
    active_right_bucket_count: usize,
    active_left_materialization_work: u128,
    active_right_materialization_work: u128,
    active_left_indices: Vec<usize>,
    active_right_indices: Vec<usize>,
    bucket_pair_samples: Vec<RowShiftFrontierBucketPairSample>,
}

struct RowShiftFrontierBucketPairSample {
    left_coefficient: [i32; 3],
    right_coefficient: [i32; 3],
    left_uv_state_count: usize,
    right_uv_state_count: usize,
    left_w_signature_count: usize,
    right_w_signature_count: usize,
    left_materialization_work: u128,
    right_materialization_work: u128,
    left_norm_value_count: usize,
    right_norm_value_count: usize,
    left_shift_value_count: usize,
    right_shift_value_count: usize,
    pair_work: u128,
}

fn row_shift_linear_forms() -> [RowShiftLinearForm; 3] {
    [
        RowShiftLinearForm {
            label: "shift1",
            base_term: row_shift1_base_term,
            coefficient: row_shift1_w_coefficient,
        },
        RowShiftLinearForm {
            label: "shift2",
            base_term: row_shift2_base_term,
            coefficient: row_shift2_w_coefficient,
        },
        RowShiftLinearForm {
            label: "shift4",
            base_term: row_shift4_base_term,
            coefficient: row_shift4_w_coefficient,
        },
    ]
}

fn row_shift_linear_form(shift: i32) -> Option<RowShiftLinearForm> {
    match shift {
        1 => Some(row_shift_linear_forms()[0]),
        2 => Some(row_shift_linear_forms()[1]),
        4 => Some(row_shift_linear_forms()[2]),
        _ => None,
    }
}

fn row_shift1_base_term(u: [i32; 3], v: [i32; 3]) -> i32 {
    dot3(u, v)
}

fn row_shift1_w_coefficient(u: [i32; 3], v: [i32; 3]) -> [i32; 3] {
    add3(v, rotate3(u, 1))
}

fn row_shift2_base_term(u: [i32; 3], v: [i32; 3]) -> i32 {
    dot3(v, rotate3(u, 1))
}

fn row_shift2_w_coefficient(u: [i32; 3], v: [i32; 3]) -> [i32; 3] {
    add3(u, rotate3(v, 1))
}

fn row_shift4_base_term(u: [i32; 3], v: [i32; 3]) -> i32 {
    dot3(u, rotate3(v, 1))
}

fn row_shift4_w_coefficient(u: [i32; 3], v: [i32; 3]) -> [i32; 3] {
    add3(rotate3(u, 2), rotate3(v, 2))
}

fn bundle_row_shift_analysis(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
    form: RowShiftLinearForm,
    materialize_row_signatures: bool,
    sample_bucket_count: usize,
) -> BundleRowShiftAnalysis {
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let ws = triples_by_sum.get(&bundle[2]).unwrap();
    let raw_uv_pairs = us.len() as u64 * vs.len() as u64;
    let raw_rows = raw_uv_pairs as u128 * ws.len() as u128;
    let mut uv_states_by_coefficient = HashMap::<[i32; 3], HashMap<(i32, i32), u64>>::new();
    for u in us {
        let norm_u = dot3(*u, *u);
        for v in vs {
            let coefficient = (form.coefficient)(*u, *v);
            let base_term = (form.base_term)(*u, *v);
            *uv_states_by_coefficient
                .entry(coefficient)
                .or_default()
                .entry((norm_u + dot3(*v, *v), base_term))
                .or_default() += 1;
        }
    }
    let reduced_uv_state_count = uv_states_by_coefficient.values().map(HashMap::len).sum();
    let coefficient_bucket_count = uv_states_by_coefficient.len();
    let coefficient_permutation_orbits =
        coefficient_permutation_orbit_distribution(uv_states_by_coefficient.keys().copied());
    let coefficient_permutation_orbit_count = coefficient_permutation_orbits.len();
    let max_coefficient_permutation_orbit_size = coefficient_permutation_orbits
        .values()
        .copied()
        .max()
        .unwrap_or(0);
    let max_coefficient_bucket_mass = uv_states_by_coefficient
        .values()
        .map(|states| states.values().copied().sum::<u64>())
        .max()
        .unwrap_or(0);
    let mut total_w_signature_count = 0usize;
    let mut max_w_signature_count = 0usize;
    let mut materialization_work_estimate = 0u128;
    let mut row_signature_histogram =
        materialize_row_signatures.then(HashMap::<(i32, i32), u128>::new);
    let mut w_histogram_cache = HashMap::<[i32; 3], HashMap<(i32, i32), u64>>::new();
    let mut bucket_work = Vec::<(u128, [i32; 3], usize, usize)>::new();
    for (coefficient, uv_states) in &uv_states_by_coefficient {
        let coefficient_orbit = canonical_coefficient_permutation(*coefficient);
        let w_histogram = w_histogram_cache
            .entry(coefficient_orbit)
            .or_insert_with(|| w_completion_histogram(ws, *coefficient));
        total_w_signature_count += w_histogram.len();
        max_w_signature_count = max_w_signature_count.max(w_histogram.len());
        materialization_work_estimate += uv_states.len() as u128 * w_histogram.len() as u128;
        bucket_work.push((
            uv_states.len() as u128 * w_histogram.len() as u128,
            *coefficient,
            uv_states.len(),
            w_histogram.len(),
        ));
        if let Some(row_signature_histogram) = row_signature_histogram.as_mut() {
            for ((norm_uv, base_term), uv_count) in uv_states.iter() {
                for ((norm_w, w_term), w_count) in w_histogram.iter() {
                    *row_signature_histogram
                        .entry((norm_uv + *norm_w, base_term + *w_term))
                        .or_default() += *uv_count as u128 * *w_count as u128;
                }
            }
        }
    }
    bucket_work.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let bucket_convolution_samples = bucket_work
        .iter()
        .take(sample_bucket_count)
        .map(|(work, coefficient, uv_state_count, w_signature_count)| {
            let coefficient_orbit = canonical_coefficient_permutation(*coefficient);
            let uv_states = uv_states_by_coefficient
                .get(coefficient)
                .expect("bucket work coefficient must exist");
            let w_histogram = w_histogram_cache
                .get(&coefficient_orbit)
                .expect("bucket work W histogram must exist");
            let local_unique = local_row_signature_count(uv_states, w_histogram);
            format!(
                "[{},{},{}]:uv_states={},w_signatures={},work={},local_unique={},local_compression={:.4}",
                coefficient[0],
                coefficient[1],
                coefficient[2],
                uv_state_count,
                w_signature_count,
                work,
                local_unique,
                fraction_u128(*work, local_unique as u128)
            )
        })
        .collect::<Vec<_>>();
    BundleRowShiftAnalysis {
        raw_uv_pairs,
        reduced_uv_state_count,
        coefficient_bucket_count,
        coefficient_permutation_orbit_count,
        max_coefficient_permutation_orbit_size,
        max_coefficient_bucket_mass,
        total_w_signature_count,
        max_w_signature_count,
        materialization_work_estimate,
        raw_rows,
        row_signature_histogram,
        bucket_convolution_samples,
    }
}

fn local_row_signature_count(
    uv_states: &HashMap<(i32, i32), u64>,
    w_histogram: &HashMap<(i32, i32), u64>,
) -> usize {
    let mut local_signatures = HashSet::<(i32, i32)>::new();
    for (norm_uv, base_term) in uv_states.keys() {
        for (norm_w, w_term) in w_histogram.keys() {
            local_signatures.insert((norm_uv + norm_w, base_term + w_term));
        }
    }
    local_signatures.len()
}

fn bundle_row_shift_frontier_buckets(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
    form: RowShiftLinearForm,
) -> Vec<RowShiftFrontierBucket> {
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let ws = triples_by_sum.get(&bundle[2]).unwrap();
    let mut uv_states_by_coefficient = HashMap::<[i32; 3], HashMap<(i32, i32), u64>>::new();
    for u in us {
        let norm_u = dot3(*u, *u);
        for v in vs {
            let coefficient = (form.coefficient)(*u, *v);
            let base_term = (form.base_term)(*u, *v);
            *uv_states_by_coefficient
                .entry(coefficient)
                .or_default()
                .entry((norm_u + dot3(*v, *v), base_term))
                .or_default() += 1;
        }
    }

    let mut w_histogram_cache = HashMap::<[i32; 3], HashMap<(i32, i32), u64>>::new();
    let mut buckets = Vec::with_capacity(uv_states_by_coefficient.len());
    for (coefficient, uv_states) in uv_states_by_coefficient {
        let coefficient_orbit = canonical_coefficient_permutation(coefficient);
        let w_histogram = w_histogram_cache
            .entry(coefficient_orbit)
            .or_insert_with(|| w_completion_histogram(ws, coefficient));
        let uv_norm_values = unique_projection(uv_states.keys().map(|(norm, _)| *norm));
        let uv_shift_values = unique_projection(uv_states.keys().map(|(_, shift)| *shift));
        let w_norm_values = unique_projection(w_histogram.keys().map(|(norm, _)| *norm));
        let w_shift_values = unique_projection(w_histogram.keys().map(|(_, shift)| *shift));
        let norm_bits =
            bounded_sumset_bitset(&uv_norm_values, &w_norm_values, 0, LP333_ROW_NORM_TARGET);
        let shift_bits = bounded_sumset_bitset(
            &uv_shift_values,
            &w_shift_values,
            -LP333_ROW_SHIFT_BOUND,
            LP333_ROW_SHIFT_BOUND,
        );
        let (norm_min, norm_max) = bitset_value_bounds(&norm_bits, 0).unwrap_or((0, 0));
        let (shift_min, shift_max) =
            bitset_value_bounds(&shift_bits, -LP333_ROW_SHIFT_BOUND).unwrap_or((0, 0));
        buckets.push(RowShiftFrontierBucket {
            coefficient,
            uv_state_count: uv_states.len(),
            w_signature_count: w_histogram.len(),
            materialization_work: uv_states.len() as u128 * w_histogram.len() as u128,
            norm_value_count: bitset_count(&norm_bits),
            shift_value_count: bitset_count(&shift_bits),
            norm_min,
            norm_max,
            shift_min,
            shift_max,
            norm_bits,
            shift_bits,
        });
    }
    buckets.sort_unstable_by(|a, b| a.coefficient.cmp(&b.coefficient));
    buckets
}

fn row_shift_frontier_join_analysis(
    left: &[RowShiftFrontierBucket],
    right: &[RowShiftFrontierBucket],
    norm_target: i32,
    shift_target: i32,
    sample_bucket_count: usize,
) -> RowShiftFrontierJoinAnalysis {
    let right_word_count = bitset_word_count(right.len());
    let right_norm_index =
        right_bucket_value_index(right, |bucket| &bucket.norm_bits, 0, LP333_ROW_NORM_TARGET);
    let right_shift_index = right_bucket_value_index(
        right,
        |bucket| &bucket.shift_bits,
        -LP333_ROW_SHIFT_BOUND,
        LP333_ROW_SHIFT_BOUND,
    );
    let mut interval_survivor_count = 0u128;
    let mut norm_survivor_count = 0u128;
    let mut shift_survivor_count = 0u128;
    let mut norm_shift_survivor_count = 0u128;
    let mut active_left = vec![false; left.len()];
    let mut active_right_bits = zero_bitset_for_len(right.len());
    let mut bucket_pair_samples = Vec::<(u128, RowShiftFrontierBucketPairSample)>::new();

    for (left_index, left_bucket) in left.iter().enumerate() {
        for right_bucket in right {
            if left_bucket.norm_value_count > 0
                && right_bucket.norm_value_count > 0
                && left_bucket.shift_value_count > 0
                && right_bucket.shift_value_count > 0
                && left_bucket.norm_min + right_bucket.norm_min <= norm_target
                && norm_target <= left_bucket.norm_max + right_bucket.norm_max
                && left_bucket.shift_min + right_bucket.shift_min <= shift_target
                && shift_target <= left_bucket.shift_max + right_bucket.shift_max
            {
                interval_survivor_count += 1;
            }
        }

        let mut norm_candidates = vec![0u64; right_word_count];
        let mut shift_candidates = vec![0u64; right_word_count];
        or_candidate_buckets_for_complements(
            &mut norm_candidates,
            &left_bucket.norm_bits,
            0,
            norm_target,
            &right_norm_index,
        );
        or_candidate_buckets_for_complements(
            &mut shift_candidates,
            &left_bucket.shift_bits,
            -LP333_ROW_SHIFT_BOUND,
            shift_target,
            &right_shift_index,
        );
        norm_survivor_count += bitset_count(&norm_candidates) as u128;
        shift_survivor_count += bitset_count(&shift_candidates) as u128;

        let norm_shift_candidates = bitset_and(&norm_candidates, &shift_candidates);
        let survivor_count = bitset_count(&norm_shift_candidates);
        norm_shift_survivor_count += survivor_count as u128;
        if survivor_count > 0 {
            active_left[left_index] = true;
            or_into(&mut active_right_bits, &norm_shift_candidates);
            for right_index in bitset_indices(&norm_shift_candidates, right.len()) {
                let right_bucket = &right[right_index];
                let pair_work =
                    left_bucket.materialization_work + right_bucket.materialization_work;
                if bucket_pair_samples.len() < sample_bucket_count {
                    bucket_pair_samples.push(frontier_bucket_pair_sample(
                        left_bucket,
                        right_bucket,
                        pair_work,
                    ));
                    bucket_pair_samples.sort_unstable_by(|a, b| b.0.cmp(&a.0));
                } else if let Some(last) = bucket_pair_samples.last() {
                    if pair_work > last.0 {
                        bucket_pair_samples.pop();
                        bucket_pair_samples.push(frontier_bucket_pair_sample(
                            left_bucket,
                            right_bucket,
                            pair_work,
                        ));
                        bucket_pair_samples.sort_unstable_by(|a, b| b.0.cmp(&a.0));
                    }
                }
            }
        }
    }

    let active_left_materialization_work = left
        .iter()
        .zip(active_left.iter())
        .filter(|(_, active)| **active)
        .map(|(bucket, _)| bucket.materialization_work)
        .sum();
    let active_right_materialization_work = right
        .iter()
        .enumerate()
        .filter(|(index, _)| bitset_get(&active_right_bits, *index))
        .map(|(_, bucket)| bucket.materialization_work)
        .sum();
    let active_left_indices = active_left
        .iter()
        .enumerate()
        .filter_map(|(index, active)| active.then_some(index))
        .collect::<Vec<_>>();
    let active_right_indices = bitset_indices(&active_right_bits, right.len());

    RowShiftFrontierJoinAnalysis {
        bucket_pair_count: left.len() as u128 * right.len() as u128,
        interval_survivor_count,
        norm_survivor_count,
        shift_survivor_count,
        norm_shift_survivor_count,
        active_left_bucket_count: active_left_indices.len(),
        active_right_bucket_count: active_right_indices.len(),
        active_left_materialization_work,
        active_right_materialization_work,
        active_left_indices,
        active_right_indices,
        bucket_pair_samples: bucket_pair_samples
            .into_iter()
            .map(|(_, sample)| sample)
            .collect(),
    }
}

fn frontier_bucket_pair_sample(
    left: &RowShiftFrontierBucket,
    right: &RowShiftFrontierBucket,
    pair_work: u128,
) -> (u128, RowShiftFrontierBucketPairSample) {
    (
        pair_work,
        RowShiftFrontierBucketPairSample {
            left_coefficient: left.coefficient,
            right_coefficient: right.coefficient,
            left_uv_state_count: left.uv_state_count,
            right_uv_state_count: right.uv_state_count,
            left_w_signature_count: left.w_signature_count,
            right_w_signature_count: right.w_signature_count,
            left_materialization_work: left.materialization_work,
            right_materialization_work: right.materialization_work,
            left_norm_value_count: left.norm_value_count,
            right_norm_value_count: right.norm_value_count,
            left_shift_value_count: left.shift_value_count,
            right_shift_value_count: right.shift_value_count,
            pair_work,
        },
    )
}

fn frontier_join_sample_summaries(
    samples: &[RowShiftFrontierBucketPairSample],
    left_bundle: [i32; 3],
    right_bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
    form: RowShiftLinearForm,
    norm_target: i32,
    shift_target: i32,
) -> Vec<String> {
    samples
        .iter()
        .map(|sample| {
            let left_histogram = row_shift_bucket_row_signature_histogram(
                left_bundle,
                triples_by_sum,
                form,
                sample.left_coefficient,
            );
            let right_histogram = row_shift_bucket_row_signature_histogram(
                right_bundle,
                triples_by_sum,
                form,
                sample.right_coefficient,
            );
            let join_count =
                row_shift_join_count(&left_histogram, &right_histogram, norm_target, shift_target);
            let raw_pair_work = sample.left_materialization_work * sample.right_materialization_work;
            format!(
                "left={}:uv_states={},w_signatures={},work={},norm_values={},shift_values={},row_signatures={}|right={}:uv_states={},w_signatures={},work={},norm_values={},shift_values={},row_signatures={},pair_work={},raw_pair_work={},exact_join_count={},exact_local_survival={:.12},exact_local_reduction={:.4}",
                format_coefficient(sample.left_coefficient),
                sample.left_uv_state_count,
                sample.left_w_signature_count,
                sample.left_materialization_work,
                sample.left_norm_value_count,
                sample.left_shift_value_count,
                left_histogram.len(),
                format_coefficient(sample.right_coefficient),
                sample.right_uv_state_count,
                sample.right_w_signature_count,
                sample.right_materialization_work,
                sample.right_norm_value_count,
                sample.right_shift_value_count,
                right_histogram.len(),
                sample.pair_work,
                raw_pair_work,
                join_count,
                fraction_u128(join_count, raw_pair_work),
                reduction_factor(raw_pair_work, join_count)
            )
        })
        .collect()
}

fn row_shift_bucket_row_signature_histogram(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
    form: RowShiftLinearForm,
    coefficient: [i32; 3],
) -> HashMap<(i32, i32), u128> {
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let ws = triples_by_sum.get(&bundle[2]).unwrap();
    let mut uv_states = HashMap::<(i32, i32), u64>::new();
    for u in us {
        let norm_u = dot3(*u, *u);
        for v in vs {
            if (form.coefficient)(*u, *v) == coefficient {
                let base_term = (form.base_term)(*u, *v);
                *uv_states
                    .entry((norm_u + dot3(*v, *v), base_term))
                    .or_default() += 1;
            }
        }
    }
    let w_histogram = w_completion_histogram(ws, coefficient);
    let mut row_histogram = HashMap::<(i32, i32), u128>::new();
    for ((norm_uv, base_term), uv_count) in uv_states.iter() {
        for ((norm_w, w_term), w_count) in w_histogram.iter() {
            *row_histogram
                .entry((norm_uv + *norm_w, base_term + *w_term))
                .or_default() += *uv_count as u128 * *w_count as u128;
        }
    }
    row_histogram
}

fn row_shift_active_bucket_row_signature_histogram(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
    form: RowShiftLinearForm,
    frontier_buckets: &[RowShiftFrontierBucket],
    active_indices: &[usize],
) -> HashMap<(i32, i32), u128> {
    let active_coefficients = active_indices
        .iter()
        .map(|index| frontier_buckets[*index].coefficient)
        .collect::<HashSet<_>>();
    if active_coefficients.is_empty() {
        return HashMap::new();
    }

    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let ws = triples_by_sum.get(&bundle[2]).unwrap();
    let mut uv_states_by_coefficient = HashMap::<[i32; 3], HashMap<(i32, i32), u64>>::new();
    for u in us {
        let norm_u = dot3(*u, *u);
        for v in vs {
            let coefficient = (form.coefficient)(*u, *v);
            if active_coefficients.contains(&coefficient) {
                let base_term = (form.base_term)(*u, *v);
                *uv_states_by_coefficient
                    .entry(coefficient)
                    .or_default()
                    .entry((norm_u + dot3(*v, *v), base_term))
                    .or_default() += 1;
            }
        }
    }

    let mut row_histogram = HashMap::<(i32, i32), u128>::new();
    let mut w_histogram_cache = HashMap::<[i32; 3], HashMap<(i32, i32), u64>>::new();
    for (coefficient, uv_states) in uv_states_by_coefficient {
        let coefficient_orbit = canonical_coefficient_permutation(coefficient);
        let w_histogram = w_histogram_cache
            .entry(coefficient_orbit)
            .or_insert_with(|| w_completion_histogram(ws, coefficient));
        for ((norm_uv, base_term), uv_count) in uv_states.iter() {
            for ((norm_w, w_term), w_count) in w_histogram.iter() {
                *row_histogram
                    .entry((norm_uv + *norm_w, base_term + *w_term))
                    .or_default() += *uv_count as u128 * *w_count as u128;
            }
        }
    }
    row_histogram
}

fn unique_projection(values: impl Iterator<Item = i32>) -> Vec<i32> {
    let mut values = values.collect::<Vec<_>>();
    values.sort_unstable();
    values.dedup();
    values
}

fn bounded_sumset_bitset(left: &[i32], right: &[i32], min_value: i32, max_value: i32) -> Vec<u64> {
    let bit_len = (max_value - min_value + 1) as usize;
    let mut result = zero_bitset_for_len(bit_len);
    let (shifts, source_values) = if left.len() <= right.len() {
        (left, right)
    } else {
        (right, left)
    };
    let source_bits = values_to_bounded_bitset(source_values, min_value, max_value);
    for shift in shifts {
        or_shifted_bits(&mut result, &source_bits, *shift, bit_len);
    }
    mask_last_word(&mut result, bit_len);
    result
}

fn values_to_bounded_bitset(values: &[i32], min_value: i32, max_value: i32) -> Vec<u64> {
    let bit_len = (max_value - min_value + 1) as usize;
    let mut bits = zero_bitset_for_len(bit_len);
    for value in values {
        if (min_value..=max_value).contains(value) {
            set_bit(&mut bits, (*value - min_value) as usize);
        }
    }
    bits
}

fn right_bucket_value_index(
    buckets: &[RowShiftFrontierBucket],
    bits: fn(&RowShiftFrontierBucket) -> &[u64],
    min_value: i32,
    max_value: i32,
) -> Vec<Vec<u64>> {
    let domain_len = (max_value - min_value + 1) as usize;
    let mut index = vec![zero_bitset_for_len(buckets.len()); domain_len];
    for (bucket_index, bucket) in buckets.iter().enumerate() {
        for value_index in bitset_indices(bits(bucket), domain_len) {
            set_bit(&mut index[value_index], bucket_index);
        }
    }
    index
}

fn or_candidate_buckets_for_complements(
    candidates: &mut [u64],
    left_values: &[u64],
    domain_min: i32,
    target: i32,
    right_index: &[Vec<u64>],
) {
    for left_value_index in bitset_indices(left_values, right_index.len()) {
        let left_value = domain_min + left_value_index as i32;
        let needed = target - left_value;
        let needed_index = needed - domain_min;
        if needed_index >= 0 {
            if let Some(right_buckets) = right_index.get(needed_index as usize) {
                or_into(candidates, right_buckets);
            }
        }
    }
}

fn bitset_word_count(bit_len: usize) -> usize {
    bit_len.div_ceil(64)
}

fn zero_bitset_for_len(bit_len: usize) -> Vec<u64> {
    vec![0u64; bitset_word_count(bit_len)]
}

fn set_bit(bits: &mut [u64], index: usize) {
    bits[index / 64] |= 1u64 << (index % 64);
}

fn bitset_get(bits: &[u64], index: usize) -> bool {
    bits.get(index / 64)
        .map(|word| word & (1u64 << (index % 64)) != 0)
        .unwrap_or(false)
}

fn bitset_count(bits: &[u64]) -> usize {
    bits.iter().map(|word| word.count_ones() as usize).sum()
}

fn bitset_indices(bits: &[u64], bit_len: usize) -> Vec<usize> {
    let mut indices = Vec::new();
    for (word_index, word) in bits.iter().enumerate() {
        let mut word = *word;
        while word != 0 {
            let bit = word.trailing_zeros() as usize;
            let index = word_index * 64 + bit;
            if index < bit_len {
                indices.push(index);
            }
            word &= word - 1;
        }
    }
    indices
}

fn bitset_and(left: &[u64], right: &[u64]) -> Vec<u64> {
    left.iter()
        .zip(right.iter())
        .map(|(left_word, right_word)| left_word & right_word)
        .collect()
}

fn or_into(target: &mut [u64], source: &[u64]) {
    for (target_word, source_word) in target.iter_mut().zip(source.iter()) {
        *target_word |= *source_word;
    }
}

fn bitset_value_bounds(bits: &[u64], min_value: i32) -> Option<(i32, i32)> {
    let first = bits
        .iter()
        .enumerate()
        .find(|(_, word)| **word != 0)
        .map(|(word_index, word)| word_index * 64 + word.trailing_zeros() as usize)?;
    let last = bits
        .iter()
        .enumerate()
        .rev()
        .find(|(_, word)| **word != 0)
        .map(|(word_index, word)| word_index * 64 + (63 - word.leading_zeros() as usize))?;
    Some((min_value + first as i32, min_value + last as i32))
}

fn or_shifted_bits(target: &mut [u64], source: &[u64], shift: i32, bit_len: usize) {
    if shift.unsigned_abs() as usize >= bit_len {
        return;
    }
    if shift >= 0 {
        let shift = shift as usize;
        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        for (source_index, word) in source.iter().enumerate() {
            if *word == 0 {
                continue;
            }
            let target_index = source_index + word_shift;
            if target_index < target.len() {
                target[target_index] |= word << bit_shift;
            }
            if bit_shift != 0 && target_index + 1 < target.len() {
                target[target_index + 1] |= word >> (64 - bit_shift);
            }
        }
    } else {
        let shift = (-shift) as usize;
        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        for (source_index, word) in source.iter().enumerate().skip(word_shift) {
            if *word == 0 {
                continue;
            }
            let target_index = source_index - word_shift;
            target[target_index] |= word >> bit_shift;
            if bit_shift != 0 && target_index > 0 {
                target[target_index - 1] |= word << (64 - bit_shift);
            }
        }
    }
    mask_last_word(target, bit_len);
}

fn mask_last_word(bits: &mut [u64], bit_len: usize) {
    let excess = bit_len % 64;
    if excess == 0 {
        return;
    }
    if let Some(last) = bits.last_mut() {
        *last &= (1u64 << excess) - 1;
    }
}

fn bundle_two_row_shifts_analysis(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
    first_index: usize,
    second_index: usize,
    sample_bucket_count: usize,
) -> BundleTwoRowShiftsAnalysis {
    let forms = row_shift_linear_forms();
    let first = forms[first_index];
    let second = forms[second_index];
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let ws = triples_by_sum.get(&bundle[2]).unwrap();
    let raw_uv_pairs = us.len() as u64 * vs.len() as u64;
    let mut states_by_coefficient = HashMap::<[[i32; 3]; 2], HashMap<(i32, [i32; 2]), u64>>::new();
    for u in us {
        let norm_u = dot3(*u, *u);
        for v in vs {
            let norm_uv = norm_u + dot3(*v, *v);
            let base_terms = [(first.base_term)(*u, *v), (second.base_term)(*u, *v)];
            let coefficients = [(first.coefficient)(*u, *v), (second.coefficient)(*u, *v)];
            *states_by_coefficient
                .entry(coefficients)
                .or_default()
                .entry((norm_uv, base_terms))
                .or_default() += 1;
        }
    }
    let reduced_uv_state_count = states_by_coefficient.values().map(HashMap::len).sum();
    let coefficient_bucket_count = states_by_coefficient.len();
    let coefficient_permutation_orbits =
        two_shift_coefficient_orbit_distribution(states_by_coefficient.keys().copied());
    let coefficient_permutation_orbit_count = coefficient_permutation_orbits.len();
    let max_coefficient_permutation_orbit_size = coefficient_permutation_orbits
        .values()
        .copied()
        .max()
        .unwrap_or(0);
    let max_coefficient_bucket_mass = states_by_coefficient
        .values()
        .map(|states| states.values().copied().sum::<u64>())
        .max()
        .unwrap_or(0);
    let mut raw_w_materialization_work = 0u128;
    let mut bucket_work = Vec::<(u128, [[i32; 3]; 2], usize)>::new();
    for (coefficients, uv_states) in &states_by_coefficient {
        let work = uv_states.len() as u128 * ws.len() as u128;
        raw_w_materialization_work += work;
        bucket_work.push((work, *coefficients, uv_states.len()));
    }
    bucket_work.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let bucket_samples = bucket_work
        .iter()
        .take(sample_bucket_count)
        .map(|(work, coefficients, uv_state_count)| {
            let uv_states = states_by_coefficient
                .get(coefficients)
                .expect("two-shift bucket coefficient must exist");
            let w_histogram = two_shift_w_completion_histogram(ws, *coefficients);
            let local_unique = local_two_shift_row_signature_count(uv_states, &w_histogram);
            format!(
                "{}:uv_states={},w_signatures={},work={},local_unique={},local_compression={:.4}",
                format_two_shift_coefficients(*coefficients),
                uv_state_count,
                w_histogram.len(),
                work,
                local_unique,
                fraction_u128(*work, local_unique as u128)
            )
        })
        .collect::<Vec<_>>();
    BundleTwoRowShiftsAnalysis {
        label: format!("{}_{}", first.label, second.label),
        raw_uv_pairs,
        reduced_uv_state_count,
        coefficient_bucket_count,
        coefficient_permutation_orbit_count,
        max_coefficient_permutation_orbit_size,
        max_coefficient_bucket_mass,
        raw_w_materialization_work,
        bucket_samples,
    }
}

fn two_shift_w_completion_histogram(
    ws: &[[i32; 3]],
    coefficients: [[i32; 3]; 2],
) -> HashMap<(i32, [i32; 2]), u64> {
    let mut histogram = HashMap::<(i32, [i32; 2]), u64>::new();
    for w in ws {
        *histogram
            .entry((
                dot3(*w, *w),
                [dot3(*w, coefficients[0]), dot3(*w, coefficients[1])],
            ))
            .or_default() += 1;
    }
    histogram
}

fn local_two_shift_row_signature_count(
    uv_states: &HashMap<(i32, [i32; 2]), u64>,
    w_histogram: &HashMap<(i32, [i32; 2]), u64>,
) -> usize {
    let mut local_signatures = HashSet::<(i32, [i32; 2])>::new();
    for (norm_uv, base_terms) in uv_states.keys() {
        for (norm_w, w_terms) in w_histogram.keys() {
            local_signatures.insert((
                norm_uv + norm_w,
                [base_terms[0] + w_terms[0], base_terms[1] + w_terms[1]],
            ));
        }
    }
    local_signatures.len()
}

fn bundle_all_row_shifts_analysis(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
    sample_bucket_count: usize,
) -> BundleAllRowShiftsAnalysis {
    let forms = row_shift_linear_forms();
    let us = triples_by_sum.get(&bundle[0]).unwrap();
    let vs = triples_by_sum.get(&bundle[1]).unwrap();
    let ws = triples_by_sum.get(&bundle[2]).unwrap();
    let raw_uv_pairs = us.len() as u64 * vs.len() as u64;
    let mut states_by_coefficient = HashMap::<[[i32; 3]; 3], HashMap<(i32, [i32; 3]), u64>>::new();
    for u in us {
        let norm_u = dot3(*u, *u);
        for v in vs {
            let norm_uv = norm_u + dot3(*v, *v);
            let base_terms = [
                (forms[0].base_term)(*u, *v),
                (forms[1].base_term)(*u, *v),
                (forms[2].base_term)(*u, *v),
            ];
            let coefficients = [
                (forms[0].coefficient)(*u, *v),
                (forms[1].coefficient)(*u, *v),
                (forms[2].coefficient)(*u, *v),
            ];
            *states_by_coefficient
                .entry(coefficients)
                .or_default()
                .entry((norm_uv, base_terms))
                .or_default() += 1;
        }
    }
    let reduced_uv_state_count = states_by_coefficient.values().map(HashMap::len).sum();
    let coefficient_bucket_count = states_by_coefficient.len();
    let coefficient_permutation_orbits =
        all_shift_coefficient_orbit_distribution(states_by_coefficient.keys().copied());
    let coefficient_permutation_orbit_count = coefficient_permutation_orbits.len();
    let max_coefficient_permutation_orbit_size = coefficient_permutation_orbits
        .values()
        .copied()
        .max()
        .unwrap_or(0);
    let max_coefficient_bucket_mass = states_by_coefficient
        .values()
        .map(|states| states.values().copied().sum::<u64>())
        .max()
        .unwrap_or(0);
    let mut estimated_materialization_work = 0u128;
    let mut bucket_work = Vec::<(u128, [[i32; 3]; 3], usize)>::new();
    for (coefficients, uv_states) in &states_by_coefficient {
        let work = uv_states.len() as u128 * ws.len() as u128;
        estimated_materialization_work += work;
        bucket_work.push((work, *coefficients, uv_states.len()));
    }
    bucket_work.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let bucket_samples = bucket_work
        .iter()
        .take(sample_bucket_count)
        .map(|(work, coefficients, uv_state_count)| {
            let uv_states = states_by_coefficient
                .get(coefficients)
                .expect("all-shifts bucket coefficient must exist");
            let local_unique = local_all_shift_row_signature_count(uv_states, ws, *coefficients);
            format!(
                "{}:uv_states={},w_candidates={},work={},local_unique={},local_compression={:.4}",
                format_all_shift_coefficients(*coefficients),
                uv_state_count,
                ws.len(),
                work,
                local_unique,
                fraction_u128(*work, local_unique as u128)
            )
        })
        .collect::<Vec<_>>();
    BundleAllRowShiftsAnalysis {
        raw_uv_pairs,
        reduced_uv_state_count,
        coefficient_bucket_count,
        coefficient_permutation_orbit_count,
        max_coefficient_permutation_orbit_size,
        max_coefficient_bucket_mass,
        estimated_materialization_work,
        bucket_samples,
    }
}

fn local_all_shift_row_signature_count(
    uv_states: &HashMap<(i32, [i32; 3]), u64>,
    ws: &[[i32; 3]],
    coefficients: [[i32; 3]; 3],
) -> usize {
    let mut local_signatures = HashSet::<(i32, [i32; 3])>::new();
    for (norm_uv, base_terms) in uv_states.keys() {
        for w in ws {
            local_signatures.insert((
                norm_uv + dot3(*w, *w),
                [
                    base_terms[0] + dot3(*w, coefficients[0]),
                    base_terms[1] + dot3(*w, coefficients[1]),
                    base_terms[2] + dot3(*w, coefficients[2]),
                ],
            ));
        }
    }
    local_signatures.len()
}

fn all_shift_coefficient_orbit_distribution(
    coefficients: impl Iterator<Item = [[i32; 3]; 3]>,
) -> HashMap<[[i32; 3]; 3], usize> {
    let mut distribution = HashMap::<[[i32; 3]; 3], usize>::new();
    for coefficient in coefficients {
        *distribution
            .entry(canonical_all_shift_coefficient_permutation(coefficient))
            .or_default() += 1;
    }
    distribution
}

fn two_shift_coefficient_orbit_distribution(
    coefficients: impl Iterator<Item = [[i32; 3]; 2]>,
) -> HashMap<[[i32; 3]; 2], usize> {
    let mut distribution = HashMap::<[[i32; 3]; 2], usize>::new();
    for coefficient in coefficients {
        *distribution
            .entry(canonical_two_shift_coefficient_permutation(coefficient))
            .or_default() += 1;
    }
    distribution
}

fn canonical_two_shift_coefficient_permutation(coefficients: [[i32; 3]; 2]) -> [[i32; 3]; 2] {
    let permutations = [
        [0usize, 1usize, 2usize],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let mut best = coefficients;
    for permutation in permutations {
        let permuted = [
            permute3(coefficients[0], permutation),
            permute3(coefficients[1], permutation),
        ];
        if permuted < best {
            best = permuted;
        }
    }
    best
}

fn canonical_all_shift_coefficient_permutation(coefficients: [[i32; 3]; 3]) -> [[i32; 3]; 3] {
    let permutations = [
        [0usize, 1usize, 2usize],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let mut best = coefficients;
    for permutation in permutations {
        let permuted = [
            permute3(coefficients[0], permutation),
            permute3(coefficients[1], permutation),
            permute3(coefficients[2], permutation),
        ];
        if permuted < best {
            best = permuted;
        }
    }
    best
}

fn permute3(values: [i32; 3], permutation: [usize; 3]) -> [i32; 3] {
    [
        values[permutation[0]],
        values[permutation[1]],
        values[permutation[2]],
    ]
}

fn format_two_shift_coefficients(coefficients: [[i32; 3]; 2]) -> String {
    format!(
        "a=[{},{},{}],b=[{},{},{}]",
        coefficients[0][0],
        coefficients[0][1],
        coefficients[0][2],
        coefficients[1][0],
        coefficients[1][1],
        coefficients[1][2]
    )
}

fn format_coefficient(coefficient: [i32; 3]) -> String {
    format!("[{},{},{}]", coefficient[0], coefficient[1], coefficient[2])
}

fn format_all_shift_coefficients(coefficients: [[i32; 3]; 3]) -> String {
    format!(
        "s1=[{},{},{}],s2=[{},{},{}],s4=[{},{},{}]",
        coefficients[0][0],
        coefficients[0][1],
        coefficients[0][2],
        coefficients[1][0],
        coefficients[1][1],
        coefficients[1][2],
        coefficients[2][0],
        coefficients[2][1],
        coefficients[2][2]
    )
}

fn coefficient_permutation_orbit_distribution(
    coefficients: impl Iterator<Item = [i32; 3]>,
) -> HashMap<[i32; 3], usize> {
    let mut distribution = HashMap::<[i32; 3], usize>::new();
    for coefficient in coefficients {
        *distribution
            .entry(canonical_coefficient_permutation(coefficient))
            .or_default() += 1;
    }
    distribution
}

fn canonical_coefficient_permutation(mut coefficient: [i32; 3]) -> [i32; 3] {
    coefficient.sort_unstable();
    coefficient
}

fn w_completion_histogram(ws: &[[i32; 3]], coefficient: [i32; 3]) -> HashMap<(i32, i32), u64> {
    let mut histogram = HashMap::<(i32, i32), u64>::new();
    for w in ws {
        *histogram
            .entry((dot3(*w, *w), dot3(*w, coefficient)))
            .or_default() += 1;
    }
    histogram
}

fn row_shift_join_count(
    left: &HashMap<(i32, i32), u128>,
    right: &HashMap<(i32, i32), u128>,
    norm_target: i32,
    shift_target: i32,
) -> u128 {
    left.iter()
        .map(|((norm, shift), count)| {
            count
                * right
                    .get(&(norm_target - *norm, shift_target - *shift))
                    .copied()
                    .unwrap_or(0)
        })
        .sum()
}

fn histogram_collision_summary(histogram: &HashMap<u128, u64>) -> String {
    let unique = histogram.len() as u64;
    let raw = histogram.values().copied().sum::<u64>();
    let singleton_keys = histogram.values().filter(|&&count| count == 1).count() as u64;
    let reused_keys = unique.saturating_sub(singleton_keys);
    let collision_excess = raw.saturating_sub(unique);
    let max_multiplicity = histogram.values().copied().max().unwrap_or(0);
    format!(
        "raw={raw},unique={unique},compression={:.4},collision_excess={collision_excess},singleton_keys={singleton_keys},reused_keys={reused_keys},max_multiplicity={max_multiplicity}",
        ratio(raw as usize, unique as usize)
    )
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn fraction_u128(numerator: u128, denominator: u128) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn reduction_factor(before: u128, after: u128) -> f64 {
    if after == 0 {
        0.0
    } else {
        before as f64 / after as f64
    }
}

fn aggregate_count_summary(raw: u128, unique: usize) -> String {
    let unique = unique as u128;
    format!(
        "raw={raw},unique={unique},compression={:.4}",
        fraction_u128(raw, unique)
    )
}

fn w_signature_summary(
    coefficient_bucket_count: usize,
    total_w_signature_count: usize,
    max_w_signature_count: usize,
    materialization_work_estimate: u128,
) -> String {
    let average = if coefficient_bucket_count == 0 {
        0.0
    } else {
        total_w_signature_count as f64 / coefficient_bucket_count as f64
    };
    format!(
        "coefficient_buckets={coefficient_bucket_count},total_w_signatures={total_w_signature_count},avg_w_signatures_per_bucket={average:.4},max_w_signature_count={max_w_signature_count},materialization_work_estimate={materialization_work_estimate}"
    )
}

fn frontier_join_summary(analysis: &RowShiftFrontierJoinAnalysis) -> String {
    format!(
        "bucket_pairs={},interval_survivors={},interval_survival={:.8},norm_survivors={},norm_survival={:.8},shift_survivors={},shift_survival={:.8},norm_shift_survivors={},norm_shift_survival={:.8},active_left_buckets={},active_right_buckets={},active_left_materialization_work={},active_right_materialization_work={}",
        analysis.bucket_pair_count,
        analysis.interval_survivor_count,
        fraction_u128(analysis.interval_survivor_count, analysis.bucket_pair_count),
        analysis.norm_survivor_count,
        fraction_u128(analysis.norm_survivor_count, analysis.bucket_pair_count),
        analysis.shift_survivor_count,
        fraction_u128(analysis.shift_survivor_count, analysis.bucket_pair_count),
        analysis.norm_shift_survivor_count,
        fraction_u128(analysis.norm_shift_survivor_count, analysis.bucket_pair_count),
        analysis.active_left_bucket_count,
        analysis.active_right_bucket_count,
        analysis.active_left_materialization_work,
        analysis.active_right_materialization_work
    )
}

fn all_shift_bucket_summary(analysis: &BundleAllRowShiftsAnalysis) -> String {
    format!(
        "raw={},reduced_states={},coefficient_buckets={},coefficient_permutation_orbits={},coefficient_orbit_reuse_factor={:.4},max_coefficient_permutation_orbit_size={},compression={:.4},max_coefficient_bucket_mass={},estimated_materialization_work={}",
        analysis.raw_uv_pairs,
        analysis.reduced_uv_state_count,
        analysis.coefficient_bucket_count,
        analysis.coefficient_permutation_orbit_count,
        ratio(
            analysis.coefficient_bucket_count,
            analysis.coefficient_permutation_orbit_count
        ),
        analysis.max_coefficient_permutation_orbit_size,
        ratio(analysis.raw_uv_pairs as usize, analysis.reduced_uv_state_count),
        analysis.max_coefficient_bucket_mass,
        analysis.estimated_materialization_work
    )
}

fn two_shift_bucket_summary(analysis: &BundleTwoRowShiftsAnalysis) -> String {
    format!(
        "raw={},reduced_states={},coefficient_buckets={},coefficient_permutation_orbits={},coefficient_orbit_reuse_factor={:.4},max_coefficient_permutation_orbit_size={},compression={:.4},max_coefficient_bucket_mass={},raw_w_materialization_work={}",
        analysis.raw_uv_pairs,
        analysis.reduced_uv_state_count,
        analysis.coefficient_bucket_count,
        analysis.coefficient_permutation_orbit_count,
        ratio(
            analysis.coefficient_bucket_count,
            analysis.coefficient_permutation_orbit_count
        ),
        analysis.max_coefficient_permutation_orbit_size,
        ratio(analysis.raw_uv_pairs as usize, analysis.reduced_uv_state_count),
        analysis.max_coefficient_bucket_mass,
        analysis.raw_w_materialization_work
    )
}

fn reduced_bucket_summary(
    raw: u64,
    reduced_states: usize,
    coefficient_buckets: usize,
    coefficient_permutation_orbits: usize,
    max_coefficient_permutation_orbit_size: usize,
    max_coefficient_bucket_mass: u64,
) -> String {
    format!(
        "raw={raw},reduced_states={reduced_states},coefficient_buckets={coefficient_buckets},coefficient_permutation_orbits={coefficient_permutation_orbits},coefficient_orbit_reuse_factor={:.4},max_coefficient_permutation_orbit_size={max_coefficient_permutation_orbit_size},compression={:.4},max_coefficient_bucket_mass={max_coefficient_bucket_mass}",
        ratio(coefficient_buckets, coefficient_permutation_orbits),
        ratio(raw as usize, reduced_states)
    )
}

fn validate_crt_row_bundle(
    bundle: [i32; 3],
    triples_by_sum: &BTreeMap<i32, Vec<[i32; 3]>>,
) -> Result<(), String> {
    if bundle.into_iter().sum::<i32>() != 1 {
        return Err(format!(
            "expected row bundle with total sum 1, got [{},{},{}]",
            bundle[0], bundle[1], bundle[2]
        ));
    }
    for value in bundle {
        if !triples_by_sum.contains_key(&value) {
            return Err(format!(
                "bundle entry {value} is not a valid length-37 odd row sum"
            ));
        }
    }
    Ok(())
}

fn crt_component_by_hub(hub: [i32; 3]) -> Result<([i32; 3], [i32; 3], [i32; 3]), String> {
    match canonical_bundle_rotation(hub) {
        [-15, 5, 11] => Ok(([-15, 5, 11], [-5, -1, 7], [-5, 7, -1])),
        [-13, -1, 15] => Ok(([-13, -1, 15], [-5, 1, 5], [-5, 5, 1])),
        [-9, -5, 15] => Ok(([-9, -5, 15], [-5, -3, 9], [-5, 9, -3])),
        other => Err(format!(
            "unsupported CRT component hub [{},{},{}]; supported hubs are [-15,5,11], [-13,-1,15], [-9,-5,15]",
            other[0], other[1], other[2]
        )),
    }
}

fn parse_i32_triple(value: &str) -> Result<[i32; 3], String> {
    let parts = value.split(',').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(format!("expected comma-separated triple, got `{value}`"));
    }
    let mut triple = [0i32; 3];
    for (slot, part) in triple.iter_mut().zip(parts) {
        *slot = part
            .parse::<i32>()
            .map_err(|_| format!("invalid integer triple `{value}`"))?;
    }
    Ok(triple)
}

fn parse_usize_flag(value: &str, flag: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|_| format!("invalid integer value `{value}` for {flag}"))
}

fn parse_row_shift_flag(value: &str) -> Result<i32, String> {
    let shift = value
        .parse::<i32>()
        .map_err(|_| format!("invalid row shift `{value}`; expected one of 1, 2, 4"))?;
    match shift {
        1 | 2 | 4 => Ok(shift),
        _ => Err(format!(
            "invalid row shift `{value}`; expected one of 1, 2, 4"
        )),
    }
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
                *next.entry(*prefix_norm + *component_norm).or_default() +=
                    prefix_count * component_count;
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
        .map(|(norm, count)| {
            count
                * right_norm_counts
                    .get(&(combined_target - *norm))
                    .copied()
                    .unwrap_or(0)
        })
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
    let length = parse_usize_flag_or_config(
        &args,
        "--length",
        file_config.as_ref().and_then(|cfg| cfg.length),
    )?;
    let compression = parse_usize_flag_or_default(
        &args,
        "--compression",
        file_config.as_ref().and_then(|cfg| cfg.compression),
        1,
    )?;
    let max_attempts = parse_u64_flag_or_default(
        &args,
        "--max-attempts",
        file_config.as_ref().and_then(|cfg| cfg.max_attempts),
        1000,
    )?;
    let row_sum_target = parse_i32_flag_or_default(
        &args,
        "--row-sum",
        file_config.as_ref().and_then(|cfg| cfg.row_sum_target),
        1,
    )?;
    let shard_spec = find_flag_value(&args, "--shard")
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
    let order = parse_flag_value(&args, "--order")?
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    let block_sizes = parse_flag_value(&args, "--block-sizes")?
        .split(',')
        .map(|value| value.trim().parse::<usize>().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let lambda = parse_flag_value(&args, "--lambda")?
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
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
    let length = parse_flag_value(&args, "--length")?
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
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
    println!(
        "effective_tail_depth={}",
        tail_depth.min(reduced_length).min(12)
    );
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
    let length = parse_flag_value(&args, "--length")?
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
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
        MitmSplitStrategy::Contiguous => {
            (reduced_length / 2, reduced_length - (reduced_length / 2))
        }
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
    println!(
        "right_states_emitted={}",
        outcome.stats.right_states_emitted
    );
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
        "{banner}\n\n{message}\n{usage_label}\n  hadamard analyze lp333-crt\n  hadamard analyze lp333-crt-bundle [--bundle a,b,c]\n  hadamard analyze lp333-crt-pair [--left a,b,c] [--right d,e,f] [--left-shift 0|1|2] [--right-shift 0|1|2] [--shift 1|2|4] [--sample-buckets N] [--frontier-join] [--frontier-exact-join] [--two-shifts] [--all-shifts] [--exact-join]\n  hadamard analyze lp333-crt-component [--hub a,b,c]\n  hadamard analyze lp333-multiplier [--col10-shift1]\n  hadamard search lp [--config PATH] --length N [--compression D] [--max-attempts M] [--shard i/n]\n  hadamard search sds --order N --block-sizes k1,k2,k3,k4 --lambda L [--max-matches M] [--shard i/n]\n  hadamard decompress lp --bucket-in PATH [--max-pairs N] [--artifact-out PATH]\n  hadamard verify lp --a +--++ --b +-+-+\n  hadamard build 2cc --a +--++ --b +-+-+\n  hadamard enumerate sds-167\n  hadamard benchmark psd [--sequence +--++] [--backend direct|fft|autocorrelation]\n  hadamard benchmark compressed-pairs --length N [--compression D] [--ordering natural|generator2] [--spectral-frequencies K] [--tail-depth T] [--row-sum R] [--max-pairs M]\n  hadamard benchmark compressed-pairs-mitm --length N [--compression D] [--split contiguous|parity] [--row-sum R] [--max-pairs M]\n  hadamard test-known lp-small|lp-seven|lp-nine|lp-eleven|lp-thirteen"
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
    use super::{
        col10_fixed_shift1_values, col10_fixed_triple_dot_sum_values, col10_self_dot_values,
        parse_lp_search_file_config, row_bundle_pair_survival, row_bundle_triples_for_row_units,
        row_mod3_bundle_pair_masses, row_units_147_full_row_lift_analysis,
        row_units_147_shift0_dot_marginal_feasible, RowUnits147Marginal, LP333_ACTUAL_SHIFT_TARGET,
        LP333_ROW_SHIFT_TARGET,
    };
    use std::collections::BTreeSet;

    #[test]
    fn lp_search_config_parser_reads_key_value_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("hadamard-lp-search.cfg");
        std::fs::write(
            &path,
            "length=333\ncompression=3\nmax_attempts=4096\nrow_sum=1\nshard=0/8\n",
        )
        .expect("write config");
        let parsed =
            parse_lp_search_file_config(path.to_str().expect("utf8 path")).expect("config");
        assert_eq!(parsed.length, Some(333));
        assert_eq!(parsed.compression, Some(3));
        assert_eq!(parsed.max_attempts, Some(4096));
        assert_eq!(parsed.row_sum_target, Some(1));
        assert_eq!(parsed.shard.as_deref(), Some("0/8"));
        std::fs::remove_file(path).expect("remove config");
    }

    #[test]
    fn multiplier_row_action_screen_matches_recorded_row_bundle_counts() {
        let pairs = row_mod3_bundle_pair_masses();
        assert_eq!(pairs.len(), 504);

        let row_units_147 = row_bundle_triples_for_row_units(&[1, 4, 7]);
        assert_eq!(row_units_147.len(), 1064);
        let (surviving_pairs, surviving_mass) = row_bundle_pair_survival(&pairs, &row_units_147);
        assert_eq!(surviving_pairs, 12);
        assert_eq!(surviving_mass, 119_903_105_952);

        let full_column_preserving = row_bundle_triples_for_row_units(&[1, 2, 4, 5, 7, 8]);
        assert_eq!(full_column_preserving.len(), 18);
        let (surviving_pairs, surviving_mass) =
            row_bundle_pair_survival(&pairs, &full_column_preserving);
        assert_eq!(surviving_pairs, 0);
        assert_eq!(surviving_mass, 0);
    }

    #[test]
    fn multiplier_row_units_147_full_row_lift_matches_recorded_counts() {
        let pairs = row_mod3_bundle_pair_masses();
        let lift = row_units_147_full_row_lift_analysis(&pairs, false);
        assert_eq!(lift.bundle_pair_count, 12);
        assert_eq!(lift.active_bundle_count, 7);
        assert_eq!(lift.active_row_marginal_count, 7_467);
        assert_eq!(lift.row_pair_candidate_count, 13_764_060);
        assert_eq!(lift.norm_compatible_row_pair_count, 6_048);
        assert_eq!(lift.exact_row_pair_count, 6_048);
        assert_eq!(lift.shift0_dot_marginal_feasible_row_pair_count, 0);
        assert_eq!(lift.col10_fixed_rows_exact_row_pair_count, 1_296);
        assert_eq!(lift.col10_shift0_e3_feasible_row_pair_count, 1_296);
        assert!(matches!(
            lift.exact_column_trivial_pattern_log10,
            Some(value) if (value - 103.315).abs() < 0.001
        ));
        assert!(matches!(
            lift.col10_fixed_rows_pattern_log10,
            Some(value) if (value - 60.818).abs() < 0.001
        ));
        assert!(matches!(
            lift.col10_shift0_e3_pattern_log10,
            Some(value) if (value - 60.818).abs() < 0.001
        ));
    }

    #[test]
    fn multiplier_shift0_dot_uses_actual_crt_shift_target() {
        let left = RowUnits147Marginal {
            rows: [0; 9],
            norm: 0,
            paf: [0; 9],
            shift0_e1_dot_sums: BTreeSet::from([0]),
            shift0_e3_dot_sums: BTreeSet::from([0]),
            column_trivial_pattern_log10: 0.0,
            col10_fixed_rows_pattern_log10: None,
            col10_shift0_e3_dot_sums: None,
            col10_shift1_dot_sums: None,
        };
        let right = RowUnits147Marginal {
            rows: [0; 9],
            norm: 0,
            paf: [0; 9],
            shift0_e1_dot_sums: BTreeSet::from([LP333_ACTUAL_SHIFT_TARGET]),
            shift0_e3_dot_sums: BTreeSet::from([LP333_ACTUAL_SHIFT_TARGET - 12 * 37]),
            column_trivial_pattern_log10: 0.0,
            col10_fixed_rows_pattern_log10: None,
            col10_shift0_e3_dot_sums: None,
            col10_shift1_dot_sums: None,
        };

        assert_eq!(LP333_ROW_SHIFT_TARGET, 37 * LP333_ACTUAL_SHIFT_TARGET);
        assert!(row_units_147_shift0_dot_marginal_feasible(&left, &right));
    }

    #[test]
    fn col10_orbit_dot_tables_include_extreme_rows() {
        assert_eq!(col10_self_dot_values(37), BTreeSet::from([37]));
        assert_eq!(col10_self_dot_values(-37), BTreeSet::from([37]));
        assert_eq!(col10_fixed_shift1_values(37), BTreeSet::from([37]));
        assert_eq!(col10_fixed_shift1_values(-37), BTreeSet::from([37]));
        assert_eq!(
            col10_fixed_triple_dot_sum_values(37, 37, 37),
            BTreeSet::from([111])
        );
        assert_eq!(
            col10_fixed_triple_dot_sum_values(37, 37, -37),
            BTreeSet::from([-37])
        );
    }
}
