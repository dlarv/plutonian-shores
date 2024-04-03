pub mod query_manager;
pub mod commands;
use crate::commands::*;
use mythos_core::{conf, logger::set_logger_id};
use help::{self, print_help};

fn main() {
    set_logger_id("STYX");

    let mut cmd = InstallCommand::new();

    unsafe {
        if let Some(conf) = conf::MythosConfig::read_file("plutonian-shores") {
            load_config_values(&mut cmd, conf);
        }
    }
    
    parse_args(&mut cmd);
    cmd.execute();
}

unsafe fn load_config_values(cmd: &mut InstallCommand, conf: conf::MythosConfig) {
    if let Some(val) = conf.try_get_boolean("use_alias_mode") {
        cmd.use_alias_mode = val;
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
            cmd.do_sync_repos = val;
        }
        if let Some(val) = conf.try_get_boolean("use_alias_mode") {
            cmd.use_alias_mode = val;
        }
    }
}

fn parse_args(cmd: &mut InstallCommand) {
    let args = mythos_core::cli::clean_cli_args();
    let mut reading_xbps_args = false;

    for arg in args {
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
                "-n" | "--dry-run" => {
                    cmd.do_dry_run = true;
                },
                "-u" | "--update" => {
                    cmd.run_sys_update = true;
                },
                "-X" | "--update-all" => {
                    cmd.run_xbps_update = true;
                    cmd.run_sys_update = true;
                },
                "-y" | "--assume-yes" => {
                    cmd.assume_yes = true;
                },
                "-x" | "--xbps-args" => reading_xbps_args = true,
                "-a" | "--alias" => {
                    cmd.use_alias_mode = true;
                    reading_xbps_args = true;
                },
                "-w" | "--wrapper" => {
                    cmd.use_alias_mode = false;
                },
                _ => { cmd.add_xbps_arg(arg); },
            };
        }
        else {
            cmd.add_pkg(arg);
        }
    }

    if cmd.pkgs().len() > 0 {
        cmd.run_pkg_install = true;
    }
}
