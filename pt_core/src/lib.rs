pub mod help;
pub mod query; 

use duct::{Expression, cmd};
use mythos_core::{cli::get_cli_input, logger::get_logger_id, printfatal, printinfo, printwarn};
use serde_derive::{Deserialize, Serialize};

use std::io::{stdout, Write, stdin};

/* FUNCTIONS */
pub fn get_user_permission(assume_yes: bool, msg: &str) -> bool{
    println!("{}", msg);
    loop {
        if assume_yes {
            println!("Would you like to proceed? Y/n: Y");
            return true;
        }

        let input = get_cli_input("Would you like to proceed? Y/n: ");
        if ["n", "no"].contains(&input.as_str()) {
            return false;
        }
        if ["y", "yes", "\n", ""].contains(&input.as_str()) {
            return true;
        }
        eprintln!("Invalid input.");
    }
}
pub fn parse_output(output: Vec<u8>) -> String {
    return output.iter().map(|x| (*x as char)).collect::<String>().trim().to_string();
}
pub fn xbps_args_to_string(xbps_args: &Vec<String>) -> String {
    if xbps_args.len() == 0 {
        return "".into();
    }
    return xbps_args.iter().fold("-".to_string(), |acc, x| {
        if x.starts_with("--") {
            printfatal!("Styx can only take the short version of xbps-install args");
        }
        acc + x.trim_start_matches("-")
    });
}
pub fn validate_pkgs<'a, T>(search_terms: T) -> Option<Vec<QueryResult>>  where T: Iterator<Item = &'a str>{
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
        // User chose to select from query results.
        // If only one pkg exists, use it.
        if query.len() == 1 {
            output.push(query.get(0).unwrap().clone());
            continue;
        }

        let msg = &format!("{}\n\n0. Exit\n1. Select from result(s)\n2. Remove query\nOption: ", query.get_short_list());

        // Get and validate user selection.
        let user_input = get_user_selection(msg, 2);
        if user_input == 0 {
            return None;
        }
        if user_input == 2 {
            printinfo!("Removed {term}");
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

pub fn get_user_selection(msg: &str, max_val: usize) -> usize {
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
/* STRUCTS */
/**
 * Public interface to results of query.
 */
#[derive(Debug, Clone)]
pub struct Query {
    pkg_name: String,
    results: Vec<QueryResult>,
    longest_name: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryResult {
    pub is_installed: bool,
    pub pkg_name: String,
    pub pkg_version: String,
    pub pkg_description: String,
    pub score: i32,
}
/**
 * TertiaryList: Package was found in tertiary list and can be installed using the contained pkg
 * manager.
 * NotFound: No packages were found that matched the search.
 */
#[derive(Debug)]
pub enum QueryError{
    TertiaryList(String),
    NotFound(String),
}

// Deprecating
pub trait MythosCommand {
    fn pkgs<'a> (&'a mut self) -> &'a mut Vec<String>;
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String>;
    fn build_cmd(&self) -> Expression;

    fn add_pkg<T: Into<String>>(&mut self, pkg: T) -> &mut Self {
        self.pkgs().push(pkg.into());
        return self;
    }
    fn add_pkgs<'a, T>(&mut self, pkgs: T) -> &mut Self where T: IntoIterator<Item = &'a str> {
        self.pkgs().extend(pkgs.into_iter().map(|x| x.to_string()));
        return self;
    }
    fn add_xbps_args<'a, T>(&mut self, args: T) -> &mut Self where T: IntoIterator<Item = &'a str> {
        self.xbps_args().extend(args.into_iter().map(|x| x.to_string()));
        return self;
    }
    fn add_xbps_arg(&mut self, arg: String) -> &mut Self {
        self.xbps_args().push(arg);
        return self;
    }
    fn list_pkgs(&mut self) -> String {
        return self.pkgs().iter().map(|x| format!("{}\n", x)).collect();
    }
}
