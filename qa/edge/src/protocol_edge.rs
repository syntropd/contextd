//! Edge tests for Varlink framing and protocol boundary conditions.

#[cfg(test)]
mod tests {
    use contextd_daemon::varlink::{VarlinkCall, VarlinkReply};
    use serde_json::json;

    #[test]
    fn test_varlink_call_deserialization_edge_cases() {
        // Missing parameters object
        let raw = b"{\"method\":\"org.varlink.service.GetInfo\"}";
        let call: Result<VarlinkCall, _> = serde_json::from_slice(raw);
        assert!(call.is_ok());
        let c = call.unwrap();
        assert_eq!(c.method, "org.varlink.service.GetInfo");
        assert!(c.parameters.is_none());

        // Malformed json
        let corrupt = b"{\"method\":";
        let err: Result<VarlinkCall, _> = serde_json::from_slice(corrupt);
        assert!(err.is_err());
    }

    #[test]
    fn test_empty_string_parameters() {
        let reply = VarlinkReply::err("", None);
        let bytes = reply.to_bytes();
        assert_eq!(*bytes.last().unwrap(), 0x00);
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes[..bytes.len() - 1]).unwrap();
        assert_eq!(parsed["error"], "");
    }

    #[test]
    fn test_deeply_nested_parameters() {
        let nested = json!({
            "level1": {
                "level2": {
                    "level3": "deep_value"
                }
            }
        });
        let reply = VarlinkReply::ok(nested);
        let bytes = reply.to_bytes();
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes[..bytes.len() - 1]).unwrap();
        assert_eq!(
            parsed["parameters"]["level1"]["level2"]["level3"],
            "deep_value"
        );
    }
}
