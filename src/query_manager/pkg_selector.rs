/*!
 * Display QueryResults
 * Handle user input
 * Handle actions concerning fixing invalid packages in styx command
 */
use std::io::{stdin, stdout, Write};
use crate::query_manager::*;

const QUERY_SHORT_THRESHOLD: usize = 0;

impl PackageSelector {
    pub fn new(pkg_name: Package) -> PackageSelector {
        return PackageSelector { 
            pkg_name, 
            query_results: None 
        };
    }

    /**
     * Simple query:
     * 1. Query with fuzzy find
     * 2. User can re-enter search term
     * 3. User can select directly from results
     */
    pub fn get_replacement_pkg(&mut self) -> Result<PackageSelection, String> {
        let results: QueryResults = QueryResults::fuzzy_query(&self.pkg_name)?; 
        let len = results.len();

        if len == 0 {
            println!("Query yielded no results for: '{}'", self.pkg_name);
            return Ok(PackageSelection::None);
        }
        self.query_results = Some(results);
        
        if len <= QUERY_SHORT_THRESHOLD {
            return Ok(self.select_in_list_mode(&self.build_msg(vec![]), false));
        } 
        else {
            return Ok(self.select_in_list_mode(&self.build_msg(vec![]), false));
            //return Ok(self.display_tui_mode());
        }
    }

    fn select_in_tui_mode(&self) -> Option<String> {
        todo!()
    }

    fn select_in_list_mode(&self, msg: &str, allow_extra_opts: bool) -> PackageSelection {
        let query = self.query_results.as_ref().unwrap();
        let mut input: String;

        loop {
            print!("{}", msg);
            let _ = stdout().flush();
            input = String::new();

            stdin().read_line(&mut input).expect("Could not read user input");
            input = input.trim().into();

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

    fn build_msg(&self, opts: Vec<&str>) -> String {
        let menu = self.query_results.as_ref().unwrap().to_menu();
        let mut msg: String = format!("Query results for: {}\n{menu}\n0. Remove pkg", self.pkg_name);

        let index = menu.len() + 1;
        for (i, opt) in opts.iter().enumerate() {
            msg += &format!("\n{}. {opt}", index + i);
        }
        msg += "\nEnter option: ";

        return msg;
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
        let mut cmd = Command::new("sudo");
        cmd.arg("xbps-query");
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
        let res = sel.get_replacement_pkg();
        println!("{:?}", res);
        assert!(true);
    }
}
