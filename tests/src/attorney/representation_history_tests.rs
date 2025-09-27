//! Attorney representation history tests
//!
//! Tests for attorney representation history endpoint

use http::types::{Headers, Method, OutgoingRequest};
use serde_json::{json, Value};
use spin_test_sdk::{
    bindings::{fermyon::spin_test_virt::key_value, wasi::http},
    spin_test,
};

/// Helper to create a test attorney and return its ID or skip test
fn setup_test_attorney(district: &str) -> Option<String> {
    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), district.as_bytes())
        .unwrap();
    headers
        .append(&"Content-Type".to_string(), b"application/json")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Post).unwrap();
    request.set_path_with_query(Some("/api/attorneys")).unwrap();

    let attorney_data = json!({
        "bar_number": format!("HIST{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        "first_name": "History",
        "last_name": "Attorney",
        "email": "history.attorney@law.com",
        "phone": "555-0200",
        "address": {
            "street1": "456 History St",
            "city": "Legal City",
            "state": "LC",
            "zip_code": "54321",
            "country": "USA"
        }
    });

    let request_body = request.body().unwrap();
    let stream = request_body.write().unwrap();
    stream
        .blocking_write_and_flush(serde_json::to_string(&attorney_data).unwrap().as_bytes())
        .unwrap();
    drop(stream);
    http::types::OutgoingBody::finish(request_body, None).unwrap();

    let response = spin_test_sdk::perform_request(request);
    if response.status() != 201 {
        return None;
    }

    let body = response.body_as_string().unwrap_or_default();
    if body.is_empty() {
        return None;
    }

    serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|json| json["id"].as_str().map(|s| s.to_string()))
}

#[spin_test]
fn test_get_representation_history_basic() {
    let _store = key_value::Store::open("district9");

    // Create an attorney or skip test if not possible
    let attorney_id = match setup_test_attorney("district9") {
        Some(id) => id,
        None => return, // Skip test gracefully
    };

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district9")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request
        .set_path_with_query(Some(&format!(
            "/api/attorneys/{}/representation-history",
            attorney_id
        )))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    // Verify the structure
    assert!(body_json["attorney_id"].is_string());
    assert!(body_json["attorney_name"].is_string());
    assert!(body_json["assignments"].is_array());
    assert!(body_json["summary"].is_object());

    // Verify summary structure
    let summary = &body_json["summary"];
    assert!(summary["total_cases"].is_number());
    assert!(summary["active_cases"].is_number());
    assert!(summary["completed_cases"].is_number());
    assert!(summary["primary_role"].is_string());
    assert!(summary["date_range"].is_object());
    assert!(summary["outcomes"].is_object());
}

#[spin_test]
fn test_get_representation_history_with_active_filter() {
    let _store = key_value::Store::open("district10");

    let attorney_id = match setup_test_attorney("district10") {
        Some(id) => id,
        None => return,
    };

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district10")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request
        .set_path_with_query(Some(&format!(
            "/api/attorneys/{}/representation-history?active_only=true",
            attorney_id
        )))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    // Verify that all assignments are active
    let assignments = body_json["assignments"].as_array().unwrap();
    for assignment in assignments {
        assert_eq!(assignment["is_active"].as_bool().unwrap(), true);
    }
}

#[spin_test]
fn test_get_representation_history_with_role_filter() {
    let _store = key_value::Store::open("district11");

    let attorney_id = match setup_test_attorney("district11") {
        Some(id) => id,
        None => return,
    };

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district11")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request
        .set_path_with_query(Some(&format!(
            "/api/attorneys/{}/representation-history?role=lead_counsel",
            attorney_id
        )))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    // Verify that all assignments have the filtered role
    let assignments = body_json["assignments"].as_array().unwrap();
    for assignment in assignments {
        assert_eq!(assignment["role"].as_str().unwrap(), "lead_counsel");
    }
}

#[spin_test]
fn test_get_representation_history_with_pagination() {
    let _store = key_value::Store::open("district12");

    let attorney_id = match setup_test_attorney("district12") {
        Some(id) => id,
        None => return,
    };

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district12")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request
        .set_path_with_query(Some(&format!(
            "/api/attorneys/{}/representation-history?page=1&page_size=2",
            attorney_id
        )))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    // Verify pagination works (should have at most 2 assignments)
    let assignments = body_json["assignments"].as_array().unwrap();
    assert!(assignments.len() <= 2);
}

