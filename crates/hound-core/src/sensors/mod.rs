//! Read-only HTTP collection from explicitly configured laboratory sensors.

mod observation;
use crate::Result;
pub use observation::{Endpoint, Observation, ProtocolEvent, Provenance};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const MAX_RESPONSE: u64 = 1_048_576;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpSensor {
    pub sensor_id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SensorBatch {
    schema_version: u32,
    observations: Vec<Observation>,
}

pub trait Sensor {
    fn observations(&self) -> Result<Vec<Observation>>;
}

impl HttpSensor {
    /// Supports only the lab's HTTP/IPv4-or-DNS endpoint format.
    /// No credentials, redirects, proxies, query strings, or arbitrary paths.
    pub fn validate(&self) -> Result<(&str, u16)> {
        if !observation::identifier(&self.sensor_id) {
            return Err("invalid sensor identifier".into());
        }
        let authority = self
            .url
            .strip_prefix("http://")
            .and_then(|s| s.strip_suffix("/observations"))
            .ok_or("sensor URL must be http://HOST:PORT/observations")?;
        let (host, port) = authority
            .rsplit_once(':')
            .ok_or("sensor URL requires a port")?;
        if host.is_empty()
            || !host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
        {
            return Err("invalid sensor host".into());
        }
        let port = port.parse::<u16>()?;
        if port == 0 {
            return Err("invalid sensor port".into());
        }
        Ok((host, port))
    }
    pub fn parse(&self, bytes: &[u8]) -> Result<Vec<Observation>> {
        self.validate()?;
        let batch: SensorBatch = serde_json::from_slice(bytes)?;
        if batch.schema_version != 1 {
            return Err("unsupported sensor schema version".into());
        }
        let mut ids = HashSet::new();
        for observation in &batch.observations {
            observation.validate()?;
            if observation.sensor_id != self.sensor_id {
                return Err("observation sensor does not match the authorized sensor".into());
            }
            if !ids.insert(&observation.observation_id) {
                return Err("duplicate observation identifier in sensor response".into());
            }
        }
        Ok(batch.observations)
    }
}

impl Sensor for HttpSensor {
    fn observations(&self) -> Result<Vec<Observation>> {
        let (host, port) = self.validate()?;
        let timeout = Duration::from_secs(3);
        let mut stream = (host, port)
            .to_socket_addrs()?
            .find_map(|addr| TcpStream::connect_timeout(&addr, timeout).ok())
            .ok_or("cannot connect to authorized sensor")?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        write!(
            stream,
            "GET /observations HTTP/1.0\r\nHost: {host}:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
        )?;
        let mut response = Vec::new();
        stream.take(MAX_RESPONSE + 1).read_to_end(&mut response)?;
        if response.len() as u64 > MAX_RESPONSE {
            return Err("sensor response exceeds size limit".into());
        }
        let split = response
            .windows(4)
            .position(|p| p == b"\r\n\r\n")
            .ok_or("invalid HTTP response")?;
        let headers = std::str::from_utf8(&response[..split])?;
        let mut lines = headers.split("\r\n");
        let status = lines.next().ok_or("missing HTTP status")?;
        let parts: Vec<_> = status.split_whitespace().collect();
        if parts.len() < 2 || !matches!(parts[0], "HTTP/1.0" | "HTTP/1.1") || parts[1] != "200" {
            return Err(format!("sensor refused request: {status}").into());
        }
        let mut length = None;
        let mut json = false;
        for line in lines {
            let (name, value) = line.split_once(':').ok_or("invalid HTTP header")?;
            match name.to_ascii_lowercase().as_str() {
                "content-length" => {
                    if length.is_some() {
                        return Err("duplicate Content-Length".into());
                    }
                    length = Some(value.trim().parse::<usize>()?);
                }
                "transfer-encoding" => {
                    return Err("chunked sensor responses are unsupported".into());
                }
                "content-type" => json = value.trim().split(';').next() == Some("application/json"),
                _ => (),
            }
        }
        let body = &response[split + 4..];
        if length != Some(body.len()) || !json {
            return Err("invalid HTTP length or content type".into());
        }
        self.parse(body)
    }
}
