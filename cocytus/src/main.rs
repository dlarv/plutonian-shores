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
use mythos_core::{cli::clean_cli_args, logger::set_id, printerror, printinfo};
use pt_core::{get_user_selection, validate_pkgs, Query, QueryResult};

fn main() {
    let _ = set_id("COCYTUS");
    let args = clean_cli_args();
    // This is passed to styx or lethe, if the user chooses to do so.
    let mut do_dry_run = false;
    let mut pkgs: Vec<String> = Vec::new();

    for arg in args {
        if arg == "-h" || arg == "--help" {
        println!("Wrapper for xbps-query -Rs (xrs). Allows the user to select from the results and pipe them to either styx or lethe.\ncocytus -h|--help\t\tPrint this menu\ncocytus [pkgs]\t\tQuery [pkgs].");
            return;
        } 
        if arg == "-n" || arg == "--dryrun" {
            do_dry_run = true;
        }
        else if !arg.starts_with("-") {
            pkgs.push(arg);
        } else {
            printerror!("Unknown opt: '{arg}'");
            return;
        }
    }


    let mut validated_pkgs = Query::from(match validate_pkgs(pkgs.into_iter()) {
        Some(pkgs) => pkgs,
        None => {
            printinfo!("Exiting...");
            return;
        }
    });

    printinfo!("\nSelected packages:\n{}\n", validated_pkgs.get_short_list());

    loop {
        match get_user_selection(&format!("0. Exit\n1. Pipe results to Styx\n2. Pipe results to Lethe\n3. Show details\nOption: "), 3) {
            0 => return,
            1 => {
                pipe_to_styx(validated_pkgs, do_dry_run);
                return;
            },
            2 => {
                pipe_to_lethe(validated_pkgs, do_dry_run);
                return;
            },
            3 => {
                validated_pkgs = match print_pkg_info(validated_pkgs) {
                    Some(pkgs) => pkgs,
                    None => return
                }
            },
            _ => panic!("User input should have been evaluated earlier")
        };
    }
}

fn print_pkg_info(query: Query) -> Option<Query> {
    let msg = "\n0. Return\n1. Previous\n2. Next\nOption: ";
    let mut pkgs: Vec<QueryResult> = Vec::new();

    for pkg in query {
        let info = pkg.display();
        println!("\nShowing info for \"{}\"", pkg.pkg_name);
        println!("{info}");
    }

    return None;
}
fn pipe_to_styx(pkgs: Query, do_dryrun: bool) {
    // Check if user has sudo privileges   
    // Check if styx is installed.
    // Execute install
    printinfo!("Piping to styx");
    let mut cmd = Command::new("sudo");
    cmd.arg("styx");

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
    let mut cmd = Command::new("sudo");
    cmd.arg("lethe");

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
