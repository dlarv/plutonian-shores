- [ ] Util behavior when no packages/args are given
- [x] Create script to automatically update version numbers.
- [x] Cocytus piping functionality
- [x] Help messages
- [x] Fuzzy find when query yields no results
- [ ] Charon syntax for changing owner of file to user
- [x] Charon syntax for creating dirs
E.g. whether to make "$MYTHOS_DIR/$UTIL_NAME/file" or "$MYTHOS_DIR/file"
- [ ] Charon syntax for optional install "?"
- [ ] Charon uninstall should read implied file names
If reading a line with format `target_dir dest_dir [opts]` (Charon install file) and `dest_dir` has no filename, Charon should try using the filename at end of `target_dir` before throwing an error.
- [x] Migrated to new logger system.

Charon (1.0.0)
- [x] Better charon console logs.
- [x] Charon can obtain version number.
    - [ ] and use it to determine whether updates are needed.
- [x] Create basic .desktop files.
- [ ] Test plan.
- [x] Charon should try to find charon file if one isn't provided.
- [ ] Charon cannot overwrite its binary when installing itself.
- [x] Better output messages when installing.
- [x] Charon overwriting index file instead of appending to it.

Cocytus (1.0.0)
- [ ] Test plan.
- [ ] Display info about packages.
- [x] Piping to styx/lethe should use sudo, even if cocytus didn't.

Lethe (1.0.0)
- [ ] Test plan.
- [ ] Test environment.

PT_Core (1.0.0)
- [ ] Test plan.
- [x] Allow/Ensure users can select multiple options per query.

Styx (1.0.0)
- [ ] Test plan.
- [ ] Test environment.
