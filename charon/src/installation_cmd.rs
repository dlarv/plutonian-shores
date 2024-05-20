use std::path::PathBuf;

use mythos_core::dirs;
use toml::Value;

/**
 * A list of all items that must be installed.
 */
#[derive(Debug)]
pub struct InstallationCmd {
    pub items: Vec<InstallItem>,
    pub mkdirs: Vec<PathBuf>,
    pub name: String,
}

/**
 * A single item to be installed.
 */
#[derive(Debug)]
pub struct InstallItem {
    /// Path of item to be installed.
    pub target: PathBuf,
    /// Path item is to be installed to.
    pub dest: PathBuf,
    /// Permission of file in 000
    pub perms: u32,
    /// Remove extension from target file name.
    pub strip_ext: bool,
    /// Optional name of installed file.
    pub alias: Option<PathBuf>,
    /// Overwrite file if it already exists?
    pub overwrite: bool,
    /// Comments made during installation process. Used for logging.
    pub comment: String,
    /// Location to look for updates.
    pub source: Option<String>,
}

impl InstallationCmd {
    pub fn new() -> InstallationCmd {
        return InstallationCmd {
            items: Vec::new(),
            mkdirs: Vec::new(),
            name: "".into(),
        };
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.into();
    }
    pub fn add_item(&mut self, target: &PathBuf, val: &Value) {
        let mut cmd = InstallItem {
            target: "".into(),
            dest: target.into(),
            perms: 000,
            strip_ext: false,
            alias: None,
            overwrite: true,
            comment: "".into(),
            source: None,
        };
        let table = match val {
            Value::String(v) => {
                cmd.dest = v.into();
                self.items.push(cmd);
                return;
            },
            Value::Table(table) => {
                table
            },
            _ => return,
        };
        let mut dest = None;
        if let Some(Value::String(val)) = table.get("target") {
            cmd.target = val.into();
        }
        if let Some(Value::String(val)) = table.get("dest") {
            dest = Some(PathBuf::from(val));
        }
        if let Some(Value::Integer(val)) = table.get("perms") {
            cmd.perms = val.to_owned() as u32;
        }
        if let Some(Value::Boolean(val)) = table.get("strip_ext") {
            cmd.strip_ext = val.to_owned();
        }
        if let Some(Value::String(val)) = table.get("alias") {
            cmd.alias = Some(val.into());
        }
        if let Some(Value::Boolean(val)) = table.get("overwrite") {
            cmd.overwrite = val.to_owned();
        }
        if let Some(Value::String(val)) = table.get("comment") {
            cmd.comment = val.to_owned();
        }
        if let Some(Value::String(val)) = table.get("source") {
            cmd.source = Some(val.to_owned());
        }
        // alias >> strip_ext >> dest >> target_file_name
        if let Some(alias) = &cmd.alias {
            cmd.dest.push(alias)
        } 
        else if let Some(dest) = dest {
            // Remove extension, if applicable.
            if cmd.strip_ext {
                cmd.dest.push(dest.file_stem().unwrap());
            } else {
                cmd.dest.push(dest);
            }
        } else {
            cmd.dest.push(cmd.target.file_name().unwrap());
        }

        self.items.push(cmd);
    }
    pub fn add_dir(&mut self, dir: &str) -> Option<PathBuf> {
        if let Some(path) = dirs::expand_mythos_shortcut(dir, "charon") {
            if !self.mkdirs.contains(&path) {
                self.mkdirs.push(path.to_owned());
            }
            return Some(path);
        }
        return None;
    }
}

impl InstallItem {
    pub fn print_dest(&self) -> String {
        return self.dest.to_string_lossy().to_string();
    }
    pub fn print_comment(&self) -> String {
        return self.comment.to_string();
    }
}

