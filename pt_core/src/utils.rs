use rust_fuzzy_search::fuzzy_compare;

use crate::QueryResult;


/**
 * Assigns a value between 0.0, 1.0
 * Returns None if value is below a given threshold
 */
fn score_result(search_term: &str, name: &str, threshold: f32) -> Option<i32> {
    let score = fuzzy_compare(&search_term, &name);
    if score >= threshold {
        return Some((score * 100.0) as i32);
    }
    return None;
}
/**
 * xbps-query -> <block-of-text> | parse_results -> <structured-data>
 */
pub fn parse_xbps_output(raw_results: Vec<u8>, search_term: &str, fuzzy_find_threshold: f32) -> (Vec<QueryResult>, usize) {
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
            match score_result(&search_term, &name, fuzzy_find_threshold) {
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
pub fn read_single_index(input: &str, query: &Vec<QueryResult>) -> Option<(QueryResult, usize)> {
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
pub fn read_multiple_index(input: &str, query: &Vec<QueryResult>) -> Option<(Vec<QueryResult>, usize)> {
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
