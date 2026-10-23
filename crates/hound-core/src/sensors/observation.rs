use crate::Result;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub address: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolEvent {
    pub protocol: String,
    pub method: String,
    pub path: String,
    pub message_id: String,
    pub operation: String,
    pub payload_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub capture_method: String,
    pub sensor_instance: String,
    pub sequence: u64,
}

/// A sensor's report of a received HTTP request, never an attribution claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub observation_id: String,
    pub sensor_id: String,
    pub timestamp_unix_ms: u64,
    pub event: ProtocolEvent,
    pub source_endpoint: Endpoint,
    pub destination_endpoint: Endpoint,
    pub provenance: Provenance,
}

pub(crate) fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
}

impl Observation {
    pub fn validate(&self) -> Result<()> {
        let p = &self.provenance;
        if !identifier(&self.sensor_id)
            || !identifier(&p.sensor_instance)
            || p.sequence == 0
            || self.observation_id
                != format!("{}:{}:{}", self.sensor_id, p.sensor_instance, p.sequence)
        {
            return Err("invalid observation identity or provenance sequence".into());
        }
        if self.timestamp_unix_ms == 0
            || p.capture_method != "http_server_receive"
            || self.source_endpoint.port == 0
            || self.destination_endpoint.port == 0
        {
            return Err("invalid observation timestamp, capture method, or endpoint".into());
        }
        let e = &self.event;
        if e.protocol != "http"
            || e.method != "POST"
            || e.path != "/ingest"
            || !identifier(&e.message_id)
            || !matches!(e.operation.as_str(), "heartbeat" | "probe" | "transfer")
            || e.payload_bytes == 0
            || e.payload_bytes > 4096
        {
            return Err("unsupported or invalid protocol event".into());
        }
        Ok(())
    }
}
