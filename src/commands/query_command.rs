use mythos_core::{printerror, logger::get_logger_id, printinfo, fatalmsg, printfatal, cli::get_cli_input};

use crate::query_manager::{PackageSelector, Package, PackageSelection, QueryResults};

use super::{QueryCommand, MythosCommand, QueryDisplayMode};

impl QueryCommand {
    pub fn new() -> QueryCommand {
        return QueryCommand { 
            pkgs: Vec::new(), 
            xbps_args: Vec::new(),
            display_mode: QueryDisplayMode::Smart,
        };
    }
    pub fn set_display_mode(&mut self, mode: QueryDisplayMode) {
        self.display_mode = mode;
    }

    pub fn execute(&mut self) {
        if matches!(self.display_mode, QueryDisplayMode::AliasMode) {
            self.execute_alias_mode();
            return;
        }

        // Packages selected by user.
        // These will/can be piped to styx/lethe
        let mut selected_pkgs: Vec<Package> = Vec::new(); 
        for pkg in &self.pkgs {
            let mut selector = match self.query_pkg(pkg) {
                Some(res) => PackageSelector::from_results(pkg.to_string(), res),
                None => continue
            };
            match selector.select_pkgs(&self.display_mode) {
                PackageSelection::Package(new_pkg) => {
                    printinfo!("Replaced '{}' with '{}'", pkg, new_pkg);
                    selected_pkgs.push(new_pkg);
                },
                PackageSelection::Packages(new_pkgs) => {
                    printinfo!("Replaced '{}' with the following: '{:?}'", pkg, *new_pkgs);
                    selected_pkgs.extend(*new_pkgs);
                }
                _ => printinfo!("Removed '{}'", pkg)
            }
        }

        self.pkgs = selected_pkgs;
        println!("The following packages have been selected:\n{pkgs}", pkgs=self.list_pkgs());
    }
    fn query_pkg(&self, pkg: &Package) -> Option<QueryResults> {
        loop {
            printinfo!("Showing results for '{pkg}'");

            let results = match smart_query(&pkg) {
                Some(res) => res,
                None => { 
                    printinfo!("Query yielded no results for: '{pkg}'");
                    return None 
                }
            };
            println!("{res}", res=results.to_list());
            
            let input = get_cli_input(&self.user_options(1));
            match input.as_str() {
                "0" => {
                    std::process::exit(0); 
                },
                "1" => return Some(results),
                "2" => return None,
                _ => {
                    printinfo!("Please select from the options above.");
                    continue
                }
            }
        }
    }

    fn execute_alias_mode(&self) {
        self.build_cmd()
            .unchecked()
            .run().unwrap();    
    }

    fn user_options(&self, offset: usize) -> String {
        return format!("0. Exit Cocytus\n{offset}. Select pkgs\n{offset2}. Query next pkg\nOption: ", offset2=offset+1);
    }
}
/**
 * Run a strict query first. If that returns no results, run a fuzzy query.
 * Returns None if no results were found in either query.
 */
fn smart_query(pkg_name: &Package) -> Option<QueryResults> {
    return match QueryResults::strict_query(&pkg_name) {
        Some(res) => Some(res),
        None => return QueryResults::fuzzy_query(&pkg_name)
    };
}
