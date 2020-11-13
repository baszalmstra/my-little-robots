mod application;

use anyhow::Context;
use anyhow::{anyhow, bail};
use itertools::Itertools;
use mlr::Battle;
use mlr::Runner;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "my-little-robots CLI", author, setting = clap::AppSettings::DeriveDisplayOrder)]
enum MyLittleRobots {
    /// Command for running a local match
    Run(Run),
}

#[derive(StructOpt)]
#[structopt(setting = clap::AppSettings::DeriveDisplayOrder)]
struct Run {
    /// The runners that should be placed in the match.
    ///
    /// A runner is specified in one of the following ways:
    /// 1. `command:$PATH` or `localrunner:$PATH`. The path to a binary file.
    #[structopt(
        parse(from_os_str),
        required = true,
        min_values = 2,
        verbatim_doc_comment
    )]
    runners: Vec<OsString>,
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("ERROR: {}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("because: {}", cause));
        std::process::exit(1)
    }
}

fn try_main() -> anyhow::Result<()> {
    env_logger::try_init()?;

    let opt: MyLittleRobots = MyLittleRobots::from_args();

    match opt {
        MyLittleRobots::Run(run_opt) => {
            let mut battle = Battle::default();

            // Parse all runner descriptions into actual runners
            let runners = run_opt
                .runners
                .iter()
                .map(|runner_desc| -> anyhow::Result<_> {
                    let runner = RunnerDesc::parse(runner_desc)?;
                    Ok(Box::new(runner.into_runner()?))
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Add all runners as players to the battle
            for runner in runners {
                battle.add_player(runner);
            }

            // Construct the future for the battle
            let (sender, receiver) = async_std::sync::channel(1);
            std::thread::spawn(|| {
                async_std::task::block_on(
                    battle.run(Some(Duration::from_millis(100)), Some(sender)),
                )
            });

            // Await the first world send by the battle
            let world = async_std::task::block_on(receiver.recv())?;

            // Spawn a task that continuously updates the latest world received by the battle.
            let (world_sender, world_receiver) = async_watch::channel(world);
            async_std::task::spawn(async move {
                while let Ok(world) = receiver.recv().await {
                    if world_sender.send(world).is_err() {
                        break;
                    }
                }
            });

            // Render our world
            application::run(world_receiver).expect("failed to render");
        }
    }

    Ok(())
}

enum RunnerDesc {
    Command { command: String, args: Vec<String> },
    Source { source: PathBuf },
}

impl RunnerDesc {
    pub fn parse(s: &OsStr) -> anyhow::Result<Self> {
        let s = match s.to_str() {
            Some(s) => s,
            None => return Self::from_path(PathBuf::from(s)),
        };

        let parse_command = |s| -> anyhow::Result<_> {
            let mut args = shell_words::split(s)
                .context("couldn't parse as shell arguments")?
                .into_iter();
            let command = args.next().ok_or_else(|| {
                anyhow!("you must have at least one shell 'word' in the command string")
            })?;
            Ok((command, args.collect_vec()))
        };

        if let Some((typ, content)) = s.splitn(2, ':').collect_tuple() {
            match typ {
                "file" | "local" => Self::from_path(PathBuf::from(content)),
                "command" => {
                    let (command, args) = parse_command(content)?;
                    Ok(Self::Command { command, args })
                }
                _ => bail!("unknown runner type {:?}", typ),
            }
        } else {
            Self::from_path(PathBuf::from(s))
        }
    }

    fn from_path(source: PathBuf) -> anyhow::Result<Self> {
        Ok(RunnerDesc::Source { source })
    }

    /// Construct a runner from this description
    pub fn into_runner(self) -> anyhow::Result<Runner> {
        match self {
            RunnerDesc::Command { command, args } => Ok(Runner::new_cmd(command, args)),
            RunnerDesc::Source { source } => Runner::new_wasm(source),
        }
    }
}
