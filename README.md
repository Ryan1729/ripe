# RIPE

RIPE stands for Randomizer-Inspired Pastime/Entertainment. This is a WIP game/thing inspired by the phenomenon of video game randomizers.

## WASM version

### Running locally

1. Install Rust via [rustup.rs](https://rustup.rs).

2. Install WebAssembly target:
```
rustup target add wasm32-unknown-unknown
```
3. Start dev server:
```
cargo run-wasm ripe --release
```
4. Visit `http://localhost:8000` with your browser.

### Extra build options

These extra features can be adding then to the run-wasm `features` flag. Note that these are comma separated. For instance to activate `invariant-checking` and `logging` you can run:
```
cargo run-wasm ripe --release --features invariant-checking,logging
```
## Desktop

The desktop version attempts to be cross platform. Only Linux and Windows have been tested at this time.

### Building/Running

1. Install Rust via [rustup.rs](https://rustup.rs).

2. Build via cargo
```
cargo build --release --bin ripe
```
3. Run the executable
```
./target/release/ripe
```

#### Linux specific notes

When building the Linux version, some additional packages may be needed to support building the [`alsa`](https://github.com/diwic/alsa-rs) library this program uses for sound, on Linux.
On Ubuntu, these packages can be installed as follows:

```
sudo apt install libasound2-dev pkg-config
```

If you don't care about sound you can build with the enabled-by-default `"non-web-sound"` feature flag turned off:

```
cargo build --release --bin ripe --no-default-features
```

##### Wayland
As of this writing, [a library that this program uses does not allow specifying that parts of the screen need to be redrawn, on Wayland](https://github.com/john01dav/softbuffer/issues/9).
For now, you can run the executable with the `WINIT_UNIX_BACKEND` environment variable set to `"x11"` as a workaround.

```
WINIT_UNIX_BACKEND="x11" ./target/release/ripe
```

## Feature flags

##### invariant-checking

With this enabled violations of certain invariants will result in a panic. These checks are disabled in default mode since (presumably) a player would prefer the game doing something weird to outright crashing.

##### logging

Enables additional generic logging. With this feature disabled, the logs will be compiled out, leaving no appreciable run-time overhead.

##### non-web-sound

Enables sound when not building for the web. On by default.

##### reload

Enables hot reloading of certain parts of the code, which is only relevant if you are making changes to the code. Off by default. Only useful on Desktop, and currently only tested on Linux.

###### Getting hot reloading working

(Assumes that the environment is set up for normal Desktop builds already.)

1.  In one terminal run
```
cargo run --release --bin ripe --features reload
```
to build and run the version of the main exe that will do the hot reloading.

2. In a second terminal run
```
cargo build --release --package app --features reload
```
after each change to the reloadable part of the code (things called from the `app` crate).

Optionally, an automated way of doing this, such as via cargo-watch can be used, like so:

```
cargo watch -w libs/app/ -x 'build --release --package app --features reload'
```

It is possible to run both commands in one terminal, and in parallel, at least on Linux, using GNU Parallel.

See https://github.com/rksm/hot-lib-reloader-rs for exampels of the GNU Parallel version, more details on limitations, etc.

___

licensed under Apache or MIT, at your option.
