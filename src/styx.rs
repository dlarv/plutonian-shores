pub mod query_manager;
pub mod commands;
use crate::commands::*;
use mythos_core::{conf, logger::set_logger_id, printinfo};
use help::{self, print_help};

static mut USE_ALIAS_MODE: bool = false;
fn main() {
    set_logger_id("STYX");
    unsafe { 
        if let Some(conf) = conf::MythosConfig::read_file("plutonian-shores") {
            load_config_values(conf);
        }
    }
    
    let mut cmd = match parse_args() {
        Some(cmd) => cmd,
        None => return
    };

    cmd.execute();
}

unsafe fn load_config_values(conf: conf::MythosConfig) {
    if let Some(val) = conf.try_get_boolean("use_alias_mode") {
        USE_ALIAS_MODE = val;
    }
    if let Some(conf) = conf.get_subsection("cocytus") { 
        if let Some(val) = conf.try_get_float("fuzzy_find_threshold") {
            query_manager::query_results::THRESHOLD = val as f32;
        }

        if let Some(val) = conf.try_get_integer("list_column_length") {
            query_manager::query_results::LIST_COLUMN_LEN = val as usize;
        }
    }
    if let Some(conf) = conf.get_subsection("styx") {
        if let Some(val) = conf.try_get_boolean("do_sync") {
            install_command::DO_SYNC_REPOS = val;
        }
        if let Some(val) = conf.try_get_boolean("use_alias_mode") {
            USE_ALIAS_MODE = val;
        }
    }
}

fn parse_args() -> Option<InstallCommand> {
    let args = mythos_core::cli::clean_cli_args();
    let mut cmd = InstallCommand::new(StyxState::DoInstall);
    let mut reading_xbps_args = false;
    let mut alias_mode: bool;
    unsafe { alias_mode = USE_ALIAS_MODE; }

    for arg in args {
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
                "-n" | "--dry-run" => {
                    cmd.set_do_dry_run(true);
                },
                "-u" | "--update" => {
                    cmd.set_initial_state(StyxState::DoSysUpdate);
                },
                "-X" | "--update-all" => {
                    cmd.set_initial_state(StyxState::DoXbpsUpdate);
                },
                "-y" | "--assume-yes" => {
                    cmd.set_assume_yes(true);
                },
                "-x" | "--xbps-args" => reading_xbps_args = true,
                "-a" | "--alias" => {
                    alias_mode = true;
                    reading_xbps_args = true;
                },
                "-w" | "--wrapper" => {
                    alias_mode = false;
                },
                _ => { cmd.add_xbps_arg(arg); },
            };
        }
        else {
            cmd.add_pkg(arg);
        }
    }

    if alias_mode {
        cmd.set_assume_yes(false);
        cmd.set_initial_state(StyxState::DoInstall);
        cmd.set_do_dry_run(false);

    }

    return Some(cmd);
}
