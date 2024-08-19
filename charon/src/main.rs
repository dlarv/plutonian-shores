/*!
 * Manual installation wizard.
 * Runs an automatic install, if given a .charon/.toml file.
*/

mod installation_cmd;
mod installer;
mod charon_file_creator;
use std::{env::current_dir, ffi::OsString, fs, path::{Path, PathBuf}};

use charon_file_creator::create_charon_file;
use installation_cmd::InstallationCmd;
use mythos_core::{cli::clean_cli_args, logger::set_id, printerror, printinfo, printwarn};
use toml::Value;

fn main() {
    let _ = set_id("CHARON");

    // If no args are provided, do guided install.
    let args = clean_cli_args();
    let mut path: Option<&str> = None;
    let mut do_dry_run = false;

    // charon
    // charon <path/to/dir/>
    // charon <path/to/charon>
    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("charon [opts] [path]\nBasic installer util that can use toml files to quickly install programs.\nopts:\n-h | --help\t\tPrint this menu\n-n | --dryrun\t\tRun command without making changes to filesystem\n-c | --create\t\tCreate a basic charon file");
                return;
            },
            "-n" | "--dryrun" => do_dry_run = true,
            "-c" | "--create" => {
                let path = match create_charon_file() {
                    Some(path) => path,
                    None => return
                };
                std::fs::write(current_dir().unwrap().join(path.1).with_extension("charon"), &path.0).unwrap();
                return;
            },
            _ => {
                if arg.starts_with("-") {
                    printerror!("Unknown arg: {arg}.");
                    return;
                }
                path = Some(arg);
            }
        }
    }

    let path: PathBuf = if let Some(path) = path{
        PathBuf::from(path)
    } else {
        match current_dir() {
            Ok(path) => path,
            Err(msg) => {
                printerror!("Could not read current dir: {msg}");
                return;
            }
        }
    };

    printinfo!("Starting auto installation\n");
    let installation_cmd = match auto_install(path) {
        Some(cmd) => cmd,
        None => {
            printinfo!("Could not install util, exiting...");
            return;
        }
    };

    printinfo!("\nBeginning installation:");
    installer::run_installation(&installation_cmd, do_dry_run);
}
fn auto_install(path: PathBuf) -> Option<InstallationCmd> {
    //! Install program using a .charon file.
    let table;

    let mut path = path.canonicalize().unwrap();
    // If user provided a directory, locate charon file.
    if path.is_dir() {
        path = match find_charon_file(&path) {
            Ok(path) => path,
            Err(msg) => {
                printwarn!("{msg}");
                return None;
            }
        }
    }
    printinfo!("Found charon file at {path:?}");

    let parent: PathBuf = path.parent().unwrap_or(&Path::new("")).to_path_buf().canonicalize().unwrap();

    match fs::read_to_string(&path) {
        Ok(file) => match toml::from_str::<Value>(&file) {
            Ok(Value::Table(v)) => table = v,
            Ok(val) => {
                printerror!("Formatting error in charon file. Expected table, found {val}");
                return None;
            },
            Err(msg) => {
                printerror!("Error in charon file: {msg}");
                return None;
            }
        },
        Err(msg) => {
            printerror!("Error reading charon file: {msg}");
            return None;
        }
    }

    // Init
    printinfo!("\nReading charon file:");
    let mut cmd = InstallationCmd::new();

    // Use filename as util name.
    cmd.set_name(&path.with_extension("").file_name().unwrap().to_string_lossy().into_owned());

    for dir in table.iter() {
        // Key is target mythos_dir.
        // This is appended to the beginning of dest.
        // Value is a list of items to install.
        if let Value::Array(list) = dir.1 {
            let path = match cmd.add_dir(dir.0) {
                Some(path) => path,
                None => {
                    printwarn!("Not a valid path shortcut: {}", dir.0);
                    continue;
                }
            };
            for item in list {
                // cmd.add_item(&parent.join(path), item);
                cmd.add_item(&parent, &path, item);
            }
        } else if let Value::Table(_) = dir.1 {
            cmd.set_info(dir.1);
        }
    }
    return Some(cmd);
}
fn find_charon_file(path: &PathBuf) -> Result<PathBuf, String>{
    //! Path is a directory which must contain a charon file.
    //! File should either be <util>.charon
    //! or <util>/<util>.toml
    let mut contents = match path.read_dir() {
        Ok(contents) => contents,
        Err(_) =>  return Err(format!("Could not read contents of path: {path:#?}"))
    };

    let res = contents.find(|file| {
        if let Ok(file) = file {
            file.path().extension() == Some(&OsString::from("charon"))
        } else {
            false
        }
    });
    if let Some(entry) = res {
        match entry {
            Ok(entry) => return Ok(entry.path()),
            Err(err) => return Err(format!("Error reading directory: {:?}", err))
        }
    }
    // No files matched pattern
    return Err(format!("Could not locate charon file. This should be a file with the form name.charon or name/name.toml."));
}

#[cfg(test)]
mod test {
    /*! # Test Plan
        * - Test mkdirs parsing.
        * - Test install commands.
        * - Test error in charon file.
        * - Test file not found.
        * - Create .desktop file.
        * - Test empty charon item (e.g. data = []).
     */
    use std::path::PathBuf;
    use mythos_core::dirs;
    use crate::*;

    //#[test]
    // If the directory already exists, charon will ignore it, making this test not work anymore.
    fn test_mkdirs() {
        let val = auto_install("data/test.toml".into()).unwrap();
        let dir1 = dirs::expand_mythos_shortcut("d", "charon").unwrap();
        let dir2 = dirs::expand_mythos_shortcut("b", "charon").unwrap();
        let path1 = PathBuf::from(dir1);
        let path2 = PathBuf::from(dir2);
        assert_eq!(val.mkdirs, vec![path2, path1]);
    }
    #[test]
    fn test_dest() {
        let val = auto_install("data/test.toml".into()).unwrap();
        let dir = dirs::expand_mythos_shortcut("d", "charon").unwrap();
        let mut path = PathBuf::from(dir);
        path.push("index.charon");
        assert_eq!(val.items[1].dest, path);
    }
    #[test]
    fn test_install() {
        let cmd = auto_install("data/test.toml".into()).unwrap();
        let dir = dirs::expand_mythos_shortcut("d", "charon").unwrap();
        let mut path = PathBuf::from(dir);
        path.push("index.charon");

        let val = installer::install(&cmd, true);
        assert!(val.contains(&path.to_string_lossy().to_string()));
    }

    //#[test]
    // If the directory already exists, charon will ignore it, making this test not work anymore.
    fn test_empty_charon_item() {
        let cmd = auto_install("data/empty_test.toml".into()).unwrap();
        assert!(cmd.mkdirs.len() > 0);
    }
    #[test]
    fn test_info() {
        let cmd = auto_install("data/test.toml".into()).unwrap();
        assert_eq!(cmd.version, Some("0.0.0".to_string()));
    }
}
