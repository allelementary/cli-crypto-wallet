use clap::{Parser, Subcommand};

mod services;
mod orchestrator;
mod commands;
mod config;

#[tokio::main]
async fn main() {
    let cli = commands::Cli::parse();
    let mut orchestrator = orchestrator::Orchestrator::new();

    orchestrator.handle_command(&cli.command).await;
}
