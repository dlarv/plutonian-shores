use crate::utils::{parse_xbps_output, read_multiple_index, read_single_index};
use std::{fs, process::{Command, Stdio}};

use mythos_core::{cli::get_cli_input, dirs, fatalmsg, printerror};
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
            return Ok(Query::from(query));
        };
        return Err(QueryError::NotFound(format!("Package not found: '{search_term}'")));
    }

    pub fn query_xbps(search_term: &str) -> Option<Query> {
        let raw_results = Command::new("xrs")
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Using search_term here works, unless no pkgs are found.
            // So if the user does cocytus 'bledner' instead of 'blender', it will return nothing.
            // Doing it this way would allow the query to find what the user likely meant.
            .arg("")
            .output()
            .expect(&fatalmsg!("Error running query for {search_term}"))
            .stdout;


        let (mut results, longest_name) = parse_xbps_output(raw_results, search_term, THRESHOLD);

        if results.len() == 0 {
            return None;
        }

        results.sort_by(|a, b| b.score.cmp(&a.score));

        if results[0].score >= 100 {
            results = vec![results.remove(0)];
        }

        return Some(Query { results, longest_name, pkg_name: search_term.into() });
    }
    pub fn query_charon(search_term: &str) -> Option<QueryResult> {
        //! Check if search term is contained inside of index.charon.
        let path = dirs::get_path(dirs::MythosDir::Data, "charon/index.charon")?;
        let res = match fs::read_to_string(path) {
            Ok(res) => res,
            Err(msg) => {
                printerror!("Could not load charon file: {msg}");
                return None;
            }
        };

        let table: Value = match toml::from_str(&res) {
            Ok(table) => table,
            Err(msg) => {
                printerror!("Could not parse charon file: {msg}");
                return None;
            }
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
    pub fn select_from_results(&self) -> Option<Query> {
        /*!
            * Allows user to select packages by indices.
            * Return None if user selected 0, thereby cancelling selection.
            * Else return Query, where its results are the packages they selected.
        */
        let msg = &format!("{list}\n0. Remove package\nEnter from the options above: ", list = self.get_short_list());
        loop {
            let input = get_cli_input(msg);
            if input == "0" {
                return None;
            }
            let results = if input.find(" ").is_none() {
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
                Some(res) => Some(Query { results: res.0, longest_name: res.1, pkg_name: format!("{} (Modified)", self.pkg_name) }),
                None => {
                    eprintln!("Please enter an option above");
                    continue;
                }
            };
        }
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
            Some(size) =>  {
                // If size.col < (...), then columns will be cast/rounded to 0.
                // This will lead to a divide by zero error later on.
                let div = (self.longest_name + num_digits + 2) as u16;
                if size.cols < div {
                    1
                } else {
                    size.cols / div
                }
            },
            None => 1
        };

        let mut row_counter = 1;
        let longest_name = self.longest_name;
        for (i, res) in self.results.iter().enumerate() {
            // Current index of number + padding zeros + '.' + name + ' '
            output += &format!("{id:0$}. {name: <longest_name$} ", num_digits, id = i + 1, name = res.pkg_name); 
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
    pub fn get_pkg_names<'a>(&'a self) -> Vec<&'a str> {
        return self.results.iter().map(|p| p.pkg_name.as_str()).collect::<Vec<&str>>();
    }

    pub fn len(&self) -> usize {
        return self.results.len();
    }
    pub fn get<'a>(&'a self, index: usize) -> Option<&'a QueryResult> {
        if index >= self.results.len() {
            return None;
        }
        return Some(&self.results[index]);
    }
}
impl From<QueryResult> for Query {
    fn from(value: QueryResult) -> Self {
        let name = &value.pkg_name;
        return Query {
            pkg_name: name.to_owned(),
            results: vec![value.clone()],
            longest_name: name.len(),
        };
    }
}
impl From<Vec<QueryResult>> for Query {
    fn from(value: Vec<QueryResult>) -> Self {
        let longest_name: usize = if value.len() > 0 {
            value.iter().reduce(|acc, x| { 
                if x.pkg_name.len() > acc.pkg_name.len() { 
                    x 
                } else { 
                    acc 
                }
            }).unwrap().pkg_name.len()
        } else {
            0
        };

        return Query {
            pkg_name: "".into(),
            results: value,
            longest_name,
        };
    }
}
impl IntoIterator for Query {
    type Item = QueryResult;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        return self.results.into_iter();
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    /*! # Test Plan
        * - Query xbps for incomplete name (e.g. ble).
        * - Query xbps for mispelled name (e.g. blneder).
        * - Query charon for uninstalled pkg.
        * - Query charon using incomplete name.
        * - Query charon using mispelled name.
        * - Select from results.
        * - Select multiple results from 1 query.
        * - Select from multiple queries.
     */
    use crate::*;

    #[test]
    fn test_xbps_query() {
        let res = Query::query_xbps("blende").unwrap();
        assert_eq!(res.results[0].pkg_name, "blender");
    }
    #[test]
    fn test_charon_query() {
        let res = Query::query_charon("charon");
        assert_eq!("charon", res.unwrap().pkg_name);
        let res2 = Query::query_charon("hello");
        assert!(matches!(res2, None));
    }
    #[test]
    fn test_exact_match() {
        let res = Query::query_xbps("blender").unwrap();
        assert_eq!(res.results[0].pkg_name, "blender");
        assert_eq!(res.results.len(), 1);
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
