#!/usr/bin/env python3
"""Tool to fix unused imports in Rust files."""

import re
import sys
from pathlib import Path

def remove_import_from_line(line, import_to_remove):
    """Remove a specific import from a use statement line."""
    original = line
    
    # Case 1: Simple import like "use std::sync::Arc;"
    simple_pattern = rf'^(\s*)use\s+[\w:]+::{import_to_remove}\s*;\s*$'
    if re.match(simple_pattern, line):
        return ''
    
    # Case 2: Full path import like "use anyhow::Result;"
    full_pattern = rf'^(\s*)use\s+{import_to_remove}\s*;\s*$'
    if re.match(full_pattern, line):
        return ''
    
    # Case 3: Nested import like "use anyhow::Result;"
    nested_pattern = rf'^(\s*)use\s+\w+::{import_to_remove}\s*;\s*$'
    if re.match(nested_pattern, line):
        return ''
    
    # Case 4: Grouped imports like "use tracing::{debug, info, warn};"
    if '{' in line and '}' in line:
        # Check if import is in the braces
        brace_match = re.search(r'\{([^}]+)\}', line)
        if brace_match:
            imports_str = brace_match.group(1)
            imports = [i.strip() for i in imports_str.split(',')]
            
            if import_to_remove in imports:
                imports.remove(import_to_remove)
                
                if len(imports) == 0:
                    # Remove entire line if no imports left
                    return ''
                elif len(imports) == 1:
                    # Convert to single import
                    prefix_match = re.match(r'^(\s*use\s+[\w:]+::)\{', line)
                    if prefix_match:
                        prefix = prefix_match.group(1)
                        return f"{prefix}{imports[0]};\n"
                else:
                    # Keep as grouped import
                    new_imports = ', '.join(imports)
                    new_line = re.sub(r'\{[^}]+\}', f'{{{new_imports}}}', line)
                    return new_line
    
    return line

def fix_file(file_path, imports_to_remove):
    """Fix unused imports in a file."""
    path = Path(file_path)
    if not path.exists():
        print(f"File not found: {file_path}")
        return False
    
    with open(path, 'r') as f:
        lines = f.readlines()
    
    modified = False
    new_lines = []
    
    # Group by line number
    line_imports = {}
    for imp in imports_to_remove:
        line_num = imp['line']
        if line_num not in line_imports:
            line_imports[line_num] = []
        line_imports[line_num].append(imp['import'])
    
    for i, line in enumerate(lines, 1):
        if i in line_imports:
            new_line = line
            for imp in line_imports[i]:
                new_line = remove_import_from_line(new_line, imp)
                if new_line != line:
                    modified = True
            if new_line:
                new_lines.append(new_line)
        else:
            new_lines.append(line)
    
    if modified:
        with open(path, 'w') as f:
            f.writelines(new_lines)
        print(f"Fixed: {file_path}")
        return True
    return False

# Files and their unused imports based on cargo check output
FILES_TO_FIX = {
    'crates/browser-core/src/config_manager.rs': [
        {'line': 12, 'import': 'std::collections::HashMap'},
        {'line': 16, 'import': 'warn'},
    ],
    'crates/browser-core/src/storage.rs': [
        {'line': 16, 'import': 'warn'},
    ],
    'crates/browser-core/src/memory_profiler.rs': [
        {'line': 10, 'import': 'Duration'},
    ],
    'crates/browser-core/src/performance_optimizer.rs': [
        {'line': 6, 'import': 'Result'},
        {'line': 10, 'import': 'Duration'},
        {'line': 12, 'import': 'warn'},
    ],
    'crates/browser-core/src/network_intelligence.rs': [
        {'line': 6, 'import': 'Result'},
        {'line': 10, 'import': 'Duration'},
        {'line': 12, 'import': 'debug'},
    ],
    'crates/browser-core/src/privacy_fortress.rs': [
        {'line': 6, 'import': 'Result'},
        {'line': 13, 'import': 'warn'},
    ],
    'crates/browser-core/src/experimental.rs': [
        {'line': 9, 'import': 'Arc'},
        {'line': 11, 'import': 'RwLock'},
        {'line': 12, 'import': 'debug'},
    ],
    'crates/browser-core/src/efficiency/cache_manager.rs': [
        {'line': 4, 'import': 'Hash'},
    ],
    'crates/browser-core/src/automation.rs': [
        {'line': 13, 'import': 'Arc'},
        {'line': 14, 'import': 'Duration'},
        {'line': 15, 'import': 'RwLock'},
        {'line': 16, 'import': 'debug'},
        {'line': 16, 'import': 'warn'},
    ],
}

def main():
    base_path = Path('/root/live-swe-agent/Proxy-desktop-browser')
    
    for file_rel, imports in FILES_TO_FIX.items():
        file_path = base_path / file_rel
        fix_file(str(file_path), imports)
    
    print("\nDone fixing unused imports!")

if __name__ == '__main__':
    main()
