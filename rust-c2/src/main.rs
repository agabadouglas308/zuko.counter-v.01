use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};

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

fn parse_observation(line: &str) -> Result<Observation, serde_json::Error> {
    let observation: Observation = serde_json::from_str(line)?;
    if observation.kind != "observation" {
        return Err(<serde_json::Error as serde::de::Error>::custom(
            "event kind is not observation",
        ));
    }
    Ok(observation)
}

fn update_track(track: &mut Track, obs: &Observation) {
    // Confidence-weighted smoothing; this is a simulator, not a fire-control tracker.
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
    // Synthetic protected zone: x/y in [-50, 50], altitude below 120 m.
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

fn main() {
    let stdin = io::stdin();
    let mut tracks: HashMap<String, Track> = HashMap::new();
    println!(r#"{{"kind":"status","message":"rust-c2-ready"}}"#);

    for line in stdin.lock().lines().map_while(Result::ok) {
        let Ok(obs) = parse_observation(&line) else {
            eprintln!("ignored malformed or non-observation event");
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
        println!(
            r#"{{"kind":"track_update","target_id":"{}","source_sensor":"{}","timestamp_ms":{},"x_m":{:.2},"y_m":{:.2},"z_m":{:.2},"confidence":{:.2},"sensor_count":{},"recommendation":"{}"}}"#,
            entry.target_id,
            obs.sensor,
            entry.last_timestamp_ms,
            entry.position.x,
            entry.position.y,
            entry.position.z,
            entry.confidence,
            entry.sensors.len(),
            action
        );
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

    #[test]
    fn malformed_json_is_rejected() {
        assert!(parse_observation("{not-json}").is_err());
    }

    #[test]
    fn non_observation_event_is_rejected() {
        let line = r#"{"kind":"status","message":"ready"}"#;
        assert!(parse_observation(line).is_err());
    }

    #[test]
    fn confidence_is_clamped_during_update() {
        let mut track = track_at(0.0, 0.0, 50.0, 0.5);
        let obs = observation(
            r#"{"kind":"observation","sensor":"radar","target_id":"test","timestamp_ms":1,"position":{"x_m":0.0,"y_m":0.0,"z_m":50.0},"velocity":{"x_mps":0.0,"y_mps":0.0,"z_mps":0.0},"confidence":4.0}"#,
        );
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
