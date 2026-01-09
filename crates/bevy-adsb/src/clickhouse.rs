use std::collections::{BinaryHeap, HashSet};
use std::time::Instant;
use std::{collections::HashMap, io::BufRead};

use bevy::log::info;
use bevy::prelude::*;
use chrono::{DateTime, NaiveDateTime, Utc};
use ehttp::Request;
use url::Url;

use crate::math::{Coordinate, Degrees};
use crate::state::{Aircraft, AircraftState};

const CLICKHOUSE_URL: &str = "http://localhost:18123";
const CLICKHOUSE_USER: &str = "default";
const CLICKHOUSE_PASSWORD: &str = "not-secret"; // Use this only locally
const CLICKHOUSE_DEBUG_RESPONSE_PATH: &str = "clickhouse/debug.csv";

#[derive(Resource, Clone, Default)]
pub struct DataFetch {
    pub create: Vec<Aircraft>,
    pub update: HashMap<String, BinaryHeap<AircraftState>>,
    pub icaos: HashSet<String>,
}

fn get_url(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Url {
    let mut url = Url::parse(CLICKHOUSE_URL).unwrap();

    {
        let mut pairs = url.query_pairs_mut();
        pairs.clear();
        pairs.append_pair("default_format", "CSV");
        pairs.append_pair("user", CLICKHOUSE_USER);
        pairs.append_pair("password", CLICKHOUSE_PASSWORD);
        pairs.append_pair("param_limit", 10_000.to_string().as_str());
        pairs.append_pair(
            "param_start_time",
            start_time.timestamp().to_string().as_str(),
        );
        pairs.append_pair("param_end_time", end_time.timestamp().to_string().as_str());
    }

    url
}

pub async fn get_planes(
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    existing_icaos: HashSet<String>,
) -> DataFetch {
    let start = Instant::now();
    let keys = ["icao", "time", "lat", "lon", "t", "r", "track_degrees"];

    let request = Request::post(
        get_url(start_time, end_time).as_str(),
        format!(
            "
            SELECT {}
            FROM planes_mercator
            WHERE time > {{start_time:DateTime64}}
            AND time < {{end_time:DateTime64}}
            AND altitude > 2000
            ORDER BY time
            LIMIT {{limit:UInt32}}
            ",
            keys.join(", ")
        )
        .as_bytes()
        .into(),
    );

    let response = ehttp::fetch_async(request)
        .await
        .expect("Failed to query clickhouse, is clickhouse running?");
    let mut lines = std::io::Cursor::new(&response.bytes).lines();

    let mut create = Vec::<Aircraft>::new();
    let mut update = HashMap::<String, BinaryHeap<AircraftState>>::new();
    let mut icaos = existing_icaos.clone();

    while let Some(Ok(line)) = lines.next() {
        let els = line.split(',').collect::<Vec<&str>>();

        if els.len() != keys.len() {
            let mut file_out = std::fs::File::create(CLICKHOUSE_DEBUG_RESPONSE_PATH).unwrap();
            std::io::copy(&mut std::io::Cursor::new(response.bytes), &mut file_out).unwrap();
            panic!(
                "Unexpected data from clickhouse: `{}`. CSV response data written to `{}`",
                line, CLICKHOUSE_DEBUG_RESPONSE_PATH
            );
        }
        let icao: String = els[0].trim_matches('\"').into();

        let data = AircraftState {
            coordinate: Coordinate {
                latitude: Degrees(els[2].parse().unwrap()),
                longitude: Degrees(els[3].parse().unwrap()),
            },
            heading: Degrees(els[6].parse().unwrap()),
            altitude: 0.0,
            ground_speed: 0.0,
            timestamp: NaiveDateTime::parse_from_str(els[1], "\"%Y-%m-%d %H:%M:%S%.3f\"")
                .unwrap_or_else(|err| panic!("Could not parse time `{}`, error: `{}`", els[1], err))
                .and_utc(),
        };

        if icaos.contains(&icao) {
            (*update.entry(icao.clone()).or_default()).push(data);
        } else {
            create.push(Aircraft {
                icao: icao.clone(),
                buffer: vec![data].into(),
                last_state: data,
                last_update: data.timestamp,
            });
        }
        icaos.insert(icao);
    }

    info!(
        "Fetching data took {}ms. Current buffer time: {}. Create: {}. Update: {}",
        start.elapsed().as_millis(),
        start_time.format("%H:%M:%S"),
        create.len(),
        update.len()
    );

    DataFetch {
        update,
        create,
        icaos,
    }
}
