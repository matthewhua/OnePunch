use std::collections::{HashSet, VecDeque};

const MAX_PROCESSED_OUTBOUND_EVENTS: usize = 4096;

/// In-memory World outbound dedupe for the online actor lifetime.
///
/// This intentionally does not serialize into `world_func`, because that column
/// stores the client-facing `WorldDataFunction` protobuf.
pub struct WorldSystem {
    processed_outbound_events: VecDeque<String>,
    processed_outbound_set: HashSet<String>,
}

impl WorldSystem {
    pub fn new() -> Self {
        Self {
            processed_outbound_events: VecDeque::new(),
            processed_outbound_set: HashSet::new(),
        }
    }

    pub fn has_processed_outbound(&self, event_token: &str) -> bool {
        self.processed_outbound_set.contains(event_token)
    }

    pub fn mark_outbound_processed(&mut self, event_token: String) {
        if event_token.is_empty() || !self.processed_outbound_set.insert(event_token.clone()) {
            return;
        }

        self.processed_outbound_events.push_back(event_token);
        while self.processed_outbound_events.len() > MAX_PROCESSED_OUTBOUND_EVENTS {
            if let Some(evicted) = self.processed_outbound_events.pop_front() {
                self.processed_outbound_set.remove(&evicted);
            }
        }
    }
}

impl Default for WorldSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processed_outbound_events_deduplicate_in_memory() {
        let mut system = WorldSystem::new();
        system.mark_outbound_processed("event-a".to_string());
        system.mark_outbound_processed("event-b".to_string());
        system.mark_outbound_processed("event-a".to_string());

        assert!(system.has_processed_outbound("event-a"));
        assert!(system.has_processed_outbound("event-b"));
        assert!(!system.has_processed_outbound("event-c"));
        assert_eq!(system.processed_outbound_events.len(), 2);
    }
}
