use anyhow::{Context, Result};
use clap::{AppSettings, Parser};
use nix::mount::MsFlags;
use nix::{mount, sched, sys, unistd, NixPath};
use std::os::raw::c_int;
use std::os::unix::process::CommandExt;
use std::process::Command;
use tracing::{debug, error, event, info, instrument, trace, Level};

#[derive(Parser, Debug)]
pub struct PullOpts {
    /// Specify the image name
    #[clap(long, short)]
    image: String,
    /// Specify the tag of the image
    tag: String,
}
