//! Edge tests for corrupt, truncated, or malformed event logs.

#[cfg(test)]
mod tests {
    use contextd_core::index::{EventStore, SystemEvent};
    use std::fs::OpenOptions;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_handles_corrupt_jsonl_gracefully() {
        let tmp = tempdir().unwrap();
        let store = EventStore::new(tmp.path()).unwrap();

        // Write a valid event first
        let evt1 = SystemEvent::new("init", None, "First boot", None);
        store.append(&evt1).unwrap();

        // Inject corrupt bytes and half-written lines directly into the file
        let log_file = tmp.path().join("events.jsonl");
        {
            let mut file = OpenOptions::new().append(true).open(&log_file).unwrap();
            file.write_all(b"{\"incomplete\": json without ending\n").unwrap();
            file.write_all(b"\x00\xFF\xFE binary garbage\n").unwrap();
            file.write_all(b"\n\n   \n").unwrap(); // Empty lines
        }

        // Write a second valid event
        let evt2 = SystemEvent::new("sentry", None, "Recovered boot", None);
        store.append(&evt2).unwrap();

        // Query should safely ignore corrupt lines and return valid ones
        let results = store.query(None, 0, 100).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].summary, "First boot");
        assert_eq!(results[1].summary, "Recovered boot");
    }

    #[test]
    fn test_zero_byte_event_store() {
        let tmp = tempdir().unwrap();
        let store = EventStore::new(tmp.path()).unwrap();
        let results = store.query(None, 0, 10).unwrap();
        assert!(results.is_empty());
    }
}
