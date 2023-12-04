pub mod install_command;
pub mod remove_command;
pub mod query_command;
use duct::{Expression, cmd};
use mythos_core::{printfatal, logger::get_logger_id};

use crate::query_manager::Package;
use std::io::{stdout, Write, stdin};

/* FUNCTIONS */
pub fn get_user_permission(assume_yes: bool, msg: &str) {
    println!("{}", msg);
    loop {
        print!("Would you like to proceed? Y/n: ");
        if assume_yes {
            println!("Y");
            return;
        }

        let _ = stdout().flush();
        let mut input = String::new();
        if let Err(msg) = stdin().read_line(&mut input) {
            printfatal!("{msg}");
        }
        input = input.trim().to_lowercase().into();

        if ["n", "no"].contains(&input.as_str()) {
            printfatal!("User cancelled command");
        }
        else if ["y", "yes", "\n", ""].contains(&input.as_str()) {
            return;
        }
        eprintln!("Invalid input.");
    }
}
pub fn parse_output(output: Vec<u8>) -> String {
    return output.iter().map(|x| (*x as char)).collect::<String>().trim().to_string();
}
pub fn xbps_args_to_string(xbps_args: &Vec<String>) -> String {
    if xbps_args.len() == 0 {
        return "".into();
    }
    return xbps_args.iter().fold("-".to_string(), |acc, x| {
        if x.starts_with("--") {
            printfatal!("Styx can only take the short version of xbps-install args");
        }
        acc + x.trim_start_matches("-")
    });
}

/* STRUCTS & ENUMS */
#[derive(Debug, Clone)]
pub enum StyxState { 
    Completed, 
    Failed,
    AliasMode,
    BadPkg(String),
    DoInstall,
    DoSysUpdate,
    DoXbpsUpdate,
}
#[derive(Debug)]
pub struct InstallCommand {
    assume_yes: bool,
    do_dry_run: bool,
    do_sync_repos: bool,
	xbps_args: Vec<String>,
	pkgs: Vec<Package>,
	current_state: StyxState,
    do_validate_pkgs: bool,
} 
#[derive(Debug)]
pub struct RemoveCommand {
    do_dry_run: bool,
    assume_yes: bool,
	xbps_args: Vec<String>,
	pkgs: Vec<Package>,
    remove_orphans: bool,
    do_recursive: bool,
    do_validate_pkgs: bool,
    bad_pkg: Option<Package>,
}
#[derive(Debug)]
pub enum QueryDisplayMode {
    AliasMode,
    List,
    Tui,
    Smart,
}
#[derive(Debug)]
pub struct QueryCommand {
    pkgs: Vec<Package>,
    xbps_args: Vec<String>,
    display_mode: QueryDisplayMode,
}
/* IMPLEMENTATION */
pub trait MythosCommand {
    fn pkgs<'a> (&'a mut self) -> &'a mut Vec<Package>;
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String>;
    fn set_do_dry_run<'a>(&'a mut self, dry_run: bool);
    fn build_cmd(&self) -> Expression;

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
    fn list_pkgs(&self) -> String {
        return self.pkgs().iter().map(|x| format!("{}\n", x)).collect();
    }
}

impl MythosCommand for RemoveCommand {
    fn pkgs<'a> (&'a mut self) -> &'a mut Vec<Package> { return &mut self.pkgs; }
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String> { return &mut self.xbps_args; }
    fn set_do_dry_run<'a>(&'a mut self, dry_run: bool) { self.do_dry_run = dry_run; }
    fn build_cmd(&self) -> Expression {
        let mut args: Vec<String> = Vec::new();
        if self.do_dry_run {
            args.push("-n".into());
        }
        if self.do_recursive {
            args.push("-R".into());
        }
        if self.remove_orphans {
            args.push("-o".into());
        }
        args.extend(self.xbps_args.to_owned());
        args.extend(self.pkgs.to_owned());
        return cmd("xbps-remove", args);
    }
}
impl MythosCommand for InstallCommand {
    fn pkgs<'a>(&'a mut self) -> &'a mut Vec<Package> { return &mut self.pkgs; }
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String> { return &mut self.xbps_args; }
    fn set_do_dry_run<'a>(&'a mut self, dry_run: bool) { self.do_dry_run = dry_run; }
    fn build_cmd(&self) -> Expression {
        let mut args: Vec<String> = Vec::new();
        if self.do_sync_repos {
            args.push("-S".into());
        }
        if self.assume_yes {
            args.push("-y".into());
        }
        if self.do_dry_run {
            args.push("-n".into());
        }
        args.extend(self.xbps_args.to_owned());
        args.extend(self.pkgs.to_owned());
        return cmd("xbps-install", args);
    }
}
impl MythosCommand for QueryCommand {
    fn pkgs<'a>(&'a mut self) -> &'a mut Vec<Package> { return &mut self.pkgs; }
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String> { return &mut self.xbps_args; }
    fn set_do_dry_run<'a>(&'a mut self, dry_run: bool) { }

    fn build_cmd(&self) -> Expression {
        let mut args = Vec::new();
        if !matches!(self.display_mode, QueryDisplayMode::AliasMode) {
            args.push("-Rs".into());
        }

        args.extend(self.xbps_args.to_owned());
        args.extend(self.pkgs.to_owned());
        return cmd("xbps-query", args);
    }
}
