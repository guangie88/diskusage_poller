#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", deny(warnings))]

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate failure;
extern crate fruently;
extern crate humantime;
extern crate nix;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use failure::Error;
use fruently::fluent::Fluent;
use fruently::forwardable::JsonForwardable;
use humantime::Duration;
use nix::sys::statvfs::vfs::Statvfs;
use std::os::raw::c_ulong;
use std::path::Path;
use std::thread;
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
                default_value = "127.0.0.1:24224", help = "Fluentd hostname")]
    addr: String,

    #[structopt(long = "off", help = "Turn off Fluentd logging")]
    fluent_off: bool,

    #[structopt(short = "t", long = "tag",
                help = "Tag to use for Fruentd logging")]
    tag: String,

    #[structopt(short = "p", long = "path",
                help = "Path to check for disk usage")]
    path: String,

    #[structopt(parse(try_from_str), short = "i", long = "interval",
                help = "Interval to get disk usage")]
    interval: Duration,
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
    used_perc: f32,
    free_perc: f32,
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
            used_perc: (1.0 - (stat.f_bfree as f32 / stat.f_blocks as f32))
                * 100.0,
            free_perc: (stat.f_bfree as f32 / stat.f_blocks as f32) * 100.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, new)]
struct StatvfsWrap {
    statvfs: StatvfsDef,
}

fn run_impl(
    addr: &str,
    tag: &str,
    path: &Path,
    fluent_off: bool,
) -> Result<()> {
    let stat = Statvfs::for_path(path)?;
    let stat_wrap = StatvfsWrap::new(StatvfsDef::from_statvfs(&stat));

    if !fluent_off {
        Fluent::new(addr, tag)
            .post(&stat_wrap)
            .map_err(|e| -> FluentError { e.into() })?;
    }

    if cfg!(debug_assertions) {
        println!("{}", serde_json::to_string_pretty(&stat_wrap)?);
    }

    Ok(())
}

fn run() -> Result<()> {
    let config = MainConfig::from_args();
    let path = Path::new(&config.path);

    loop {
        if let Err(e) =
            run_impl(&config.addr, &config.tag, path, config.fluent_off)
        {
            eprintln!("dup run ERROR: {}", e);
        }

        thread::sleep(*config.interval);
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("dup main ERROR: {}", e);
    }
}
