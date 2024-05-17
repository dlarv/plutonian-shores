/*!
 * Functionality for querying void repo and displaying the results 
 *
 */
use crate::query_manager::*;
use std::process::{Command, Stdio};
use mythos_core::{fatalmsg, logger::get_logger_id};
use rust_fuzzy_search::fuzzy_compare;

pub static mut THRESHOLD: f32 = 0.3;
pub static mut LIST_COLUMN_LEN: usize = 20;
impl QueryResults {
    /**
     * Uses fuzzy search to match packages in void repo to {search_term}
     */
    pub fn fuzzy_query(search_term: &Package) -> Option<QueryResults> {
        let results = search_repo(&"".into());
        let mut output = QueryResults::parse_results(results, search_term);  
        output.sort(); 
        if output.len() == 0 {
            return None;
        }
        return Some(output);
    }

    /**
     * Queries void repo using exactly {search_term}
     */
    pub fn strict_query(search_term: &Package) -> Option<QueryResults> {
        let results = search_repo(search_term);
        let mut output = QueryResults::parse_results(results, search_term);
        output.sort();
        if output.len() == 0 {
            return None;
        }
        return Some(output);
    }

    /**
     * xbps-query -> <block-of-text> | parse_results -> <structured-data>
     */
    pub fn parse_results(raw_results: Vec<u8>, search_term: &Package) -> QueryResults {
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
        return QueryResults(output, longest_name);
    }
    pub fn to_list(&self) -> String {
        let spacer = " ".repeat(self.1);
        let mut output: Vec<String> = Vec::with_capacity(self.0.len());
        
        for result in &self.0 {
            let is_installed_marker = if result.is_installed {
                "*"
            } 
            else { 
                "-"
            };
            output.push(format!("[{is_installed_marker}] {}-{}{spacer}{}", result.pkg_name, result.pkg_version, result.pkg_description));
        }
        return output.join("\n");
    }

    /**
     * Sorts list in descending order by {score}
     */
    pub fn sort(&mut self) {
        self.0.sort_by(|a, b| b.score.cmp(&a.score));
    }

    pub fn filter_include(&mut self, filter_term: &str) { 
        self.0.retain(|query| query.pkg_name.contains(filter_term) || query.pkg_description.contains(filter_term));
    }
    pub fn filter_exclude(&mut self, filter_term: &str) { 
        self.0.retain(|query| !query.pkg_name.contains(filter_term) && !query.pkg_description.contains(filter_term));
    }
    pub fn len(&self) -> usize {
        return self.0.len();
    }
    pub fn to_menu(&self) -> String {
        let mut output: String = String::new();
        let col_count: usize;
        unsafe {
            col_count = std::cmp::max(self.len() / LIST_COLUMN_LEN, 1);
        }

        for (i, item) in self.0.iter().enumerate() {
            output += &format!("{}. {}", i + 1, item.pkg_name);

            if i % col_count == 0 {
                output += "\n";
            }
            else {
                output += &" ".repeat(self.1 - item.pkg_name.len() + 1);
            }
        }
        return output;
    }
}

/// Expects {search_term} to already be formatted.
fn search_repo(search_term: &Package) -> Vec<u8> {
    // I think xrs searches through cache
    // It doesn't take sudo to run 
    return Command::new("xrs")
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg(search_term)
        .output()
        .expect(&fatalmsg!("Error running query for {search_term}"))
        .stdout;
}

/**
 * Wraps rust_fuzzy_search (crate)
 * Assigns a value between 0.0, 1.0
 * Returns None if value is below a given threshold
 */
fn score_result(search_term: &Package, name: &Package) -> Option<i32> {
    let score = fuzzy_compare(&search_term, &name);
    unsafe {
        if score >= THRESHOLD {
            return Some((score * 100.0) as i32);
        }
    }
    return None;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_command(search_term: &str) -> Command {
        let mut cmd = Command::new("xrs");
        cmd.stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        cmd.arg(search_term);
        return cmd;
    }

    #[test]
    fn test_parse_results() {
        const TERM: &str = "feh";
        let mut cmd = build_command(&TERM);
        let res = cmd.output().unwrap().stdout;

        let p = QueryResults::parse_results(res, &TERM.into()).0;

        assert!(p[0].pkg_name.starts_with("feh"));
        assert_eq!(p[0].is_installed, true); 

        assert!(p[1].pkg_name.starts_with("fehQlibs"));
        assert_eq!(p[1].is_installed, false); 
    }

    #[test]
    fn test_fuzzy_search() {
        const TERM: &str = "blneder";
        let mut cmd = build_command("");
        let res = cmd.output().unwrap().stdout;
        let p = QueryResults::parse_results(res, &TERM.into()).0;
        //println!("{:#?}", p);
        assert_ne!(p.len(), 0);
    } 

    #[test]
    fn test_sorting() {
        const TERM: &str = "blen";
        let mut cmd = build_command("");
        let res = cmd.output().unwrap().stdout;
        let mut p = QueryResults::parse_results(res, &TERM.into()).0;
        p.sort_by(|a, b| b.score.cmp(&a.score));
        assert_eq!(p[0].pkg_name, "blender");
    } 
}

