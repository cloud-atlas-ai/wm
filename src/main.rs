use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod compile;
mod extract;
mod init;
mod show;
mod state;
mod types;

#[derive(Parser)]
#[command(name = "wm")]
#[command(about = "Working memory for AI coding assistants")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .wm/ in current project
    Init,

    /// Run LLM extraction from transcript
    Extract {
        /// Path to transcript file
        #[arg(long)]
        transcript: Option<String>,

        /// Run in background (detach from terminal)
        #[arg(long)]
        background: bool,
    },

    /// Compile working set for current state
    Compile {
        /// User's current message (for intent detection)
        #[arg(long)]
        intent: Option<String>,
    },

    /// Display state, working set, or nodes
    Show {
        /// What to show: state, working, nodes, conflicts
        #[arg(default_value = "state")]
        what: String,
    },

    /// Hook entry points (called by Claude Code hooks)
    Hook {
        #[command(subcommand)]
        command: HookCommands,
    },
}

#[derive(Subcommand)]
enum HookCommands {
    /// Called by post-submit hook
    Compile,

    /// Called by sg after clearing (or manually)
    Extract,
}

fn main() -> ExitCode {
    // Check if disabled
    if std::env::var("WM_DISABLED").is_ok() {
        return ExitCode::SUCCESS;
    }

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => init::run(),
        Commands::Extract {
            transcript,
            background,
        } => extract::run(transcript, background),
        Commands::Compile { intent } => compile::run(intent),
        Commands::Show { what } => show::run(&what),
        Commands::Hook { command } => match command {
            HookCommands::Compile => compile::run_hook(),
            HookCommands::Extract => extract::run_hook(),
        },
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}
