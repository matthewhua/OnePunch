use anyhow::{anyhow, Result};
use dashmap::DashMap;
use proto::slg::GarrisonTroop;

/// In-memory index of garrison troops grouped by map position.
pub struct GarrisonState {
    by_pos: DashMap<i32, Vec<GarrisonTroop>>,
}

impl GarrisonState {
    pub fn new() -> Self {
        Self {
            by_pos: DashMap::new(),
        }
    }

    pub fn place(&self, pos: i32, troop: GarrisonTroop) -> Result<()> {
        let troop_key = troop_key(&troop)?;
        if self.contains_troop(troop_key) {
            return Err(anyhow!("garrison troop {} already exists", troop_key));
        }

        let mut troops = self.by_pos.entry(pos).or_default();
        troops.push(troop);
        troops.sort_by_key(|troop| troop.troop_key_id.unwrap_or_default());
        Ok(())
    }

    pub fn list(&self, pos: Option<i32>) -> Vec<GarrisonTroop> {
        let mut troops = match pos {
            Some(pos) => self
                .by_pos
                .get(&pos)
                .map(|entry| entry.value().clone())
                .unwrap_or_default(),
            None => self
                .by_pos
                .iter()
                .flat_map(|entry| entry.value().clone())
                .collect(),
        };
        troops.sort_by_key(|troop| troop.troop_key_id.unwrap_or_default());
        troops
    }

    pub fn repatriate_one(&self, troop_key: i32) -> Result<GarrisonTroop> {
        let mut positions: Vec<i32> = self.by_pos.iter().map(|entry| *entry.key()).collect();
        positions.sort_unstable();

        for pos in positions {
            let mut should_remove_pos = false;
            let removed = {
                let mut entry = self
                    .by_pos
                    .get_mut(&pos)
                    .ok_or_else(|| anyhow!("garrison position {} not found", pos))?;
                if let Some(index) = entry
                    .iter()
                    .position(|troop| troop.troop_key_id == Some(troop_key))
                {
                    let troop = entry.remove(index);
                    should_remove_pos = entry.is_empty();
                    Some(troop)
                } else {
                    None
                }
            };

            if should_remove_pos {
                self.by_pos.remove(&pos);
            }
            if let Some(troop) = removed {
                return Ok(troop);
            }
        }

        Err(anyhow!("garrison troop {} not found", troop_key))
    }

    pub fn repatriate_all(&self) -> Vec<GarrisonTroop> {
        let mut positions: Vec<i32> = self.by_pos.iter().map(|entry| *entry.key()).collect();
        positions.sort_unstable();

        let mut troops = Vec::new();
        for pos in positions {
            if let Some((_, mut removed)) = self.by_pos.remove(&pos) {
                troops.append(&mut removed);
            }
        }
        troops.sort_by_key(|troop| troop.troop_key_id.unwrap_or_default());
        troops
    }

    fn contains_troop(&self, troop_key: i32) -> bool {
        self.by_pos.iter().any(|entry| {
            entry
                .value()
                .iter()
                .any(|troop| troop.troop_key_id == Some(troop_key))
        })
    }
}

impl Default for GarrisonState {
    fn default() -> Self {
        Self::new()
    }
}

fn troop_key(troop: &GarrisonTroop) -> Result<i32> {
    troop
        .troop_key_id
        .ok_or_else(|| anyhow!("garrison troop key is required"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn troop(key: i32) -> GarrisonTroop {
        GarrisonTroop {
            role_id: Some(1000 + i64::from(key)),
            name: Some(format!("player-{key}")),
            troop_key_id: Some(key),
            end_time: Some(10_000 + i64::from(key)),
            ..Default::default()
        }
    }

    #[test]
    fn place_and_list_garrison_troops_by_position() {
        let state = GarrisonState::new();

        state.place(101, troop(2)).unwrap();
        state.place(101, troop(1)).unwrap();
        state.place(202, troop(3)).unwrap();

        let troops = state.list(Some(101));
        assert_eq!(
            troops
                .iter()
                .map(|troop| troop.troop_key_id.unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(state.list(None).len(), 3);
        assert!(state.list(Some(303)).is_empty());
    }

    #[test]
    fn rejects_duplicate_or_missing_garrison_troop_key() {
        let state = GarrisonState::new();

        state.place(101, troop(1)).unwrap();
        assert!(state.place(202, troop(1)).is_err());
        assert!(state
            .place(
                202,
                GarrisonTroop {
                    troop_key_id: None,
                    ..Default::default()
                },
            )
            .is_err());
    }

    #[test]
    fn repatriate_one_removes_matching_troop() {
        let state = GarrisonState::new();

        state.place(101, troop(1)).unwrap();
        state.place(101, troop(2)).unwrap();

        let removed = state.repatriate_one(1).unwrap();

        assert_eq!(removed.troop_key_id, Some(1));
        assert_eq!(state.list(Some(101)).len(), 1);
        assert!(state.repatriate_one(1).is_err());
    }

    #[test]
    fn repatriate_all_clears_state() {
        let state = GarrisonState::new();

        state.place(101, troop(2)).unwrap();
        state.place(202, troop(1)).unwrap();

        let removed = state.repatriate_all();

        assert_eq!(
            removed
                .iter()
                .map(|troop| troop.troop_key_id.unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert!(state.list(None).is_empty());
    }
}
