pub mod query_manager;
pub mod commands;
use crate::commands::{RemoveCommand, MythosCommand};
use commands::remove_command;
use mythos_core::conf;

fn main() {
    unsafe { 
        if let Some(conf) = conf::MythosConfig::read_file("plutonian-shores") {
            load_config_values(conf);
        }
    }
    let mut cmd = match parse_args() {
        Some(cmd) => cmd,
        None => return
    };
    loop {
        match cmd.execute() {
            Ok(_) => break,
            Err(_) => println!()
        }
    }
}
unsafe fn load_config_values(conf: conf::MythosConfig) {
    if let Some(conf) = conf.get_subsection("cocytus") { 
        if let Some(val) = conf.try_get_float("fuzzy_find_threshold") {
            query_manager::query_results::THRESHOLD = val as f32;
        }

        if let Some(val) = conf.try_get_integer("list_column_length") {
            query_manager::query_results::LIST_COLUMN_LEN = val as usize;
        }
    }

    if let Some(conf) = conf.get_subsection("lethe") {
        if let Some(val) = conf.try_get_boolean("remove_orphans") {
            remove_command::REMOVE_ORPHANS = val;
        }
        if let Some(val) = conf.try_get_boolean("do_recursive_removal") {
            remove_command::DO_RECURSIVE = val;
        }
    }
}

fn parse_args() -> Option<RemoveCommand> {
    let args = mythos_core::cli::clean_cli_args();
    let mut cmd = RemoveCommand::new();
    let mut reading_xbps_args = false;

    for arg in args {
        if !reading_xbps_args {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("TODO: help msg");
                    return None;
                },
                "-R" | "--recursive" => {
                    cmd.set_do_recursive(true);
                },
                "-o" | "--remove-orphans" => {
                    cmd.set_remove_orphans(true);
                },
                "-y" | "--assume-yes" => {
                    cmd.set_assume_yes(true);
                },
                "-x" | "--xbps-args" => reading_xbps_args = true,
                _ => { cmd.add_pkg(arg); () },
            };
        }
        else if arg.starts_with("-"){
            cmd.add_xbps_arg(arg);
        }
        else {
            cmd.add_pkg(arg);
        }
    }

    return Some(cmd);
}

#[cfg(test)]
mod tests {
    use crate::commands::remove_command;

    use super::*;

    #[test]
    pub fn parse_config() {
        std::env::set_var("MYTHOS_LOCAL_CONFIG_DIR", "tests/config");
        let conf = conf::MythosConfig::read_file("plutonian-shores").unwrap();

        unsafe {
            let prev_col_length = query_manager::query_results::LIST_COLUMN_LEN;
            load_config_values(conf);
            assert_eq!(query_manager::query_results::THRESHOLD, 1.0);
            assert_eq!(query_manager::query_results::LIST_COLUMN_LEN, prev_col_length);
            assert_eq!(remove_command::REMOVE_ORPHANS, true);
            assert_eq!(remove_command::DO_RECURSIVE, false);
        }

    }
}
