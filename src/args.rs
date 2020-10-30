use serde::Deserialize;
use structopt::StructOpt;

#[derive(Debug, Deserialize, StructOpt)]
#[structopt(name = "velvet")]
pub struct Args {
    #[structopt(short = "c", long = "config-file-path", default_value = "config.json")]
    pub config_file_path: String,
}
