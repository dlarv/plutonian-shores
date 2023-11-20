use std::{io::{BufReader, BufRead}, process::{Command, Stdio}};
use duct::{cmd, Expression};
use mythos_core::{printfatal, logger::get_logger_id, printerror, fatalmsg, printinfo};

use crate::query_manager::{PackageSelector, PackageSelection};
use crate::commands::*;

pub static mut DO_SYNC_REPOS: bool = false;

impl InstallCommand {
    pub fn new(initial_state: StyxState) -> InstallCommand {
        unsafe {
            return InstallCommand {
                do_dry_run: false,
                assume_yes: false,
                do_sync_repos: DO_SYNC_REPOS,
                xbps_args: Vec::new(),
                pkgs: Vec::new(),
                current_state: initial_state,
                do_validate_pkgs: true,
            };
        }
    }
    pub fn set_assume_yes(&mut self, val: bool) {
        self.assume_yes = val;
    }
    pub fn set_initial_state(&mut self, state: StyxState) {
        self.current_state = state;
    }
    pub fn is_completed(&self) -> bool {
        return matches!(self.current_state, StyxState::Failed) 
            || matches!(self.current_state, StyxState::Completed);
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
    fn build_sys_update_cmd(&self) -> Expression {
        return cmd!("xbps-install", "-Syu");
    }
    fn build_xbps_update_cmd(&self) -> Expression {
        return cmd!("xbps-install", "-Syu", "xbps");
    }

    pub fn execute(&mut self) {
        if self.do_validate_pkgs && self.pkgs.len() > 0 {
            self.validate_pkgs();
        }

        while !self.is_completed() {
            match &self.current_state.to_owned() {
                StyxState::DoXbpsUpdate => self.try_execute_xbps_update(),
                StyxState::DoSysUpdate => self.try_execute_sys_update(),
                StyxState::DoInstall => self.try_execute_install(),
                _ => return
            };
        }
        printinfo!("Installation Complete!");
    }
    fn validate_pkgs(&mut self) {
        let mut require_update = false;
        let mut bad_pkg_index: Vec<usize> = Vec::new();

        for (i, pkg) in self.pkgs.iter().enumerate() {

            let res = cmd!("xbps-install", "-n", pkg)
                .stderr_to_stdout()
                .stdout_capture()
                .unchecked()
                .read().expect(&fatalmsg!("Could not execute install command."));

            // System needs to be updated
            if res.contains("broken, unresolvable shlib") {
                self.current_state = StyxState::DoSysUpdate;
                require_update = true;
                continue;
            }
            // Pkg must be replaced
            else if res.starts_with("Package '") && res.ends_with("' not found in repository pool.") {
                bad_pkg_index.push(i);
            }
        }
        // Fix & remove bad pkgs
        bad_pkg_index.reverse();
        for index in bad_pkg_index {
            let pkg = self.pkgs[index].to_owned();
            match self.fix_bad_pkg(&pkg) {
                Ok(msg) => printinfo!("{msg}"), 
                Err(msg) => printerror!("{msg}"),
            }
            self.pkgs.remove(index);
        }
        if require_update {
            println!("System must be updated.");
        }
    }

    // Replace or remove pkg 
    fn fix_bad_pkg(&mut self, pkg: &str) -> Result<String, String>{
        self.current_state = StyxState::DoInstall;
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
            }
            _ => Ok(format!("Removed '{}'", pkg)),
        };
    }

    // Leads to DoXbpsUpdate, Failed
    fn try_execute_xbps_update(&mut self) {
        self.current_state = StyxState::DoSysUpdate;
        // Update gives no output for dry runs
        if self.do_dry_run {
            printinfo!("Updated xbps.");
            return;
        }
        get_user_permission(self.assume_yes, "Updating xbps");
        let mut cmd = Command::new("xbps-install");
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .args(["-Syu", "xbps"]);

        self.build_xbps_update_cmd().run().expect(&fatalmsg!("xbps could not be updated."));
        printinfo!("xbps has been updated");
    }
    fn try_execute_sys_update(&mut self) {
        // Update gives no output for dry runs
        if self.do_dry_run {
            printinfo!("Updated system.");
            self.current_state = StyxState::DoInstall;
            return;
        }

        get_user_permission(self.assume_yes, "Updating system");
        let cmd = self.build_sys_update_cmd()
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
                self.current_state = StyxState::DoInstall;
                printinfo!("System has been updated");
            }
            else if msg.contains("The 'xbps' package must be updated") {
                self.current_state = StyxState::DoXbpsUpdate;
                printinfo!("xbps must be updated");
            } 
            // Print to stdout
            else {
                print!("{msg}");
                msg = String::new();
                continue;
            }
            return;
        }
    }

    // Install pkgs 
    // NOTE: assumes 'broken shlib' error is caught in validate_pkgs method. I'm not 100% sure if
    // this is the case, it might need to be caught in this method.
    fn try_execute_install(&mut self) {
        if self.pkgs.len() == 0 {
            self.current_state = StyxState::Completed;
            return;
        }

        let fmt_pkgs: String = self.pkgs.iter().map(|x| format!("{}\n", x)).collect();
        get_user_permission(self.assume_yes, &format!("The following packages will be installed:\n{}", fmt_pkgs));

        let cmd = &mut self.build_install_cmd().unchecked();
        cmd.run().expect(&fatalmsg!("Could not run install command."));
        self.current_state = StyxState::Completed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_validate_pkgs() {
        let mut cmd = InstallCommand::new(StyxState::DoInstall);
        cmd.add_pkgs(["firefox", "bledner", "feh"]);
        cmd.validate_pkgs();

        println!("{:?}", cmd.pkgs);
    }
}
