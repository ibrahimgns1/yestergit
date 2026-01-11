use anyhow::Result;
use clap::Parser;
use yestergit::{
    commands,
    config::{Args, Commands},
};

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Some(Commands::Scan { path }) => {
            commands::scan(path.clone())?;
        }
        Some(Commands::Check { path }) => {
            commands::check(path.clone(), &args)?;
        }
        Some(Commands::List) => {
            commands::list()?;
        }
        Some(Commands::Note { message }) => {
            commands::note(message.clone())?;
        }
        Some(Commands::Config {
            set_key,
            set_url,
            set_model,
            set_prompt,
            set_lang,
        }) => {
            commands::config(
                set_key.clone(),
                set_url.clone(),
                set_model.clone(),
                set_prompt.clone(),
                set_lang.clone(),
            )?;
        }
        Some(Commands::Summarize) => {
            commands::summarize(&args)?;
        }
        None => {
            commands::report_all(&args)?;
        }
    }

    Ok(())
}
