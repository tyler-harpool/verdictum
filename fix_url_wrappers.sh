#!/bin/bash

# Fix all URL wrapper handlers to use proper header handling

for file in src/handlers/opinion_url.rs src/handlers/sentencing_url.rs src/handlers/order_url.rs src/handlers/docket_url.rs; do
    echo "Fixing $file..."

    # Replace the simple header.set() approach with the working Request::builder() approach
    sed -i '' '
/fn add_district_header/,/^}/ {
    s/let headers = req\.headers();/let method = req.method();\
    let uri = req.uri();\
\
    \/\/ Create a new request with the district header\
    let headers = spin_sdk::http::Headers::new();\
\
    \/\/ Copy existing headers\
    for (name, value) in req.headers() {\
        let _ = headers.append(\&name.to_string(), \&value.as_bytes().to_vec());\
    }/

    s/headers\.set(&"X-Court-District"\.to_string(), &district\.as_bytes()\.to_vec());/\/\/ Add the district header\
    let _ = headers.set(\&"x-court-district".to_string(), \&vec![district.as_bytes().to_vec()]);\
\
    let body = req.into_body();\
    let new_req = Request::builder()\
        .method(method)\
        .uri(\&uri)\
        .headers(headers)\
        .body(body)\
        .build();/

    s/Ok(req)/Ok(new_req)/
}' "$file"

    # Also add IntoResponse to imports if not present
    if ! grep -q "IntoResponse" "$file"; then
        sed -i '' 's/use spin_sdk::http::{Params, Request, Response};/use spin_sdk::http::{IntoResponse, Params, Request, Response};/' "$file"
    fi

    # Fix handler calls to use .map(|r| r.into_response()) for ApiResult returns
    sed -i '' 's/=> crate::handlers::[^:]*::\([^(]*\)(\([^)]*\))$/=> crate::handlers::\1(\2)\
            .map(|r| r.into_response())/g' "$file"

    # Clean up any double .map() calls that might have been created
    sed -i '' ':a; /\.map(|r| r\.into_response())$/ { N; s/\.map(|r| r\.into_response())\n[[:space:]]*\.map(|r| r\.into_response())/\.map(|r| r.into_response())/; ba; }' "$file"
done

echo "All files fixed!"