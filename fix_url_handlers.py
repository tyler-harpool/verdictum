#!/usr/bin/env python3
import os
import re

def fix_url_handler(filepath):
    """Fix URL handler functions to return Response instead of ApiResult"""

    with open(filepath, 'r') as f:
        content = f.read()

    # Fix the import statements - change ApiResult to Response
    content = re.sub(
        r'use crate::error::ApiResult;',
        'use spin_sdk::http::Response;',
        content
    )

    # Fix function signatures - change ApiResult<impl IntoResponse> to Response
    content = re.sub(
        r'pub fn (\w+)\(req: Request, params: Params\) -> ApiResult<impl IntoResponse> \{',
        r'pub fn \1(req: Request, params: Params) -> Response {',
        content
    )

    # Fix the error handling in add_district_header calls
    # Change: let req = add_district_header(req, &params)?;
    # To: let req = match add_district_header(req, &params) { Ok(r) => r, Err(e) => return crate::utils::json_response::error_response(&e), };
    content = re.sub(
        r'    let req = add_district_header\(req, &params\)\?;',
        r'    let req = match add_district_header(req, &params) {\n        Ok(r) => r,\n        Err(e) => return crate::utils::json_response::error_response(&e),\n    };',
        content
    )

    # Add json_response import if not present
    if 'use crate::utils::json_response' not in content:
        # Add it after the other imports
        content = re.sub(
            r'(use spin_sdk::http::\{[^}]+\};)',
            r'\1\nuse crate::utils::json_response;',
            content
        )

    with open(filepath, 'w') as f:
        f.write(content)

    print(f"Fixed: {filepath}")

# List of URL handler files to fix
url_handlers = [
    'src/handlers/attorney_url.rs',
    'src/handlers/criminal_case_url.rs',
    'src/handlers/deadline_url.rs',
    'src/handlers/docket_url.rs',
    'src/handlers/judge_url.rs',
    'src/handlers/opinion_url.rs',
    'src/handlers/order_url.rs',
    'src/handlers/sentencing_url.rs',
]

for handler in url_handlers:
    if os.path.exists(handler):
        fix_url_handler(handler)
    else:
        print(f"File not found: {handler}")

print("\nDone!")