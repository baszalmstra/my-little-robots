use async_trait::async_trait;
use futures::future::join_all;
use futures::{StreamExt, TryFutureExt};
use serde_derive::{Deserialize, Serialize};
use std::iter::FromIterator;
use std::time::Duration;
use thiserror::Error;

/// A `PlayerId` uniquely describes a single Player
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PlayerId(pub usize);

/// A `World` defines the state of the world.
#[derive(Clone, Eq, Debug, PartialEq, Hash, Serialize, Deserialize)]
pub struct World {}

impl World {
    /// Applies the specified `actions` to an instance and returns a modified instance where these
    /// actions have been applied.
    fn apply(self, actions: impl IntoIterator<Item = Action>) -> Self {
        self
    }

    /// Creates a snapshot of the world as seen by the given Player.
    fn player_world(&self, player_id: PlayerId) -> PlayerWorld {
        PlayerWorld { player_id }
    }
}

/// A `PlayerWorld` represents only the visible parts of a world for a specific player.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct PlayerWorld {
    pub player_id: PlayerId,
}

/// Describes a possible action that can be performed in the world as ordered by a specific player.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum PlayerAction {
    DoNothing,
}

/// Describes an action in the world which may have been undertaken by any player
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum Action {}

/// The PlayerRunner can be implemented to produce actions for a current snapshot of the world.
#[async_trait]
pub trait PlayerRunner {
    /// Given the current state of the world, returns the actions that should be executed.
    async fn run(&mut self, input: RunnerInput) -> RunnerOutput;
}

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum RunnerError {
    #[error("internal error")]
    InternalError,

    #[error("the program exited before it returned any data")]
    NoData,

    #[error("the program took too long, past the time limit of {0:?}")]
    Timeout(Duration),
}

/// The input for a `PlayerRunner`
pub type RunnerInput = PlayerWorld;

/// The output of a `PlayerRunner`
pub type RunnerOutput = Result<Vec<PlayerAction>, RunnerError>;

// Implement `PlayerRunner` for a functions
#[async_trait]
impl<F> PlayerRunner for F
where
    F: FnMut(RunnerInput) -> RunnerOutput + Send,
{
    async fn run(&mut self, input: RunnerInput) -> RunnerOutput {
        (self)(input)
    }
}

/// Represents everything of a specific player.
pub struct Player {
    pub id: PlayerId,
    pub runner: Box<dyn PlayerRunner>,
}

/// Runs a single turn on the world
pub async fn turn(players: &mut [Player], world: World) -> World {
    // Get the actions from all the players
    let actions = join_all(players.iter_mut().map(|player| {
        let player_id = player.id;
        let world_ref = &world;
        async move {
            let player_world = world_ref.player_world(player_id);
            player.runner.run(player_world).await.map_or_else(
                |err| {
                    log::error!("Player {:?}: {}", player_id, err);
                    None
                },
                move |player_actions| {
                    Some(
                        player_actions
                            .into_iter()
                            .map(|action| validate_action(action, player_id, world_ref))
                            .filter_map(|action| match action {
                                Ok(action) => Some(action),
                                Err(err) => {
                                    log::error!("Player {:?}: invalid action: {}", player_id, err);
                                    None
                                }
                            })
                            .collect::<Vec<Action>>(),
                    )
                },
            )
        }
    }))
    .await
    .into_iter()
    .filter_map(|a| a)
    .flatten();

    // Run all actions on the world
    world.apply(actions.into_iter())
}

/// An error that might occur when a user sends an action that is not possible.
#[derive(Error, Clone, Debug)]
pub enum ActionValidationError {
    #[error("Invalid action")]
    InvalidAction(String),
}

/// Given an action from a player turn it into an action that can be applied to the world. Returns
/// an error if the action cannot be performed by the player.
fn validate_action(
    action: PlayerAction,
    player: PlayerId,
    world: &World,
) -> Result<Action, ActionValidationError> {
    Err(ActionValidationError::InvalidAction("".to_string()))
}
