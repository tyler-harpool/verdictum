//! Query string parsing utilities for Spin HTTP handlers

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Parse a query string into key-value pairs
pub fn parse_query_string(query: &str) -> Vec<(&str, &str)> {
    if query.is_empty() {
        return Vec::new();
    }

    query.split('&')
        .filter_map(|pair| {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() == 2 && !parts[1].is_empty() {
                Some((parts[0], parts[1]))
            } else {
                None
            }
        })
        .collect()
}

/// Get a string value from parsed query parameters
pub fn get_string<'a>(params: &'a [(&'a str, &'a str)], key: &str) -> Option<String> {
    params.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| (*v).to_string())
}

/// Get a UUID value from parsed query parameters
pub fn get_uuid(params: &[(&str, &str)], key: &str) -> Option<Uuid> {
    params.iter()
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| Uuid::parse_str(v).ok())
}

/// Get a usize value from parsed query parameters
pub fn get_usize(params: &[(&str, &str)], key: &str) -> Option<usize> {
    params.iter()
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| v.parse().ok())
}

/// Get an i64 value from parsed query parameters
pub fn _get_i64(params: &[(&str, &str)], key: &str) -> Option<i64> {
    params.iter()
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| v.parse().ok())
}

/// Get an i64 value with a default
pub fn _get_i64_or(params: &[(&str, &str)], key: &str, default: i64) -> i64 {
    _get_i64(params, key).unwrap_or(default)
}

/// Get a bool value from parsed query parameters
pub fn get_bool(params: &[(&str, &str)], key: &str) -> Option<bool> {
    params.iter()
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| v.parse().ok())
}

/// Get a DateTime<Utc> value from parsed query parameters (RFC3339 format)
pub fn get_datetime(params: &[(&str, &str)], key: &str) -> Option<DateTime<Utc>> {
    params.iter()
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| DateTime::parse_from_rfc3339(v).ok())
        .map(|d| d.with_timezone(&Utc))
}

/// Parse a JSON value from query parameters
pub fn get_json<T: serde::de::DeserializeOwned>(params: &[(&str, &str)], key: &str) -> Option<T> {
    params.iter()
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| {
            // Try to parse as quoted JSON string first
            serde_json::from_str(&format!("\"{}\"", v)).ok()
        })
}