use std::{io::{stdout, Write, stdin}, process::{Command, Stdio}};
use crate::query_manager::{Package, PackageSelector};

#[derive(Debug, Clone)]
pub enum States { 
    Completed, 
    NeedsXbpsUpdate, 
    NeedsSysUpdate, 
    BadPkg(String),
    DoPkgValidation,
    DoInstall,
    DoSysUpdate,
    DoXbpsUpdate,
}

#[derive(Debug)]
pub struct InstallCommand {
    pub assume_yes: bool,
	pub do_system_update: bool,
	pub xbps_args: Vec<String>,
	pub pkgs: Vec<Package>,
	pub current_state: States,
    pub do_validate_pkgs: bool,
} 

impl InstallCommand {
    pub fn new(initial_state: States) -> InstallCommand {
        let do_system_update: bool = matches!(initial_state, States::DoSysUpdate) 
            || matches!(initial_state, States::DoXbpsUpdate);

        return InstallCommand { 
            assume_yes: false,
            do_system_update, 
            xbps_args: Vec::new(), 
            pkgs: Vec::new(), 
            current_state: initial_state,
            do_validate_pkgs: true
        };
    }

    pub fn xbps_args<I>(&mut self, args: I) where I: Iterator<Item = String> {
        self.xbps_args.extend(args);
    }

    pub fn try_run(&mut self) -> Result<String, String>{
        /*!
         * Attempts to run xbps-install to install pkgs and/or update system.
         * Cmd can run into 3 errors:
         * - System needs updating 
         * - xbps needs updating
         * - pkg does not exist
         *
         * If command fails, caller will have the option to cancel command or fix.
         */
        match self.current_state {
            States::DoXbpsUpdate => {
                match self.update_xbps() {
                    Ok(_) => {
                        self.current_state = States::DoSysUpdate;
                        return Ok("STYX (Success): Updated the 'xbps' package".into());
                    },
                    Err(_) => {
                        self.current_state = States::Completed;
                        return Err("STYX (Fatal Error): Could not update the 'xbps' package.".into());
                    },
                };
            },
            States::DoSysUpdate => {
                match self.update_sys() {
                    Ok(_) => {
                        self.current_state = if self.pkgs.len() > 0 {
                            States::DoInstall
                        } else { States::Completed };
                        return Ok("STYX (Success): Updated system.".into());
                    },
                    Err((needs_xbps_update, msg)) => {
                        if needs_xbps_update { 
                            self.current_state = States::NeedsXbpsUpdate;
                            return Err(format!("STYX (Error): Xbps needs to be updated."));
                        } else {
                            self.current_state = States::Completed;
                            return Err(format!("STYX (Fatal Error): Could not update system.\n {}", msg)); 
                        } 
                    }
                };
            },
            States::DoInstall => {
                if let Err(bad_pkg) = self.validate_pkgs() {
                    if bad_pkg.len() == 0 {
                        self.current_state = States::DoSysUpdate;
                        return Err(format!("STYX (Error): System needs to be updated."));
                    }
                    let msg = format!("STYX (Error): Could not find package '{}', must be replaced.", bad_pkg);
                    self.current_state = States::BadPkg(bad_pkg);
                    return Err(msg);
                }
                if let Err(msg) = self.install_pkgs() {
                    self.current_state = States::Completed;
                    return Err(format!("STYX (Fatal Error): Could not run install command.\n{}", msg));
                }
                self.current_state = States::Completed;
                return Ok("STYX (Success): Installation complete!".into());
            },
            _ => panic!("STYX (Fatal Error): try_run should only be called if the command state == DoXbpsUpdate || DoSysUpdate || DoInstall. State is = {:?}", self.current_state),
        };
    }

    pub fn apply_fix(&mut self) -> String {
        let mut msg: String = "".into();
        self.current_state = match self.current_state.clone() {
            States::NeedsSysUpdate =>  States::DoSysUpdate,
            States::NeedsXbpsUpdate => States::DoXbpsUpdate,
            States::BadPkg(pkg) => {
                let new_pkg = self.replace_pkg(&pkg);
                if new_pkg.len() == 0 {
                    msg = format!("STYX: Removed '{}'", pkg);
                }
                else {
                    msg = format!("STYX: Replaced '{}' with '{}'", pkg, new_pkg);
                    self.pkgs.push(new_pkg);
                }

                States::DoInstall
            },
            _ => panic!("STYX (Fatal Error): apply_fix should only be called if the command state == NeedsSysUpdate || NeedsXbpsUpdate || BadPkg. State is = {:?}", self.current_state),
        };

        return msg.into();
    }

