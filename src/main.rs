#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", deny(warnings))]

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate failure;
extern crate fruently;
extern crate nix;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use failure::Error;
use fruently::fluent::Fluent;
use fruently::forwardable::JsonForwardable;
use nix::sys::statvfs::vfs::Statvfs;
use std::os::raw::c_ulong;
use std::path::Path;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, Fail)]
enum FluentError {
    #[fail(display = "")] InnerFluentError { e: fruently::error::FluentError },
}

impl From<fruently::error::FluentError> for FluentError {
    fn from(e: fruently::error::FluentError) -> FluentError {
        FluentError::InnerFluentError { e: e }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(StructOpt, Debug)]
#[structopt(name = "dup", about = "Disk Usage Poller")]
struct MainConfig {
    #[structopt(short = "a", long = "addr",
                default_value = "127.0.0.1:24224", help = "Fruentd hostname")]
    addr: String,

    #[structopt(short = "t", long = "tag",
                help = "Tag to use for Fruentd logging")]
    tag: String,

    #[structopt(short = "p", long = "path",
                help = "Path to check for disk usage")]
    path: String,

    #[structopt(parse(try_from_str), short = "i", long = "interval",
                help = "Interval in seconds")]
    interval: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct StatvfsDef {
    bsize: c_ulong,
    frsize: c_ulong,
    blocks: u64,
    bfree: u64,
    bavail: u64,
    files: u64,
    ffree: u64,
    favail: u64,
    fsid: c_ulong,
    flagstr: String,
    namemax: c_ulong,
}

impl StatvfsDef {
    fn from_statvfs(stat: &Statvfs) -> StatvfsDef {
        StatvfsDef {
            bsize: stat.f_bsize,
            frsize: stat.f_frsize,
            blocks: stat.f_blocks,
            bfree: stat.f_bfree,
            bavail: stat.f_bavail,
            files: stat.f_files,
            ffree: stat.f_ffree,
            favail: stat.f_favail,
            fsid: stat.f_fsid,
            flagstr: format!("{:?}", stat.f_flag),
            namemax: stat.f_namemax,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, new)]
struct StatvfsWrap {
    statvfs: StatvfsDef,
}

fn run_impl(addr: &str, tag: &str, path: &Path) -> Result<()> {
    let stat = Statvfs::for_path(path)?;
    let stat_wrap = StatvfsWrap::new(StatvfsDef::from_statvfs(&stat));

    Fluent::new(addr, tag)
        .post(&stat_wrap)
        .map_err(|e| -> FluentError { e.into() })?;

    Ok(())
}

fn run() -> Result<()> {
    let config = MainConfig::from_args();
    let path = Path::new(&config.path);
    let interval = Duration::from_secs(config.interval);

    loop {
        if let Err(e) = run_impl(&config.addr, &config.tag, path) {
            eprintln!("dup run ERROR: {}", e);
        }

        thread::sleep(interval);
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("dup main ERROR: {}", e);
    }
}
