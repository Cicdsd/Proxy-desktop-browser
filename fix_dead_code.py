#!/usr/bin/env python3
"""Tool to fix dead_code warnings by adding allow(dead_code) attributes."""

import re
from pathlib import Path

def add_allow_dead_code_to_struct(file_path, struct_name):
    """Add #[allow(dead_code)] attribute to a struct."""
    path = Path(file_path)
    if not path.exists():
        print(f"File not found: {file_path}")
        return False
    
    with open(path, 'r') as f:
        content = f.read()
    
    # Pattern to find struct definition
    pattern = rf'(#\[derive\([^\]]+\)\]\s*)?(pub\s+struct\s+{struct_name}\s*\{{)'
    
    def replace_func(match):
        derive_part = match.group(1) or ''
        struct_part = match.group(2)
        
        # Check if already has allow(dead_code)
        if '#[allow(dead_code)]' in derive_part:
            return match.group(0)
        
        return f'{derive_part}#[allow(dead_code)]\n{struct_part}'
    
    new_content = re.sub(pattern, replace_func, content)
    
    if new_content != content:
        with open(path, 'w') as f:
            f.write(new_content)
        print(f"Fixed struct {struct_name} in {file_path}")
        return True
    return False

# Structs that need fixing based on warnings
STRUCTS_TO_FIX = [
    ('crates/browser-core/src/network_intelligence.rs', 'TrafficAnalyzer'),
    ('crates/browser-core/src/network_intelligence.rs', 'BandwidthManager'),
    ('crates/browser-core/src/privacy_fortress.rs', 'FingerprintProtector'),
    ('crates/browser-core/src/privacy_fortress.rs', 'PrivacyScore'),
    ('crates/browser-core/src/experimental.rs', 'MultiEngineSystem'),
    ('crates/browser-core/src/experimental.rs', 'DnsResolver'),
    ('crates/browser-core/src/experimental.rs', 'DifferentialPrivacy'),
    ('crates/browser-core/src/experimental.rs', 'AntiCorrelationSystem'),
    ('crates/browser-core/src/experimental.rs', 'LocalLlm'),
    ('crates/browser-core/src/experimental.rs', 'VisualEngine'),
    ('crates/browser-core/src/experimental.rs', 'PostQuantumCrypto'),
    ('crates/browser-core/src/experimental.rs', 'BlockchainDns'),
    ('crates/browser-core/src/experimental.rs', 'QuantumRng'),
    ('crates/browser-core/src/automation.rs', 'NaturalLanguageAutomation'),
    ('crates/browser-core/src/content_enhancement/language_detector.rs', 'AdvancedLanguageDetector'),
]

def main():
    base_path = Path('/root/live-swe-agent/Proxy-desktop-browser')
    
    for file_rel, struct_name in STRUCTS_TO_FIX:
        file_path = base_path / file_rel
        add_allow_dead_code_to_struct(str(file_path), struct_name)
    
    print("\nDone fixing dead_code warnings!")

if __name__ == '__main__':
    main()
