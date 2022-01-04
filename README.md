# zeus-rs
[![Continuous Integration](https://github.com/AlecGoncharow/zeus-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/AlecGoncharow/project-name-tbd/actions/workflows/rust.yml)

WIP city builder game with custom engine using [wgpu-rs](https://github.com/gfx-rs/wgpu-rs) for rendering and async, multithreading, and networking using [tokio](https://github.com/tokio-rs/tokio)

# Building
Requirements (Mostly due to [shaderc-rs](https://github.com/google/shaderc-rs#building-from-source)): 
 * [cargo](https://www.rust-lang.org/tools/install)
 * [cmake](https://cmake.org/)
 * [git](https://git-scm.com/)
 * [python3 (must be called python.exe on Windows)](https://www.python.org/)
 * [ninja (on Windows)](https://github.com/ninja-build/ninja/releases)
 * Support for a graphics API supported by [wgpu](https://github.com/gfx-rs/wgpu#supported-platforms)

### Compilation Time Optimization Requirements: 
By default this repo builds with some flags that speed up compilation times on the listed platforms considerably but are completely optional in terms of generating a functional binary.  
You can modify `.cargo/config.toml` to relax any of these requirements.

#### Linux x86_64:
* Currently requires nightly toolchain.   
* [clang](https://releases.llvm.org/download.html)

#### MacOS x86_64:
* Currently requires nightly toolchain.   
* You must manually install https://github.com/michaeleisel/zld. you can easily do this with the "brew" package manager:  
  * `brew install michaeleisel/zld/zld`  

#### Windows x86_64:
* Currently requires nightly toolchain.   
* May not compile due to `wgpu` switch to `wgpu-hal` and this project's usage of push constants, unverified.

# Running
This repo contains 2 binaries currently, `game-client` and `server`, this is likely to change. The game will work without the server for now, 
the server only acts as a sort of entiity orchestrator at the moment.

### Game
`cargo run --bin game-client --release`

### Server
`cargo run --bin server --release`
