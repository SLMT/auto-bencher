# Auto Bencher

A deployment and benchmarking tool that aims to benchmark [ElaSQL](https://github.com/elasql/elasql) with [ElaSQL-Bench](https://github.com/elasql/elasqlbench) on a cluster.

## Prerequisite

### Environments

- Operating System: UNIX-based OS
  - Have been tested on CentOS 7
- An account that can login to all the machines in the cluster with the same username and without entering password via SSH
  - In other words, the system on every machine should have a user with same username.
  - Logging in without using a password can be achieved by `ssh-copy-id` (see [this](https://linuxhint.com/use-ssh-copy-id-command/) for more information).
- Rust Development Environment
  - Only need to be installed on the machine where this tool runs.
  - Since this tool is written in Rust, we need its package manager to setup dependency and compile the program.
  - See [this page](https://www.rust-lang.org/tools/install) for how to install Rust environment.
- Java Development Kit (JDK) 8
  - We just need a `tar.gz` file. No need to be installed on the machines. Auto Bencher will deploy the given JDK on the machines automatically.

### Knowledge

All the following knowledge can be learned from [Getting Started Guide](https://github.com/elasql/documentation/blob/master/doc/getting_started.pdf) of ElaSQL.

- Knowing how to package [ElaSQL-Bench](https://github.com/elasql/elasqlbench) into runnable JARs.
  - Specifically, we need a `server.jar` for servers and a `client.jar` for clients.
- Knowing how to configure [ElaSQL-Bench](https://github.com/elasql/elasqlbench).

## Usage Guide

We have prepared [a comprehensive guide](doc/usage-guide.pdf) for any one who want to learn how to use Auto Bencher. Check the document about the usage.

## Available Commands

- `cargo run init-env`
  - Initialzes the testing environment on all the machines.
- `cargo run load [db name] [parameter file]`
  - Loads a testbed with the given parameters in `[parameter file]` with a assigned `[db name]`.
- `cargo run bench [db name] [parameter file]`
  - Benchmarks ElaSQL with the given parameters in `[parameter file]` and the testbed loaded in `[db name]` DB.
- `cargo run all-exec [command]`
  - Executes the given command `[command]` on all the machines.
- `cargo run pull [pattern]`
  - Pulls the files with the names that match `[pattern]` on all the machines.

## Debugging Messages

To enable debugging message for Auto Bencher, set environment variable `RUST_LOG` with `auto_bencher=DEBUG`.

For example, to check the debugging message when benchmarking:

```
> RUST_LOG=auto_bencher=DEBUG cargo run bench my-db my-parameter-file
```
