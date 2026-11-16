use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::PathBuf;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
struct Vec3 {
    #[serde(rename = "x_m", alias = "x_mps")]
    x: f64,
    #[serde(rename = "y_m", alias = "y_mps")]
    y: f64,
    #[serde(rename = "z_m", alias = "z_mps")]
    z: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct Observation {
    kind: String,
    schema_version: u32,
    event_id: String,
    sensor: String,
    target_id: String,
    timestamp_ms: i64,
    position: Vec3,
    velocity: Vec3,
    confidence: f64,
}

#[derive(Debug, Default)]
struct Track {
    target_id: String,
    position: Vec3,
    velocity: Vec3,
    confidence: f64,
    sensors: HashSet<String>,
    last_timestamp_ms: i64,
}

struct AuditLog {
    writer: Option<BufWriter<File>>,
    sequence: u64,
}

impl AuditLog {
    fn disabled() -> Self {
        Self {
            writer: None,
            sequence: 0,
        }
    }

    fn open(path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            writer: Some(BufWriter::new(file)),
            sequence: 0,
        })
    }

    fn record_track_update(
        &mut self,
        track: &Track,
        source_event_id: &str,
        source_sensor: &str,
        action: &str,
    ) -> io::Result<String> {
        self.sequence += 1;
        let audit_id = format!("audit-{}", self.sequence);
        let record = json!({
            "kind": "audit_track_update",
            "schema_version": SCHEMA_VERSION,
            "event_id": audit_id,
            "source_event_id": source_event_id,
            "target_id": track.target_id,
            "source_sensor": source_sensor,
            "timestamp_ms": track.last_timestamp_ms,
            "position": { "x_m": track.position.x, "y_m": track.position.y, "z_m": track.position.z },
            "confidence": track.confidence,
            "sensor_count": track.sensors.len(),
            "recommendation": action,
        });
        if let Some(writer) = self.writer.as_mut() {
            serde_json::to_writer(&mut *writer, &record)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
        Ok(audit_id)
    }
}

fn parse_observation(line: &str) -> Result<Observation, String> {
    let observation: Observation = serde_json::from_str(line).map_err(|err| err.to_string())?;
    if observation.kind != "observation" {
        return Err("event kind is not observation".into());
    }
    if observation.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "unsupported schema version {}",
            observation.schema_version
        ));
    }
    if observation.event_id.is_empty()
        || observation.sensor.is_empty()
        || observation.target_id.is_empty()
    {
        return Err("event_id, sensor, and target_id must not be empty".into());
    }
    if observation.timestamp_ms < 0 {
        return Err("timestamp_ms must not be negative".into());
    }
    Ok(observation)
}

fn update_track(track: &mut Track, obs: &Observation) {
    let confidence = obs.confidence.clamp(0.0, 1.0);
    let alpha = (0.25 + confidence * 0.35).clamp(0.25, 0.60);
    track.position.x = alpha * obs.position.x + (1.0 - alpha) * track.position.x;
    track.position.y = alpha * obs.position.y + (1.0 - alpha) * track.position.y;
    track.position.z = alpha * obs.position.z + (1.0 - alpha) * track.position.z;
    track.velocity.x = alpha * obs.velocity.x + (1.0 - alpha) * track.velocity.x;
    track.velocity.y = alpha * obs.velocity.y + (1.0 - alpha) * track.velocity.y;
    track.velocity.z = alpha * obs.velocity.z + (1.0 - alpha) * track.velocity.z;
    track.confidence = (track.confidence * 0.55 + confidence * 0.45).clamp(0.0, 1.0);
    track.sensors.insert(obs.sensor.clone());
    track.last_timestamp_ms = obs.timestamp_ms;
}

fn recommendation(track: &Track) -> &'static str {
    let inside = track.position.x.abs() <= 50.0
        && track.position.y.abs() <= 50.0
        && track.position.z <= 120.0;
    if inside && track.confidence >= 0.70 {
        "request_operator_review"
    } else if inside {
        "continue_observation"
    } else {
        "notify_site_security"
    }
}

fn audit_path() -> Result<Option<PathBuf>, String> {
    let mut args = env::args().skip(1);
    let mut result = None;
    while let Some(arg) = args.next() {
        if arg == "--audit-log" {
            let path = args.next().ok_or("--audit-log requires a file path")?;
            result = Some(PathBuf::from(path));
        } else if arg == "--help" || arg == "-h" {
            println!("Usage: rust-c2 [--audit-log PATH]");
            std::process::exit(0);
        } else {
            return Err(format!("unknown argument: {arg}"));
        }
    }
    Ok(result)
}

