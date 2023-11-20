use crate::{query_manager::{PackageSelector, PackageSelection}, commands::*};
use std::{process::{Command, Stdio}, io::Read};
use duct::cmd;

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
                Ok(msg) => println!("LETHE: {}", msg),
                Err(msg) => eprintln!("LETHE (Error): {}", msg)
            }
        }
        if let Err(msg) = self.validate_pkgs() {
            eprintln!("LETHE (Error): {}", msg);
            return Err(());
        }
        self.execute_removal();
        println!("Completed successfully!");
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
                .reader().expect("Could not run remove commmand");

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
        let new_pkg = match PackageSelector::new(pkg.to_owned()).get_replacement_pkg() {
            Ok(pkg) => pkg,
            Err(msg) => { 
                return Err(format!("{}\n'{}' was removed", msg, pkg));
            }
        };

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
            panic!("LETHE (Fatal Error): {msg}");
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
