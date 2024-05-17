pub mod help;
pub mod query_manager;

use duct::{Expression, cmd};
use mythos_core::{printfatal, logger::get_logger_id};
use query_manager::QueryDisplayMode;

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
        if ["y", "yes", "\n", ""].contains(&input.as_str()) {
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

pub trait MythosCommand {
    fn pkgs<'a> (&'a mut self) -> &'a mut Vec<Package>;
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String>;
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
    fn list_pkgs(&mut self) -> String {
        return self.pkgs().iter().map(|x| format!("{}\n", x)).collect();
    }
}
