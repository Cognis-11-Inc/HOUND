use hound_core::{
    memory::{CollectionProvenance, EvidenceRecord, record_evidence},
    sensors::{HttpSensor, Observation, Sensor},
    truth::{Hypothesis, HypothesisStatus},
};
use serde_json::json;
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::TcpListener;

fn observation() -> Observation {
    serde_json::from_str(include_str!("../../../tests/fixtures/observation.json")).unwrap()
}

fn sensor() -> HttpSensor {
    HttpSensor {
        sensor_id: "node-c".into(),
        url: "http://node-c:8080/observations".into(),
    }
}

fn batch(obs: &[Observation]) -> Vec<u8> {
    serde_json::to_vec(&json!({"schema_version": 1, "observations": obs})).unwrap()
}

#[test]
fn parses_and_preserves_provenance() {
    let original = observation();
    let decoded = sensor()
        .parse(&batch(std::slice::from_ref(&original)))
        .unwrap();
    assert_eq!(decoded, vec![original]);
}

#[test]
fn rejects_invalid_measurements() {
    let mut cases = Vec::new();
    let mut o = observation();
    o.timestamp_unix_ms = 0;
    cases.push(o);
    let mut o = observation();
    o.source_endpoint.port = 0;
    cases.push(o);
    let mut o = observation();
    o.provenance.capture_method = "model_prediction".into();
    cases.push(o);
    let mut o = observation();
    o.observation_id = "rewritten-id".into();
    cases.push(o);
    let mut o = observation();
    o.event.path = "/ground-truth".into();
    cases.push(o);
    let mut o = observation();
    o.event.operation = "intruder".into();
    cases.push(o);
    let mut o = observation();
    o.event.payload_bytes = 4097;
    cases.push(o);
    for o in cases {
        assert!(sensor().parse(&batch(&[o])).is_err());
    }
    let mut value = serde_json::to_value(observation()).unwrap();
    value["source_endpoint"]["address"] = json!("invented-host");
    assert!(serde_json::from_value::<Observation>(value).is_err());
}

#[test]
fn rejects_hidden_fields_wrong_versions_sensors_and_duplicates() {
    let mut value = json!({"schema_version": 1, "observations": [observation()]});
    value["observations"][0]["intrusion_origin"] = json!("node-a");
    assert!(
        sensor()
            .parse(&serde_json::to_vec(&value).unwrap())
            .is_err()
    );
    value = json!({"schema_version": 2, "observations": []});
    assert!(
        sensor()
            .parse(&serde_json::to_vec(&value).unwrap())
            .is_err()
    );
    let wrong = HttpSensor {
        sensor_id: "node-d".into(),
        ..sensor()
    };
    assert!(wrong.parse(&batch(&[observation()])).is_err());
    assert!(
        sensor()
            .parse(&batch(&[observation(), observation()]))
            .is_err()
    );
}

#[test]
fn restricts_sensor_urls() {
    for url in [
        "https://node-c:8080/observations",
        "http://node-c:8080/ground-truth",
        "http://node-c:8080/observations?file=secret",
        "http://user@node-c:8080/observations",
        "http://node-c:0/observations",
        "http://node-c\r\nInjected:8080/observations",
    ] {
        assert!(
            HttpSensor {
                url: url.into(),
                ..sensor()
            }
            .validate()
            .is_err()
        );
    }
}

#[test]
fn journal_preserves_original_deduplicates_and_rejects_rewrites() {
    let path = std::env::temp_dir().join(format!(
        "hound-evidence-{}-{}.jsonl",
        std::process::id(),
        hound_core::memory::now_unix_ms().unwrap()
    ));
    let original = EvidenceRecord {
        observation: observation(),
        collection: CollectionProvenance {
            sensor_url: sensor().url,
            collected_at_unix_ms: 1792771201000,
        },
    };
    assert_eq!(
        record_evidence(&path, std::slice::from_ref(&original)).unwrap(),
        1
    );
    let contents = std::fs::read(&path).unwrap();
    let decoded: EvidenceRecord = serde_json::from_slice(&contents).unwrap();
    assert_eq!(original, decoded);
    let mut recollected = original.clone();
    recollected.collection.collected_at_unix_ms += 1;
    assert_eq!(record_evidence(&path, &[recollected]).unwrap(), 0);
    let mut rewritten = original.clone();
    rewritten.observation.event.operation = "heartbeat".into();
    assert!(record_evidence(&path, &[rewritten]).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), contents);
    std::fs::write(&path, &contents[..contents.len() - 1]).unwrap();
    assert!(record_evidence(&path, &[]).is_err());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn hypotheses_remain_separate_and_references_must_exist() {
    let id = observation().observation_id;
    let known = HashSet::from([id.clone()]);
    let mut h = Hypothesis {
        hypothesis_id: "hypothesis-1".into(),
        suspected_origin: Some("node-a".into()),
        supporting_observations: vec![id.clone()],
        contradicting_observations: vec![],
        status: HypothesisStatus::Supported,
    };
    h.validate(&known).unwrap();
    assert!(serde_json::from_value::<Observation>(serde_json::to_value(&h).unwrap()).is_err());
    assert!(
        serde_json::from_value::<Hypothesis>(serde_json::to_value(observation()).unwrap()).is_err()
    );
    h.contradicting_observations.push(id);
    assert!(h.validate(&known).is_err());
    h.contradicting_observations = vec!["unknown".into()];
    assert!(h.validate(&known).is_err());
    h.supporting_observations.clear();
    h.contradicting_observations.clear();
    h.suspected_origin = None;
    h.status = HypothesisStatus::Unresolved;
    h.validate(&known).unwrap();
}

fn http_response(response: Vec<u8>) -> hound_core::Result<Vec<Observation>> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
            assert!(request.len() < 4096);
        }
        assert!(request.starts_with(b"GET /observations HTTP/1.0\r\n"));
        stream.write_all(&response).unwrap();
    });
    let result = HttpSensor {
        url: format!("http://{addr}/observations"),
        ..sensor()
    }
    .observations();
    server.join().unwrap();
    result
}

#[test]
fn reads_actual_http_and_rejects_redirects_and_bad_framing() {
    let body = batch(&[observation()]);
    let header = format!(
        "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    assert_eq!(
        http_response([header.as_bytes(), &body].concat()).unwrap(),
        vec![observation()]
    );
    for response in [
        "HTTP/1.0 302 Found\r\nLocation: http://other/ground-truth\r\n\r\n",
        "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\nContent-Length: 999\r\n\r\n{}",
        "HTTP/1.0 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
    ] {
        assert!(http_response(response.as_bytes().to_vec()).is_err());
    }
}
