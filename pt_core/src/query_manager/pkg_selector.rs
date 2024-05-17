/*!
 * Display QueryResults
 * Handle user input
 * Handle actions concerning fixing invalid packages in styx command
 */
use mythos_core::{cli::get_cli_input, printinfo, logger::get_logger_id};
use crate::query_manager::*;

const QUERY_SHORT_THRESHOLD: usize = 0;

impl PackageSelector {
    pub fn new(pkg_name: Package) -> PackageSelector {
        return PackageSelector { 
            pkg_name, 
            query_results: None 
        };
    }
    pub fn from_results(pkg_name: Package, results: QueryResults) -> PackageSelector {
        return PackageSelector { pkg_name, query_results: Some(results) }
    }

    pub fn select_replacement_pkgs(&mut self) -> PackageSelection {
        let results = match QueryResults::fuzzy_query(&self.pkg_name) {
            Some(res) => res,
            None => { 
                printinfo!("Query yielded no results for: '{name}'", name=self.pkg_name);
                return PackageSelection::None;
            } 
        };
        let len = results.len();
        self.query_results = Some(results);

        if len <= QUERY_SHORT_THRESHOLD {
            return self.select_in_list_mode(&self.build_msg(), false);
        } 
        else {
            return self.select_in_list_mode(&self.build_msg(), false);
            //return self.display_tui_mode();
        }
    }

    pub fn select_pkgs(&mut self, display_mode: &QueryDisplayMode) -> PackageSelection {
        if self.query_results.is_none() {
            self.query_results = match QueryResults::fuzzy_query(&self.pkg_name) {
                Some(res) => Some(res),
                None => { 
                    printinfo!("Query yielded no results for: '{name}'", name=self.pkg_name);
                    return PackageSelection::None;
                } 
            };
        }
        
        let use_list_mode = match display_mode {
            QueryDisplayMode::AliasMode | QueryDisplayMode::List => true,
            QueryDisplayMode::Tui => false,
            QueryDisplayMode::Smart => self.query_results.as_mut().unwrap().len() <= QUERY_SHORT_THRESHOLD,
        };
        if use_list_mode {
            return self.select_in_list_mode(&self.build_msg(), true);
        } 
        else {
            return self.select_in_list_mode(&self.build_msg(), true);
            //return self.display_tui_mode();
        }
    }

    fn select_in_tui_mode(&self) -> Option<String> {
        todo!()
    }

    fn select_in_list_mode(&self, msg: &str, allow_extra_opts: bool) -> PackageSelection {
        let query = self.query_results.as_ref().unwrap();

        loop {
            let input = get_cli_input(msg);
            let res = if input.find(" ") == None {
                read_single_index(&input, query)
            }
            else {
                read_multiple_index(&input, query)
            };

            if !allow_extra_opts && matches!(res, Some(PackageSelection::OtherOpt(_))) {
                eprintln!("Please enter an option above");
                continue;
            }

            return match res {
                Some(res) => res,
                None => {
                    eprintln!("Please enter an option above");
                    continue;
                }
            };
        }
    }

    fn build_msg(&self) -> String {
        let menu = self.query_results.as_ref().unwrap().to_menu();
        return format!("Query results for: {name}\n{menu}\n0. Remove pkg\nEnter option: ", name=self.pkg_name);
    }
}
// Return None if index was invalid
fn read_single_index(input: &str, query: &QueryResults) -> Option<PackageSelection> {
    let num_input = match input.parse::<usize>() {
        Ok(num) => num,
        Err(_) => return None 
    };
    return if num_input == 0 {
        Some(PackageSelection::None)
    }
    else if num_input > 0 && num_input <= query.len(){
        Some(PackageSelection::Package(query.0[num_input - 1].pkg_name.to_string()))
    }
    else if num_input > query.len() {
        Some(PackageSelection::OtherOpt(num_input - query.len()))
    }
    else {
        None
    }
}
fn read_multiple_index(input: &str, query: &QueryResults) -> Option<PackageSelection> {
    let mut pkgs: Vec<Package> = Vec::new();
    for num in input.split(" ") {
        match read_single_index(num, query) {
            Some(PackageSelection::Package(pkg)) => pkgs.push(pkg),
            _ => {
                eprintln!("You cannot use '0' or {}+ while selecting multiple packages!", query.len());
                return None;
            }
        }
    }
    return Some(PackageSelection::Packages(Box::new(pkgs)));
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use super::*;

    fn build_command(search_term: &str) -> Command {
        println!("{}", search_term);
        let mut cmd = Command::new("xbps-query");
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::piped());
        cmd.args(["-R", "--regex", "-s"]);
        cmd.arg(search_term);
        return cmd;
    }

    fn get_pkg_selector() -> PackageSelector {
        const TERM: &str = "blen";
        let mut cmd = build_command(&TERM);
        let res = cmd.output().unwrap().stdout;

        let p = QueryResults::parse_results(res, &TERM.into());

        return PackageSelector {
            pkg_name: "blen".to_string(),
            query_results: Some(p),
        };
    }

    //#[test]
    fn get_user_selection() {
        let mut sel = get_pkg_selector();
        let res = sel.select_replacement_pkgs();
        println!("{:?}", res);
        assert!(true);
    }
}

