use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about= None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, global = true)]
    pub author: Option<String>,

    #[arg(short, long, default_value_t = 1, global = true)]
    pub days: u64,

    #[arg(long, default_value_t = false, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Scan {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    Check {
        #[arg(short, long)]
        path: PathBuf,
    },
    List,

    Note {
        message: String,
    },
}
