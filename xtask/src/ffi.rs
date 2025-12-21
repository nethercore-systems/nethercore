//! FFI binding generation commands

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum FfiCommand {
    /// Generate FFI bindings for C and Zig
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

pub fn execute(command: FfiCommand) -> Result<()> {
    match command {
        FfiCommand::Generate { console } => generate(console),
        FfiCommand::Check { console } => check(console),
        FfiCommand::Verify { console } => verify(console),
        FfiCommand::All { console } => all(console),
    }
}

fn generate(console: Option<String>) -> Result<()> {
    println!("Generating FFI bindings...");

    match console {
        Some(c) => {
            ffi_gen::generate_for_console(&c)?;
        }
        None => {
            let consoles = ffi_gen::get_consoles()?;
            for c in consoles {
                ffi_gen::generate_for_console(&c)?;
            }
        }
    }

    println!("✓ Done!");
    Ok(())
}

fn check(console: Option<String>) -> Result<()> {
    println!("Checking FFI bindings are in sync...");

    let mut all_in_sync = true;

    match console {
        Some(c) => {
            if !ffi_gen::check_for_console(&c)? {
                all_in_sync = false;
            }
        }
        None => {
            let consoles = ffi_gen::get_consoles()?;
            for c in consoles {
                if !ffi_gen::check_for_console(&c)? {
                    all_in_sync = false;
                }
            }
        }
    }

    if all_in_sync {
        println!("\n✓ All bindings are in sync!");
        Ok(())
    } else {
        anyhow::bail!("Bindings are out of sync. Run 'cargo xtask ffi generate' to regenerate.")
    }
}

fn verify(console: Option<String>) -> Result<()> {
    println!("Verifying FFI bindings compile...");

    match console {
        Some(c) => {
            ffi_gen::verify_with_zig(&c)?;
        }
        None => {
            let consoles = ffi_gen::get_consoles()?;
            for c in consoles {
                ffi_gen::verify_with_zig(&c)?;
            }
        }
    }

    Ok(())
}

fn all(console: Option<String>) -> Result<()> {
    generate(console.clone())?;
    println!();
    check(console.clone())?;
    println!();
    verify(console)?;
    Ok(())
}