    pub fn try_apply_fix(&mut self) -> Result<String, String> {
        /*!
         * Ask user for permission to apply fix 
        */
        loop {
            print!("Would you like to do this now? Y/n: ");
            if self.assume_yes {
                print!("y\n");
                return Ok(self.apply_fix());
            }

            let _ = stdout().flush();
            let mut input = String::new();
            stdin().read_line(&mut input).expect("STYX (Fatal Error): Could not get user input");
            input = input.trim().to_lowercase().into();

            if ["n", "no"].contains(&input.as_str()) {
                match &self.current_state {
                    States::NeedsSysUpdate | States::NeedsXbpsUpdate => {
                        self.current_state = States::Completed;
                        return Err("STYX: User cancelled command.".into());
                    },
                    States::BadPkg(pkg) => {
                        let msg = format!("STYX: Removed package: '{}'", pkg);
                        self.current_state = States::DoInstall;
                        return Ok(msg);
                    },

                    _ => panic!("STYX (Fatal Error): try_apply_fix should only be called if the command state == NeedsSysUpdate || NeedsXbpsUpdate || BadPkg. State is = {:?}", self.current_state),
                }
            }
            else if ["y", "yes", "", "\n"].contains(&input.as_str()) {
                return Ok(self.apply_fix());
            }
            else {
                eprintln!("Please select from the options above.");
            }
        }
    }

    pub fn is_completed(&self) -> bool {
        return matches!(self.current_state, States::Completed);
    }

    pub fn validate_pkgs(&mut self) -> Result<(), Package> {
        if !self.do_validate_pkgs {
            return Ok(());
        }

        let mut res: Result<(), Package> = Ok(());
        let mut i: i32 = -1;

        for pkg in &self.pkgs {
            i += 1;
            let mut cmd = Command::new("xbps-install");
            cmd.arg("-n")
                .arg(pkg)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let output = match cmd.output() {
                Ok(cmd) => cmd,
                Err(_) => {
                    res = Err(pkg.to_owned());
                    break;
                }
            }.stderr;
             
            // Convert error message to string
            let msg = output.iter().map(|x| (*x as char)).collect::<String>().trim().to_string();

            // No errors were given
            if msg.len() == 0 {
                continue; 
            }

            if msg.starts_with("Package '") && msg.ends_with("' not found in repository pool.") {
                res = Err(pkg.to_owned());
                break;
            }
            else if msg.contains("shlibs") {
                return Err("".into());
            }
        }

        if res.is_err() {
            self.pkgs.remove(i as usize);
        }
        else {
            self.do_validate_pkgs = false;
        }
        return res;
    }

    pub fn replace_pkg(&self, pkg: &Package) -> Package {
        let pkg = match PackageSelector::new(pkg.to_owned()).get_replacement_pkg() {
            Ok(pkg) => pkg,
            Err(msg) => { 
                eprintln!("STYX (Error): {}", msg);
                return "".into();
            }
        };

        return match pkg {
            Some(pkg) => pkg,
            None => "".into(),
        };
    }

    /* ACTIONS */
    fn install_pkgs(&mut self) -> Result<(), String>{
        let mut cmd = Command::new("xbps-install");
        cmd.stderr(Stdio::piped())
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit());

        for pkg in &self.pkgs {
            cmd.arg(pkg);
        }

        let res = match cmd.output() {
            Ok(res) => res,
            Err(msg) => return Err(msg.to_string()),
        };

        let msg = res.stderr.iter().map(|x| (*x as char)).collect::<String>().trim().to_string();
        if msg.len() == 0 {
            return Ok(());
        }
        return Err(msg.to_string());
    }
    fn update_xbps(&mut self) -> Result<(), ()> {
        let mut cmd = Command::new("xbps-install");
        cmd.args(["-Syu", "xbps"]);
        cmd.stderr(Stdio::piped())
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit());

        match cmd.output() {
            Ok(_) => return Ok(()),
            Err(_) => return Err(())
        };
    }
    fn update_sys(&mut self) -> Result<(), (bool, String)> {
        let mut cmd = Command::new("xbps-install");
        cmd.arg("-Syu");
        cmd.stderr(Stdio::piped())
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit());

        let res = match cmd.output() {
            Ok(output) => output,
            Err(msg) => return Err((false, msg.to_string()))
        };

        let msg = res.stderr.iter().map(|x| (*x as char)).collect::<String>().trim().to_string();
        if msg.len() == 0 {
            return Ok(());
        }
        else if msg.contains("The 'xbps' package must be updated") {
            return Err((true, "".into()));
        }
        return Err((false, msg));
    }
}
