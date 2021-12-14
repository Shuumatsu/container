A tiny container runtime written in Rust. The idea is to support a minimal isolated containers without using existing runtimes.

|        Feature        |                   Description                   |                                                State                                                |
| :-------------------: | :---------------------------------------------: | :-------------------------------------------------------------------------------------------------: |
|      pivot_root       |            Change the root directory            |                                                  ✅                                                  |
|        Mounts         |    Mount files and directories to container     |                                                  ✅                                                  |
|      Namespaces       |         Isolation of various resources          |                                                  ✅                                                  |
|      Cgroups v2       |            Resource limitations, etc            |                                                  TODO                                                 |
|         Hooks         | Add custom processing during container creation |                                                  TODO                                                 |


## Usage 

```
container-run 

USAGE:
    container run --fsroot <FSROOT> <APP> [ARGUMENTS]...

ARGS:
    <APP>             Specify the path to the application to run
    <ARGUMENTS>...    Arguments to be passed to the app

OPTIONS:
    -f, --fsroot <FSROOT>    Specify the root directory path
    -h, --help               Print help information
```

### Create and run a container


```
$ mkdir alpine-rootfs
# use docker to export alpine into the rootfs directory
$ docker export $(docker create alpine) | tar -C alpine-rootfs -xvf -
```

Then you can run applications inside the container:

```
cargo build && sudo ./target/debug/container run -f alpine-rootfs sh
```