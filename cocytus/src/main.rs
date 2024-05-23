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
use pt_core::{Query, QueryError, QueryResult};
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

fn validate_pkgs(search_terms: Vec<&str>) -> Option<Vec<QueryResult>> {
    /*!
     * Iterate over pkgs, searching for each one in repo. 
     * Allows user to select from results or remove it.
     * User also has opportunity to exit.
     * Returns None if all packages are removed or user exits.
     */
    let mut output: Vec<QueryResult> = Vec::new();

    for term in search_terms {
        let query = match Query::query(&term) {
            Ok(res) => res,
            Err(QueryError::NotFound(msg)) | Err(QueryError::TertiaryList(msg)) => {
                printwarn!("{msg}");
                continue;
            }
        };
        // No results, exit early.
        if query.len() == 0 {
            printwarn!("No results found for {term}");
            continue;
        }

        let msg = &format!("{}\n0. Exit\n1. Select from result(s)\n2. Remove query\nOption: ", query.get_short_list());

        // Get and validate user selection.
        let user_input = get_user_selection(msg, 2);
        if user_input == 0 {
            return None;
        }
        if user_input == 2 {
            printinfo!("Removed {term}");
            continue;
        }


        // User chose to select from query results.
        // If only one pkg exists, use it.
        if query.len() == 1 {
            output.push(query.get(0).unwrap().clone());
            continue;
        }
        // Display results
        let msg = query.get_short_list();
        let selected_pkg_index = get_user_selection(&format!("{msg}\n0. Remove package\nEnter from the options above: "), query.len());

        // User chose to remove package.
        if selected_pkg_index == 0 {
            printinfo!("Removed {term}");
            continue;
        }
        // Add pkg
        println!("Selected: {selected_pkg_index}");
        output.push(query.get(selected_pkg_index - 1).unwrap().clone());
    }

    return Some(output);
}
fn get_user_selection(msg: &str, max_val: usize) -> usize {
    /*!
     * Prints a msg to the console prompting the user for input.
     * Validates input. Input must be an integer [0,max_val].
     * Returns that value
     */ 
    loop {
        let input = match get_cli_input(msg).parse::<usize>() {
            Ok(input) => input,
            Err(_) => {
                printwarn!("Please enter a valid number from the options above");
                continue;
            }
        };
        if input <= max_val {
            return input;
        }
        printwarn!("Please enter a valid number from the options above");
    }
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
