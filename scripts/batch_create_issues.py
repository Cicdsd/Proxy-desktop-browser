#!/usr/bin/env python3
"""Batch create GitHub issues from JSON file."""

import json
import subprocess
import sys
import time
import os

def create_issue(title, body):
    """Create a single GitHub issue."""
    cmd = ['gh', 'issue', 'create', '--title', title, '--body', body]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        if result.returncode == 0:
            url = result.stdout.strip()
            return True, url
        else:
            return False, result.stderr
    except subprocess.TimeoutExpired:
        return False, "Timeout"
    except Exception as e:
        return False, str(e)

def main():
    if len(sys.argv) < 2:
        print("Usage: python batch_create_issues.py <issues.json> [start_idx] [count]")
        sys.exit(1)
    
    json_file = sys.argv[1]
    start_idx = int(sys.argv[2]) if len(sys.argv) > 2 else 0
    count = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    
    with open(json_file, 'r') as f:
        issues = json.load(f)
    
    if count > 0:
        issues = issues[start_idx:start_idx + count]
    else:
        issues = issues[start_idx:]
    
    print(f"Creating {len(issues)} issues starting from index {start_idx}...")
    
    created = 0
    failed = 0
    
    for i, issue in enumerate(issues):
        idx = start_idx + i + 1
        title = issue['title'][:250]  # GitHub title limit
        body = issue['body'][:65000]  # GitHub body limit
        
        success, result = create_issue(title, body)
        
        if success:
            created += 1
            print(f"✅ {idx}: {title[:50]}...")
        else:
            failed += 1
            print(f"❌ {idx}: {title[:40]}... ({result[:30]})")
        
        # Small delay to avoid rate limiting
        if (i + 1) % 10 == 0:
            print(f"Progress: {created} created, {failed} failed")
            time.sleep(0.5)
    
    print(f"\n=== Summary ===")
    print(f"Created: {created}")
    print(f"Failed: {failed}")
    print(f"Total: {created + failed}")

if __name__ == "__main__":
    main()
