# Sherpa S3 – a friendly Rust TUI/CLI for S3-compatible object storage

*Through the wilderness of object storage, Sherpa S3 will be your guide.*

> [!WARNING]
> This project is still in early development. Features may be incomplete or unstable. Installation methods and documentation are also a work in progress.

Using CLI tools for S3-compatible storage can be very hard to use, especially for newcomers. Sherpa S3 aims to provide a friendly and easy-to-use TUI (Text User Interface) and CLI (Command Line Interface) for managing these storage services. It aims to be compatible with services like Amazon S3, Backblaze B2 and others. 

**Source Repositories:**
- Main: [Codeberg](https://codeberg.org/raphael-denni/sherpa-s3)
- Mirror: [GitHub](https://github.com/raphael-denni/sherpa-s3)

## ✨ Features

- Easy-to-use TUI for managing S3-compatible storage
- CLI for quick operations and scripting
- Support for multiple S3-compatible services
- Configuration management for different profiles
- Basic operations: list, upload, download, delete objects
- Future plans for advanced features like transfer acceleration and parallelism

## 🗺️ Roadmap

### Phase 1: CLI
- [X] Implement basic configuration system
- [X] Add core command: `ls`
- [X] Add core command: `cp`
- [X] Add core command: `rm`

### Phase 2: TUI
- [X] Panes for buckets, objects, etc.
- [X] Status bar
- [X] Conditional launch
- [ ] Connect S3 logics

### Phase 3: Advanced features
- [ ] Configure Forgejo actions in Codeberg for package distribution
- [ ] Complete the wiki for documentation
- [ ] Multi-profiles configuration
- [ ] Transfer and parallelism
- [ ] Expand CLI
- [ ] Enhance TUI

### Sides
- The wiki will be completed along the project
- The quest for a proper logo
- World domination plans (Just kidding... or am I?)

## 📦 Installation

### From precompiled binaries
You can install Sherpa S3 by downloading the precompiled binaries from the [Releases](https://codeberg.org/raphael-denni/sherpa-s3/releases) page on Codeberg. Choose the appropriate binary for your operating system and architecture.

### From source
To build Sherpa S3 from source, you need to have Rust and Cargo installed on your system. You can then clone the repository and build the project using the following commands:

```bash
git clone https://codeberg.org/raphael-denni/sherpa-s3.git
cd sherpa-s3
cargo build --release
```

This will create an executable in the `target/release` directory.

## 🚀 Usage

After installation, you can run Sherpa S3 from the command line. For example, to start the TUI, simply run:

```bash
sherpa
```

To use the CLI, you can run commands like:

```bash
sherpa ls
sherpa cp source destination
sherpa rm object
```

## 📚 Documentation

You can find all information related to this project and how to use it on the [Codeberg wiki](https://codeberg.org/raphael-denni/sherpa-s3/wiki).

## 📜 License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## 🙌 Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
