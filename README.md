# S3 Wayfinder – a friendly Rust TUI/CLI for S3-compatible object storage

_Through the wilderness of object storage, S3 Wayfinder will be your guide._

> [!WARNING]
> This project is still in early development. Features may be incomplete or unstable. Installation methods and documentation are also a work in progress.

Using CLI tools for S3-compatible storage can be very hard to use, especially for newcomers. S3 Wayfinder aims to provide a friendly and easy-to-use TUI (Text User Interface) and CLI (Command Line Interface) for managing these storage services. It aims to be compatible with services like Amazon S3, Backblaze B2 and others.

**Source Repositories:**

- Main: [Codeberg](https://codeberg.org/raphael-denni/s3-wayfinder)
- Mirror: [GitHub](https://github.com/raphael-denni/s3-wayfinder)

## ✨ Features

- Easy-to-use TUI for managing S3-compatible storage
- CLI for quick operations and scripting
- Support for multiple S3-compatible services
- Configuration management for different profiles
- Basic operations: list, upload, download, delete objects
- Future plans for advanced features like transfer acceleration and parallelism

## 🗺️ Roadmap

### Phase 1 & 2 : CLI + TUI (v0.1.0)

- [x] Implement basic configuration system
- [x] Add core command: `ls`
- [x] Add core command: `cp`
- [x] Add core command: `rm`
- [x] Panes for buckets, objects, etc.
- [x] Status bar
- [x] Conditional launch
- [x] Connect S3 logics

### Phase 3: Advanced features (v0.2.0)

- [x] Configure Forgejo actions in Codeberg for package distribution
- [x] Add 'mk' command for bucket and folder creation
- [x] Add 'mv' command for moving/renaming objects/buckets
- [ ] Integrate 'mk' and 'mv' into the TUI

### Phase 4: Enhancements and polish (v0.3.0)

- [ ] Multi-profiles configuration with default
- [ ] Nesting of buckets and folders
- [ ] Complete the wiki for documentation

### Sides

- The wiki will be completed along the project
- The quest for a proper logo
- World domination plans (Just kidding... or am I?)

## 📦 Installation

### From precompiled binaries

You can install S3 Wayfinder by downloading the precompiled binaries from the [Releases](https://codeberg.org/raphael-denni/s3-wayfinder/releases) page on Codeberg. Choose the appropriate binary for your operating system and architecture. (No binaries for macOS, I don't have a Mac to build them, sorry!)

### From source

To build S3 Wayfinder from source, you need to have Rust and Cargo installed on your system. You can then clone the repository and build the project using the following commands:

```bash
git clone https://codeberg.org/raphael-denni/s3-wayfinder.git
cd s3-wayfinder
cargo build --release
```

This will create an executable in the `target/release` directory.

## 🚀 Usage

After installation, you can run S3 Wayfinder from the command line. For example, to start the TUI, simply run:

```bash
s3wf
```

To use the CLI, you can run commands like:

```bash
s3wf ls
s3wf cp source destination
s3wf rm object
```

## 📚 Documentation

You can find all information related to this project and how to use it on the [Codeberg wiki](https://codeberg.org/raphael-denni/s3-wayfinder/wiki).

## 📜 License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## 🤖 genAI declaration

genAI tools were used to:

- Draft the documentation and some comments (currently, most of the documentation is human-written, even the emojis yes)
- Review the code and suggest improvements (without code edits or snippets proposing)

Other than that, the code was written by me, Raphaël Denni, with my brain and documentation reading. I take full responsibility for the code and its quality.

## 🙌 Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
