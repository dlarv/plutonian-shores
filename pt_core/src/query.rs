use std::{fs, num, process::{Command, Stdio}};

use mythos_core::{cli::get_cli_input, dirs, fatalmsg, logger::get_logger_id};
use rust_fuzzy_search::fuzzy_compare;
use toml::Value;

use crate::{Query, QueryError, QueryResult};

// Minimum score package must get using fuzzy find to be included in results.
const THRESHOLD: f32 = 0.3;

impl Query{
    pub fn query(search_term: &str) -> Result<Query, QueryError> {
        /*!
            * Find packages that match search_term.
            * Tries to find package using xrs.
            * Then checks to see if program was installed using charon.
            * Finally, checks list of tertiary package managers.
         */
        if let Some(query) = Query::query_xbps(search_term) {
            return Ok(query);
        };
        if let Some(query) = Query::query_charon(search_term) {
            return Ok(Query::from_query_result(query));
        };
        return Err(QueryError::NotFound(format!("Package not found: '{search_term}'")));
    }
    pub fn from_query_result(res: QueryResult) -> Query {
        let name = &res.pkg_name;
        return Query {
            pkg_name: name.to_owned(),
            results: vec![res.clone()],
            longest_name: name.len(),
        };
    }

    pub fn query_xbps(search_term: &str) -> Option<Query> {
        let raw_results = Command::new("xrs")
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg(search_term)
            .output()
            .expect(&fatalmsg!("Error running query for {search_term}"))
            .stdout;

        let (mut results, longest_name) = parse_xbps_output(raw_results, search_term);

        if results.len() == 0 {
            return None;
        }

        results.sort_by(|a, b| a.score.cmp(&b.score));
        return Some(Query { results, longest_name, pkg_name: search_term.into() });
    }

    pub fn query_charon(search_term: &str) -> Option<QueryResult> {
        //! Check if search term is contained inside of index.charon.
        let path = dirs::get_dir(dirs::MythosDir::Data, "charon/index.charon")?;
        let res = match fs::read_to_string(path) {
            Ok(res) => res,
            Err(_) => return None
        };

        let table: Value = match toml::from_str(&res) {
            Ok(table) => table,
            Err(_) => return None,
        };

        // Get table value
        if let Value::Table(table) = table {
            if let Some(Value::Table(val)) = &table.get(search_term) {
                return Some(QueryResult {
                    is_installed: true,
                    pkg_name: search_term.into(),
                    pkg_version: if let Some(Value::String(val)) = val.get("version") { val.to_owned() } else { "".into() },
                    pkg_description: if let Some(Value::String(val)) = val.get("description") { val.to_owned() } else { "".into() },
                    score: 100,
                });
            }
        }

        return None;
    }

    pub fn select_from_results(&self) -> Query {
        /*!
            * Allows user to select packages by indices.
        */
        loop {
            let input = get_cli_input(&self.build_msg());
            let results = if input.find(" ") == None {
                let r = read_single_index(&input, &self.results);
                if let Some(vals) = r {
                    Some((vec![vals.0], vals.1))
                } else {
                    None
                }
            }
            else {
                read_multiple_index(&input, &self.results)
            };

            return match results {
                Some(res) => Query { results: res.0, longest_name: res.1, pkg_name: format!("{} (Modified)", self.pkg_name) },
                None => {
                    eprintln!("Please enter an option above");
                    continue;
                }
            };
        }
    }
    pub fn replace_package(&mut self) {
        /*!
         * 
        */
    }
    pub fn get_short_list(&self) -> String {
        /*!
         * Display list of results in separate columns.
         * Columns = termsize.width / (self.longest_name + id), where id = '1. ', '01. ', etc.
        */
        let mut output = String::new();

        let num_digits = self.results.len() / 10;
        let columns = match termsize::get() {
            // Index of number + padding zeros + '.' + ' ' + longest_name + ' '
            Some(size) =>  size.cols / (self.longest_name + num_digits + 2) as u16,
            None => 1
        };

        let mut row_counter = 1;
        let longest_name = self.longest_name;
        for (i, res) in self.results.iter().enumerate() {
            // Current index of number + padding zeros + '.' + name + ' '
            output += &format!("{i:0$}. {name: <longest_name$}", num_digits, name = res.pkg_name); 

            // Loop down to next row
            if row_counter % columns == 0 {
                output += "\n";
                row_counter = 1;
            } else {
                row_counter += 1;
            }
        }
        return output;
    }
    fn build_msg(&self) -> String {
        let mut menu : String = String::new();
        let col_count: usize = 20;

        for (i, item) in self.results.iter().enumerate() {
            menu += &format!("{}. {}", i + 1, item.pkg_name);

            if i % col_count == 0 {
                menu += "\n";
            }
            else {
                menu += &" ".repeat(self.longest_name - item.pkg_name.len() + 1);
            }
        }
        return format!("Query results for: {name}\n{menu}\n0. Remove pkg\nEnter option: ", name=self.pkg_name);
    }
}
/**
 * Wraps rust_fuzzy_search (crate)
 * Assigns a value between 0.0, 1.0
 * Returns None if value is below a given threshold
 */
