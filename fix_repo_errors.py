#!/usr/bin/env python3
import re
import sys

def fix_repository_calls(content):
    """Fix RepositoryFactory calls that use ? operator in non-Result returning functions"""

    # Pattern to find RepositoryFactory calls with ?
    pattern = r'(\s+)let (\w+) = RepositoryFactory::(\w+)\(&req\)\?;'

    def replace_func(match):
        indent = match.group(1)
        var_name = match.group(2)
        method_name = match.group(3)
        return f"""{indent}let {var_name} = match RepositoryFactory::{method_name}(&req) {{
{indent}    Ok(r) => r,
{indent}    Err(e) => return json::error_response(&e),
{indent}}};"""

    # Replace all occurrences
    fixed_content = re.sub(pattern, replace_func, content)

    return fixed_content

def process_file(filepath):
    """Process a single file"""
    try:
        with open(filepath, 'r') as f:
            content = f.read()

        # Check if file has the pattern we need to fix
        if 'RepositoryFactory::' in content and '(&req)?' in content:
            fixed_content = fix_repository_calls(content)

            if fixed_content != content:
                with open(filepath, 'w') as f:
                    f.write(fixed_content)
                print(f"Fixed: {filepath}")
            else:
                print(f"No changes needed: {filepath}")
        else:
            print(f"Skipped: {filepath}")
    except Exception as e:
        print(f"Error processing {filepath}: {e}")

# List of files to process
files = [
    'src/handlers/attorney.rs',
    'src/handlers/criminal_case.rs',
    'src/handlers/config.rs',
    'src/handlers/deadline.rs',
    'src/handlers/judge.rs',
    'src/handlers/sentencing.rs',
    'src/handlers/admin.rs',
]

for filepath in files:
    process_file(filepath)

print("Done!")