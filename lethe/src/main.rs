/*!
 * Simple wrapper for xbps-remove command -Ro.
 */

use duct::cmd;
use mythos_core::{cli::{clean_cli_args, get_user_permission}, printerror, printfatal, printinfo, logger::set_id};
use pt_core::{validate_pkgs, Query, QueryResult};
fn main() {
    let _ = set_id("LETHE");
    let args = clean_cli_args();
    let mut pkgs: Vec<&str> = Vec::new();
    let mut do_dry_run = false;

    // Parse opts.
    for arg in &args {
        if arg == "-h" || arg == "--help" {
            println!("Wrapper util for xbps-remove -Ryo");
            println!("lethe [opts] pkgs");
            println!("opts:");
            println!("-h | --help\t\tPrint this menu.\n-n | --dryrun\t\tRun command w/o making changes to system.");
            return;
        } 
        if arg == "-n" || arg == "--dryrun" {
            do_dry_run = true;
        }
        else if !arg.starts_with("-") {
            pkgs.push(&arg);
        } else {
            printerror!("Unknown opt: '{arg}'");
            return;
        }
    }

    // Validate packages
    // Ensure package(s) actually exist.
    let mut removed_pkgs = false;
    let validated_pkgs = Query::from(match validate_pkgs(pkgs.into_iter()) {
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
    let msg = format!("The following packages will be removed.{}", validated_pkgs.get_short_list());
    if get_user_permission(false, &msg) {
        printinfo!("Exiting");
        return;
    }

    // Create args list
    let pkg_names = validated_pkgs.get_pkg_names();
    let mut args = vec!["-Ryo"];
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

#[cfg(test)]
mod test {
    /*! # Test Plan
        * - Rm pkg not installed.
        * - Rm bad pkg.
        * - Rm pkg w/ orphans.
        * - Rm pkg that shares dependencies w/ an installed pkg.
     */
}
