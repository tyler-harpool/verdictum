#!/usr/bin/env python3
import os
import re

def fix_url_handler(filepath):
    """Fix URL handler functions to properly convert IntoResponse to Response"""

    with open(filepath, 'r') as f:
        content = f.read()

    # Since handlers return ApiResult<impl IntoResponse> and IntoResponse can be converted to Response,
    # we need to call .into_response() on the Ok value
    # First, check if we need the IntoResponse import
    if 'use spin_sdk::http::{IntoResponse' not in content:
        # Add IntoResponse to existing imports
        content = re.sub(
            r'(use spin_sdk::http::\{)([^}]+)(\};)',
            r'\1IntoResponse, \2\3',
            content,
            count=1
        )

    # Fix the Ok branch to call into_response()
    content = re.sub(
        r'        Ok\(resp\) => Response::new\(resp\.into\(\)\),',
        r'        Ok(resp) => resp.into_response(),',
        content
    )

    # If that pattern doesn't match, try the original pattern
    content = re.sub(
        r'        Ok\(resp\) => resp\.into\(\),',
        r'        Ok(resp) => resp.into_response(),',
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