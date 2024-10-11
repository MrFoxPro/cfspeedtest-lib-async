use super::{Measurement, format_bytes};

use reqwest::Client;
use serde::Serialize;
use std::{
    fmt::Display,
    time::{Duration, Instant},
};

const BASE_URL: &str = "https://speed.cloudflare.com";
const DOWNLOAD_URL: &str = "__down?bytes=";
const UPLOAD_URL: &str = "__up";

#[derive(Clone, Copy, Debug, Hash, Serialize, Eq, PartialEq)]
pub enum TestType {
    Download,
    Upload,
}

#[derive(Clone, Debug)]
pub enum PayloadSize {
    K100 = 100_000,
    M1 = 1_000_000,
    M10 = 10_000_000,
    M25 = 25_000_000,
    M100 = 100_000_000,
}

impl Display for PayloadSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format_bytes(self.clone() as usize))
    }
}

impl PayloadSize {
    pub fn from(payload_string: String) -> Result<Self, String> {
        match payload_string.to_lowercase().as_str() {
            "100_000" | "100000" | "100k" | "100kb" => Ok(Self::K100),
            "1_000_000" | "1000000" | "1m" | "1mb" => Ok(Self::M1),
            "10_000_000" | "10000000" | "10m" | "10mb" => Ok(Self::M10),
            "25_000_000" | "25000000" | "25m" | "25mb" => Ok(Self::M25),
            "100_000_000" | "100000000" | "100m" | "100mb" => Ok(Self::M100),
            _ => Err("Value needs to be one of 100k, 1m, 10m, 25m or 100m".to_string()),
        }
    }

    pub fn sizes_from_max(max_payload_size: PayloadSize) -> Vec<usize> {
        let payload_bytes: Vec<usize> =
            vec![100_000, 1_000_000, 10_000_000, 25_000_000, 100_000_000];
        match max_payload_size {
            PayloadSize::K100 => payload_bytes[0..1].to_vec(),
            PayloadSize::M1 => payload_bytes[0..2].to_vec(),
            PayloadSize::M10 => payload_bytes[0..3].to_vec(),
            PayloadSize::M25 => payload_bytes[0..4].to_vec(),
            PayloadSize::M100 => payload_bytes[0..5].to_vec(),
        }
    }
}

pub struct Metadata {
    city: String,
    country: String,
    ip: String,
    asn: String,
    colo: String,
}

impl Display for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "City: {}\nCountry: {}\nIp: {}\nAsn: {}\nColo: {}",
            self.city, self.country, self.ip, self.asn, self.colo
        )
    }
}

pub async fn run_latency_test(
    client: &Client,
    nr_latency_tests: u32,
) -> (Vec<f64>, f64) {
    let mut measurements: Vec<f64> = Vec::new();
    for i in 0..=nr_latency_tests {
        let latency = test_latency(client).await;
        measurements.push(latency);
    }
    let avg_latency = measurements.iter().sum::<f64>() / measurements.len() as f64;
    (measurements, avg_latency)
}

pub async fn test_latency(client: &Client) -> f64 {
    let url = &format!("{}/{}{}", BASE_URL, DOWNLOAD_URL, 0);
    let req_builder = client.get(url);

    let start = Instant::now();
    let response = req_builder.send().await.expect("failed to get response");
    let duration = start.elapsed().as_secs_f64() * 1_000.0;

    let server_timing = response.headers().get("Server-Timing")
        .expect("No Server-Timing in response header")
        .to_str().unwrap();

    let cf_req_duration: f64 = server_timing.split(';').find_map(|part| part.trim_start_matches("dur=").parse().ok()).unwrap_or(0.0);

    let mut req_latency = duration - cf_req_duration;
    if req_latency < 0.0 {
        // TODO investigate negative latency values
        req_latency = 0.0
    }
    req_latency
}

const TIME_THRESHOLD: Duration = Duration::from_secs(5);

pub fn run_tests(
    client: &Client,
    test_fn: fn(&Client, usize) -> f64,
    test_type: TestType,
    payload_sizes: Vec<usize>,
    nr_tests: u32,
    disable_dynamic_max_payload_size: bool,
) -> Vec<Measurement> {
    let mut measurements: Vec<Measurement> = Vec::new();
    for payload_size in payload_sizes {
        let start = Instant::now();
        for i in 0..nr_tests {
            let mbit = test_fn(client, payload_size);
            measurements.push(Measurement {
                test_type,
                payload_size,
                mbit,
            });
        }

        let duration = start.elapsed();

        // only check TIME_THRESHOLD if dynamic max payload sizing is not disabled
        if !disable_dynamic_max_payload_size && duration > TIME_THRESHOLD {
            break;
        }
    }
    measurements
}

pub async fn test_upload(client: &Client, payload_size_bytes: usize) -> f64 {
    let url = &format!("{BASE_URL}/{UPLOAD_URL}");
    let payload: Vec<u8> = vec![1; payload_size_bytes];
    let req_builder = client.post(url).body(payload);

	let start = Instant::now();
	let _ = req_builder.send().await.expect("failed to get response");
	let duration = start.elapsed();

	let mbits = (payload_size_bytes as f64 * 8.0 / 1_000_000.0) / duration.as_secs_f64();
    mbits
}

pub async fn test_download(client: &Client, payload_size_bytes: usize) -> f64 {
    let url = &format!("{BASE_URL}/{DOWNLOAD_URL}{payload_size_bytes}");
    let req_builder = client.get(url);

	let start = Instant::now();
	let _ = req_builder.send().await.expect("failed to get response");
	let duration = start.elapsed();

	let mbits = (payload_size_bytes as f64 * 8.0 / 1_000_000.0) / duration.as_secs_f64();

    mbits
}

pub async fn fetch_metadata(client: &Client) -> Metadata {
    let url = &format!("{}/{}{}", BASE_URL, DOWNLOAD_URL, 0);
    let headers = client.get(url).send().await
        .expect("failed to get response")
        .headers()
        .to_owned();
	
    Metadata {
        city: extract_header_value(&headers, "cf-meta-city", "City N/A"),
        country: extract_header_value(&headers, "cf-meta-country", "Country N/A"),
        ip: extract_header_value(&headers, "cf-meta-ip", "IP N/A"),
        asn: extract_header_value(&headers, "cf-meta-asn", "ASN N/A"),
        colo: extract_header_value(&headers, "cf-meta-colo", "Colo N/A"),
    }
}

fn extract_header_value(headers: &reqwest::header::HeaderMap, header_name: &str, na_value: &str) -> String {
    headers.get(header_name).and_then(|value| value.to_str().ok()).unwrap_or(na_value).to_owned()
}
