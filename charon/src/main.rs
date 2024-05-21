/*!
 * Manual installation wizard.
 * Runs an automatic install, if given a .charon/.toml file.
 * Otherwise, runs a guided install. 
*/

mod installation_cmd;

use std::{fs::{self, Permissions}, os::unix::fs::PermissionsExt, path::PathBuf};

use installation_cmd::InstallationCmd;
use mythos_core::{cli::clean_cli_args, dirs, logger::{get_logger_id, set_logger_id}, printinfo, printwarn};
use toml::Value;

fn main() {
    set_logger_id("CHARON");

    // If no args are provided, do guided install.
    let args = clean_cli_args();
    let do_dry_run = args.contains(&"-n".into()) || args.contains(&"--dryrun".into());
    let path = args.iter().find(|x| !x.starts_with("-"));

    let installation_cmd = if path.is_none() {
        guided_install().unwrap()
    } else {
        auto_install(PathBuf::from(path.unwrap())).unwrap()
    };

    // List of all files installed for this util.
    let new_charon_file = install(&installation_cmd, do_dry_run);
    modify_index(&installation_cmd, do_dry_run);

    // Read old charon file into memory.
    let old_charon_file = read_charon_file(&installation_cmd.name);

    // If file is in old && not in new --> remove
    remove_orphans(old_charon_file, &new_charon_file, do_dry_run);
    // Overwrite util.charon file.
    write_charon_file(&installation_cmd.name, new_charon_file, do_dry_run);
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
                let path = match cmd.add_dir(&dir.0) {
                    Some(path) => path,
                    None => {
                        printwarn!("Not a valid path shortcut: {}", dir.0);
                        continue;
                    }
                };
                cmd.add_item(&path, item);
            }
        }
    }
    return Some(cmd);
}
fn install(installation_cmd: &InstallationCmd, do_dry_run: bool) -> Vec<String> {
    //! Run installation using an installation cmd.
    let mut new_charon_file = Vec::new();
    for item in &installation_cmd.items {
        let mut comments = vec!["#"];
        // Add files to new_charon_file
        new_charon_file.push(item.print_dest());

        if item.comment.len() > 0 {
            comments.push(&item.comment);
        }

        if item.dest.exists() && !item.overwrite {
            comments.push("File exists && !overwrite");
        }
        if !do_dry_run && item.overwrite {
            match fs::copy(&item.target, &item.dest) {
                Ok(_) => comments.push("Successfully installed"),
                Err(_) => comments.push("Error, could not copy file"),
            }
            item.dest.metadata().unwrap().permissions().set_mode(item.perms);
        }
        let comment = comments.join("; ");
        printinfo!("{}", comment);
        new_charon_file.push(comment);
    }
    return new_charon_file;
}
fn remove_orphans(old_charon_file: Option<Vec<String>>, new_charon_file: &Vec<String>, do_dry_run: bool) {
    //! Remove deprecated files installed in previous versions.
    if let Some(mut old_files) = old_charon_file {
        old_files.retain(|x| !new_charon_file.contains(x));
        for file in old_files {
            // Skip comments
            if file.starts_with("#") {
                continue;
            }
            let path = PathBuf::from(file);
            if !do_dry_run && path.exists() {
                match fs::remove_file(&path) {
                    Ok(_) => printinfo!("Removed: {path:?}"),
                    Err(msg) => printwarn!("{msg:?}")
                }
            }
        }
    }
}
fn modify_index(cmd: &InstallationCmd, do_dry_run: bool) {
    //! Modify index.charon file
    if do_dry_run {
        return;
    }

    // Saves data as a entry in a toml file.
    // This file can be read directly into a pt_core::QueryResult
    // name, version, description, source

    if let Some(mut index) = read_charon_file("index") {
        if !index.contains(&cmd.name) {
            index.push(cmd.to_toml_str().to_owned());
            write_charon_file("index", index, do_dry_run);
        }

    } else {
        write_charon_file("index", vec![cmd.to_toml_str().to_owned()], do_dry_run);
    }
}
fn read_charon_file(util_name: &str) -> Option<Vec<String>> {
    //! Read file inside $MYTHOS_DATA_DIR/$util_name.charon
    let mut path = match dirs::get_dir(dirs::MythosDir::Data, util_name) {
        Some(path) => path,
        None => return None
    };
    path.push(util_name.to_owned() + ".charon");

    let contents: Vec<String> = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return None
    }.trim()
        .split("\n")
        .filter(|x| x.len() > 0)
        .map(|x| x.to_string())
        .collect();

    return Some(contents);
}
fn write_charon_file(util_name: &str, data: Vec<String>, do_dry_run: bool) {
    //! Write file to $MYTHOS_DATA_DIR/$util_name.charon
    let mut path = match dirs::get_dir(dirs::MythosDir::Data, "charon") {
        Some(path) => path,
        None => return 
    };
    let extension = if do_dry_run {
        ".dryrun"
    } else {
        ""
    }.to_owned() + ".charon";

    path.push(util_name.to_owned() + &extension);
    let contents = data.join("\n");
    fs::write(path, contents).unwrap();
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use mythos_core::dirs;
    use crate::*;

    #[test]
    fn test_mkdirs() {
        let val = auto_install("data/charon.toml".into()).unwrap();
        let dir = dirs::expand_mythos_shortcut("d", "charon").unwrap();
        let path = PathBuf::from(dir);
        assert_eq!(val.mkdirs, vec![path]);
    }
    #[test]
    fn test_dest() {
        let val = auto_install("data/charon.toml".into()).unwrap();
        let dir = dirs::expand_mythos_shortcut("d", "charon").unwrap();
        let mut path = PathBuf::from(dir);
        path.push("test.txt");
        assert_eq!(val.items[0].dest, path);
    }
    #[test]
    fn test_install() {
        let cmd = auto_install("data/charon.toml".into()).unwrap();
        let dir = dirs::expand_mythos_shortcut("d", "charon").unwrap();
        let mut path = PathBuf::from(dir);
        path.push("test.txt");

        let val = install(&cmd, true);

        assert!(val.contains(&path.to_string_lossy().to_string()));
    }
}
