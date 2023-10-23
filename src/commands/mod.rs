pub mod install_command;
pub mod remove_command;
use crate::query_manager::Package;
use std::io::{stdout, Write, stdin};

/* OBJECTS */
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

#[derive(Debug)]
pub struct RemoveCommand {
    assume_yes: bool,
	xbps_args: Vec<String>,
	pkgs: Vec<Package>,
    remove_all_orphans: bool,
    do_validate_pkgs: bool,
    bad_pkg: Option<Package>,
}

/* FUNCTIONS */
pub fn get_user_permission(assume_yes: bool, msg: &str) -> Result<(), String> {
    println!("{}", msg);
    loop {
        print!("Would you like to proceed? Y/n: ");
        if assume_yes {
            println!("Y");
            return Ok(());
        }

        let _ = stdout().flush();
        let mut input = String::new();
        if let Err(_) = stdin().read_line(&mut input) {
            return Err("Could not get user input".into());
        }
        input = input.trim().to_lowercase().into();

        if ["n", "no"].contains(&input.as_str()) {
            return Err("User cancelled command".into());
        }
        else if ["y", "yes", "\n", ""].contains(&input.as_str()) {
            return Ok(());
        }
        eprintln!("Invalid input");
    }
}
pub fn parse_output(output: Vec<u8>) -> String {
    return output.iter().map(|x| (*x as char)).collect::<String>().trim().to_string();
}

/* IMPLEMENTATION */
pub trait MythosCommand {
    fn pkgs<'a> (&'a mut self) -> &'a mut Vec<Package>;
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String>;

    fn add_pkg<T: Into<String>>(&mut self, pkg: T) -> &mut Self {
        self.pkgs().push(pkg.into());
        return self;
    }
    fn add_pkgs<'a, T>(&mut self, pkgs: T) -> &mut Self where T: IntoIterator<Item = &'a str> {
        self.pkgs().extend(pkgs.into_iter().map(|x| x.to_string()));
        return self;
    }
    fn add_xbps_args<'a, T>(&mut self, args: T) -> &mut Self where T: IntoIterator<Item = &'a str> {
        self.xbps_args().extend(args.into_iter().map(|x| x.to_string()));
        return self;
    }
    fn add_xbps_arg(&mut self, arg: String) -> &mut Self {
        self.xbps_args().push(arg);
        return self;
    }
}

impl MythosCommand for RemoveCommand {
    fn pkgs<'a> (&'a mut self) -> &'a mut Vec<Package> { return &mut self.pkgs; }
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String> { return &mut self.xbps_args; }
}

impl MythosCommand for InstallCommand {
    fn pkgs<'a>(&'a mut self) -> &'a mut Vec<Package> { return &mut self.pkgs; }
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String> { return &mut self.xbps_args; }
}
