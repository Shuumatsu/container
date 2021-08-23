#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate anyhow;

mod cgroup;
mod commands;
mod docker;
mod namespace;
mod storage;

use anyhow::{Context, Result};
use clap::{AppSettings, Parser};
use commands::{run, RunOpts};
use nix::{sched, sys, unistd};
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::{env, os::raw::c_int};
use tracing::{debug, error, event, info, instrument, trace, Level};
use tracing_subscriber::prelude::*;

#[derive(Parser, Debug)]
#[clap(about, version)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Run(RunOpts),
}

#[instrument]
fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .pretty() // enable everything
        .with_max_level(tracing::Level::TRACE)
        .init();

    let opts: Opts = Opts::parse();

    println!("{:?}", opts);
    match opts.subcmd {
        SubCommand::Run(opts) => {
            run(opts)?;
        }
    }

    // // sched::setns(fd, nstype)?;
    // // sched::unshare(sched::CloneFlags::CLONE_NEWNS)?;

    // // let mut cgroup = rand::random::<String>();
    // // let cgroup_path = PathBuf::from(format!("/sys/fs/cgroup/{}", cgroup));

    // debug!("container_pid = {}", unistd::Pid::this());

    Ok(())
}
