//! Unit QA tests for EventStore append-only persistence and query engine.

#[cfg(test)]
mod tests {
    use contextd_core::index::{EventStore, SystemEvent};
    use tempfile::tempdir;

    #[test]
    fn test_append_and_query_all() {
        let tmp = tempdir().unwrap();
        let store = EventStore::new(tmp.path()).unwrap();

        let event1 = SystemEvent::new("sentry", Some("nginx.service"), "Crash detected", None);
        let event2 = SystemEvent::new("inferenced", None, "VRAM eviction started", None);

        store.append(&event1).unwrap();
        store.append(&event2).unwrap();

        let results = store.query(None, 0, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].source, "sentry");
        assert_eq!(results[1].source, "inferenced");
    }

    #[test]
    fn test_filter_by_unit() {
        let tmp = tempdir().unwrap();
        let store = EventStore::new(tmp.path()).unwrap();

        let event1 = SystemEvent::new("sentry", Some("unit-a.service"), "Unit A failure", None);
        let event2 = SystemEvent::new("sentry", Some("unit-b.service"), "Unit B failure", None);

        store.append(&event1).unwrap();
        store.append(&event2).unwrap();

        let results = store.query(Some("unit-a.service"), 0, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].unit.as_deref(), Some("unit-a.service"));
    }

    #[test]
    fn test_query_respects_limit() {
        let tmp = tempdir().unwrap();
        let store = EventStore::new(tmp.path()).unwrap();

        for i in 0..10 {
            let event = SystemEvent::new("test", None, &format!("Event {}", i), None);
            store.append(&event).unwrap();
        }

        let results = store.query(None, 0, 3).unwrap();
        assert_eq!(results.len(), 3);
    }
}
