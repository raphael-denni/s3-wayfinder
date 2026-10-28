## [0.2.0] - 2026-10-27

### 🚀 Features

- *(error)* Anyhow for better errors handling
- Add mk & mv commands to CLI
- *(tui)* Add mk and mv into tui

### ⚡ Performance

- Use BufWriter in cp.rs for file upload

### 🚜 Refactor

- Remove redundant allocations across src
- *(tracing)* Harmonize the use of tracing macros and std print
- Use built-in methods in ls.rs and cp.rs
- Code cleanup in tui.rs
- *(cli)* Harmonize usage of s3 paths + remove unnecessary deps

### ⚙️ Miscellaneous Tasks

- Add Forgejo Actions for development & release
- Bump deps
- Bump version to v0.2.0 & update docs

## [0.1.0] - 2026-05-25

### 🚀 Features

- Basic configuration system + init CLI
- Implement ls command for buckets and objects
- Implement comprehensive cp command
- Implement rm command with confirmation
- Implement interactive TUI
- *(tui)* Better error display w/ invalid url handling
- Cp & rm core commands in TUI

### 📚 Documentation

- Proper README.md and and deps in Cargo.toml
- Comment docs added + structure fix
- Improve private functions comments
- Rename from sherpa-s3 to s3-wayfinder

### 🚜 Refactor

- Move config logic & handling in module
- Move commands logic and ls in module
- Core logic as callable functions
- *(tui)* Functions renaming & ariane string

### ⚙️ Miscellaneous Tasks

- Init commit
- Release preparations (clippy fixes and copyright update)

