use std::{io::Read, process::{Command, Stdio}};

use duct::{cmd, Expression};
use mythos_core::{cli::get_cli_input, fatalmsg, logger::get_logger_id, printerror, printfatal, printinfo, printwarn};
use pt_core::{get_user_permission, query_manager::{Package, PackageSelection, PackageSelector}, xbps_args_to_string, MythosCommand};


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
impl MythosCommand for RemoveCommand {
    fn pkgs<'a> (&'a mut self) -> &'a mut Vec<Package> { return &mut self.pkgs; }
    fn xbps_args<'a> (&'a mut self) -> &'a mut Vec<String> { return &mut self.xbps_args; }
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
pub static mut DO_RECURSIVE: bool = false;
pub static mut REMOVE_ORPHANS: bool = true;

impl RemoveCommand {
    pub fn new() -> RemoveCommand {
        unsafe {
            return RemoveCommand { 
                do_dry_run: false,
                assume_yes: false, 
                xbps_args: Vec::new(), 
                pkgs: Vec::new(), 
                do_validate_pkgs: true,
                remove_orphans: REMOVE_ORPHANS,
                do_recursive: DO_RECURSIVE,
                bad_pkg: None,
            };
        }
    }
    pub fn set_assume_yes(&mut self, val: bool) -> &mut Self{
        self.assume_yes = val;
        return self;
    }
    pub fn set_remove_orphans(&mut self, val: bool) -> &mut Self {
        self.remove_orphans = val;
        return self;
    }
    pub fn set_do_recursive(&mut self, val: bool) -> &mut Self {
        self.do_recursive = val;
        return self;
    }

    pub fn execute(&mut self) -> Result<(), ()> {
        if let Some(pkg) = self.bad_pkg.take() {
            match self.fix_bad_pkg(&pkg) {
                Ok(msg) => printinfo!("{msg}"),
                Err(msg) =>printerror!("{msg}")
            }
        }
        if let Err(msg) = self.validate_pkgs() {
            printwarn!("{msg}");
            return Err(());
        }
        self.execute_removal();
        printinfo!("Completed successfully!");
        return Ok(());
    }

    fn validate_pkgs(&mut self) -> Result<(), String> {
        if !self.do_validate_pkgs  {
            return Ok(());
        }
        let mut index: isize = -1;
        for (i, pkg) in self.pkgs.iter().enumerate() {
            let mut msg = String::new();
            let mut cmd = cmd!("xbps-remove", "-n", pkg)
                .stderr_to_stdout()
                .stdout_capture()
                .reader().expect(&fatalmsg!("Could not run remove commmand"));

            let _ = cmd.read_to_string(&mut msg);
            if msg.trim().ends_with("not currently installed."){
                index = i as isize;
                break;
            }
        };
    
        if index != -1 {
            let pkg = self.pkgs.remove(index as usize);
            self.bad_pkg = Some(pkg.to_owned());
            return Err(format!("Package {}' not installed.", pkg));
        }
        self.do_validate_pkgs = false;
        return Ok(());
    }
    fn fix_bad_pkg(&mut self, pkg: &String) -> Result<String, String>{
        let new_pkg = PackageSelector::new(pkg.to_owned()).select_replacement_pkgs();

        return match new_pkg {
            PackageSelection::Package(new_pkg) => {
                let msg = format!("Replaced '{}' with '{}'", pkg, new_pkg);
                self.pkgs.push(new_pkg);
                Ok(msg)
            },
            PackageSelection::Packages(new_pkgs) => {
                let msg = format!("Replaced '{}' with the following: '{:?}'", pkg, *new_pkgs);
                self.pkgs.extend(*new_pkgs);
                Ok(msg)
            },
            _ => Ok(format!("Removed '{}'", pkg)),
        };
    }
    fn execute_removal(&mut self) {
        get_user_permission(self.assume_yes, &format!("The following packages will be removed:\n{pkgs}", pkgs=self.list_pkgs()));
        let mut cmd = Command::new("xbps-remove");
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        if self.do_dry_run {
            cmd.arg("-n");
        }
        if self.do_recursive {
            cmd.arg("-R");
        }
        if self.remove_orphans {
            cmd.arg("-o");
        }
        cmd.arg(xbps_args_to_string(&self.xbps_args));
        cmd.args(self.pkgs());

        if let Err(msg) = cmd.output() {
            printfatal!("{msg}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_validation() {
        let mut cmd = RemoveCommand::new();
        cmd.add_pkgs(["blen", "godot"])
            .set_assume_yes(true);

        let _ = cmd.validate_pkgs();
        assert_eq!(cmd.bad_pkg, Some("blen".into()));
    }
}

