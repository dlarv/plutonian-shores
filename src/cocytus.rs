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
use mythos_core::{conf, logger::{get_logger_id, set_logger_id}, printfatal};
use commands::{QueryDisplayMode, QueryCommand, MythosCommand};
use query_manager::Package;

use crate::query_manager::PackageSelector;

static mut DISPLAY_MODE: QueryDisplayMode = QueryDisplayMode::Smart;
fn main() {
    set_logger_id("COCYTUS");
    unsafe { 
        if let Some(conf) = conf::MythosConfig::read_file("plutonian-shores") {
            load_config_values(conf);
        }
    }
    let mut cmd = parse_args();
    cmd.execute();
}

unsafe fn load_config_values(conf: conf::MythosConfig) {
    if conf.try_get_boolean("use_alias_mode").is_some() {
        DISPLAY_MODE = QueryDisplayMode::AliasMode;
    }
    if let Some(conf) = conf.get_subsection("cocytus") { 
        if let Some(val) = conf.try_get_float("fuzzy_find_threshold") {
            query_manager::query_results::THRESHOLD = val as f32;
        }

        if let Some(val) = conf.try_get_integer("list_column_length") {
            query_manager::query_results::LIST_COLUMN_LEN = val as usize;
        }

        if conf.try_get_boolean("use_alias_mode").is_some() {
            DISPLAY_MODE = QueryDisplayMode::AliasMode;
        }

        if let Some(val) = conf.try_get_string("default_display_mode") {
            match val.to_lowercase().as_str() {
                "list" => DISPLAY_MODE = QueryDisplayMode::List,
                "tui" => DISPLAY_MODE = QueryDisplayMode::Tui,
                "alias" => DISPLAY_MODE = QueryDisplayMode::AliasMode,
                "smart" => DISPLAY_MODE = QueryDisplayMode::Smart,
                _ => ()
            }
        }
    }
}
fn parse_args() -> QueryCommand {
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
                    std::process::exit(0);
                },
                "-l" | "--list" => cmd.set_display_mode(QueryDisplayMode::List),
                "-t" | "--tui" => cmd.set_display_mode(QueryDisplayMode::Tui),
                "-a" | "--alias" => cmd.set_display_mode(QueryDisplayMode::AliasMode),
                "-x" | "--xbps-args" => reading_xbps_args = true,
                _ => { cmd.add_xbps_arg(arg); },
            };
        }
        else {
            cmd.add_pkg(arg);
        }
    }

    return cmd;
}
