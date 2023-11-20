/*!
 * Fuzzy find
 * Show query results
 * Allow user to run install/remove/exit
 *
 * No opts --> xbps-query -Rs pkg 
 */
pub mod query_manager;
pub mod commands;
use duct;

use help::print_help;
use mythos_core::{logger::{get_logger_id, set_logger_id}, printfatal};
use commands::{QueryDisplayMode, QueryCommand, MythosCommand};
use query_manager::Package;

use crate::query_manager::PackageSelector;

fn main() {
    set_logger_id("COCYTUS");
    let pkgs = parse_args();
}

fn parse_args() -> Option<QueryCommand> {
    let mut cmd = QueryCommand::new();
    let mut reading_xbps_args = false;

    for arg in mythos_core::cli::clean_cli_args() {
        if arg.starts_with("-") {
            if reading_xbps_args {
                cmd.add_xbps_arg(arg);
                continue;
            }
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    return None;
                },
                "-l" | "--list" => cmd.set_display_mode(QueryDisplayMode::List),
                "-t" | "--tui" => cmd.set_display_mode(QueryDisplayMode::Tui),
                "-x" | "--xbps-args" => reading_xbps_args = true,
                _ => printfatal!("Unknown arg: {arg}"),
            };
        }
        else {
            cmd.add_pkg(arg);
        }
    }

    return Some(cmd);
}
