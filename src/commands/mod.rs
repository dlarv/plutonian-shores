pub mod install_command;
use crate::query_manager::Package;

#[derive(Debug, Clone)]
pub enum StyxState { 
    Completed, 
    Failed,
    BadPkg(String),
    DoInstall,
    DoSysUpdate,
    DoXbpsUpdate,
}

#[derive(Debug)]
pub struct InstallCommand {
    assume_yes: bool,
    do_sync_repos: bool,
	xbps_args: Vec<String>,
	pkgs: Vec<Package>,
	current_state: StyxState,
    do_validate_pkgs: bool,
} 
