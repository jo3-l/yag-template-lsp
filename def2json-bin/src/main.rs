use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use clap::Parser;
use serde::Serialize;
use yag_template_envdefs::EnvDefSource;

mod discordmd;

/// Export `ydef` files to JSON.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The `ydef` files to process.
    #[clap(required = true)]
    files: Vec<PathBuf>,

    /// Whether to pretty-print the output.
    #[clap(short, long, default_value = "true")]
    pretty: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let sources = args
        .files
        .iter()
        .map(|path| -> anyhow::Result<EnvDefSource> {
            let name = path
                .file_name()
                .with_context(|| anyhow!("invalid file {}", path.display()))?
                .to_string_lossy();
            let contents = fs::read_to_string(path).map_err(|_| anyhow!("failed to read file {}", path.display()))?;
            Ok(EnvDefSource::new(name, contents))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let defs = yag_template_envdefs::parse(&sources).map_err(|err| anyhow!("failed to parse definitions: {err}"))?;
    let processed_defs: HashMap<String, ProcessedFunc> =
        defs.funcs.into_iter().map(|(name, func)| (name, func.into())).collect();

    let serialized = if args.pretty {
        serde_json::to_string_pretty(&processed_defs)
    } else {
        serde_json::to_string(&processed_defs)
    };
    let serialized = serialized.map_err(|err| anyhow!("failed serializing to JSON: {err}"))?;
    println!("{serialized}");

    Ok(())
}

#[derive(Debug, Serialize)]
struct ProcessedFunc {
    pub name: String,
    pub signature: String,
    pub doc: String,
}

impl From<yag_template_envdefs::Func> for ProcessedFunc {
    fn from(f: yag_template_envdefs::Func) -> Self {
        Self {
            name: f.name.clone(),
            signature: f.signature(),
            doc: discordmd::render(&f.doc),
        }
    }
}
