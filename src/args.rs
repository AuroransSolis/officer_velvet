use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Deserialize, Parser)]
#[command(name = "velvet")]
pub struct Args {
    #[arg(short = 'c', long = "config-file-path", default_value = "config.json")]
    pub config_file_path: String,
}
