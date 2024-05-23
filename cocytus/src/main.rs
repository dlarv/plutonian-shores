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

use duct::cmd;
use mythos_core::{cli::{clean_cli_args, get_cli_input}, logger::{get_logger_id, set_logger_id}, printinfo, printwarn};
use pt_core::{get_user_selection, validate_pkgs, Query, QueryError, QueryResult};
fn main() {
    set_logger_id("COCYTUS");
    let args = clean_cli_args();
    let mut pkgs: Vec<&str> = Vec::new();
    let mut opts: Vec<&str> = Vec::new();

    // Parse opts.
    for arg in &args {
        if arg.starts_with("-") {
            opts.push(&arg);
        } else {
            pkgs.push(&arg);
        }
    }

    let validated_pkgs = Query::from(match validate_pkgs(pkgs) {
        Some(pkgs) => pkgs,
        None => {
            printinfo!("Exiting");
            return;
        }
    });

    println!("\nSelected packages:\n{}\n", validated_pkgs.get_short_list());

    match get_user_selection(&format!("0. Exit\n1. Pipe results to Styx\n2. Pipe results to Lethe\n3. Show details\nOption: "), 3) {
        0 => return,
        1 => pipe_to_styx(validated_pkgs),
        2 => pipe_to_lethe(validated_pkgs),
        3 => print_pkg_info(validated_pkgs),
        _ => panic!("User input should have been evaluated earlier")
    };
}


fn print_pkg_info(pkgs: Query) {
    for pkg in pkgs {
        println!("\nShowing {}", pkg.pkg_name);
        let _ = cmd!("xbps-query", "-R", pkg.pkg_name).pipe(cmd!("head")).run();
    }
}
fn pipe_to_styx(pkgs: Query) {
    // Check if user has sudo privileges
    // If not, prompt 
    // Execute install
    printinfo!("Piped to styx");
}
fn pipe_to_lethe(pkgs:Query) {
    printinfo!("Piped to lethe");
}
