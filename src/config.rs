use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about= None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, global = true)]
    pub author: Option<String>,

    #[arg(short, long, global = true)]
    pub days: Option<u64>,

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
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    List,

    Note {
        message: String,
    },

    Summarize,

    Config {
        #[arg(long)]
        set_key: Option<String>,
        #[arg(long)]
        set_url: Option<String>,
        #[arg(long)]
        set_model: Option<String>,
        #[arg(long)]
        set_prompt: Option<String>,
        #[arg(long)]
        set_lang: Option<String>,
    },
}
