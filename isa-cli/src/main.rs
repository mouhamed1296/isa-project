use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod commands;

const BANNER: &str = r#"
    __  ______        __________
   /  |/  /   |      /  _/ ___/   /\
  / /|_/ / /| |______/ / \__ \   /  \
 / /  / / ___ /_____/ / ___/ /  / /\ \
/_/  /_/_/  |_|   /___//____/  /_/  \_\
"#;

#[derive(Parser)]
#[command(
    name = "isa",
    author,
    version,
    about = "MA-ISA — Multi-Axis Integrity State Accumulation",
    long_about = "A developer toolkit for integrating cryptographic integrity tracking into your applications.\nSupports NestJS, Tauri, Rust, and WASM targets.",
    after_help = "Examples:\n  isa init                    Interactive project setup\n  isa install --target nestjs Generate NestJS dependency config\n  isa config add-dimension    Add a custom integrity dimension\n  isa device init my-pos      Initialize a device state\n  isa device record my-pos    Record an event\n  isa demo                    Launch the interactive web demo"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive project setup — configure MA-ISA for your stack
    Init {
        /// Project directory (default: current directory)
        #[arg(short, long)]
        path: Option<String>,

        /// Skip interactive prompts, use defaults
        #[arg(long)]
        defaults: bool,
    },

    /// Generate install instructions and dependency configs for your target
    Install {
        /// Target platform: nestjs, tauri, rust, wasm
        #[arg(short, long)]
        target: Option<String>,

        /// Output the config to a file instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Manage dimensions, policies, and constraints
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Device state management — init, record, verify, show, compare
    Device {
        #[command(subcommand)]
        action: DeviceAction,
    },

    /// Launch the interactive web demo
    Demo {
        /// Port to serve on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Which demo to open: iot, government, mobile-money
        #[arg(short, long, default_value = "iot")]
        variant: String,
    },

    /// Show project status and health
    Status {
        /// Project directory
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Display version and system info
    Info,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Add a new integrity dimension
    AddDimension {
        /// Dimension name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// List all configured dimensions
    ListDimensions,

    /// Set a policy for a dimension
    SetPolicy {
        /// Dimension name or index
        dimension: String,

        /// Policy strategy: ImmediateHeal, Quarantine, MonitorOnly
        #[arg(short, long)]
        strategy: Option<String>,

        /// Divergence threshold
        #[arg(short = 'T', long)]
        threshold: Option<u64>,
    },

    /// Add a constraint across dimensions
    AddConstraint {
        /// Constraint type: MaxRatio, SumBelow
        #[arg(short = 't', long)]
        constraint_type: Option<String>,
    },

    /// Show current configuration
    Show,

    /// Generate a config file from current settings
    Generate {
        /// Output format: yaml, toml, json
        #[arg(short, long, default_value = "yaml")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum DeviceAction {
    /// Initialize a new device state
    Init {
        /// Device identifier
        device_id: String,

        /// Output file path (default: <device_id>.state)
        #[arg(short, long)]
        output: Option<String>,

        /// Master seed as hex string (generates random if not provided)
        #[arg(short, long)]
        seed: Option<String>,
    },

    /// Record a custom event
    Record {
        /// State file path
        #[arg(short, long, default_value = "device.state")]
        file: String,

        /// Event data (hex or string)
        event: String,

        /// Entropy data (hex or string, optional)
        #[arg(short, long)]
        entropy: Option<String>,

        /// Time delta in milliseconds
        #[arg(short, long, default_value = "1000")]
        delta_t: u64,
    },

    /// Verify state integrity
    Verify {
        /// State file path
        file: String,

        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show current device state
    Show {
        /// State file path
        file: String,

        /// Output format: text, json, hex
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Compare two device states
    Compare {
        /// First state file
        file1: String,

        /// Second state file
        file2: String,

        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

fn print_banner() {
    println!("{}", BANNER.bright_cyan().bold());
    println!(
        "  {} {}\n",
        "Multi-Axis Integrity State Accumulation".white().bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
}

fn print_welcome() {
    print_banner();
    println!("{}", "  Getting Started:".yellow().bold());
    println!(
        "    {}  {}",
        "isa init".cyan(),
        "Interactive project setup".dimmed()
    );
    println!(
        "    {}  {}",
        "isa install".cyan(),
        "Generate install config for your stack".dimmed()
    );
    println!(
        "    {}  {}",
        "isa config show".cyan(),
        "View current configuration".dimmed()
    );
    println!(
        "    {}  {}",
        "isa device init".cyan(),
        "Initialize a device state".dimmed()
    );
    println!(
        "    {}  {}",
        "isa demo".cyan(),
        "Launch interactive web demo".dimmed()
    );
    println!(
        "    {}  {}",
        "isa --help".cyan(),
        "Show all commands".dimmed()
    );
    println!();
    println!(
        "  {}",
        "Run `isa init` to get started with your project!".green()
    );
    println!();
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None => {
            print_welcome();
            Ok(())
        }
        Some(Commands::Init { path, defaults }) => commands::init_project::run(path, defaults),
        Some(Commands::Install { target, output }) => commands::install::run(target, output),
        Some(Commands::Config { action }) => match action {
            ConfigAction::AddDimension { name } => commands::config::add_dimension(name),
            ConfigAction::ListDimensions => commands::config::list_dimensions(),
            ConfigAction::SetPolicy {
                dimension,
                strategy,
                threshold,
            } => commands::config::set_policy(dimension, strategy, threshold),
            ConfigAction::AddConstraint { constraint_type } => {
                commands::config::add_constraint(constraint_type)
            }
            ConfigAction::Show => commands::config::show(),
            ConfigAction::Generate { format, output } => {
                commands::config::generate(format, output)
            }
        },
        Some(Commands::Device { action }) => match action {
            DeviceAction::Init {
                device_id,
                output,
                seed,
            } => commands::init::run(device_id, output, seed),
            DeviceAction::Record {
                file,
                event,
                entropy,
                delta_t,
            } => commands::record::run(file, event, entropy, delta_t),
            DeviceAction::Verify { file, verbose } => commands::verify::run(file, verbose),
            DeviceAction::Show { file, format } => commands::show::run(file, format),
            DeviceAction::Compare {
                file1,
                file2,
                format,
            } => commands::compare::run(file1, file2, format),
        },
        Some(Commands::Demo { port, variant }) => commands::demo::run(port, variant),
        Some(Commands::Status { path }) => commands::status::run(path),
        Some(Commands::Info) => commands::info::run(),
    };

    if let Err(e) = result {
        eprintln!("\n{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}
