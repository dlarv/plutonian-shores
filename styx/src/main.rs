use std::io::{BufRead, BufReader};

use duct::cmd;
use mythos_core::{cli::{clean_cli_args, get_user_permission}, logger, printerror};
use pt_core::{validate_pkgs, Query};
enum StartState {
    Install,
    SysUpdate,
    XbpsUpdate,
}
fn main() {
    let _ = logger::set_id("STYX");
    // let args = std::env::args().skip(1);
    let args = clean_cli_args();
    let mut pkgs: Vec<String> = Vec::new();
    let mut starting_state = StartState::Install;
    let mut do_dry_run = false;

    for arg in args {
        if arg == "-h" || arg == "--help" {
            println!("Wrapper util for xbps-install");
            println!("styx [opts] packages");
            println!("opts:");
            println!("-h | --help\t\tPrint this menu.\n-u | --update\t\tRun a system update. Equiv to xbps-install -Syu.\n-x | --xbps-update\t\tUpdate xbps. Contains an implicit '-u'.\n-n | --dryrun\t\tRun command w/o making changes to system.");
            return;
        } 
        if arg == "-u" || arg == "--update" {
            starting_state = StartState::SysUpdate;
        } else if arg == "-x" || arg == "--xbps-update" {
            starting_state = StartState::XbpsUpdate;
        } else if arg == "-n" || arg == "--dryrun" { 
            do_dry_run = true;
        } else if arg.starts_with("-") {
            printerror!("Unknown arg: {arg}");
        } else {
            pkgs.push(arg);
        }
    }

    let _ = match starting_state {
        StartState::Install => install_pkgs(pkgs, do_dry_run, false),
        StartState::SysUpdate => sys_update(false),
        StartState::XbpsUpdate => {
            match xbps_update() {
                Ok(_) => sys_update(false),
                Err(err) => Err(err),
            }
        },
    };
}

fn install_pkgs(pkgs: Vec<String>, do_dry_run: bool, assume_yes: bool) -> Result<(), std::io::Error> { 
    //! Validate and install packages.
    let query = Query::from(match validate_pkgs(pkgs.into_iter()) {
        Some(pkgs) => pkgs,
        None => {
            println!("All packages removed. Exiting...");
            return Ok(());
        },
    });

    // Build args for command.
    let pkg_names = query.get_pkg_names();
    let mut args = vec!["-Sy"];
    if do_dry_run {
        args.push("-n".into());
    }
    args.extend(pkg_names);

    // Build command.
    let cmd = cmd("xbps-install", args)
        .stderr_to_stdout()
        .unchecked();

    // If an update is required, run the install command again.
    loop {
        let mut lines = BufReader::new(cmd.reader()?).lines();
        // Loop through lines in bufreader.
        loop {
            let line = match lines.next() {
                Some(line) => line?,
                None => return Ok(())
            };

            println!("{line}");

            if line.contains("shlibs") 
                && get_user_permission(assume_yes, "System needs to be updated.") {
                    sys_update(assume_yes)?;
                    break;
            } 
            // This is here just in case.
            if line.contains("The 'xbps' package must be updated") 
                && get_user_permission(assume_yes, "xbps package needs to be updated."){
                    xbps_update()?;
                    break;
            }
        }
        
    }
}
fn sys_update(assume_yes: bool) -> Result<(), std::io::Error>{
    let cmd = cmd("xbps-install", vec!["-Syu"])
        .stderr_to_stdout()
        .unchecked();
    loop {
        // If xbps update is needed, rerun this command.
        let mut lines = BufReader::new(cmd.reader()?).lines();
        // Iterate over lines in output.
        loop {
            let line = match lines.next() {
                Some(line) => line?,
                None => return Ok(())
            };
            println!("{line}");
            if line.contains("The 'xbps' package must be updated") 
                && get_user_permission(assume_yes, "xbps package needs to be updated."){
                    xbps_update()?;
                    break;
            }
        }
    }
}
fn xbps_update()-> Result<(), std::io::Error> {
    cmd("xbps-install", vec!["-Syu", "xbps"]).unchecked().run()?;
    return Ok(());
}

#[cfg(test)]
mod test {
    /*!
        * # Test Plan
        * - Try install pkg, but system is out of date.
        * - Try install pkg, but both system and xbps are ood.
        * - Try install bad pkg.
        * - Try update, but system is not ood.
        * - Try update, but xbps is also ood.
        * - Try update xbps.
     */

}
