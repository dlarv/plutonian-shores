/*
 * Styx: xbps-install wrapper
 * Shares args with cmd
 *
 * styx -U
 *
 * If user tries to install a pkg which cannot be found
 * 1. Run query command
 * 2. Display results to user
 * 3. Allow user to select one or none of results 
 *
 * If user tries to update system & xbps must be updated first
 * Ask user if they want to do so (Y/n)
 * If user passed -y | --assume-yes, do so automatically
 * 
 * NOTE: xbps-install confirmation message (... (Y/n)) is sent via stderr 
 * styx will do a dry-run where it captures stderr/etc
 * If stderr is fine, do actual install
 */
pub mod query_manager;
pub mod commands;
use crate::commands::install_command::{ InstallCommand, States };

fn main() {
    let mut cmd = match parse_args() {
        Some(cmd) => cmd,
        None => return
    };

    while !cmd.is_completed() { 
        match cmd.try_run() {
            Ok(msg) => {
                println!("{}", msg);
                continue;
            },
            Err(msg) => eprintln!("{}", msg)
        };

        // Fatal errors mark command as completed 
        if cmd.is_completed() {
            break;
        }

        match cmd.try_apply_fix() {
            Ok(msg) => println!("{}", msg),
            Err(msg) => eprintln!("{}", msg)
        };
    }
}

fn parse_args() -> Option<InstallCommand> {
    let args: Vec<String> = std::env::args().into_iter().skip(1).flat_map(|x| {
        if x.starts_with("--") || !x.starts_with("-") {
            vec![x]
        }
        else {
            x.chars().into_iter().skip(1).map(|x| format!("-{}", x)).collect()
        }
    }).collect();

    let mut assume_yes = false;
    let mut do_system_update = false;
    let mut pkgs: Vec<String> = Vec::new();
    let mut xbps_args: Vec<String> = Vec::new();
    let mut reading_xbps_args = false;
    let mut initial_state: States = States::DoInstall;

    for arg in args {
        if !reading_xbps_args {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("TODO: help msg");
                    return None;
                },
                "-U" | "--update" => {
                    do_system_update = true;
                },
                "-X" | "--update-all" => {
                    initial_state = States::DoXbpsUpdate;
                    do_system_update = true;
                },
                "-y" | "--assume-yes" => {
                    assume_yes = true;
                },
                "-x" | "--xbps-args" => reading_xbps_args = true,
                _ => pkgs.push(arg),
            };
        }
        else if arg.starts_with("-"){
            xbps_args.push(arg);
        }
        else {
            pkgs.push(arg);
        }
    }

    return Some(InstallCommand {
        assume_yes,
        do_system_update,
        xbps_args,
        pkgs,
        current_state: initial_state,
        do_validate_pkgs: true,
    });
}
