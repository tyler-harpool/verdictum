#!/usr/bin/env python3
import os
import re

def fix_url_handler(filepath):
    """Fix URL handler functions to properly handle ApiResult returns"""

    with open(filepath, 'r') as f:
        content = f.read()

    # Check if already has the right pattern
    if 'match crate::handlers::' in content and '.into()' in content:
        print(f"Already fixed: {filepath}")
        return

    # Fix the wrapper function bodies to handle ApiResult
    # We need to match the Result and convert to Response
    pattern = r'(pub fn \w+\(req: Request, params: Params\) -> Response \{\n    let req = match add_district_header\(req, &params\) \{\n        Ok\(r\) => r,\n        Err\(e\) => return crate::utils::json_response::error_response\(&e\),\n    \};\n    )(crate::handlers::\w+::\w+)\(req, params\)\n\}'

    def replace_func(match):
        prefix = match.group(1)
        handler_call = match.group(2)
        return f'{prefix}match {handler_call}(req, params) {{\n        Ok(resp) => resp.into(),\n        Err(e) => crate::utils::json_response::error_response(&e),\n    }}\n}}'

    content = re.sub(pattern, replace_func, content)

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