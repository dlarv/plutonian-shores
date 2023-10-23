use crate::{query_manager::PackageSelector, commands::*};
use std::{process::{Command, Stdio}, io::Read};
use duct::cmd;

impl RemoveCommand {
    pub fn new() -> RemoveCommand {
        return RemoveCommand { 
            assume_yes: false, 
            xbps_args: Vec::new(), 
            pkgs: Vec::new(), 
            remove_all_orphans: false,
            do_validate_pkgs: true,
            bad_pkg: None,
        };
    }
    pub fn set_assume_yes(&mut self, val: bool) -> &mut Self{
        self.assume_yes = val;
        return self;
    }
    pub fn set_remove_all_orphans(&mut self, val: bool) -> &mut Self {
        self.remove_all_orphans = val;
        return self;
    }

    pub fn execute(&mut self) {
        if let Some(pkg) = self.bad_pkg.take() {
            match self.fix_bad_pkg(&pkg) {
                Ok(msg) => println!("LETHE: {}", msg),
                Err(msg) => eprintln!("LETHE (Error): {}", msg)
            }
        }
        if let Err(msg) = self.validate_pkgs() {
            eprintln!("LETHE (Error): {}", msg);
            return;
        }
        self.do_removal();
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
            Some(new_pkg) => {
                let msg = format!("Replaced '{}' with '{}'", pkg, new_pkg);
                self.pkgs.push(new_pkg);
                Ok(msg)
            },
            None => Ok(format!("Removed '{}'", pkg)),
        };
    }

    fn do_removal(&mut self) {
        /*
         * 
         */

        todo!()
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
