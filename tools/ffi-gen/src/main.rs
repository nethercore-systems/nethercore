//! FFI binding generator CLI

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ffi-gen")]
#[command(about = "Generate FFI bindings from Rust source", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate all FFI bindings
    Generate {
        /// Console to generate bindings for (default: all)
        #[arg(short, long)]
        console: Option<String>,
    },

    /// Check that bindings are in sync with source
    Check {
        /// Console to check (default: all)
        #[arg(short, long)]
        console: Option<String>,
    },

    /// Verify bindings compile with Zig
    Verify {
        /// Console to verify (default: all)
        #[arg(short, long)]
        console: Option<String>,
    },

    /// Generate, check, and verify all bindings
    All {
        /// Console to process (default: all)
        #[arg(short, long)]
        console: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { console } => {
            println!("Generating FFI bindings...");
            match console {
                Some(c) => ffi_gen::generate_for_console(&c)?,
                None => {
                    for c in ffi_gen::get_consoles()? {
                        ffi_gen::generate_for_console(&c)?;
                    }
                }
            }
            println!("✓ Done!");
        }
        Commands::Check { console } => {
            println!("Checking FFI bindings are in sync...");
            let mut all_in_sync = true;
            match console {
                Some(c) => {
                    if !ffi_gen::check_for_console(&c)? {
                        all_in_sync = false;
                    }
                }
                None => {
                    for c in ffi_gen::get_consoles()? {
                        if !ffi_gen::check_for_console(&c)? {
                            all_in_sync = false;
                        }
                    }
                }
            }
            if all_in_sync {
                println!("\n✓ All bindings are in sync!");
            } else {
                anyhow::bail!("Bindings are out of sync. Run 'ffi-gen generate' to regenerate.");
            }
        }
        Commands::Verify { console } => {
            println!("Verifying FFI bindings compile...");
            match console {
                Some(c) => ffi_gen::verify_with_zig(&c)?,
                None => {
                    for c in ffi_gen::get_consoles()? {
                        ffi_gen::verify_with_zig(&c)?;
                    }
                }
            }
        }
        Commands::All { console } => {
            // Generate
            println!("Generating FFI bindings...");
            let consoles: Vec<String> = match &console {
                Some(c) => vec![c.clone()],
                None => ffi_gen::get_consoles()?,
            };
            for c in &consoles {
                ffi_gen::generate_for_console(c)?;
            }
            println!("✓ Done!\n");

            // Check
            println!("Checking FFI bindings are in sync...");
            let mut all_in_sync = true;
            for c in &consoles {
                if !ffi_gen::check_for_console(c)? {
                    all_in_sync = false;
                }
            }
            if !all_in_sync {
                anyhow::bail!("Bindings are out of sync after generation!");
            }
            println!("\n✓ All bindings are in sync!\n");

            // Verify
            println!("Verifying FFI bindings compile...");
            for c in &consoles {
                ffi_gen::verify_with_zig(c)?;
            }
        }
    }

    Ok(())
}
