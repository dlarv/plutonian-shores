use mythos_core::{printerror, logger::get_logger_id, printinfo, fatalmsg, printfatal};

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
            printinfo!("Showing results for '{pkg}'");

            let results = match smart_query(&pkg) {
                Some(res) => res,
                None => { 
                    printinfo!("Query yielded no results for: '{pkg}'");
                    continue
                }
            };
            println!("{res}", res=results.to_list());

        }

        println!("The following packages have been selected:\n{pkgs}", pkgs=self.list_selected_pkgs());
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
