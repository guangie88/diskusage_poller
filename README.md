# diskusage_poller

Polls for storage usage (include `inode` usage) on file system via the Linux
system call [`statvfs`](http://man7.org/linux/man-pages/man3/statvfs.3.html) for
`Rust`, to `Fluentd` unified logging layer.

`dup` is the executable short form for Disk Usage Poller.

Requires Linux and supports compilation on stable `rustc`, and also works for
target `x86_64-unknown-linux-musl`.

## Installation of `Rust` environment

Follow the instructions from [`rustup`](https://www.rustup.rs/). In order to
build for target `x86_64-unknown-linux-musl`, the following command should be
run **after** the installation of the toolchain:

```bash
rustup target add x86_64-unknown-linux-musl
```

The benefit building for target `x86_64-unknown-linux-musl` is that the compiled
executable is fully statically linked, _i.e_. does not depend on any system
shared objects and is very portable across all Linux distributions.

## Recommended Build Command

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

## Run Example

Polls for:

* (Default `Fluentd` server at: `127.0.0.1:24224`)
* Interval: 5 seconds
* Path at: `/`
* `Fluentd` tag: `elastic.rs`

```bash
./target/x86_64-unknown-linux-musl/release/dup -i 5 -p / -t elastic.rs
```

Note the program loops forever until `CTRL-C` is pressed.

## JSON Log Example

```json
{
  "bsize": 4096,
  "frsize": 4096,
  "blocks": 59699623,
  "bfree": 36878532,
  "bavail": 33828523,
  "files": 15237120,
  "ffree": 13868856,
  "favail": 13868856,
  "fsid": 91332353,
  "flagstr": "RELATIME",
  "namemax": 255
}
```
