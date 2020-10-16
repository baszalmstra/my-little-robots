use crate::{Coord, PlayerId};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UnitId(pub(crate) usize);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Unit {
    pub id: UnitId,
    pub player: PlayerId,
    pub location: Coord,
}
