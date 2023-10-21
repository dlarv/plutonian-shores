pub mod query_manager;
pub mod commands;
use crate::commands::*;

fn main() {
    let mut cmd = match parse_args() {
        Some(cmd) => cmd,
        None => return
    };

    while !cmd.is_completed() { 
        cmd.execute();
        println!();
    }
}

fn parse_args() -> Option<InstallCommand> {
    let args = mythos_core::cli::clean_cli_args();
    let mut cmd = InstallCommand::new(StyxState::DoInstall);
    let mut reading_xbps_args = false;

    for arg in args {
        if !reading_xbps_args {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("TODO: help msg");
                    return None;
                },
                "-U" | "--update" => {
                    cmd.set_initial_state(StyxState::DoSysUpdate);
                },
                "-X" | "--update-all" => {
                    cmd.set_initial_state(StyxState::DoXbpsUpdate);
                },
                "-y" | "--assume-yes" => {
                    cmd.set_assume_yes(true);
                },
                "-x" | "--xbps-args" => reading_xbps_args = true,
                _ => cmd.add_pkg(arg),
            };
        }
        else if arg.starts_with("-"){
            cmd.add_xbps_arg(arg);
        }
        else {
            cmd.add_pkg(arg);
        }
    }

    return Some(cmd);
}
