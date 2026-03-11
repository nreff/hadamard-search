use hadamard_construct::build_two_circulant_hadamard;
use hadamard_core::{
    available_psd_backends, exact_row_sum_square_candidates_167, get_psd_backend, CheckpointState,
    LegendrePair, Sequence,
};
use hadamard_search::{
    decompress_bucket_artifact, parse_bucket_artifact_text, run_legendre_search, DecompressionConfig,
    SearchConfig,
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
    if args.first().map(String::as_str) != Some("lp") {
        return Err("expected `search lp`".to_string());
    }
    let length = parse_flag_value(&args, "--length")?.parse::<usize>().map_err(|e| e.to_string())?;
    let compression = parse_flag_value_or(&args, "--compression", "1")
        .parse::<usize>()
        .map_err(|e| e.to_string())?;
    let max_attempts = parse_flag_value_or(&args, "--max-attempts", "1000")
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    let row_sum_target = parse_flag_value_or(&args, "--row-sum", "1")
        .parse::<i32>()
        .map_err(|e| e.to_string())?;
    let shard_spec = parse_flag_value_or(&args, "--shard", "0/1");
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
    if args.first().map(String::as_str) != Some("psd") {
        return Err("expected `benchmark psd`".to_string());
    }
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

fn cmd_test_known(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("lp-small") => test_known_pair("+--++", "+-+-+"),
        Some("lp-seven") => test_known_pair("+--+-++", "+--+-++"),
        Some("lp-nine") => test_known_pair("+---+-+++", "+--++-+-+"),
        Some("lp-eleven") => test_known_pair("+++-++-+---", "+++-++-+---"),
        _ => Err(
            "expected `test-known lp-small`, `test-known lp-seven`, `test-known lp-nine`, or `test-known lp-eleven`"
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
        "{banner}\n\n{message}\n{usage_label}\n  hadamard search lp --length N [--compression D] [--max-attempts M] [--shard i/n]\n  hadamard decompress lp --bucket-in PATH [--max-pairs N] [--artifact-out PATH]\n  hadamard verify lp --a +--++ --b +-+-+\n  hadamard build 2cc --a +--++ --b +-+-+\n  hadamard enumerate sds-167\n  hadamard benchmark psd [--sequence +--++] [--backend direct|autocorrelation]\n  hadamard test-known lp-small|lp-seven|lp-nine|lp-eleven"
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
