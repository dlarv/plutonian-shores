/*!
 * Simple wrapper for xbps-remove command -Ro.
 */

use std::ffi::OsString;

use duct::cmd;
use mythos_core::{cli::{clean_cli_args, get_cli_input}, logger::{get_logger_id, set_logger_id}, printfatal, printinfo};
use pt_core::{validate_pkgs, Query, QueryResult};
fn main() {
    set_logger_id("COCYTUS");
    let args = clean_cli_args();
    let mut pkgs: Vec<&str> = Vec::new();
    let mut opts: Vec<&str> = Vec::new();
    let mut do_dry_run = false;

    // Parse opts.
    for arg in &args {
        if arg == "-n" || arg == "--dryrun" {
            do_dry_run = true;
        }
        else if arg.starts_with("-") {
            opts.push(&arg);
        } else {
            pkgs.push(&arg);
        }
    }

    // Validate packages
    // Ensure package(s) actually exist.
    let mut removed_pkgs = false;
    let validated_pkgs = Query::from(match validate_pkgs(pkgs) {
        // Only grab packages that are installed.
        Some(pkgs) => pkgs.into_iter().filter(|p| { removed_pkgs = true; p.is_installed}).collect::<Vec<QueryResult>>(),
        None => {
            printinfo!("Exiting");
            return;
        }
    });

    // If all packages were removed, exit
    if removed_pkgs {
        println!("Removed packages not currently installed");
    }
    if validated_pkgs.len() == 0 {
        printinfo!("All packages were removed. Exiting...");
        return;
    }

    // Give user option to exit.
    let msg = format!("The following packages will be removed.{}\nWould you like to continue? Y/n ", validated_pkgs.get_short_list());
    let user_input = get_cli_input(&msg).to_lowercase();
    if user_input == "n" || user_input == "no" {
        printinfo!("Exiting");
        return;
    }

    // Create args list
    let pkg_names = validated_pkgs.into_iter().map(|p| p.pkg_name.into()).collect::<Vec<String>>();
    let mut args = vec!["-Ryo".to_string()];
    if do_dry_run {
        args.push("-n".into());
    }
    args.extend(pkg_names);

    // Remove packages.
    match cmd("xbps-remove", args).run() {
        Ok(_) => printinfo!("Success! Exiting"),
        Err(msg) => printfatal!("{msg}"),
    }

}
