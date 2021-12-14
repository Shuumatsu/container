use anyhow::{Context, Result};
use clap::Parser;
use nix::mount::MsFlags;
use nix::{mount, sched, sys, unistd, NixPath};
use std::os::unix::process::CommandExt;
use std::process::Command;
use tracing::{debug, error, event, info, instrument, trace, Level};

#[derive(Parser, Debug)]
pub struct RunOpts {
    /// Specify the root directory path
    #[clap(long, short)]
    fsroot: String,
    /// Specify the path to the application to run
    app: String,
    /// Arguments to be passed to the app
    arguments: Vec<String>,
}

#[instrument]
pub fn run(opts: RunOpts) -> Result<()> {
    debug!("host nodename = {:?}", sys::utsname::uname().nodename());

    sched::unshare(sched::CloneFlags::CLONE_NEWNS)?;

    sched::unshare(sched::CloneFlags::CLONE_NEWPID)?;

    sched::unshare(sched::CloneFlags::CLONE_NEWUTS)?; // for hostname
    unistd::sethostname("container")?;
    debug!(
        "container nodename = {:?}",
        sys::utsname::uname().nodename()
    );

    let mut command = Command::new("/proc/self/exe");
    command.arg0("init").arg("start");
    command.args(["--fsroot", &opts.fsroot]);
    command.arg(&opts.app).args(opts.arguments);
    command.env("PATH", "/bin");

    let status = command.spawn()?.wait()?;
    debug!("container exited with status {:?}", status);

    Ok(())
}

#[instrument]
pub fn start(opts: RunOpts) -> Result<()> {
    unistd::chroot(&opts.fsroot[..])?;
    unistd::chdir("/")?;

    mount::mount(
        None::<&str>,
        "/proc",
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )?;
    mount::mount(
        None::<&str>,
        "/tmp",
        Some("tmpfs"),
        MsFlags::empty(),
        None::<&str>,
    )?;

    let mut command = Command::new(opts.app);
    command.args(opts.arguments);

    let status = command.spawn()?.wait()?;
    debug!("application exited with status {:?}", status);

    mount::umount("/proc")?;
    mount::umount("/tmp")?;

    Ok(())
}

// 其实我们要运行的程序是在 container 里的 init 线程后再运行的 （是的 app 是 init 的子进程，
// 我们不能直接创建 app 的进程，因为 container 进程我们是希望运行在宿主环境的
// 而如果我们直接创建 app 的进程，
//  1. pre_exec 和 post_exec 不好办
//  2. 信号等会不好处理，因为 init 进程不会执行未注册信号的默认逻辑，比如我们不能 ctrl c 停止一个死循环程序

// 我试了一下这个 ctrl c 把 container 停了都还在跑
// #include <stdio.h>
// #include <unistd.h>

// int main() {
//     for (;;) {
//         printf("Hello, World!\n");
//         sleep(1);
//     }
//     return 0;
// }

// 所以其实可以在 init 线程里执行 pre_exec 和 post_exec
