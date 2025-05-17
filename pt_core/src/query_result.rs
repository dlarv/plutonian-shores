use duct::cmd;
use mythos_core::printerror;
use crate::{QueryResult, parse_output};

// Runs xq {pkg} | head -n HEAD_LINE_COUNT
const HEAD_LINE_COUNT: &str = "12";

impl QueryResult {
    pub fn long_display(&self) -> String {
        /*!
            * Run xq {pkg} | head -n HEAD_LINE_COUNT
        */
        let cmd = cmd!("xq", &self.pkg_name).pipe(cmd!("head", "-n", HEAD_LINE_COUNT));

        let output = match cmd.run() {
            Ok(output) => parse_output(output.stdout),
            Err(msg) => {
                printerror!("{msg}");
                return "".into();
            }
        };
        return output;
    }
    pub fn display(&self) -> String {
        /*!
            * Show info about package. Including description, version, etc.
            * {PKG_NAME} [{*|-}]
            * Version: {version}
            * Description: {description}
        */
        let output = format!("{} [{}]\nVersion: {}\nDescription: {}",
            self.pkg_name,
            if self.is_installed { "*" } else { "-" },
            self.pkg_version,
            self.pkg_description);

        return output;
    }
}

impl PartialEq for QueryResult {
    fn eq(&self, other: &Self) -> bool {
        return self.pkg_name == other.pkg_name;
    }
}
