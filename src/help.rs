use mythos_core::logger::get_logger_id;

pub fn print_help() {
    match get_logger_id().to_lowercase().as_str() {
        "styx" => println!("{STYX_HELP}"),
        "cocytus" => println!("{COCYTUS_HELP}"),
        "lethe" => println!("{LETHE_HELP}"),
        _ => panic!("Unknown mythos-util: {}", get_logger_id())
    }
}

const STYX_HELP: &str = "styx [opts] [pkgs]\n\
                             Wrapper for xbps-install.\n\
                             Styx will apply a common fix if it encounters any of the following errors:\n\
                             1. \"Invalid pkg\": Query repos using fuzzy finder. The user can then select from the results or remove the bad pkg.\n\
                             2. \"Broken shlib\": Run system update (xbps-install -Syu).\n\
                             3. \"Must update xbps package: Update xbps (xbps-install -Syu xbps) and then update the system.\n

                             opts:\n\
                                 -h | --help			Print this menu.\n\
                                 -a | --alias           Treat this cmd as an alias.\n\
                                 -w | --wrapper         Opposite of -a.\n\
                                 -n | --dry-run         Run command without making changes to system\n\
                                 -u | --update		    Run system update.\n\
                                 -X | --update-all	    Run xbps and system update.\n\
                                 -y | --assume-yes	    Don't ask user for confirmation.\n\
                                 -x | --xbps-args	    Pass all following opts directly to xbps-install.";

const LETHE_HELP: &str = "\n\
                              lethe [opts] [pkgs]\n\
                              Wrapper for xbps-remove\n\
                              Lethe will query repos if given a bad pkg.
                              
                              opts:\n\
                                 -h | --help			Print this menu.\n\
                                 -a | --alias           Treat this cmd as an alias.\n\
                                 -w | --wrapper         Opposite of -a.\n\
                                 -n | --dry-run         Run command without making changes to system\n\
                                 -R | --recursive       Recursively remove dependencies.\n\
                                 -o | --remove-orphans  Also remove orphaned pkgs.\n\
                                 -y | --assume-yes	    Don't ask user for confirmation.\n\
                                 -x | --xbps-args	    Pass all following opts directly to xbps-install.";
const COCYTUS_HELP: &str = "\n\
                            cocytus [opts] [pkgs]\n\
                            Wrapper for xbps-query\n\
                            Cocytus queries remove repos. User can select from results tp install/remove pkgs.

                            opts:\n\
                                 -h | --help			Print this menu.\n\
                                 -a | --alias           Treat this cmd as an alias.\n\
                                 -w | --wrapper         Opposite of -a.\n\
                                 -n | --dry-run         Run command without making changes to system\n\
                                 -t | --tui             Display results in TUI mode\n\
                                 -l | --list            Display results in list mode\n\
                                 -x | --xbps-args	    Pass all following opts directly to xbps-install.";
