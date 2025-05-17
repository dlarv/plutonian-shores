use crate::utils::{parse_xbps_output, read_multiple_index, read_single_index};
use std::{fs, io::{stdin, stdout, Read, Write}, process::{Command, Stdio}};

use mythos_core::{cli::{self, get_cli_input}, dirs, fatalmsg, printerror};
use toml::Value;

use termion::{self, clear};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use crate::{Query, QueryError, QueryResult};

// Minimum score package must get using fuzzy find to be included in results.
const THRESHOLD: f32 = 0.3;
const SMALL_LIST_SIZE: usize = 50;

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
        if self.len() <= SMALL_LIST_SIZE {
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
                } else {
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

        // List is too large to display, use less-like format.
        let res = self.show_long_list("0 to Remove package or select from the options above: ")?;
        return Some(Query { 
            results: res.0, 
            longest_name: res.1, 
            pkg_name: format!("{} (Modified)", self.pkg_name) 
        });
    }
    pub fn get_short_list(&self) -> String {
        return self.generate_list(0, None);
    }
    fn generate_list(&self, skip_row: usize, end_row: Option<usize>) -> String {
        /*!
         * Display list of results in separate columns.
         * Columns = termsize.width / (self.longest_name + id), where id = '1. ', '01. ', etc.
         *
         * skip_row: Number of rows to skip.
        */

        let mut output = String::new();
        let num_digits = (self.results.len()as f32).log10() as usize + 1;
        let columns = match termsize::get() {
            Some(size) =>  {
                // Index of number + padding zeros + '.' + ' ' + longest_name + ' '
                let div = (self.longest_name + num_digits + 2) as u16;
                // If size.col < (...), then columns will be cast/rounded to 0.
                // This will lead to a divide by zero error later on.
                if size.cols < div {
                    1
                } else {
                    size.cols / div
                }
            },
            None => 1
        };

        let start_index = columns as usize * skip_row;
        let end_index = if let Some(end) = end_row {
            columns as usize * end
        } else {
            self.len()
        };

        let mut row_counter = 1;
        let longest_name = self.longest_name;

        let mut i = start_index as isize - 1;
        for res in self.results.iter().skip(start_index) {
            i += 1;
            if i == end_index as isize { break; }
            // Current index of number + padding zeros + '.' + name + ' '
            output += &format!("{id:0$}. {name: <longest_name$} ", num_digits, id = i + 1, name = res.pkg_name); 
            // Loop down to next row
            if row_counter % columns == 0 {
                output += "\n\r";
                row_counter = 1;
            } else {
                row_counter += 1;
            }
        }
        return output;
    }
    fn show_long_list(&self, msg: &str) -> Option<(Vec<QueryResult>, usize)> {
        // Hides cursor for lifetime of object.
        let c = termion::cursor::Hide;

        let mut indices: String = String::new();
        let mut start_row = 0;
        let page_length: usize = match termsize::get() {
            // -- for the footer.
            Some(size) => size.rows as usize - 4,
            None => 1
        };
        let mut end_row = page_length;

        // These calculations are repeated inside of generate_list(...)
        let final_row: usize = {
            let num_digits = (self.results.len()as f32).log10() as usize + 1;
            let columns = match termsize::get() {
                Some(size) =>  {
                    // Index of number + padding zeros + '.' + ' ' + longest_name + ' '
                    let div = (self.longest_name + num_digits + 2) as u16;
                    // If size.col < (...), then columns will be cast/rounded to 0.
                    // This will lead to a divide by zero error later on.
                    if size.cols < div {
                        1
                    } else {
                        size.cols / div
                    }
                },
                None => 1
            };
            self.results.len() / columns as usize
        };

        let refresh = |start_row: &usize, end_row: &usize, indices: &String| {
            print!("{}{}\n\r{}{}\n", 
                termion::clear::All,
                self.generate_list(*start_row, Some(*end_row)), 
                msg,
                &indices);
            // let c = termion::cursor::Up;
        };


        let stdout = std::io::stdout().into_raw_mode().unwrap();
        // Hides cursor for lifetime of object.
        let c = termion::cursor::HideCursor::from(stdout);

        let mut stdin = termion::async_stdin().keys();

        refresh(&start_row, &end_row, &indices);

        loop {
            let input = match stdin.next() {
                Some(Ok(input)) => input,
                Some(Err(err)) => {
                    printerror!("{err}");
                    return None;
                },
                None => continue
            };

            match input {
                termion::event::Key::Char('j') => {
                    if start_row == final_row {
                        continue;
                    }
                    start_row += 1;
                    end_row = start_row + page_length;
                    refresh(&start_row, &end_row, &indices);
                },
                termion::event::Key::Char('k') => {
                    if start_row == 0 {
                        continue;
                    }
                    start_row = (start_row as isize - 1) as usize;
                    end_row = start_row + page_length;
                    refresh(&start_row, &end_row, &indices);
                },
                termion::event::Key::Char('G') => {
                    start_row = final_row;
                    end_row = start_row + page_length;
                    refresh(&start_row, &end_row, &indices);
                },
                termion::event::Key::Char('g') => {
                    start_row = 0;
                    end_row = start_row + page_length;
                    refresh(&start_row, &end_row, &indices);
                },
                termion::event::Key::Ctrl('d') => {
                    start_row = std::cmp::max(start_row as isize - 10, 0) as usize;
                    end_row = start_row + page_length;
                    refresh(&start_row, &end_row, &indices);
                },
                termion::event::Key::Ctrl('u') => {
                    start_row = std::cmp::min(start_row + 10, final_row);
                    end_row = start_row + page_length;
                    refresh(&start_row, &end_row, &indices);
                },
                termion::event::Key::Char('q') => break,
                termion::event::Key::Backspace => {
                    indices.pop();
                    refresh(&start_row, &end_row, &indices);
                },
                termion::event::Key::Char(ch) => {
                    if ch >= '1' && ch <= '9' || ch == ' ' {
                        indices.push(ch);
                        refresh(&start_row, &end_row, &indices);
                    } else if ch == '0' {
                        return None;
                    } if ch == '\n' {
                        break;
                    }
                },
                _ => continue
            }
        }

        let results = if indices.find(" ").is_none() {
            let r = read_single_index(&indices, &self.results);
            if let Some(vals) = r {
                Some((vec![vals.0], vals.1))
            } else {
                None
            }
        } else {
            read_multiple_index(&indices, &self.results)
        };
        return results;
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
    // #[test]
    fn test_long_display_list() {
        let res = Query::query_xbps("b").unwrap();
        let output = res.show_long_list("...");
        assert!(true);
    }
}
