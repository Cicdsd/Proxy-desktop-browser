#!/bin/bash
# Fast issue creator - creates issues from JSON file

JSON_FILE="generated_issues.json"
CREATED=0
FAILED=0

# Function to create a single issue
create_issue() {
    local title="$1"
    local body="$2"
    local labels="$3"
    
    result=$(gh issue create --title "$title" --body "$body" --label "$labels" 2>&1)
    if [[ $? -eq 0 ]]; then
        echo "✅ $result"
        return 0
    else
        echo "❌ Failed: $result"
        return 1
    fi
}

# Read and create issues using Python to parse JSON
python3 << 'PYTHON'
import json
import subprocess
import sys

with open("generated_issues.json", "r") as f:
    issues = json.load(f)

print(f"Creating {len(issues)} issues...")

for i, issue in enumerate(issues):
    title = issue['title']
    body = issue['body']
    labels = ','.join(issue.get('labels', []))
    
    # Escape special characters for shell
    title = title.replace('"', '\\"').replace('`', '\\`').replace('$', '\\$')
    body = body.replace('"', '\\"').replace('`', '\\`').replace('$', '\\$')
    
    cmd = f'gh issue create --title "{title}" --body "{body}" --label "{labels}"'
    
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=30)
    
    if result.returncode == 0:
        print(f"[{i+1}/{len(issues)}] ✅ {result.stdout.strip()}")
    else:
        print(f"[{i+1}/{len(issues)}] ❌ {result.stderr.strip()[:80]}")
    
    # Show progress
    if (i + 1) % 25 == 0:
        print(f"--- Progress: {i+1}/{len(issues)} ---")

PYTHON
