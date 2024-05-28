use std::path::PathBuf;

use mythos_core::dirs;
use pt_core::QueryResult;
use toml::Value;

/**
 * A list of all items that must be installed.
 */
#[derive(Debug)]
pub struct InstallationCmd {
    pub items: Vec<InstallItem>,
    pub mkdirs: Vec<PathBuf>,
    pub name: String,
    /// Location to look for updates.
    pub source: Option<String>,
    /// Package version
    pub version: Option<String>,
    /// Package description
    pub description: Option<String>,
}

/**
 * A single item to be installed.
 */
#[derive(Debug, Clone)]
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
}

impl InstallationCmd {
    pub fn new() -> InstallationCmd {
        return InstallationCmd {
            items: Vec::new(),
            mkdirs: Vec::new(),
            name: "".into(),
            source: None,
            version: None,
            description: None,
            
        };
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.into();
    }
    pub fn add_item(&mut self, parent: &PathBuf, dest: &PathBuf, val: &Value) {
        let mut cmd = InstallItem {
            target: parent.into(),
            dest: dest.into(),
            perms: 000,
            strip_ext: false,
            alias: None,
            overwrite: true,
            comment: "".into(),
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
            // cmd.target = val.into();
            cmd.target = parent.join(&val).canonicalize().unwrap_or(val.into());
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
        println!("Copying {target:#?} --> {dest:#?}", target = cmd.target, dest = cmd.dest);
        self.items.push(cmd);
    }
    pub fn add_simple_item(&mut self, target: PathBuf, dest: PathBuf, perms:u32, overwrite: bool, strip_ext: bool) {
        //! Add item without using a toml file.
        let dest = if strip_ext {
            dest.join(PathBuf::from(target.file_stem().unwrap_or_default()).file_name().unwrap_or_default())
        } else {
            dest
        };
        let item = InstallItem {
            target,
            dest,
            perms,
            strip_ext,
            alias: None,
            overwrite,
            comment: "".to_string(),
        };
        self.items.push(item);
    }
    pub fn add_dir(&mut self, dir: &str) -> Option<PathBuf> {
        if let Some(path) = dirs::expand_mythos_shortcut(dir, "charon") {
            if !self.mkdirs.contains(&path) {
                println!("Creating directory: {path:#?}");
                self.mkdirs.push(path.to_owned());
            }
            return Some(path);
        }
        return None;
    }
    pub fn to_toml_str(&self) -> String {
        let mut output = format!("{} = {{", self.name);
        if let Some(val) = &self.version {
            output += &format!("version = \"{val}\", ");
        }
        if let Some(val) = &self.description{
            output += &format!("description= \"{val}\", ");
        }
        let src = if let Some(val) = &self.source {
            val
        } else {
            "\"charon\""
        };
        output += &format!("source = \"{src}\" }}");
        return output;
    }
}

impl InstallItem {
    pub fn new() -> InstallItem {
        return InstallItem {
            target: PathBuf::new(),
            dest: PathBuf::new(),
            perms: 0,
            strip_ext: false,
            alias: None,
            overwrite: true,
            comment: "".into(),
        };
    }
    pub fn target(&mut self, target: PathBuf) -> &mut InstallItem {
        self.target = target;
        return self;
    }
    pub fn perms(&mut self, val: u32) -> &mut InstallItem {
        self.perms = val;
        return self;
    }
    pub fn strip_ext(&mut self, val: bool) -> &mut InstallItem {
        self.strip_ext = val;
        return self;
    }
    pub fn overwrite(&mut self, val: bool) -> &mut InstallItem {
        self.overwrite = val;
        return self;
    }
    pub fn comment(&mut self, val: String) -> &mut InstallItem {
        self.comment = val;
        return self;
    }
    pub fn print_dest(&self) -> String {
        return self.dest.to_string_lossy().to_string();
    }
    pub fn to_toml_str(&self) -> String {
        let mut output = format!("{{ target = {:?}", self.target);

        if self.perms > 0 {
            output += &format!(", perms = {}", self.perms);
        }
        if self.strip_ext {
            output += ", strip_ext = true";
        }
        if !self.overwrite {
            output += ", overwrite = true";
        }
        if let Some(alias) = &self.alias {
            output += &format!(", alias = {alias:?}");
        }
        if self.comment.len() > 0 {
            output += &format!(", comment = {:?}", self.comment);
        }
        output += "}";
        return output;
    }
}

