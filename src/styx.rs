pub mod query_manager;
pub mod commands;
use crate::commands::*;
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

    while !cmd.is_completed() { 
        cmd.execute();
        println!();
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
    if let Some(conf) = conf.get_subsection("styx") {
        if let Some(val) = conf.try_get_boolean("do_sync") {
            install_command::DO_SYNC_REPOS = val;
        }
    }
}

fn parse_args() -> Option<InstallCommand> {
    let args = mythos_core::cli::clean_cli_args();
    let mut cmd = InstallCommand::new(StyxState::DoInstall);
    let mut reading_xbps_args = false;

    for arg in args {
        if !reading_xbps_args && arg.starts_with("-") {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("TODO: help msg");
                    return None;
                },
                "-U" | "--update" => {
                    cmd.set_initial_state(StyxState::DoSysUpdate);
                },
                "-X" | "--update-all" => {
                    cmd.set_initial_state(StyxState::DoXbpsUpdate);
                },
                "-y" | "--assume-yes" => {
                    cmd.set_assume_yes(true);
                },
                "-x" | "--xbps-args" => reading_xbps_args = true,
                _ => { cmd.add_xbps_arg(arg); () },
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
    use super::*;

    #[test]
    pub fn parse_config() {
        std::env::set_var("MYTHOS_LOCAL_CONFIG_DIR", "tests/config");
        let conf = conf::MythosConfig::read_file("plutonian-shores").unwrap();

        unsafe {
            let prev_col_length = query_manager::query_results::LIST_COLUMN_LEN;
            let threshold_value = query_manager::query_results::THRESHOLD;
            load_config_values(conf);
            assert_eq!(query_manager::query_results::THRESHOLD, 1.0);
            assert_eq!(query_manager::query_results::LIST_COLUMN_LEN, prev_col_length);
            query_manager::query_results::THRESHOLD = threshold_value;
        }

    }
}
