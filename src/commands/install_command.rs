use std::{io::{BufReader, BufRead}, process::{Command, Stdio}};
use duct::{cmd, Expression};
use mythos_core::{printfatal, logger::get_logger_id, printerror, fatalmsg, printinfo};

use crate::query_manager::{PackageSelector, PackageSelection};
use crate::commands::*;

impl InstallCommand {
    pub fn new() -> InstallCommand {
        return InstallCommand {
            do_dry_run: false,
            assume_yes: false,
            do_sync_repos: true,
            use_alias_mode: false,
            xbps_args: Vec::new(),
            pkgs: Vec::new(),

            run_xbps_update: true,
            run_sys_update: true,
            run_pkg_install: true,
        };
    }

    pub fn execute(&mut self) {
        if self.use_alias_mode {
            self.execute_alias_mode();
            return;
        }

        while self.is_executing() {
            let cmd = self.build_cmd();

            if self.run_xbps_update {
                self.try_execute_xbps_update();
            }
            else if self.run_sys_update {
                self.try_execute_sys_update();
            }
            else {
                self.try_execute_install();
            }
        }

        printinfo!("Installation Complete!");
    }

    /* Helper methods */
    fn validate(&mut self, cmd: Expression) -> bool {
        let res = cmd
            .stderr_to_stdout()
            .stdout_capture()
            .unchecked()
            .read().expect(&fatalmsg!("Could not execute install command."));

        if res.contains("The 'xbps' package must be updated")  {
            self.run_sys_update = true;
            self.run_xbps_update = true;
            return false;
        }
        if res.contains("broken, unresolvable shlib") {
            self.run_sys_update = true;
            return false;
        }
        if res.starts_with("Package '") && res.ends_with("' not found in repository pool.") {
            return false;
        }
        return true;
    }

    fn execute_alias_mode(&mut self) {
        self.build_cmd().unchecked().run().unwrap();
    }

    // Replace or remove pkg 
    fn fix_bad_pkg(&mut self, pkg: &str) -> Result<String, String>{
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
            }
            _ => Ok(format!("Removed '{}'", pkg)),
        };
    }

    // If this function fails, entire operation fails too.
    fn try_execute_xbps_update(&mut self) {
        // Update gives no output for dry runs
        if self.do_dry_run {
            printinfo!("Updated xbps.");
            return;
        }
        get_user_permission(self.assume_yes, "Updating xbps");
        self.build_cmd().run().expect(&fatalmsg!("xbps could not be updated."));
        self.run_xbps_update = false;
        printinfo!("xbps has been updated");
    }
    fn try_execute_sys_update(&mut self) -> bool{
        // Update gives no output for dry runs
        if self.do_dry_run {
            printinfo!("Updated system.");
            return true;
        }

        get_user_permission(self.assume_yes, "Updating system");
        let cmd = self.build_cmd()
            .stderr_to_stdout()
            .stdout_capture()
            .reader()
            .expect(&fatalmsg!("Could not run install command"));

        // Intercept and Read stdout line by line 
        let mut reader = BufReader::new(cmd);
        let mut msg = String::new();
        loop {
            if reader.read_line(&mut msg).is_err() {
                printfatal!("System could not be updated");
            }
            // Command has finished
            if msg.len() == 0 {
                printinfo!("System has been updated");
                break;
            }

            if msg.contains("The 'xbps' package must be updated") {
                printinfo!("xbps must be updated");
                self.run_xbps_update = true;
                self.run_sys_update = true;
                return false;
            } 
            // Print to stdout
            print!("{msg}");
            msg = String::new();
        }
        self.run_sys_update = false;
        self.run_xbps_update = false;
        return true;
    }

    // Install pkgs 
    // NOTE: assumes 'broken shlib' error is caught in validate_pkgs method. I'm not 100% sure if
    // this is the case, it might need to be caught in this method.
    fn try_execute_install(&mut self) {
        if self.pkgs.len() == 0 {
            return;
        }

        get_user_permission(self.assume_yes, &format!("The following packages will be installed:\n{pkgs}", pkgs=self.list_pkgs()));

        let cmd = &mut self.build_install_cmd().unchecked();
        cmd.run().expect(&fatalmsg!("Could not run install command."));
    }

    fn is_executing(&self) -> bool {
        return self.run_xbps_update
            && self.run_sys_update
            && self.run_pkg_install;
    }
    fn build_install_cmd(&self) -> Expression {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_validate_pkgs() {
        let mut cmd = InstallCommand::new();
        cmd.do_dry_run = true;
        cmd.add_pkgs(["firefox", "bledner", "feh"]);

        println!("{:?}", cmd.pkgs);
    }
}
