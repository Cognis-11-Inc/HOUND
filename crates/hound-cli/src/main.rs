use hound_core::{
    Result,
    memory::{CollectionProvenance, EvidenceRecord, now_unix_ms, record_evidence},
    sensors::{HttpSensor, Sensor},
};
use std::collections::HashSet;
use std::path::PathBuf;

const HELP: &str = "HOUND — Cognis 11 Inc.\nUsage: hound sniff [--sensor ID=http://HOST:PORT/observations]... [--output PATH]\nDefault sensors: node-c, node-d, node-e on port 8080 (Compose laboratory).\nEvidence output: artifacts/observations.jsonl. Parent directory must exist.\nOnly explicitly authorized laboratory sensors may be configured.";

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--help" | "-h") | None => {
            println!("{HELP}");
            return Ok(());
        }
        Some("sniff") => (),
        _ => return Err(HELP.into()),
    }
    let mut sensors = Vec::new();
    let mut output = PathBuf::from("artifacts/observations.jsonl");
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--sensor" => {
                let spec = args.next().ok_or("--sensor requires ID=URL")?;
                let (id, url) = spec.split_once('=').ok_or("--sensor requires ID=URL")?;
                sensors.push(HttpSensor {
                    sensor_id: id.into(),
                    url: url.into(),
                });
            }
            "--output" => output = args.next().ok_or("--output requires PATH")?.into(),
            "--help" | "-h" => {
                println!("{HELP}");
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {flag}").into()),
        }
    }
    if sensors.is_empty() {
        sensors = ["node-c", "node-d", "node-e"]
            .into_iter()
            .map(|id| HttpSensor {
                sensor_id: id.into(),
                url: format!("http://{id}:8080/observations"),
            })
            .collect();
    }
    let mut ids = HashSet::new();
    let mut urls = HashSet::new();
    for sensor in &sensors {
        sensor.validate()?;
        if !ids.insert(&sensor.sensor_id) || !urls.insert(&sensor.url) {
            return Err("duplicate configured sensor identity or URL".into());
        }
    }
    let mut records = Vec::new();
    // Validate all responses before opening the evidence journal.
    for sensor in &sensors {
        let observations = sensor
            .observations()
            .map_err(|e| format!("{}: {e}", sensor.sensor_id))?;
        let collected_at_unix_ms = now_unix_ms()?;
        for observation in observations {
            records.push(EvidenceRecord {
                observation,
                collection: CollectionProvenance {
                    sensor_url: sensor.url.clone(),
                    collected_at_unix_ms,
                },
            });
        }
    }
    let added = record_evidence(&output, &records)?;
    for record in &records {
        let o = &record.observation;
        println!(
            "{} | {} ms | {}:{} -> {}:{} | {} {} | {} [{}] | sensor={} via {}",
            o.observation_id,
            o.timestamp_unix_ms,
            o.source_endpoint.address,
            o.source_endpoint.port,
            o.destination_endpoint.address,
            o.destination_endpoint.port,
            o.event.method,
            o.event.path,
            o.event.operation,
            o.event.message_id,
            o.sensor_id,
            record.collection.sensor_url
        );
    }
    println!(
        "Retrieved {} observations; recorded {added} new records in {}. No attribution performed.",
        records.len(),
        output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("hound: {error}");
        std::process::exit(1);
    }
}
