mod application;

use anyhow::Context;
use anyhow::{anyhow, bail};
use itertools::Itertools;
use mlr::GameState;
use mlr::Player;
use mlr::Runner;
use mlr::World;
use mlr_api::{Coord, PlayerId};
use serde_json::json;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
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
            let players = run_opt
                .runners
                .iter()
                .enumerate()
                .map(|(i, r)| -> anyhow::Result<Player> {
                    let runner = RunnerDesc::parse(r)?;
                    Ok(Player {
                        id: PlayerId(i),
                        runner: Box::new(runner.into_runner()?),
                        memory: json!({}),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let mut game_state = GameState {
                players,
                world: World::default(),
            };

            // Spawn a unit for every player
            for (i, player) in game_state.players.iter().enumerate() {
                game_state
                    .world
                    .spawn_unit(player.id, Coord::new(10 + i as isize * 10, 10));
            }

            // Create the world
            let (sender, receiver) = async_watch::channel(game_state.world.clone());

            std::thread::spawn(|| {
                async_std::task::block_on(async move {
                    // Run the turn in a loop
                    loop {
                        game_state = game_state.turn().await;
                        if sender.send(game_state.world.clone()).is_err() {
                            break; // Sender closed
                        }
                        if game_state.world.units_on_exits().next().is_some() {
                            break;
                        }
                        async_std::task::sleep(std::time::Duration::from_millis(100)).await;
                    }
                });
            });

            // Render our world
            application::run(receiver).expect("failed to render");
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
