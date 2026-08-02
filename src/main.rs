use chrono::Utc;
use clap::Parser;
use std::collections::HashMap;
use std::path::Path;
use std::process::exit;

mod exporter;
mod generator;
mod schedule;

use generator::RotationOverride;

// 3 weeks by default
const DEFAULT: usize = 21 * 24 / 2;

// If no start time is specified, use the last even hour
#[must_use] 
pub fn default_timestamp() -> i64 {
    let now = Utc::now().timestamp();
    let block = 2 * 60 * 60;
    now - (now % block)
}

const OVERRIDE_HELP: &str = concat!(
    "Override flags (stateful, order-dependent):\n",
    "  --interval <X>       Set the 1-based rotation index for subsequent flags\n",
    "  --regular <X,Y>      Set RegularStages (exactly 2 IDs)\n",
    "  --ranked <X,Y>       Set GachiStages (exactly 2 IDs)\n",
    "  --rule <RULE>        Set GachiRule (cVar, cVgl, or cVlf)\n",
    "\n",
    "  --regular, --ranked & --rule must follow an --interval.\n",
    "  Missing properties for an interval are filled by the standard RNG logic.\n",
    "  Both --flag value and --flag=value syntaxes are accepted.\n",
    "\n",
    "Example:\n",
    "  ikamaki --yml --count 10 --interval 4 --regular \"1,0\" --ranked \\\n",
    "          \"6,12\" --rule \"cVar\" --interval 6 --regular \"10,13\"",
);

#[derive(Parser, Debug)]
#[command(name = "ikamaki", about = "Friendly Sushi Roll generating rotations for Splatoon", after_help = OVERRIDE_HELP)]
struct Args {
    /// How many rotations to generate
    #[arg(short, long)]
    count: Option<usize>,

    /// UNIX timestamp to start the schedule from
    #[arg(short, long)]
    start: Option<i64>,

    /// Output file path
    #[arg(short, long, default_value = "VSSetting.byaml")]
    output: String,

    /// Generate YAML in addition to the BYAML
    #[arg(long)]
    yml: bool,

    /// Set `DisconnectByMemoryHash` to true
    #[arg(long)]
    hash_kick: bool,
}

const VALID_RULES: &[&str] = &["cVar", "cVgl", "cVlf"];

fn parse_map_ids(raw: &str, flag: &str) -> Vec<i32> {
    let parts: Vec<&str> = raw.split(',').collect();
    if parts.len() != 2 {
        eprintln!(
            "error: {flag} requires exactly 2 comma-separated map IDs, got '{raw}'"
        );
        exit(1);
    }
    let mut ids: Vec<i32> = Vec::with_capacity(2);
    for p in &parts {
        if let Ok(n) = p.trim().parse::<i32>() { ids.push(n) } else {
            eprintln!("error: {} contains invalid map ID '{}'", flag, p.trim());
            exit(1);
        }
    }
    ids
}

fn split_eq_value(arg: &str, flag_name: &str) -> Option<String> {
    let prefix = format!("{flag_name}=");
    if arg.starts_with(&prefix) {
        Some(arg[prefix.len()..].to_string())
    } else {
        None
    }
}

fn require_interval(current: Option<usize>) -> usize {
    if let Some(n) = current { n } else {
        eprintln!("error: --regular/--ranked/--rule must follow an --interval flag");
        exit(1);
    }
}

fn parse_overrides() -> (HashMap<usize, RotationOverride>, Vec<String>) {
    let raw: Vec<String> = std::env::args().collect();
    let mut overrides: HashMap<usize, RotationOverride> = HashMap::new();
    let mut remaining: Vec<String> = Vec::with_capacity(raw.len());
    if let Some(prog) = raw.first() {
        remaining.push(prog.clone());
    }

    let mut current_interval: Option<usize> = None;
    let mut i = 1;
    while i < raw.len() {
        let arg = &raw[i];

        if arg == "--interval" || split_eq_value(arg, "--interval").is_some() {
            let value = if let Some(v) = split_eq_value(arg, "--interval") {
                v
            } else {
                i += 1;
                if i >= raw.len() {
                    eprintln!("error: --interval requires a value");
                    exit(1);
                }
                raw[i].clone()
            };
            if let Ok(n) = value.parse::<usize>() { current_interval = Some(n) } else {
                eprintln!(
                    "error: --interval value '{value}' is not a valid positive integer"
                );
                exit(1);
            }
        } else if arg == "--regular" || split_eq_value(arg, "--regular").is_some() {
            let value = if let Some(v) = split_eq_value(arg, "--regular") {
                v
            } else {
                i += 1;
                if i >= raw.len() {
                    eprintln!("error: --regular requires a value");
                    exit(1);
                }
                raw[i].clone()
            };
            let interval = require_interval(current_interval);
            let ids = parse_map_ids(&value, "--regular");
            overrides.entry(interval).or_default().regular = Some(ids);
        } else if arg == "--ranked" || split_eq_value(arg, "--ranked").is_some() {
            let value = if let Some(v) = split_eq_value(arg, "--ranked") {
                v
            } else {
                i += 1;
                if i >= raw.len() {
                    eprintln!("error: --ranked requires a value");
                    exit(1);
                }
                raw[i].clone()
            };
            let interval = require_interval(current_interval);
            let ids = parse_map_ids(&value, "--ranked");
            overrides.entry(interval).or_default().ranked = Some(ids);
        } else if arg == "--rule" || split_eq_value(arg, "--rule").is_some() {
            let value = if let Some(v) = split_eq_value(arg, "--rule") {
                v
            } else {
                i += 1;
                if i >= raw.len() {
                    eprintln!("error: --rule requires a value");
                    exit(1);
                }
                raw[i].clone()
            };
            let interval = require_interval(current_interval);
            if !VALID_RULES.contains(&value.as_str()) {
                eprintln!(
                    "error: --rule must be one of {VALID_RULES:?}, got '{value}'"
                );
                exit(1);
            }
            overrides.entry(interval).or_default().rule = Some(value);
        } else {
            remaining.push(arg.clone());
        }

        i += 1;
    }

    (overrides, remaining)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (overrides, remaining) = parse_overrides();
    let args = Args::parse_from(remaining);
    let count = args.count.unwrap_or(DEFAULT);
    let start_time = args.start.unwrap_or_else(default_timestamp);
    let schedule = generator::generate(count, start_time, &overrides);
    exporter::export(&schedule, &args.output, args.hash_kick)?;

    if args.yml {
        let yml_path = derive_yml_path(&args.output);
        exporter::export_text(&schedule, &yml_path, args.hash_kick)?;
        println!(
            "Successfully generated {} rotation(s) -> {} + {}",
            count, args.output, yml_path
        );
    } else {
        println!(
            "Successfully generated {} rotation(s) -> {}",
            count, args.output
        );
    }

    Ok(())
}

fn derive_yml_path(output: &str) -> String {
    let path = Path::new(output);
    if let Some(stem) = path.file_stem() {
        if let Some(parent) = path.parent() {
            return parent
                .join(format!("{}.yml", stem.to_string_lossy()))
                .to_string_lossy()
                .into_owned();
        }
        return format!("{}.yml", stem.to_string_lossy());
    }
    format!("{output}.yml")
}
