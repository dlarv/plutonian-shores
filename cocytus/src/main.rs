/*!
 * CLI interface to the query functionality in pt_core.
 * Essentially, this acts as a wrapper for xbps-query -Rs 
 * Allows user to select from results and pipe them to lethe or styx.
 * Pipe selection to xbps-query -S to see detail info.
 *
 * TODO:
 * - Allow user to select multiple packages from one query search term
 * - Show details can display description etc contained inside QueryResult
 * - Show details interface allows user to skip between info
 */

use std::process::Command;
use duct::cmd;
use mythos_core::{cli::clean_cli_args, logger::{self, set_id}, printerror, printinfo, printwarn};
use pt_core::{get_user_selection, validate_pkgs, Query};

fn main() {
    let _ = set_id("COCYTUS");
    let args = clean_cli_args();
    let mut print_help = false;
    // This is passed to styx or lethe, if the user chooses to do so.
    let mut do_dryrun = false;

    // Filter out packages.
    let pkgs: Vec<&str> = args.iter().filter(|x| {
        if x == &"-h" || x == &"--help" {
            print_help = true;
        } else if x == &"-n" || x == &"--dryrun" {
            do_dryrun = true;
        }
        !x.starts_with("-")
    }).map(|x| x.as_str()).collect();

    if print_help || pkgs.len() == 0 {
        println!("Wrapper for xbps-query -Rs (xrs). Allows the user to select from the results and pipe them to either styx or lethe.\ncocytus -h|--help\t\tPrint this menu\ncocytus [pkgs]\t\tQuery [pkgs].");
        return;
    } 

    let validated_pkgs = Query::from(match validate_pkgs(pkgs.into_iter()) {
        Some(pkgs) => pkgs,
        None => {
            printinfo!("Exiting...");
            return;
        }
    });

    printinfo!("\nSelected packages:\n{}\n", validated_pkgs.get_short_list());

    match get_user_selection(&format!("0. Exit\n1. Pipe results to Styx\n2. Pipe results to Lethe\n3. Show details\nOption: "), 3) {
        0 => return,
        1 => pipe_to_styx(validated_pkgs, do_dryrun),
        2 => pipe_to_lethe(validated_pkgs, do_dryrun),
        3 => print_pkg_info(validated_pkgs),
        _ => panic!("User input should have been evaluated earlier")
    };
}

fn print_pkg_info(pkgs: Query) {
    for pkg in pkgs {
        printinfo!("\nShowing {}", pkg.pkg_name);
        let _ = cmd!("xbps-query", "-R", pkg.pkg_name).pipe(cmd!("head")).run();
    }
}
fn pipe_to_styx(pkgs: Query, do_dryrun: bool) {
    // Check if user has sudo privileges   
    // Check if styx is installed.
    // Execute install
    printinfo!("Piping to styx");
    let mut cmd = Command::new("styx");
    if do_dryrun {
        cmd.arg("-n");
    } 
    cmd.args(pkgs.get_pkg_names());
    let _ = match cmd.spawn() {
        Ok(mut child) => child.wait(),
        Err(msg) => {
            printerror!("{msg:?}");
            return;
        }
    };
}
fn pipe_to_lethe(pkgs:Query, do_dryrun: bool) {
    printinfo!("Piped to lethe");
    let mut cmd = Command::new("lethe");
    if do_dryrun {
        cmd.arg("-n");
    } 
    cmd.args(pkgs.get_pkg_names());
    let _ = match cmd.spawn() {
        Ok(mut child) => child.wait(),
        Err(msg) => {
            printerror!("{msg:?}");
            return;
        }
    };
}

#[cfg(test)]
mod test {
    /*! # Test Plan
        * - Display query result info.
        * - Pipe to styx.
        * - Pipe to lethe.
        * - Select multiple query results from one query.
     */
}
