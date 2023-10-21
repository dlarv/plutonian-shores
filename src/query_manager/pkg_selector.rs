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
    pub fn get_replacement_pkg(&mut self) -> Result<Option<Package>, String> {
        let results: QueryResults = QueryResults::fuzzy_query(&self.pkg_name)?; 
        let len = results.len();

        if len == 0 {
            println!("Query yielded no results for: '{}'", self.pkg_name);
            return Ok(None);
        }
        self.query_results = Some(results);
        
        if len <= QUERY_SHORT_THRESHOLD {
            return Ok(self.display_list_mode());
        } 
        else {
            return Ok(self.display_list_mode());
            //return Ok(self.display_tui_mode());
        }
    }
    /**
     */
    fn display_tui_mode(&self) -> Option<String> {
        todo!()
    }

    fn display_list_mode(&self) -> Option<String> {
        let query = self.query_results.as_ref().unwrap();
        let msg: &str = &format!("Query results for: {}\n0. Remove package\n{}\nEnter option: ", self.pkg_name, query.to_menu());
        let mut input: String;
        let mut num_input: usize;

        loop {
            print!("{}", msg);
            let _ = stdout().flush();
            input = String::new();

            stdin().read_line(&mut input).expect("Could not read user input");
            input = input.trim().into();
            num_input = match input.parse::<usize>() {
                Ok(num) => num,
                Err(_) => {
                    println!("Please enter an option above");
                    continue;
                }
            };

            let _ = stdout().flush();

            if num_input == 0 {
                return None;
            }
            else if num_input > 0 && num_input <= query.len(){
                return Some(query.0[num_input - 1].pkg_name.to_string());
            }
            else {
                println!("Please enter an option above");
                continue;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
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
