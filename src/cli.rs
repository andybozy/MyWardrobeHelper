use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::app;
use crate::config::{AppConfig, ConfigOverrides, EnvConfig};
use crate::error::{AppError, AppResult};

const HELP_TEXT: &str = "\
MyWardrobeHelper

Usage:
  mywardrobehelper [--data-dir PATH] [--host HOST] [--port PORT] [--lan] <command>

Commands:
  init         Create the external data directory layout.
  doctor       Check config resolution and data directory readiness.
  serve        Resolve runtime config and print the planned server bind URLs.
  backup       Copy the current database file into the backups directory.
  export       Write a placeholder JSON export into the exports directory.
  mcp serve    Reserve the embedded MCP command surface for SEC-007.
  help         Show this message.

Flags:
  --data-dir PATH  Override the external data directory.
  --host HOST      Override the bind host.
  --port PORT      Override the bind port.
  --lan            Use 0.0.0.0 as the default bind host when none is provided.
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Init,
    Doctor,
    Serve,
    Backup,
    Export,
    McpServe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub command: Command,
    pub config: AppConfig,
}

pub fn run() -> ExitCode {
    match parse_from_process().and_then(dispatch) {
        Ok(()) => ExitCode::SUCCESS,
        Err(AppError::InvalidArgument(message)) => {
            eprintln!("error: {message}\n");
            eprintln!("{HELP_TEXT}");
            ExitCode::from(2)
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn parse_from_process() -> AppResult<Cli> {
    let cwd = env::current_dir()
        .map_err(|error| AppError::io("resolve current working directory", error))?;
    let args: Vec<String> = env::args().skip(1).collect();
    let env_config = EnvConfig::from_process_env()?;

    parse_args(&args, &cwd, env_config)
}

fn parse_args(args: &[String], cwd: &Path, env_config: EnvConfig) -> AppResult<Cli> {
    let mut overrides = ConfigOverrides::default();
    let mut position = 0usize;
    let mut command = None;

    while position < args.len() {
        match args[position].as_str() {
            "-h" | "--help" | "help" => {
                command = Some(Command::Help);
                position += 1;
            }
            "--data-dir" => {
                let value = next_value(args, &mut position, "--data-dir")?;
                overrides.data_dir = Some(PathBuf::from(value));
            }
            "--host" => {
                let value = next_value(args, &mut position, "--host")?;
                overrides.host = Some(value.to_string());
            }
            "--port" => {
                let value = next_value(args, &mut position, "--port")?;
                let port = value.parse::<u16>().map_err(|_| {
                    AppError::invalid_argument(format!(
                        "--port expects a valid TCP port, got {value}"
                    ))
                })?;

                if port == 0 {
                    return Err(AppError::invalid_argument("--port must be greater than 0"));
                }

                overrides.port = Some(port);
            }
            "--lan" => {
                overrides.lan = true;
                position += 1;
            }
            "init" => {
                set_command(&mut command, Command::Init)?;
                position += 1;
            }
            "doctor" => {
                set_command(&mut command, Command::Doctor)?;
                position += 1;
            }
            "serve" => {
                set_command(&mut command, Command::Serve)?;
                position += 1;
            }
            "backup" => {
                set_command(&mut command, Command::Backup)?;
                position += 1;
            }
            "export" => {
                set_command(&mut command, Command::Export)?;
                position += 1;
            }
            "mcp" => {
                if position + 1 >= args.len() || args[position + 1] != "serve" {
                    return Err(AppError::invalid_argument(
                        "expected `mcp serve` for the embedded MCP command",
                    ));
                }

                set_command(&mut command, Command::McpServe)?;
                position += 2;
            }
            other if other.starts_with("--") => {
                return Err(AppError::invalid_argument(format!(
                    "unknown flag `{other}`"
                )));
            }
            other => {
                return Err(AppError::invalid_argument(format!(
                    "unknown command or argument `{other}`"
                )));
            }
        }
    }

    let command = command.unwrap_or(Command::Help);
    let config = AppConfig::from_sources(overrides, env_config, cwd)?;

    Ok(Cli { command, config })
}

fn dispatch(cli: Cli) -> AppResult<()> {
    match cli.command {
        Command::Help => {
            println!("{HELP_TEXT}");
            Ok(())
        }
        Command::Init => {
            let report = app::init_app(&cli.config)?;
            println!(
                "Initialized data directory at {}",
                report.layout.root.display()
            );
            println!("Database file: {}", report.layout.database_file.display());
            println!(
                "Item media directory: {}",
                report.layout.item_media_root.display()
            );
            println!(
                "Backups directory: {}",
                report.layout.backups_root.display()
            );
            println!(
                "Exports directory: {}",
                report.layout.exports_root.display()
            );
            if report.created_database_file {
                println!(
                    "Created placeholder database file for SEC-003 at {}",
                    report.layout.database_file.display()
                );
            } else {
                println!(
                    "Reused existing database file at {}",
                    report.layout.database_file.display()
                );
            }
            Ok(())
        }
        Command::Doctor => {
            let report = app::doctor(&cli.config);
            let has_failures = report.has_failures();
            println!("Doctor report for {}", report.layout.root.display());
            for check in &report.checks {
                let prefix = match check.status {
                    app::CheckStatus::Pass => "PASS",
                    app::CheckStatus::Warn => "WARN",
                    app::CheckStatus::Fail => "FAIL",
                };
                println!("{prefix} [{}] {}", check.label, check.message);
            }

            if has_failures {
                return Err(AppError::config(
                    "doctor found one or more actionable problems",
                ));
            }

            Ok(())
        }
        Command::Serve => {
            let plan = app::plan_serve(&cli.config)?;
            println!("HTTP serve is still a placeholder for SEC-005.");
            println!("Resolved data directory: {}", plan.layout.root.display());
            println!("Planned bind URL: {}", plan.bind_url);
            println!("Local URL: {}", plan.local_url);
            match plan.lan_url {
                Some(url) => println!("LAN URL: {url}"),
                None => println!("LAN URL: disabled (bind host is loopback only)"),
            }
            Ok(())
        }
        Command::Backup => {
            let report = app::create_backup(&cli.config)?;
            println!(
                "Created database backup at {}",
                report.backup_file.display()
            );
            println!("Media backup included: {}", report.media_included);
            println!("Media files are not included in this SEC-002 backup placeholder.");
            Ok(())
        }
        Command::Export => {
            let report = app::export_layout(&cli.config)?;
            println!(
                "Wrote placeholder export to {}",
                report.export_file.display()
            );
            Ok(())
        }
        Command::McpServe => {
            let plan = app::plan_mcp(&cli.config)?;
            println!("Embedded MCP server is reserved for SEC-007.");
            println!("Resolved data directory: {}", plan.layout.root.display());
            println!("Use `cargo run -- doctor` until the MCP transport is implemented.");
            Ok(())
        }
    }
}

fn next_value<'a>(args: &'a [String], position: &mut usize, flag: &str) -> AppResult<&'a str> {
    let value_position = *position + 1;
    let Some(value) = args.get(value_position) else {
        return Err(AppError::invalid_argument(format!(
            "{flag} expects a value"
        )));
    };

    *position += 2;
    Ok(value.as_str())
}

fn set_command(slot: &mut Option<Command>, command: Command) -> AppResult<()> {
    if slot.is_some() {
        return Err(AppError::invalid_argument(
            "only one top-level command may be used at a time",
        ));
    }

    *slot = Some(command);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DEFAULT_HOST, DEFAULT_PORT};

    #[test]
    fn parses_init_with_custom_data_dir() {
        let args = vec![
            "init".to_string(),
            "--data-dir".to_string(),
            "custom-data".to_string(),
        ];
        let cwd = Path::new("/tmp/mywardrobehelper-tests");
        let cli = parse_args(&args, cwd, EnvConfig::default()).expect("cli parses");

        assert_eq!(cli.command, Command::Init);
        assert_eq!(cli.config.data_dir, cwd.join("custom-data"));
        assert_eq!(cli.config.host, DEFAULT_HOST);
        assert_eq!(cli.config.port, DEFAULT_PORT);
    }

    #[test]
    fn parses_mcp_serve_subcommand() {
        let args = vec![
            "--data-dir".to_string(),
            "custom-data".to_string(),
            "mcp".to_string(),
            "serve".to_string(),
        ];
        let cwd = Path::new("/tmp/mywardrobehelper-tests");
        let cli = parse_args(&args, cwd, EnvConfig::default()).expect("cli parses");

        assert_eq!(cli.command, Command::McpServe);
        assert_eq!(cli.config.data_dir, cwd.join("custom-data"));
    }

    #[test]
    fn rejects_multiple_commands() {
        let args = vec!["init".to_string(), "doctor".to_string()];
        let cwd = Path::new("/tmp/mywardrobehelper-tests");
        let error = parse_args(&args, cwd, EnvConfig::default()).expect_err("cli should fail");

        assert!(matches!(error, AppError::InvalidArgument(_)));
    }
}