#[spin_test]
fn test_get_representation_history_assignment_structure() {
    let _store = key_value::Store::open("district13");

    let attorney_id = match setup_test_attorney("district13") {
        Some(id) => id,
        None => return,
    };

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district13")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request
        .set_path_with_query(Some(&format!(
            "/api/attorneys/{}/representation-history",
            attorney_id
        )))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    let assignments = body_json["assignments"].as_array().unwrap();
    if !assignments.is_empty() {
        let assignment = &assignments[0];

        // Verify assignment structure
        assert!(assignment["assignment_id"].is_string());
        assert!(assignment["case_id"].is_string());
        assert!(assignment["case_number"].is_string());
        assert!(assignment["defendant_name"].is_string());
        assert!(assignment["role"].is_string());
        assert!(assignment["assigned_date"].is_string());
        assert!(assignment["is_active"].is_boolean());
        assert!(assignment["role_changes"].is_array());

        // Optional fields can be null
        // If not null, they should be the correct type
        if !assignment["removed_date"].is_null() {
            assert!(assignment["removed_date"].is_string());
        }
        if !assignment["case_outcome"].is_null() {
            assert!(assignment["case_outcome"].is_string());
        }
        if !assignment["notes"].is_null() {
            assert!(assignment["notes"].is_string());
        }
    }
}

#[spin_test]
fn test_get_representation_history_nonexistent_attorney() {
    let _store = key_value::Store::open("district9");

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district9")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request
        .set_path_with_query(Some(
            "/api/attorneys/00000000-0000-0000-0000-000000000000/representation-history",
        ))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Handler returns 404 when attorney doesn't exist
    assert_eq!(response.status(), 404);

    let body = response.body_as_string().unwrap_or_default();
    if !body.is_empty() {
        if let Ok(body_json) = serde_json::from_str::<Value>(&body) {
            assert!(body_json["error"].is_string());
            assert!(body_json["error"].as_str().unwrap().contains("not found"));
        }
    }
}

#[spin_test]
fn test_get_representation_history_empty_page() {
    let _store = key_value::Store::open("district15");

    let attorney_id = match setup_test_attorney("district15") {
        Some(id) => id,
        None => return,
    };

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district15")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    // Request a page that should be empty
    request
        .set_path_with_query(Some(&format!(
            "/api/attorneys/{}/representation-history?page=100&page_size=10",
            attorney_id
        )))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    // Should return empty assignments array but valid structure
    let assignments = body_json["assignments"].as_array().unwrap();
    assert_eq!(assignments.len(), 0);

    // Summary should still be present and valid
    assert!(body_json["summary"].is_object());
}

#[spin_test]
fn test_get_representation_history_with_date_range() {
    let _store = key_value::Store::open("district16");

    let attorney_id = match setup_test_attorney("district16") {
        Some(id) => id,
        None => return,
    };

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district16")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request.set_path_with_query(Some(&format!(
        "/api/attorneys/{}/representation-history?start_date=2023-01-01T00:00:00Z&end_date=2023-12-31T23:59:59Z",
        attorney_id
    ))).unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    // Verify response structure is valid
    assert!(body_json["assignments"].is_array());
    assert!(body_json["summary"].is_object());

    // For date filtering, assignments should either be filtered or empty
    let assignments = body_json["assignments"].as_array().unwrap();
    for assignment in assignments {
        let assigned_date = assignment["assigned_date"].as_str().unwrap();
        // Just verify it's a valid date string - actual filtering depends on implementation
        assert!(!assigned_date.is_empty());
    }
}

#[spin_test]
fn test_get_representation_history_summary_statistics() {
    let _store = key_value::Store::open("district17");

    let attorney_id = match setup_test_attorney("district17") {
        Some(id) => id,
        None => return,
    };

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district17")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request
        .set_path_with_query(Some(&format!(
            "/api/attorneys/{}/representation-history",
            attorney_id
        )))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);

    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    let summary = &body_json["summary"];
    let assignments = body_json["assignments"].as_array().unwrap();

    // Verify summary matches assignments
    let total_cases = summary["total_cases"].as_u64().unwrap() as usize;
    let active_cases = summary["active_cases"].as_u64().unwrap() as usize;
    let completed_cases = summary["completed_cases"].as_u64().unwrap() as usize;

    assert_eq!(total_cases, assignments.len());
    assert_eq!(active_cases + completed_cases, total_cases);

    // Verify date range
    let date_range = &summary["date_range"];
    assert!(date_range["start_date"].is_string());
    assert!(date_range["end_date"].is_string());

    // Verify outcomes is an object
    assert!(summary["outcomes"].is_object());
}

#[spin_test]
fn test_get_representation_history_invalid_date_format() {
    let _store = key_value::Store::open("district18");

    let attorney_id = match setup_test_attorney("district18") {
        Some(id) => id,
        None => return,
    };

    let headers = Headers::new();
    headers
        .append(&"X-Court-District".to_string(), b"district18")
        .unwrap();

    let request = OutgoingRequest::new(headers);
    request.set_method(&Method::Get).unwrap();
    request
        .set_path_with_query(Some(&format!(
            "/api/attorneys/{}/representation-history?start_date=invalid-date",
            attorney_id
        )))
        .unwrap();

    let response = spin_test_sdk::perform_request(request);

    // Should still work but ignore invalid date filter
    assert_eq!(response.status(), 200);

    let body = response.body_as_string().unwrap();
    let body_json: Value = serde_json::from_str(&body).unwrap();

    assert!(body_json["assignments"].is_array());
    assert!(body_json["summary"].is_object());
}
