//! Unit QA tests for Varlink framing and method dispatch.

#[cfg(test)]
mod tests {
    use contextd_core::index::EventStore;
    use contextd_core::watcher::DiffStore;
    use contextd_daemon::varlink::{
        handle_service_call, Context1Handler, VarlinkReply,
    };
    use serde_json::json;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn test_varlink_reply_to_bytes_nul_terminated() {
        let reply = VarlinkReply::ok(json!({ "status": "ok" }));
        let bytes = reply.to_bytes();
        assert_eq!(*bytes.last().unwrap(), 0x00);
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes[..bytes.len() - 1]).unwrap();
        assert_eq!(parsed["parameters"]["status"], "ok");
    }

    #[test]
    fn test_handle_service_get_info() {
        let reply = handle_service_call("org.varlink.service.GetInfo", None);
        assert!(reply.is_some());
        let r = reply.unwrap();
        let params = r.parameters.unwrap();
        assert_eq!(params["product"], "contextd");
    }

    #[test]
    fn test_handle_service_get_interface_description() {
        let params = json!({ "interface": "io.syntrop.Context1" });
        let reply = handle_service_call(
            "org.varlink.service.GetInterfaceDescription",
            Some(&params),
        );
        assert!(reply.is_some());
        let r = reply.unwrap();
        let desc = r.parameters.unwrap();
        assert!(desc["description"]
            .as_str()
            .unwrap()
            .contains("interface io.syntrop.Context1"));
    }

    #[test]
    fn test_context1_record_and_list_events() {
        let tmp = tempdir().unwrap();
        let diff_store = Arc::new(DiffStore::new(tmp.path()).unwrap());
        let event_store = Arc::new(EventStore::new(tmp.path()).unwrap());
        let handler = Context1Handler::new(diff_store, event_store, Arc::new(vec![]));

        let record_params = json!({
            "source": "sentry",
            "unit": "test.service",
            "summary": "Process terminated by SIGSEGV",
            "details": "core dumped"
        });

        let rec_reply = handler
            .handle_call("io.syntrop.Context1.RecordEvent", Some(&record_params))
            .unwrap();
        assert!(rec_reply.error.is_none());
        assert!(rec_reply.parameters.unwrap().get("event_id").is_some());

        let list_params = json!({
            "unit": "test.service",
            "since_seconds": 3600,
            "limit": 10
        });

        let list_reply = handler
            .handle_call("io.syntrop.Context1.ListEvents", Some(&list_params))
            .unwrap();
        assert!(list_reply.error.is_none());
        let events = list_reply.parameters.unwrap();
        assert_eq!(events["events"].as_array().unwrap().len(), 1);
    }
}
