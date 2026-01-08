#!/usr/bin/env python3
"""Tool to fix unused imports in Rust files based on cargo check output."""

import re
import sys
import subprocess
from pathlib import Path
from collections import defaultdict

def parse_warnings(output):
    """Parse cargo check output for unused import warnings."""
    warnings = defaultdict(list)
    
    # Pattern for unused import warnings
    pattern = r'warning: unused import: `([^`]+)`\s+-->\s+([^:]+):(\d+):(\d+)'
    
    for match in re.finditer(pattern, output):
        import_name = match.group(1)
        file_path = match.group(2)
        line_num = int(match.group(3))
        warnings[file_path].append({
            'import': import_name,
            'line': line_num
        })
    
    # Pattern for multiple unused imports
    pattern2 = r'warning: unused imports: ([^`]+)\s+-->\s+([^:]+):(\d+):(\d+)'
    for match in re.finditer(pattern2, output):
        imports_str = match.group(1)
        file_path = match.group(2)
        line_num = int(match.group(3))
        # Parse imports like "`debug` and `warn`"
        imports = re.findall(r'`([^`]+)`', imports_str)
        for imp in imports:
            warnings[file_path].append({
                'import': imp,
                'line': line_num
            })
    
    return warnings

def fix_file(file_path, warnings):
    """Fix unused imports in a file."""
    if not Path(file_path).exists():
        print(f"File not found: {file_path}")
        return False
    
    with open(file_path, 'r') as f:
        lines = f.readlines()
    
    # Group warnings by line
    line_warnings = defaultdict(list)
    for w in warnings:
        line_warnings[w['line']].append(w['import'])
    
    modified = False
    new_lines = []
    
    for i, line in enumerate(lines, 1):
        if i in line_warnings:
            imports_to_remove = line_warnings[i]
            new_line = line
            
            for imp in imports_to_remove:
                # Handle different import patterns
                # Pattern: use foo::{a, b, c};
                if '{' in new_line and '}' in new_line:
                    # Remove from braced imports
                    new_line = re.sub(rf',\s*{imp}(?=\s*[,}}])', '', new_line)
                    new_line = re.sub(rf'{imp}\s*,\s*', '', new_line)
                    new_line = re.sub(rf'{imp}(?=\s*}})', '', new_line)
                else:
                    # Single import: use foo::bar;
                    if re.search(rf'use\s+[^;]*::{imp};', new_line):
                        new_line = ''
            
            # Clean up empty braces
            new_line = re.sub(r'use\s+\w+::\{\s*\};?\n?', '', new_line)
            # Clean up trailing commas before closing brace
            new_line = re.sub(r',\s*}', '}', new_line)
            # Clean up double commas
            new_line = re.sub(r',\s*,', ',', new_line)
            
            if new_line != line:
                modified = True
            
            if new_line.strip():
                new_lines.append(new_line)
        else:
            new_lines.append(line)
    
    if modified:
        with open(file_path, 'w') as f:
            f.writelines(new_lines)
        print(f"Fixed: {file_path}")
    
    return modified

def main():
    # Run cargo check and capture output
    print("Running cargo check...")
    result = subprocess.run(
        ['cargo', 'check', '2>&1'],
        shell=True,
        capture_output=True,
        text=True,
        cwd='/root/live-swe-agent/Proxy-desktop-browser'
    )
    
    output = result.stdout + result.stderr
    
    # Parse warnings
    warnings = parse_warnings(output)
    
    if not warnings:
        print("No unused import warnings found.")
        return
    
    print(f"Found warnings in {len(warnings)} files:")
    for file_path, file_warnings in warnings.items():
        print(f"  {file_path}: {len(file_warnings)} warnings")
    
    # Fix files
    for file_path, file_warnings in warnings.items():
        fix_file(file_path, file_warnings)

if __name__ == '__main__':
    main()
