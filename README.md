# Plutonian Shores
Wrapper for xbps

## Styx
Wraps xpbs-install.

styx [opts] [pkgs]
- If a pkg is invalid, run query using fuzzy-find. Then allow the user to either select from the results or remove pkg from command.
- If install command throws 'unresolved shlib' error, run system update.
- If the xbps pkg must be updated, do so and then update system.

## Lethe
Wraps xbps-remove.

lethe [opts] [pkgs]
- If a pkg is invalid, run query using fuzzy-find. Then allow the user to either select from the results or remove pkg from command.

## Cocytus
Wraps xbps-query.

cocytus [opts] [pkgs]
- Run query using fuzzy-find.
- Allow user to select pkg to remove or install.