fn run() -> Result<(), String> {
    let audit_path = audit_path()?;
    let mut audit = match audit_path {
        Some(path) => {
            AuditLog::open(path).map_err(|err| format!("cannot open audit log: {err}"))?
        }
        None => AuditLog::disabled(),
    };
    let stdin = io::stdin();
    let mut tracks: HashMap<String, Track> = HashMap::new();
    println!(
        "{}",
        json!({"kind": "status", "schema_version": SCHEMA_VERSION, "message": "rust-c2-ready"})
    );

    for line in stdin.lock().lines().map_while(Result::ok) {
        let Ok(obs) = parse_observation(&line) else {
            eprintln!("ignored malformed or unsupported event");
            continue;
        };
        let entry = tracks
            .entry(obs.target_id.clone())
            .or_insert_with(|| Track {
                target_id: obs.target_id.clone(),
                position: obs.position,
                velocity: obs.velocity,
                confidence: obs.confidence.clamp(0.0, 1.0),
                sensors: HashSet::new(),
                last_timestamp_ms: obs.timestamp_ms,
            });
        update_track(entry, &obs);
        let action = recommendation(entry);
        let audit_id = audit
            .record_track_update(entry, &obs.event_id, &obs.sensor, action)
            .map_err(|err| format!("cannot write audit record: {err}"))?;
        println!(
            "{}",
            json!({
                "kind": "track_update",
                "schema_version": SCHEMA_VERSION,
                "event_id": audit_id,
                "source_event_id": obs.event_id,
                "target_id": entry.target_id,
                "source_sensor": obs.sensor,
                "timestamp_ms": entry.last_timestamp_ms,
                "position": { "x_m": entry.position.x, "y_m": entry.position.y, "z_m": entry.position.z },
                "confidence": entry.confidence,
                "sensor_count": entry.sensors.len(),
                "recommendation": action,
            })
        );
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("fatal: {err}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(json: &str) -> Observation {
        parse_observation(json).expect("valid observation")
    }

    fn track_at(x: f64, y: f64, z: f64, confidence: f64) -> Track {
        Track {
            target_id: "test".into(),
            position: Vec3 { x, y, z },
            velocity: Vec3::default(),
            confidence,
            sensors: HashSet::new(),
            last_timestamp_ms: 0,
        }
    }

    const VALID: &str = r#"{"kind":"observation","schema_version":1,"event_id":"obs-1","sensor":"radar","target_id":"test","timestamp_ms":1,"position":{"x_m":0.0,"y_m":0.0,"z_m":50.0},"velocity":{"x_mps":0.0,"y_mps":0.0,"z_mps":0.0},"confidence":0.8}"#;

    #[test]
    fn malformed_json_is_rejected() {
        assert!(parse_observation("{not-json}").is_err());
    }

    #[test]
    fn non_observation_event_is_rejected() {
        assert!(parse_observation(r#"{"kind":"status"}"#).is_err());
    }

    #[test]
    fn unsupported_schema_is_rejected() {
        assert!(
            parse_observation(&VALID.replace("\"schema_version\":1", "\"schema_version\":2"))
                .is_err()
        );
    }

    #[test]
    fn confidence_is_clamped_during_update() {
        let mut track = track_at(0.0, 0.0, 50.0, 0.5);
        let obs = observation(&VALID.replace("0.8", "4.0"));
        update_track(&mut track, &obs);
        assert!(track.confidence <= 1.0);
        assert_eq!(track.sensors.len(), 1);
    }

    #[test]
    fn geofence_includes_boundary() {
        assert_eq!(
            recommendation(&track_at(50.0, -50.0, 120.0, 0.9)),
            "request_operator_review"
        );
    }

    #[test]
    fn geofence_excludes_outside_position() {
        assert_eq!(
            recommendation(&track_at(50.01, 0.0, 50.0, 0.9)),
            "notify_site_security"
        );
    }

    #[test]
    fn low_confidence_inside_zone_remains_observation_only() {
        assert_eq!(
            recommendation(&track_at(0.0, 0.0, 50.0, 0.69)),
            "continue_observation"
        );
    }
}
