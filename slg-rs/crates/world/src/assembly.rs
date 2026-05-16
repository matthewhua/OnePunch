use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblyTroops {
    pub assembly_id: i32,
    pub troop_keys: Vec<i32>,
}

/// In-memory index of assembly membership.
pub struct AssemblyState {
    by_assembly: DashMap<i32, BTreeSet<i32>>,
    troop_to_assembly: DashMap<i32, i32>,
}

impl AssemblyState {
    pub fn new() -> Self {
        Self {
            by_assembly: DashMap::new(),
            troop_to_assembly: DashMap::new(),
        }
    }

    pub fn create(&self, assembly_id: i32, troop_key: i32) -> Result<AssemblyTroops> {
        if self.by_assembly.contains_key(&assembly_id) {
            return Err(anyhow!("assembly {} already exists", assembly_id));
        }
        if self.troop_to_assembly.contains_key(&troop_key) {
            return Err(anyhow!("troop {} already joined an assembly", troop_key));
        }

        let mut troop_keys = BTreeSet::new();
        troop_keys.insert(troop_key);
        self.by_assembly.insert(assembly_id, troop_keys);
        self.troop_to_assembly.insert(troop_key, assembly_id);
        Ok(self.snapshot(assembly_id).unwrap())
    }

    pub fn add_troop(&self, assembly_id: i32, troop_key: i32) -> Result<AssemblyTroops> {
        if self.troop_to_assembly.contains_key(&troop_key) {
            return Err(anyhow!("troop {} already joined an assembly", troop_key));
        }

        let mut entry = self
            .by_assembly
            .get_mut(&assembly_id)
            .ok_or_else(|| anyhow!("assembly {} not found", assembly_id))?;
        entry.insert(troop_key);
        self.troop_to_assembly.insert(troop_key, assembly_id);

        Ok(AssemblyTroops {
            assembly_id,
            troop_keys: entry.iter().copied().collect(),
        })
    }

    pub fn repatriate(&self, assembly_id: i32, troop_key: i32) -> Result<AssemblyTroops> {
        let recorded_assembly = self
            .troop_to_assembly
            .get(&troop_key)
            .map(|entry| *entry.value())
            .ok_or_else(|| anyhow!("assembly troop {} not found", troop_key))?;
        if recorded_assembly != assembly_id {
            return Err(anyhow!(
                "troop {} belongs to assembly {}, not {}",
                troop_key,
                recorded_assembly,
                assembly_id
            ));
        }

        let mut should_remove_assembly = false;
        let snapshot = {
            let mut entry = self
                .by_assembly
                .get_mut(&assembly_id)
                .ok_or_else(|| anyhow!("assembly {} not found", assembly_id))?;
            entry.remove(&troop_key);
            should_remove_assembly = entry.is_empty();
            AssemblyTroops {
                assembly_id,
                troop_keys: entry.iter().copied().collect(),
            }
        };

        self.troop_to_assembly.remove(&troop_key);
        if should_remove_assembly {
            self.by_assembly.remove(&assembly_id);
        }

        Ok(snapshot)
    }

    pub fn cancel(&self, assembly_id: i32) -> Result<Vec<i32>> {
        let (_, troop_keys) = self
            .by_assembly
            .remove(&assembly_id)
            .ok_or_else(|| anyhow!("assembly {} not found", assembly_id))?;

        let troop_keys: Vec<i32> = troop_keys.into_iter().collect();
        for troop_key in &troop_keys {
            self.troop_to_assembly.remove(troop_key);
        }
        Ok(troop_keys)
    }

    pub fn snapshot(&self, assembly_id: i32) -> Option<AssemblyTroops> {
        self.by_assembly
            .get(&assembly_id)
            .map(|entry| AssemblyTroops {
                assembly_id,
                troop_keys: entry.iter().copied().collect(),
            })
    }
}

impl Default for AssemblyState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_add_troops_to_assembly() {
        let state = AssemblyState::new();

        let assembly = state.create(10, 100).unwrap();
        assert_eq!(
            assembly,
            AssemblyTroops {
                assembly_id: 10,
                troop_keys: vec![100],
            }
        );

        let assembly = state.add_troop(10, 101).unwrap();
        assert_eq!(assembly.troop_keys, vec![100, 101]);
    }

    #[test]
    fn rejects_duplicate_assembly_or_troop() {
        let state = AssemblyState::new();

        state.create(10, 100).unwrap();

        assert!(state.create(10, 101).is_err());
        assert!(state.create(11, 100).is_err());
        assert!(state.add_troop(10, 100).is_err());
        assert!(state.add_troop(999, 102).is_err());
    }

    #[test]
    fn repatriate_removes_one_troop() {
        let state = AssemblyState::new();

        state.create(10, 100).unwrap();
        state.add_troop(10, 101).unwrap();

        let assembly = state.repatriate(10, 101).unwrap();

        assert_eq!(assembly.troop_keys, vec![100]);
        assert!(state.repatriate(10, 101).is_err());
        assert_eq!(state.snapshot(10).unwrap().troop_keys, vec![100]);
    }

    #[test]
    fn repatriate_rejects_wrong_assembly() {
        let state = AssemblyState::new();

        state.create(10, 100).unwrap();

        assert!(state.repatriate(11, 100).is_err());
    }

    #[test]
    fn cancel_removes_assembly_and_membership() {
        let state = AssemblyState::new();

        state.create(10, 100).unwrap();
        state.add_troop(10, 101).unwrap();

        let troop_keys = state.cancel(10).unwrap();

        assert_eq!(troop_keys, vec![100, 101]);
        assert!(state.snapshot(10).is_none());
        assert!(state.cancel(10).is_err());
        state.create(11, 100).unwrap();
    }
}
