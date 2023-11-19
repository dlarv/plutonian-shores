/*!
 * Fuzzy find
 * Show query results
 * Allow user to run install/remove/exit
 *
 * No opts --> xbps-query -Rs pkg 
 */
pub mod query_manager;
use duct;

use mythos_core::logger::{get_logger_id, set_logger_id};
use query_manager::Package;

use crate::query_manager::PackageSelector;

fn main() {
    set_logger_id("COCYTUS");
    let pkgs = parse_args();
    for pkg in pkgs {
        let mut selector = PackageSelector::new(pkg);


    }
}

fn parse_args() -> Vec<Package> {
    let mut pkgs: Vec<Package> = Vec::new();

    for arg in mythos_core::cli::clean_cli_args() {
        if arg.starts_with("-") {
            // TODO: if/when opts are added to cocytus, they'll be parsed here
            todo!("No opts for cocytus at this point.");
        }
        else {
            pkgs.push(arg);
        }
    }

    return pkgs;
}
