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
        "{banner}\n\n{message}\n{usage_label}\n  hadamard search lp [--config PATH] --length N [--compression D] [--max-attempts M] [--shard i/n]\n  hadamard search sds --order N --block-sizes k1,k2,k3,k4 --lambda L [--max-matches M] [--shard i/n]\n  hadamard decompress lp --bucket-in PATH [--max-pairs N] [--artifact-out PATH]\n  hadamard verify lp --a +--++ --b +-+-+\n  hadamard build 2cc --a +--++ --b +-+-+\n  hadamard enumerate sds-167\n  hadamard benchmark psd [--sequence +--++] [--backend direct|fft|autocorrelation]\n  hadamard benchmark compressed-pairs --length N [--compression D] [--ordering natural|generator2] [--spectral-frequencies K] [--tail-depth T] [--row-sum R] [--max-pairs M]\n  hadamard benchmark compressed-pairs-mitm --length N [--compression D] [--split contiguous|parity] [--row-sum R] [--max-pairs M]\n  hadamard test-known lp-small|lp-seven|lp-nine|lp-eleven|lp-thirteen"
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
