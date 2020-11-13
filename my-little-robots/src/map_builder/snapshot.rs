use super::Map;

/// A trait that optionally enables creating snapshots of `Map`s
pub trait SnapshotableMap {
    /// Calls the given function with a mutable map that can be edited.
    fn with_snapshot<T, F: FnMut(&mut Map) -> T>(&mut self, f: F) -> T;
}

#[derive(Default)]
pub struct MapWithSnapshots {
    snapshots: Vec<Map>,
}

impl From<Map> for MapWithSnapshots {
    fn from(map: Map) -> Self {
        MapWithSnapshots {
            snapshots: vec![map],
        }
    }
}

impl From<MapWithSnapshots> for Map {
    fn from(history: MapWithSnapshots) -> Self {
        history
            .snapshots
            .into_iter()
            .last()
            .expect("there has to be an initial map version")
    }
}

impl From<MapWithSnapshots> for Vec<Map> {
    fn from(history: MapWithSnapshots) -> Self {
        history.snapshots
    }
}

impl SnapshotableMap for MapWithSnapshots {
    fn with_snapshot<T, F: FnMut(&mut Map) -> T>(&mut self, mut f: F) -> T {
        let mut new_map = self
            .snapshots
            .last()
            .expect("there has to be an initial map version")
            .clone();
        let result = f(&mut new_map);
        self.snapshots.push(new_map);
        result
    }
}

impl SnapshotableMap for Map {
    fn with_snapshot<T, F: FnMut(&mut Map) -> T>(&mut self, mut f: F) -> T {
        f(self)
    }
}