fn score_result(search_term: &str, name: &str) -> Option<i32> {
    let score = fuzzy_compare(&search_term, &name);
    if score >= THRESHOLD {
        return Some((score * 100.0) as i32);
    }
    return None;
}
/**
 * xbps-query -> <block-of-text> | parse_results -> <structured-data>
 */
fn parse_xbps_output(raw_results: Vec<u8>, search_term: &str) -> (Vec<QueryResult>, usize) {
    enum Mode { IsInstalled, NameBlock, Gap, Description }
    let mut output: Vec<QueryResult> = Vec::new();

    let mut traversal_mode: Mode = Mode::IsInstalled;
    let mut is_installed: bool = false;
    let mut name_block: String = "".into();
    let mut index: usize;
    let mut name: String = "".into();
    let mut version: String = "".into();
    let mut desc: String = "".into();
    let mut longest_name: usize = 0;

    // Query result format:
    // [<installed>] <name_block><ws*><description>\n  
    // <name_block> -> <name>-<version>
    // <name> can contain '-'
    // Last '-' in <name_block> is considered beginning of <version>
    for ch in raw_results {
        // Start new package
        if ch == b'\n' {
            match score_result(&search_term, &name) {
                Some(score) => {
                    if name.len() > longest_name {
                        longest_name = name.len();
                    }
                    output.push(QueryResult { 
                        is_installed, 
                        pkg_name: name.clone(),
                        pkg_version: version.clone(),
                        pkg_description: desc.clone(), 
                        score,
                    })
                },
                None => ()
            }
            name_block.clear();
            desc.clear();
            traversal_mode = Mode::IsInstalled;
        }
        match traversal_mode {
            Mode::IsInstalled => {
                if ch == b' ' {
                    traversal_mode = Mode::NameBlock;
                } else if ch == b'*' {
                    is_installed = true;
                } else if ch == b'-' {
                    is_installed = false; 
                }
            },
            Mode::NameBlock => {
                if ch.is_ascii_whitespace() {
                    traversal_mode = Mode::Gap;

                    // Separate <name>-<version>
                    index = match name_block.rfind('-') {
                        Some(index) => index, 
                        None => name_block.len()
                    };

                    name = name_block[0..index].to_string();
                    version = name_block[index..].to_string();
                } else {
                    name_block.push(ch as char);
                }
            },
            Mode::Description => {
                desc.push(ch as char);
            },
            Mode::Gap => {
                if !ch.is_ascii_whitespace() {
                    traversal_mode = Mode::Description;
                    desc.push(ch as char);
                }
            },
        }; // end match()
    } // end loop
    return (output, longest_name);
}
fn read_single_index(input: &str, query: &Vec<QueryResult>) -> Option<(QueryResult, usize)> {
    /*!
        * If input is a valid usize, get query[input]
        * Else, return None
        *
        * input is not 0-indexed, it starts at 1.
      */
    let num_input = match input.parse::<usize>() {
        Ok(0) | Err(_) => return None,
        Ok(num) => num,
    };
    if num_input > query.len() {
        return None;
    }
    let output = query[num_input - 1].clone();
    return Some((output.to_owned(), output.pkg_name.len()));
}
fn read_multiple_index(input: &str, query: &Vec<QueryResult>) -> Option<(Vec<QueryResult>, usize)> {
    let mut pkgs: Vec<QueryResult> = Vec::new();
    let mut longest_name: usize = 0;
    for num in input.split(" ") {
        match read_single_index(num, query) {
            Some((pkg, len)) => { 
                pkgs.push(pkg);
                longest_name = if len > longest_name {
                    len
                } else {
                    longest_name
                };
            },
            _ => {
                eprintln!("You cannot use '0' or {}+ while selecting multiple packages!", query.len());
                return None;
            }
        }
    }
    return Some((pkgs, longest_name));
}
#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_xbps_query() {
        let res = Query::query_xbps("blender").unwrap();
        assert_eq!(res.results[0].pkg_name, "blender");
    }
    #[test]
    fn test_charon_query() {
        let res = Query::query_charon("charon");
        assert_eq!("charon", res.unwrap().pkg_name);
        let res2 = Query::query_charon("hello");
        assert!(matches!(res2, None));
    }
    // #[test]
    fn test_selection() {
        let res = Query::query_xbps("blen").unwrap();
        let output = res.select_from_results();
        println!("{:?}", output);
    }
    // #[test]
    fn test_short_display_list() {
        let res = Query::query_xbps("bl").unwrap();
        let output = res.get_short_list();
        println!("{output}");
    }
}
