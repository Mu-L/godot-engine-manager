use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(
    name = "gdem",
    version = "1.5.1",
    about = "Godot Engine Manager is a Godot Engine version management tool developed based on the GodotHub.",
    after_help = "Before using, please first sync the data with `gdem sync`."
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Configure the Godot Engine Manager.
    #[clap(name = "config", alias = "cfg")]
    Config {
        /// The proxy to use.
        #[clap(short, long)]
        proxy: Option<String>,
    },
    /// Sync the data from GodotHub.
    #[clap(name = "sync", alias = "s")]
    Sync,
    /// List the local engines.
    #[clap(name = "list", alias = "ls")]
    List {
        /// List the remote engines.
        #[clap(short, long)]
        remote: bool,
        /// List the engine assets.
        #[clap(short, long)]
        version: Option<String>,
    },
    /// Install the engine.
    #[clap(name = "install", alias = "i")]
    Install {
        /// The engine version to install.
        /// Godot_v4.4.1-stable_mono_win64.zip
        engine: String,
        #[clap(short, long)]
        /// Force install.
        force: bool,
        #[clap(short = 'k', long)]
        /// Skip sha512 check.
        skip_check: bool,
    },
    /// Switch the engine.
    #[clap(name = "switch", alias = "sw")]
    Switch {
        /// The local engine to switch.
        /// Godot_v4.4.1-stable_mono_win64
        engine: String,
    },
    /// Remove the engine.
    #[clap(name = "remove", alias = "rm")]
    Remove {
        /// The local engine to remove.
        /// Godot_v4.4.1-stable_mono_win64
        engine: String,
    },
}
