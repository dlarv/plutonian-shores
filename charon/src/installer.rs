use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
use mythos_core::{dirs, printerror, printinfo, printwarn};
use crate::installation_cmd::InstallationCmd;

pub fn run_installation(installation_cmd: &InstallationCmd, do_dry_run: bool) {
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
pub fn install(installation_cmd: &InstallationCmd, do_dry_run: bool) -> Vec<String> {
    //! Run installation using an installation cmd.
    let mut new_charon_file = Vec::new();
    for item in &installation_cmd.items {
        printinfo!("Installing {:?} --> {:?}", item.target, item.dest);

        let mut comments = vec!["#".to_string()];

        // Add files to new_charon_file
        new_charon_file.push(item.print_dest());

        if item.comment.len() > 0 {
            comments.push(item.comment.to_string());
        }

        if item.dest.exists() && !item.overwrite {
            comments.push("File exists && !overwrite".into());
        }
        if !do_dry_run && item.overwrite {
            match fs::copy(&item.target, &item.dest) {
                Ok(_) => comments.push("Successfully installed".into()),
                Err(msg) => {
                    comments.push(format!("Could not copy file: {msg}"));
                }
            }
            item.dest.metadata().unwrap().permissions().set_mode(item.perms);
        }
        let comment = comments.join("; ");
        printinfo!("{comment}");
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
    printinfo!("\nUpdating index:");

    // Saves data as a entry in a toml file.
    // This file can be read directly into a pt_core::QueryResult
    // name, version, description, source

    if let Some(mut index) = read_charon_file("charon/index") {
        printinfo!("Original index:\n{}\n", index.join("\n"));

        if index.iter().filter(|x| x.contains(&cmd.name)).collect::<Vec<&String>>().len() == 0 {
            index.push(cmd.to_toml_str().to_owned());
            printinfo!("Updated index:\n{}\n", index.join("\n"));
            write_charon_file("index", index, do_dry_run);
        } else {
            printinfo!("Util already found in index, no changes were made");
        }

    } else {
        printinfo!("No index found, creating new");
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
    let dest = if do_dry_run {
        dirs::MythosDir::LocalData
    } else {
        dirs::MythosDir::Data
    };

    let mut path = match dirs::get_dir(dest, "charon") {
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
