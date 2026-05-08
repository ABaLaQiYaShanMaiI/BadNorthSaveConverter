use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

mod json;
mod model;
mod parser;
mod records;
mod serializer;

use json::{JsonDecoder, JsonEncoder};
use parser::parse;
use serializer::serialize_checked;

#[derive(Parser, Debug)]
#[command(name = "bad-north-save-editor")]
#[command(about = "Bad North save converter (binary <-> JSON)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Convert binary save to JSON
    Bin2json {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    /// Convert JSON back to binary save
    Json2bin {
        input: PathBuf,
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Bin2json { input, output } => {
            let output = output.unwrap_or_else(|| default_json_output(&input));
            bin2json(&input, &output).map(|_| {
                format!(
                    "✓ Successfully converted {} to {}",
                    input.display(),
                    output.display()
                )
            })
        }
        Command::Json2bin { input, output } => {
            let output = output.unwrap_or_else(|| default_binary_output(&input));
            json2bin(&input, &output).map(|_| {
                format!(
                    "✓ Successfully converted {} to {}",
                    input.display(),
                    output.display()
                )
            })
        }
    };

    match result {
        Ok(message) => println!("{}", message),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn default_json_output(input: &Path) -> PathBuf {
    let mut output = input.as_os_str().to_os_string();
    output.push(".json");
    PathBuf::from(output)
}

fn default_binary_output(input: &Path) -> PathBuf {
    let input_str = input.to_string_lossy();
    if let Some(stripped) = input_str.strip_suffix(".json") {
        PathBuf::from(format!("{}.new", stripped))
    } else {
        PathBuf::from(format!("{}.new", input_str))
    }
}

fn bin2json(input: &Path, output: &Path) -> Result<(), String> {
    let bytes = fs::read(input).map_err(|e| format!("Failed to read input file: {}", e))?;

    let rec = parse(&bytes).map_err(|e| format!("Failed to parse binary: {}", e))?;

    let json = JsonEncoder::new(rec).encode();

    let json_str = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    fs::write(output, json_str).map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

fn json2bin(input: &Path, output: &Path) -> Result<(), String> {
    let json_str =
        fs::read_to_string(input).map_err(|e| format!("Failed to read input file: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let rec = JsonDecoder::decode(&json)?;

    let output_bytes = serialize_checked(&rec)
        .map_err(|e| format!("Failed to serialize binary with validation: {}", e))?;

    fs::write(output, output_bytes).map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}
