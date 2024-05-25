/*!
 * Manual installation wizard.
 * Runs an automatic install, if given a .charon/.toml file.
 * Otherwise, runs a guided install. 
*/

mod installation_cmd;
mod installer;
use std::{fs, os::unix::fs::PermissionsExt, path::{Path, PathBuf}};

use installation_cmd::InstallationCmd;
use mythos_core::{cli::clean_cli_args, dirs, logger::{get_logger_id, set_logger_id}, printerror, printinfo, printwarn};
use toml::Value;

fn main() {
    set_logger_id("CHARON");

    // If no args are provided, do guided install.
    let args = clean_cli_args();
    let mut path: Option<&str> = None;
    let mut do_dry_run = false;
    let mut do_guided_install = false;

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            return;
        } 
        if arg == "-n" || arg == "--dryrun" {
            do_dry_run = true;
        } else if arg == "-g" || arg == "--guided" {
            do_guided_install = true;
        } else if arg == "-a" || arg == "--auto" {
            do_guided_install = false;
        } else if arg.starts_with("-") {
            printerror!("Unknown arg: {arg}.");
            return;
        } else { 
            path = Some(arg);
            break;
        }
    }
    // charon
    // charon <path>
    //  - Do auto install
    // charon -g | --guided
    // charon -a | --auto [path]
    //  - Do auto install

    let installation_cmd = if path.is_none() {
        println!("Starting guided installation");
        guided_install().unwrap()
    } else {
        println!("Starting auto installation");
        auto_install(PathBuf::from(path.unwrap())).unwrap()
    };

    installer::run_installation(&installation_cmd, do_dry_run);
}
fn guided_install() -> Option<InstallationCmd> {
    //! Install program using guided input from user.
    let cmd = InstallationCmd::new();

    // Get mkdirs
    // Get binary
    // Get source
    // Get config
    // Make .desktop?
    todo!();
    // return Some(cmd);
}
fn auto_install(path: PathBuf) -> Option<InstallationCmd> {
    //! Install program using a .charon file.
    let table;
    let parent: PathBuf = path.parent().unwrap_or(&Path::new("")).to_path_buf();

    if let Ok(file) = fs::read_to_string(&path) {
        if let Ok(Value::Table(v)) = toml::from_str::<Value>(&file) {
            table = v;
        } else {
            return None;
        }
    } else {
        return None;
    }

    // Init
    let mut cmd = InstallationCmd::new();

    // Use filename as util name.
    cmd.set_name(&path.with_extension("").file_name().unwrap().to_string_lossy().into_owned());

    for dir in table.iter() {
        // Key is target mythos_dir.
        // This is appended to the beginning of dest.
        // Value is a list of items to install.
        if let Value::Array(list) = dir.1 {
            for item in list {
                let path = match cmd.add_dir(dir.0) {
                    Some(path) => path,
                    None => {
                        printwarn!("Not a valid path shortcut: {}", dir.0);
                        continue;
                    }
                };
                // cmd.add_item(&parent.join(path), item);
                cmd.add_item(&path, item);
            }
        }
    }
    return Some(cmd);
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use mythos_core::dirs;
    use crate::*;

    #[test]
    fn test_mkdirs() {
        let val = auto_install("data/charon.toml".into()).unwrap();
        let dir1 = dirs::expand_mythos_shortcut("d", "charon").unwrap();
        let dir2 = dirs::expand_mythos_shortcut("b", "charon").unwrap();
        let path1 = PathBuf::from(dir1);
        let path2 = PathBuf::from(dir2);
        assert_eq!(val.mkdirs, vec![path2, path1]);
    }
    #[test]
    fn test_dest() {
        let val = auto_install("data/charon.toml".into()).unwrap();
        let dir = dirs::expand_mythos_shortcut("d", "charon").unwrap();
        let mut path = PathBuf::from(dir);
        path.push("index.charon");
        assert_eq!(val.items[1].dest, path);
    }
    #[test]
    fn test_install() {
        let cmd = auto_install("data/charon.toml".into()).unwrap();
        let dir = dirs::expand_mythos_shortcut("d", "charon").unwrap();
        let mut path = PathBuf::from(dir);
        path.push("index.charon");

        let val = installer::install(&cmd, true);

        assert!(val.contains(&path.to_string_lossy().to_string()));
    }
}
