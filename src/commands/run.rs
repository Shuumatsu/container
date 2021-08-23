use anyhow::{Context, Result};
use clap::{AppSettings, Parser};
use nix::mount::MsFlags;
use nix::{mount, sched, sys, unistd, NixPath};
use std::os::raw::c_int;
use std::os::unix::process::CommandExt;
use std::process::Command;
use tracing::{debug, error, event, info, instrument, trace, Level};

const STAK_SIZE: usize = 4096 * 1024;
const SIGCHLD: c_int = 17;

#[derive(Parser, Debug)]
pub struct RunOpts {
    /// Specify the root directory path
    #[clap(long, short, default_value = ".")]
    fsroot: String,
    /// Specify the path to the application to run
    app: String,
    /// Arguments to be passed to the app
    arguments: Vec<String>,
}

pub fn run(opts: RunOpts) -> Result<()> {
    unistd::chroot(&opts.fsroot[..])?;

    let child_pid = sched::clone(
        Box::new(|| contained_main(&opts.app[..], &opts.arguments[..]) as isize),
        Box::new([0u8; STAK_SIZE]).as_mut(),
        sched::CloneFlags::CLONE_NEWPID
            | sched::CloneFlags::CLONE_NEWNS
            | sched::CloneFlags::CLONE_NEWUTS,
        Some(SIGCHLD),
    )
    .context("error cloning new process")?;
    debug!("clone() = {}", child_pid);

    let status = sys::wait::wait().context("error waiting child process")?;
    debug!("waitpid() = {:?}", status);

    debug!("host nodename = {:?}", sys::utsname::uname().nodename());

    Ok(())
}

#[instrument]
fn contained_main(app: &str, arguments: &[String]) -> i32 {
    debug!(
        "this() = {}, parent() = {}",
        unistd::Pid::this(),
        unistd::Pid::parent()
    );

    let mut command = Command::new(app);
    command.env("PATH", "/bin").args(arguments);
    unsafe {
        command.pre_exec(|| {
            unistd::chdir("/")?;

            // sched::unshare(sched::CloneFlags::CLONE_NEWUTS)?;
            unistd::sethostname("container")?;
            debug!(
                "container nodename = {:?}",
                sys::utsname::uname().nodename()
            );

            // sched::unshare(sched::CloneFlags::CLONE_NEWNS)?;

            // mount::umount("/proc")?;
            // mount::umount("/tmp")?;

            Ok(())
        });
    };
    // let child = command.spawn().expect(&format!("{} failed to start", app));

    // On success this function will not return, and otherwise it will return an error indicating why the exec (or another part of the setup of the Command) failed.
    let err = command.exec();
    error!("error executing {}: {}", app, err);
    err.raw_os_error().unwrap()
}
