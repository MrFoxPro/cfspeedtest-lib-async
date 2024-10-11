use crate::speedtest::TestType;
use serde::Serialize;
use std::fmt::Display;

#[derive(Serialize)]
struct StatMeasurement {
    test_type: TestType,
    payload_size: usize,
    min: f64,
    q1: f64,
    median: f64,
    q3: f64,
    max: f64,
    avg: f64,
}

#[derive(Serialize)]
pub struct Measurement {
    pub test_type: TestType,
    pub payload_size: usize,
    pub mbit: f64,
}

impl Display for Measurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}: \t{}\t-> {}",
            self.test_type,
            format_bytes(self.payload_size),
            self.mbit,
        )
    }
}

fn calc_stats(mbit_measurements: Vec<f64>) -> Option<(f64, f64, f64, f64, f64, f64)> {
    let length = mbit_measurements.len();
    if length < 4 {
        return None;
    }

    let mut sorted_data = mbit_measurements.clone();
    sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less));

    let q1 = if length % 2 == 0 {
        median(&sorted_data[0..length / 2])
    } else {
        median(&sorted_data[0..(length + 1) / 2])
    };

    let q3 = if length % 2 == 0 {
        median(&sorted_data[length / 2..length])
    } else {
        median(&sorted_data[(length + 1) / 2..length])
    };

    Some((
        *sorted_data.first().unwrap(),
        q1,
        median(&sorted_data),
        q3,
        *sorted_data.last().unwrap(),
        mbit_measurements.iter().sum::<f64>() / mbit_measurements.len() as f64,
    ))
}

fn median(data: &[f64]) -> f64 {
    let length = data.len();
    if length % 2 == 0 {
        (data[length / 2 - 1] + data[length / 2]) / 2.0
    } else {
        data[length / 2]
    }
}

pub(crate) fn format_bytes(bytes: usize) -> String {
    match bytes {
        1_000..=999_999 => format!("{}KB", bytes / 1_000),
        1_000_000..=999_999_999 => format!("{}MB", bytes / 1_000_000),
        _ => format!("{bytes} bytes"),
    }
}
