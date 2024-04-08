- [ ] xbps-query -S option for cocytus.
- [ ] pass to xbps arg -x not working.
- [ ] Styx remove installation complete message when program is cancelled etc.
- [ ] Make acheron backup utility.

v1.0.0
- [ ] Styx will state that a system update is needed, but will not try to run it.
- [ ] If Cocytus is ran w/o sudo and then piped to Styx/Lethe, prompt user for password.
- [ ] Lethe receiving extra args?

v2.0.0
Agnostic package manager.

Query/Cocytus
- Detect and query distro-specific package manager.
- Optionally search other package managers (cargo, npm).
    - User has list of managers to automatically check.
- Format and display results in either list or tui form.

Install/Styx

Uninstall/Lethe

Guided Install/Charon?
- Move charon from mythos-core to plutonian-shores?
- Install packages from source.
- Create /bin/{pkg} and .desktop entries.
- Detect when manually install pkgs need updating.

Backend details
- Match proper commands and args to packages?
    - Allow user level bindings
- Detect when system needs updating without necessarily reading output logs.
