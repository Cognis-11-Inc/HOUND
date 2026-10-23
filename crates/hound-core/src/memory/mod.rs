//! Append-only JSONL collection journal. Conflicting evidence is an error.
use crate::{
    Result,
    sensors::{HttpSensor, Observation},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix_ms() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProvenance {
    pub sensor_url: String,
    pub collected_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    pub observation: Observation,
    pub collection: CollectionProvenance,
}

impl EvidenceRecord {
    pub fn validate(&self) -> Result<()> {
        self.observation.validate()?;
        HttpSensor {
            sensor_id: self.observation.sensor_id.clone(),
            url: self.collection.sensor_url.clone(),
        }
        .validate()?;
        if self.collection.collected_at_unix_ms == 0 {
            return Err("missing collection timestamp".into());
        }
        Ok(())
    }
}

/// Validate batch and journal before appending. Single writer; not tamper-proof.
pub fn record_evidence(path: &Path, records: &[EvidenceRecord]) -> Result<usize> {
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(path)?;
    let mut known = HashMap::new();
    for line in BufReader::new(&file).split(b'\n') {
        let record: EvidenceRecord = serde_json::from_slice(&line?)?;
        record.validate()?;
        if known
            .insert(
                record.observation.observation_id.clone(),
                record.observation,
            )
            .is_some()
        {
            return Err("duplicate identity in journal".into());
        }
    }
    if file.metadata()?.len() > 0 {
        file.seek(SeekFrom::End(-1))?;
        let mut last = [0];
        file.read_exact(&mut last)?;
        if last[0] != b'\n' {
            return Err("journal has an incomplete final line".into());
        }
    }
    let mut pending = Vec::new();
    for record in records {
        record.validate()?;
        match known.get(&record.observation.observation_id) {
            Some(previous) if previous != &record.observation => {
                return Err("conflicting observation identity; refusing evidence rewrite".into());
            }
            Some(_) => (),
            None => {
                known.insert(
                    record.observation.observation_id.clone(),
                    record.observation.clone(),
                );
                pending.push(record);
            }
        }
    }
    for record in &pending {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    file.sync_all()?;
    Ok(pending.len())
}
