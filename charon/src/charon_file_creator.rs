use std::{env::current_dir, path::{Path, PathBuf}};

use mythos_core::{cli::{get_cli_input, get_user_permission}, dirs::{get_dir, MythosDir}, logger::get_logger_id, printerror};
use pt_core::get_user_selection;

use crate::installation_cmd::{InstallItem, InstallationCmd};

pub fn create_charon_file() -> Option<(String, String)> {
    //! Creates a basic charon file.
    //! Returns Some(file_contents, util_name) or None if user cancels.
    let util_name = get_util_name();

    let input = get_user_selection(
        &format!( "Creating a Charon File for {}:\n0. Exit\n1. Basic Skeleton\n2. Guided creation\nEnter Option: ", util_name), 
        2);
    match input {
        0 => return None,
        1 => return Some((skeleton().to_string(), util_name)),
        2 => (),
        _ => panic!("Input should have been validated previously")
    }
    // TODO: Create file
    let mut file_contents = String::new();
    let mut items: Vec<InstallItem> = Vec::new();
    
    if let Some(path) = get_file("binary") {
        file_contents += "bin = [\n\t";
        file_contents += &InstallItem::new().target(path).strip_ext(true).perms(0x755).to_toml_str();
        file_contents += "\n]\n";
    }
    if let Some(path) = get_file("config") {
        file_contents += "config = [\n\t";
        file_contents += &InstallItem::new().target(path.clone()).perms(0x644).to_toml_str();
        file_contents += "\n]\n";

        file_contents += "localconfig = [\n\t";
        file_contents += &InstallItem::new().target(path).perms(0x644).overwrite(false).to_toml_str();
        file_contents += "\n]\n";
    }
    if let Some(path) = get_file("alias") {
        file_contents += "alias = [\n\t";
        file_contents += &InstallItem::new().target(path).perms(0x755).to_toml_str();
        file_contents += "\n]\n";
    }
    if let Some(path) = get_file("lib") {
        file_contents += "lib = [\n\t";
        file_contents += &InstallItem::new().target(path).perms(0x755).to_toml_str();
        file_contents += "\n]\n";
    }

    return Some((file_contents, util_name));
}

fn get_util_name() -> String {
    let binding = current_dir().unwrap();
    let default_util_name = binding.file_name().unwrap_or_default().to_string_lossy();
    let msg = format!("Enter util name (Press enter to use {default_util_name:?}): "); 
    loop {
        let util_name = get_cli_input(&msg);
        // Input is trimmed inside of get_cli_input method.
        if util_name == "" {
            return default_util_name.to_string();
        } 
        if get_user_permission(false, &format!("Use {util_name}?")) {
            return util_name;
        }
    }
}

fn get_file(msg: &str) -> Option<PathBuf> {
    loop {
        let input = get_cli_input(&format!("Enter path to {msg} file (or enter to skip): "));
        if input == "" {
            return None;
        }

        let path = match PathBuf::from(input).canonicalize() {
            Ok(path) => path,
            Err(msg) => {
                printerror!("Could not get path. Reason: {msg}");
                continue;
            }
        };

        if path.is_file() {
            return Some(path);
        }
        println!("Please enter a valid path to a file");
    }
}
const fn skeleton() -> &'static str {
    return "# Example: config = [{ target = \"path/to/local\", alias = \"alt_file_name\", perms = 0x544, strip_ext = false, overwrite = false, comment = \"\" }]\nalias = []\nbin = []\nconfig = []\ndata = []\nlocalconfig = []\nlocaldata = []\nlib = []" ;
}

