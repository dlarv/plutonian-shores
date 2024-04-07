use std::io::{BufReader, BufRead};
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

            run_xbps_update: false,
            run_sys_update: false,
            run_pkg_install: false,
        };
    }

    pub fn execute(&mut self) {
        if self.use_alias_mode {
            self.execute_alias_mode();
            return;
        }

        while self.is_executing() {
            if self.run_xbps_update {
                self.try_execute_xbps_update();
            }
            else if self.run_sys_update {
                self.try_execute_sys_update();
            }
            else if self.validate_pkgs() {
                self.try_execute_install();
            }
        }

        printinfo!("Installation Complete!");
    }

    /* Helper methods */
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
            self.run_xbps_update = false;
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
            self.run_sys_update = false;
            self.run_xbps_update = false;
            return true;
        }

        get_user_permission(self.assume_yes, "Updating system");
        // User consents in previous step.
        self.assume_yes = true;

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

    fn try_execute_install(&mut self) {
        if self.pkgs.len() == 0 {
            self.run_pkg_install = false;
            return;
        }
        get_user_permission(
            self.assume_yes, 
            &format!("The following packages will be installed:\n{pkgs}", pkgs=self.list_pkgs())
        );

        self.build_cmd().unchecked().run().expect(&fatalmsg!("Could not run install command."));
        self.run_pkg_install = false;
    }
    // returns true if system doesn't need updating.
    fn validate_pkgs(&mut self) -> bool {
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
                require_update = true;
                self.run_sys_update = true;
                continue;
            }
            // XBPS must be updated
            if res.contains("The 'xbps' package must be updated") {
                require_update = true;
                self.run_sys_update = true;
                self.run_xbps_update = true;
                break;
            }
            // Pkg must be replaced
            if res.starts_with("Package '") && res.ends_with("' not found in repository pool.") {
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
            return false;
        }
        return true;
    }

    fn is_executing(&self) -> bool {
        return self.run_xbps_update
            || self.run_sys_update
            || self.run_pkg_install;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_validate_pkgs() {
        let mut cmd = InstallCommand::new();
        cmd.do_dry_run = true;
        cmd.assume_yes = true;
        cmd.run_sys_update = false;
        cmd.run_xbps_update = false;
        cmd.add_pkgs(["firefox", "bledner", "feh"]);
        let _ = cmd.build_cmd().run();

        // println!("{:?}", cmd.pkgs);
    }
}
