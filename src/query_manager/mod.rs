#![allow(dead_code)]
pub mod query_results;
pub mod pkg_selector;

pub type Package = String;

#[derive(Debug)]
pub struct QueryResult {
    is_installed: bool,
    pkg_name: String,
    pkg_version: String,
    pkg_description: String,
    score: i32,
}

#[derive(Debug)]
pub struct QueryResults (Vec<QueryResult>, usize);

#[derive(Debug)]
pub struct PackageSelector {
    pkg_name: Package,
    query_results: Option<QueryResults>,
}

#[derive(Debug)]
pub enum PackageSelection {
    Package(Package),
    Packages(Box<Vec<Package>>),
    OtherOpt(usize),
    None
}
