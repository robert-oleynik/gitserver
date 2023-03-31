use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init,
    Apply,
    Destroy,
}

impl Cli {
    pub fn command(&self) -> &Command {
        &self.command
    }
}
