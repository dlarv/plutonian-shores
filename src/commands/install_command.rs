use std::{io::{BufReader, BufRead}, process::{Command, Stdio}};
use duct::cmd;

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

    /*
     * Announce/print next step --> get_user_permission
     * Get user permission to proceed --> get_user_permission
     * try execute next step --> try_execute_*
     * print results --> execute
     * update state --> try_execute_*
     */
    pub fn execute(&mut self) {
        let res = match &self.current_state.clone() {
            StyxState::DoXbpsUpdate => self.try_execute_xbps_update(),
            StyxState::DoSysUpdate => self.try_execute_sys_update(),
            StyxState::BadPkg(pkg) => self.fix_bad_pkg(&pkg),
            StyxState::DoInstall => self.try_execute_install(),
            _ => return
        };

        match res {
            Ok(msg) => println!("STYX: {}", msg),
            Err(msg) => {
                if matches!(self.current_state, StyxState::Failed) {
                    eprintln!("STYX (FATAL ERROR): {}", msg);
                    return;
                }
                eprintln!("STYX (ERROR): {}", msg);
            }
        };
    }

    // Leads to DoInstall, BadPkg, Failed
    fn validate_pkgs(&mut self) -> Result<(), String> {
        // Method only needs to be ran once
        if !self.do_validate_pkgs {
            return Ok(());
        }

        // self.pkgs must be mutated outside of loop
        let mut counter: isize = -1;
        for pkg in self.pkgs.iter() {
            counter += 1;

            let mut cmd = Command::new("xbps-install");
            cmd.args(["-n", &pkg])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let msg = match cmd.output() {
                Ok(res) => parse_output(res.stderr),
                Err(msg) =>  {
                    if msg.to_string().contains("broken, unresolvable shlib") {
                        self.current_state = StyxState::DoSysUpdate;
                        return Err("System must be updated".into());
                    }
                    self.current_state = StyxState::Failed;
                    return Err(msg.to_string());
                }
            };

            // PKG did not fail
            if msg.len() == 0 {
                continue;
            }

            // Failed bc pkg dne
            if msg.starts_with("Package '") && msg.ends_with("' not found in repository pool.") {
                self.current_state = StyxState::BadPkg(pkg.to_owned());
                break;
            }
            if msg.contains("broken, unresolvable shlib") {
                self.current_state = StyxState::DoSysUpdate;
                return Err(format!("System must be updated"));
            }

            // Failed for other reasons
            self.current_state = StyxState::Failed;
            return Err(msg);
        }

        if matches!(self.current_state, StyxState::BadPkg(_)) {
            let pkg = self.pkgs.remove(counter as usize);
            return Err(format!("Package not found: '{}'", pkg));
        }
        self.do_validate_pkgs = false;
        return Ok(());
    }

    // Replace or remove pkg 
    // If user has removed all pkgs, abort command
    fn fix_bad_pkg(&mut self, pkg: &String) -> Result<String, String>{
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
    fn try_execute_xbps_update(&mut self) -> Result<String, String> {
        get_user_permission(self.assume_yes, "Updating xbps")?;
        let mut cmd = Command::new("xbps-install");
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .args(["-Syu", "xbps"]);

        if self.do_dry_run {
            cmd.arg("-n");
        }

        return match cmd.output() {
            Ok(_) => { 
                self.current_state = StyxState::DoSysUpdate;
                Ok("xbps has been updated".into()) 
            },
            Err(msg) => { 
                self.current_state = StyxState::Failed;
                Err(format!("xbps could not be updated\nError Message:\n{}", msg))
            }
        };
    }
    // Leads to DoSysUpdate, Failed
    fn try_execute_sys_update(&mut self) -> Result<String, String> {
        get_user_permission(self.assume_yes, "Updating system")?;
        let opts = if self.do_dry_run {
            "-Syun"
        } else {
            "-Syu"
        };
        let cmd = cmd!("xbps-install", opts)
            .stderr_to_stdout()
            .stdout_capture()
            .reader()
            .expect("STYX (Fatal Error): Could not run install command");
        let mut reader = BufReader::new(cmd);
        let mut msg = String::new();

        loop {
            if reader.read_line(&mut msg).is_err() {
                self.current_state = StyxState::Failed;
                return Err(format!("System could not be updated\nError Message:\n{}", msg));
            }
            if msg.len() == 0 {
                self.current_state = StyxState::DoInstall;
                return Ok("System has been updated".into());
            }
            if msg.contains("The 'xbps' package must be updated") {
                self.current_state = StyxState::DoXbpsUpdate;
                return Err(format!("xbps must be updated"));
            }
            print!("{msg}");
            msg = String::new();
        }
    }

    // Leads to BadPkg, DoSysUpdate, Completed
    fn try_execute_install(&mut self) -> Result<String, String> {
        if self.pkgs.len() == 0 {
            self.current_state = StyxState::Completed;
            return Ok("Completed".into());
        }

        self.validate_pkgs()?;
        let fmt_pkgs: String = self.pkgs.iter().map(|x| format!("{}\n", x)).collect();
        get_user_permission(self.assume_yes, &format!("The following packages will be installed:\n{}", fmt_pkgs))?;

        let mut opt = if self.do_sync_repos { "-Sy" } else { "-y" };
        
        for pkg in &self.pkgs {
            let cmd = match cmd!("xbps-install", opt, xbps_args_to_string(&self.xbps_args), pkg).stderr_to_stdout().stdout_capture().reader() {
                    Ok(cmd) => cmd,
                    Err(msg) => {
                        if msg.to_string().contains("broken, unresolvable shlib") {
                            self.current_state = StyxState::DoSysUpdate;
                            return Err(format!("System must be updated"));
                        }
                        self.current_state = StyxState::Failed;
                        return Err("STYX (Fatal Error): Could not run install command".into());
                    }
            };

            let mut reader = BufReader::new(cmd);
            let mut msg = String::new();

            loop {
                if reader.read_line(&mut msg).is_err() {
                    self.current_state = StyxState::Failed;
                    return Err(format!("Could not install packages\nError Message:\n{}", msg));
                }
                if msg.len() == 0 {
                    break;
                }
                else if msg.contains("broken, unresolvable shlib") {
                    self.current_state = StyxState::DoSysUpdate;
                    return Err("System must be updated".into());
                }
                print!("{msg}");
                msg = String::new();
            }
        }
        self.current_state = StyxState::Completed;
        return Ok("Installation Complete!".into());
    }
}

fn parse_output(output: Vec<u8>) -> String {
    return output.iter().map(|x| (*x as char)).collect::<String>().trim().to_string();
}
