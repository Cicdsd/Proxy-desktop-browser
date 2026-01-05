#!/usr/bin/env python3
"""Comprehensive Code Issue Analyzer for Proxy Desktop Browser"""

import os
import re
import json
import subprocess
from pathlib import Path
from typing import List

class IssueAnalyzer:
    def __init__(self, repo_path: str):
        self.repo_path = Path(repo_path)
        self.issues = []
        self.issue_id = 0
        
    def add_issue(self, title: str, body: str, labels: List[str], priority: str = "medium"):
        self.issue_id += 1
        self.issues.append({
            "id": self.issue_id,
            "title": title,
            "body": body,
            "labels": labels,
            "priority": priority
        })
    
    def analyze_rust_files(self):
        rust_files = list(self.repo_path.glob("**/*.rs"))
        for rust_file in rust_files:
            if ".git" in str(rust_file):
                continue
            try:
                content = rust_file.read_text(encoding='utf-8', errors='ignore')
                rel_path = str(rust_file.relative_to(self.repo_path))
                
                unwrap_matches = re.findall(r'\.unwrap\(\)', content)
                if len(unwrap_matches) > 3:
                    self.add_issue(
                        "[Code Quality] Replace unwrap() calls in " + rel_path,
                        "## Description\nThe file `" + rel_path + "` contains " + str(len(unwrap_matches)) + " uses of `.unwrap()` which can cause panics.\n\n## Suggested Fix\nReplace with proper error handling using `?` operator or `.expect()`.\n\n## File Location\n`" + rel_path + "`",
                        ["code-quality", "rust", "error-handling"],
                        "medium"
                    )
                
                todo_matches = re.findall(r'//\s*TODO[:\s](.+)', content, re.IGNORECASE)
                for todo in todo_matches[:3]:
                    self.add_issue(
                        "[TODO] " + todo[:60],
                        "## Description\nFound TODO in `" + rel_path + "`:\n```\n" + todo + "\n```\n\n## File Location\n`" + rel_path + "`",
                        ["todo", "enhancement"],
                        "low"
                    )
                
                fixme_matches = re.findall(r'//\s*FIXME[:\s](.+)', content, re.IGNORECASE)
                for fixme in fixme_matches:
                    self.add_issue(
                        "[FIXME] " + fixme[:60],
                        "## Description\nFound FIXME in `" + rel_path + "`:\n```\n" + fixme + "\n```\n\n## File Location\n`" + rel_path + "`",
                        ["bug", "fixme"],
                        "high"
                    )
                
                pub_fn_count = len(re.findall(r'pub\s+(?:async\s+)?fn\s+\w+', content))
                doc_count = len(re.findall(r'///.*\n\s*pub\s+(?:async\s+)?fn', content))
                if pub_fn_count - doc_count > 5:
                    self.add_issue(
                        "[Documentation] Add docs to functions in " + rel_path,
                        "## Description\nThe file has " + str(pub_fn_count - doc_count) + " undocumented public functions.\n\n## File Location\n`" + rel_path + "`",
                        ["documentation", "rust"],
                        "low"
                    )
                
                clone_count = len(re.findall(r'\.clone\(\)', content))
                if clone_count > 10:
                    self.add_issue(
                        "[Performance] Review clone() usage in " + rel_path,
                        "## Description\nThe file has " + str(clone_count) + " clone() calls. Consider using references.\n\n## File Location\n`" + rel_path + "`",
                        ["performance", "rust"],
                        "medium"
                    )
                
                unsafe_count = len(re.findall(r'unsafe\s*\{', content))
                if unsafe_count > 0:
                    self.add_issue(
                        "[Security] Audit unsafe code in " + rel_path,
                        "## Description\nThe file has " + str(unsafe_count) + " unsafe blocks that need review.\n\n## File Location\n`" + rel_path + "`",
                        ["security", "rust", "audit"],
                        "high"
                    )
                
                panic_count = len(re.findall(r'panic!\s*\(', content))
                if panic_count > 0:
                    self.add_issue(
                        "[Error Handling] Replace panic! in " + rel_path,
                        "## Description\nThe file has " + str(panic_count) + " panic! calls. Use Result instead.\n\n## File Location\n`" + rel_path + "`",
                        ["error-handling", "rust"],
                        "medium"
                    )
                
                lines = len(content.split('\n'))
                if lines > 500:
                    self.add_issue(
                        "[Refactoring] Split large file " + rel_path,
                        "## Description\nThe file has " + str(lines) + " lines. Consider splitting into modules.\n\n## File Location\n`" + rel_path + "`",
                        ["refactoring", "rust"],
                        "medium"
                    )
            except:
                pass
    
    def analyze_typescript_files(self):
        ts_files = list(self.repo_path.glob("**/*.ts")) + list(self.repo_path.glob("**/*.tsx"))
        for ts_file in ts_files:
            if ".git" in str(ts_file) or "node_modules" in str(ts_file):
                continue
            try:
                content = ts_file.read_text(encoding='utf-8', errors='ignore')
                rel_path = str(ts_file.relative_to(self.repo_path))
                
                any_count = len(re.findall(r':\s*any\b', content))
                if any_count > 0:
                    self.add_issue(
                        "[TypeScript] Replace 'any' types in " + rel_path,
                        "## Description\nFound " + str(any_count) + " uses of 'any' type.\n\n## File Location\n`" + rel_path + "`",
                        ["typescript", "code-quality"],
                        "medium"
                    )
                
                console_count = len(re.findall(r'console\.(log|warn|error)\s*\(', content))
                if console_count > 2:
                    self.add_issue(
                        "[Code Quality] Remove console in " + rel_path,
                        "## Description\nFound " + str(console_count) + " console statements.\n\n## File Location\n`" + rel_path + "`",
                        ["code-quality", "typescript"],
                        "low"
                    )
                
                todo_matches = re.findall(r'//\s*TODO[:\s](.+)', content, re.IGNORECASE)
                for todo in todo_matches[:2]:
                    self.add_issue(
                        "[TODO] " + todo[:60],
                        "## Description\nFound TODO in `" + rel_path + "`: " + todo + "\n\n## File Location\n`" + rel_path + "`",
                        ["todo", "enhancement"],
                        "low"
                    )
            except:
                pass
    
    def analyze_svelte_files(self):
        svelte_files = list(self.repo_path.glob("**/*.svelte"))
        for svelte_file in svelte_files:
            if ".git" in str(svelte_file):
                continue
            try:
                content = svelte_file.read_text(encoding='utf-8', errors='ignore')
                rel_path = str(svelte_file.relative_to(self.repo_path))
                
                if '<img' in content and 'alt=' not in content:
                    self.add_issue(
                        "[Accessibility] Add alt attributes in " + rel_path,
                        "## Description\nImages may lack alt attributes.\n\n## File Location\n`" + rel_path + "`",
                        ["accessibility", "svelte"],
                        "medium"
                    )
                
                inline_count = len(re.findall(r'style\s*=\s*["\']', content))
                if inline_count > 5:
                    self.add_issue(
                        "[Code Quality] Extract inline styles in " + rel_path,
                        "## Description\nFound " + str(inline_count) + " inline styles.\n\n## File Location\n`" + rel_path + "`",
                        ["code-quality", "svelte"],
                        "low"
                    )
                
                lines = len(content.split('\n'))
                if lines > 200:
                    self.add_issue(
                        "[Refactoring] Split large component " + rel_path,
                        "## Description\nComponent has " + str(lines) + " lines.\n\n## File Location\n`" + rel_path + "`",
                        ["refactoring", "svelte"],
                        "medium"
                    )
            except:
                pass
    
    def generate_feature_issues(self):
        features = [
            ("Multi-language support (i18n)", "Add internationalization support for the UI", ["enhancement", "i18n"]),
            ("Dark/Light theme toggle", "Add theme switching capability with user preference persistence", ["enhancement", "ui"]),
            ("Proxy health monitoring dashboard", "Real-time dashboard showing proxy status, latency, and uptime", ["enhancement", "feature"]),
            ("Export proxy configurations", "Allow exporting proxy settings to JSON/YAML formats", ["enhancement", "feature"]),
            ("Import proxy configurations", "Allow importing proxy settings from configuration files", ["enhancement", "feature"]),
            ("Customizable keyboard shortcuts", "User-defined hotkeys for common operations", ["enhancement", "ux"]),
            ("Session save and restore", "Persist and restore browser sessions across restarts", ["enhancement", "feature"]),
            ("Scheduled proxy rotation", "Time-based automatic proxy switching with schedules", ["enhancement", "feature"]),
            ("Bandwidth usage monitoring", "Track and display data usage per proxy and tab", ["enhancement", "feature"]),
            ("Connection pool optimization", "Optimize connection reuse for better performance", ["enhancement", "performance"]),
            ("Browser extension support", "Enable installation and management of browser extensions", ["enhancement", "feature"]),
            ("Custom user-agent manager", "UI for creating and managing custom user agents", ["enhancement", "feature"]),
            ("Proxy chaining support", "Support for multi-hop proxy connections", ["enhancement", "feature"]),
            ("Geolocation spoofing", "Ability to fake browser geolocation data", ["enhancement", "privacy"]),
            ("WebRTC leak prevention", "Block WebRTC from leaking real IP addresses", ["enhancement", "security"]),
            ("DNS over HTTPS (DoH)", "Implement encrypted DNS queries for privacy", ["enhancement", "security"]),
            ("Automatic proxy failover", "Automatically switch to backup proxy on failure", ["enhancement", "reliability"]),
            ("Built-in proxy speed test", "Test and compare proxy latency and throughput", ["enhancement", "feature"]),
            ("Usage analytics dashboard", "Detailed statistics on browsing and proxy usage", ["enhancement", "feature"]),
            ("API rate limiting", "Implement rate limiting for API endpoints", ["enhancement", "security"]),
            ("Proxy auth credential caching", "Securely cache proxy authentication credentials", ["enhancement", "security"]),
            ("Tab grouping by proxy", "Organize and group tabs by their proxy settings", ["enhancement", "ui"]),
            ("Bookmark manager", "Save and organize favorite pages with sync support", ["enhancement", "feature"]),
            ("Download manager", "Built-in download manager with proxy support", ["enhancement", "feature"]),
            ("Screenshot capture", "Capture and save tab screenshots", ["enhancement", "feature"]),
            ("Page archiver", "Save complete pages for offline viewing", ["enhancement", "feature"]),
            ("Cookie manager per tab", "Isolated cookie management for each tab", ["enhancement", "privacy"]),
            ("Local storage manager", "Manage local storage data per tab", ["enhancement", "privacy"]),
            ("Network request inspector", "Debug and inspect network requests", ["enhancement", "debugging"]),
            ("Console output viewer", "View JavaScript console output from pages", ["enhancement", "debugging"]),
            ("SOCKS4 proxy support", "Add support for SOCKS4 protocol", ["enhancement", "feature"]),
            ("Enhanced SOCKS5 support", "Improved SOCKS5 handling with UDP", ["enhancement", "feature"]),
            ("HTTP/2 proxy support", "Support for HTTP/2 protocol through proxies", ["enhancement", "feature"]),
            ("Tor network integration", "Built-in Tor network connectivity", ["enhancement", "privacy"]),
            ("VPN integration", "Support for VPN connections", ["enhancement", "feature"]),
            ("Ad blocker integration", "Built-in advertisement blocking", ["enhancement", "feature"]),
            ("Tracker blocker", "Block third-party tracking scripts", ["enhancement", "privacy"]),
            ("Password manager integration", "Secure password storage and autofill", ["enhancement", "security"]),
            ("Form autofill", "Automatic form completion", ["enhancement", "feature"]),
            ("Reading mode", "Distraction-free reading experience", ["enhancement", "feature"]),
            ("Print to PDF", "Save pages as PDF documents", ["enhancement", "feature"]),
            ("Tab hibernation", "Suspend inactive tabs to reduce memory", ["enhancement", "performance"]),
            ("Memory usage monitor", "Track memory consumption per tab", ["enhancement", "feature"]),
            ("CPU usage monitor", "Monitor CPU usage by the application", ["enhancement", "feature"]),
            ("Proxy rotation rules engine", "Condition-based proxy rotation logic", ["enhancement", "feature"]),
            ("Site-specific proxy rules", "Configure different proxies per website", ["enhancement", "feature"]),
            ("URL pattern matching", "Regex-based URL matching for rules", ["enhancement", "feature"]),
            ("Whitelist/Blacklist manager", "Manage allowed and blocked sites", ["enhancement", "feature"]),
            ("Auto-update mechanism", "Automatic application updates", ["enhancement", "feature"]),
            ("Crash recovery", "Automatic session recovery after crashes", ["enhancement", "reliability"]),
        ]
        for title, desc, labels in features:
            self.add_issue(
                "[Feature] " + title,
                "## Description\n" + desc + "\n\n## Acceptance Criteria\n- Feature is fully implemented\n- UI is intuitive and accessible\n- Tests are added\n- Documentation is updated",
                labels, "medium"
            )
    
    def generate_testing_issues(self):
        tests = [
            ("Unit tests for proxy module", "Comprehensive unit tests for proxy functionality", ["testing", "rust"]),
            ("Unit tests for tab manager", "Test tab creation, switching, and destruction", ["testing", "rust"]),
            ("Unit tests for config manager", "Test configuration loading and saving", ["testing", "rust"]),
            ("Unit tests for security module", "Test security features and validation", ["testing", "rust"]),
            ("Unit tests for network module", "Test network operations and error handling", ["testing", "rust"]),
            ("Unit tests for fingerprint module", "Test browser fingerprinting features", ["testing", "rust"]),
            ("Unit tests for storage module", "Test data persistence and retrieval", ["testing", "rust"]),
            ("Unit tests for automation module", "Test browser automation features", ["testing", "rust"]),
            ("Integration tests for API", "Test all API endpoints end-to-end", ["testing", "integration"]),
            ("Integration tests for proxy rotation", "Test proxy rotation logic", ["testing", "integration"]),
            ("Integration tests for authentication", "Test authentication flows", ["testing", "integration"]),
            ("Integration tests for tab isolation", "Test tab isolation features", ["testing", "integration"]),
            ("E2E tests for tab creation", "Test tab creation UI flow", ["testing", "e2e"]),
            ("E2E tests for proxy configuration", "Test proxy settings UI", ["testing", "e2e"]),
            ("E2E tests for navigation", "Test browser navigation", ["testing", "e2e"]),
            ("E2E tests for settings panel", "Test settings UI", ["testing", "e2e"]),
            ("Performance benchmarks for proxy", "Measure proxy connection performance", ["testing", "performance"]),
            ("Performance benchmarks for tabs", "Measure tab operation speed", ["testing", "performance"]),
            ("Load testing for connections", "Stress test concurrent connections", ["testing", "performance"]),
            ("Memory leak testing", "Detect and prevent memory leaks", ["testing", "performance"]),
            ("Security penetration testing", "Find security vulnerabilities", ["testing", "security"]),
            ("Fuzz testing for inputs", "Test input handling robustness", ["testing", "security"]),
            ("Regression test suite", "Prevent feature regressions", ["testing", "automation"]),
            ("Visual regression testing", "Test UI visual consistency", ["testing", "ui"]),
            ("Accessibility testing", "Test accessibility compliance", ["testing", "accessibility"]),
        ]
        for title, desc, labels in tests:
            self.add_issue(
                "[Testing] " + title,
                "## Description\n" + desc + "\n\n## Requirements\n- Tests are comprehensive\n- Edge cases are covered\n- Tests are maintainable and documented",
                labels, "medium"
            )
    
    def generate_security_issues(self):
        security = [
            ("Input validation audit", "Audit and validate all user inputs", ["security", "audit"]),
            ("SQL injection prevention", "Ensure parameterized queries are used", ["security", "audit"]),
            ("XSS prevention audit", "Sanitize all output to prevent XSS", ["security", "audit"]),
            ("CSRF protection implementation", "Add CSRF tokens to state-changing operations", ["security", "audit"]),
            ("Secrets management system", "Implement secure secret storage", ["security", "infrastructure"]),
            ("Certificate pinning", "Pin SSL certificates for security", ["security", "network"]),
            ("Memory safety audit", "Review all unsafe Rust code", ["security", "rust"]),
            ("Dependency vulnerability scan", "Scan dependencies for vulnerabilities", ["security", "dependencies"]),
            ("Authentication hardening", "Strengthen authentication mechanisms", ["security", "auth"]),
            ("Session security review", "Ensure secure session management", ["security", "auth"]),
            ("Encryption at rest", "Encrypt all sensitive stored data", ["security", "data"]),
            ("Encryption in transit", "Ensure all communications are encrypted", ["security", "network"]),
            ("Access control audit", "Review and verify access permissions", ["security", "auth"]),
            ("Log sanitization", "Remove sensitive data from logs", ["security", "logging"]),
            ("Error message review", "Ensure errors don't leak sensitive info", ["security", "audit"]),
            ("Rate limiting implementation", "Prevent API abuse with rate limits", ["security", "api"]),
            ("Brute force protection", "Block repeated authentication attempts", ["security", "auth"]),
            ("Security headers implementation", "Add security HTTP headers", ["security", "web"]),
            ("Content Security Policy", "Implement strict CSP", ["security", "web"]),
            ("Subresource integrity", "Verify external resource integrity", ["security", "web"]),
        ]
        for title, desc, labels in security:
            self.add_issue(
                "[Security] " + title,
                "## Description\n" + desc + "\n\n## Security Impact\nCritical for protecting user data and system integrity.\n\n## Acceptance Criteria\n- Vulnerability is addressed\n- Security tests are added",
                labels, "high"
            )
    
    def generate_documentation_issues(self):
        docs = [
            ("API reference documentation", "Complete API endpoint documentation", ["documentation"]),
            ("Architecture diagrams", "Create system architecture diagrams", ["documentation"]),
            ("Developer setup guide", "Step-by-step development environment setup", ["documentation"]),
            ("User manual", "Comprehensive end-user documentation", ["documentation"]),
            ("Configuration reference", "Document all configuration options", ["documentation"]),
            ("Troubleshooting guide", "Common issues and solutions", ["documentation"]),
            ("Security best practices", "Security guidelines for users", ["documentation"]),
            ("Changelog maintenance", "Keep changelog up to date", ["documentation"]),
            ("Code comments improvement", "Improve inline code documentation", ["documentation"]),
            ("README enhancements", "Expand and improve README", ["documentation"]),
            ("Contributing guidelines", "Create CONTRIBUTING.md file", ["documentation"]),
            ("Code of conduct", "Add CODE_OF_CONDUCT.md", ["documentation"]),
            ("License documentation", "Clarify licensing terms", ["documentation"]),
            ("Deployment guide", "Document deployment procedures", ["documentation"]),
            ("Backup procedures", "Document backup and restore", ["documentation"]),
            ("API versioning documentation", "Document API version policy", ["documentation"]),
            ("Error code reference", "Document all error codes", ["documentation"]),
            ("FAQ section", "Frequently asked questions", ["documentation"]),
            ("Video tutorials", "Create tutorial videos", ["documentation"]),
            ("Quick start guide", "5-minute getting started guide", ["documentation"]),
        ]
        for title, desc, labels in docs:
            self.add_issue(
                "[Documentation] " + title,
                "## Description\n" + desc + "\n\n## Requirements\n- Clear and comprehensive\n- Examples provided where helpful\n- Kept in sync with code",
                labels, "low"
            )
    
    def generate_performance_issues(self):
        perf = [
            ("Memory leak detection and fixes", "Identify and fix memory leaks", ["performance"]),
            ("Startup time optimization", "Reduce application startup time", ["performance"]),
            ("Tab switching performance", "Optimize tab switching speed", ["performance"]),
            ("Proxy connection pooling", "Implement efficient connection reuse", ["performance"]),
            ("Lazy loading implementation", "Load resources on demand", ["performance"]),
            ("Cache optimization", "Improve caching strategies", ["performance"]),
            ("Request batching", "Batch network requests efficiently", ["performance"]),
            ("UI rendering optimization", "Improve rendering performance", ["performance"]),
            ("Database query optimization", "Optimize database operations", ["performance"]),
            ("Resource cleanup automation", "Ensure proper resource cleanup", ["performance"]),
            ("Async operation optimization", "Improve async patterns", ["performance"]),
            ("Memory footprint reduction", "Reduce overall memory usage", ["performance"]),
            ("CPU usage optimization", "Reduce CPU consumption", ["performance"]),
            ("Network bandwidth optimization", "Minimize bandwidth usage", ["performance"]),
            ("Bundle size reduction", "Reduce JavaScript bundle size", ["performance"]),
        ]
        for title, desc, labels in perf:
            self.add_issue(
                "[Performance] " + title,
                "## Description\n" + desc + "\n\n## Measurement\n- Baseline performance measured\n- Improvements quantified",
                labels, "medium"
            )
    
    def generate_ui_issues(self):
        ui = [
            ("Responsive design improvements", "Better support for different screen sizes", ["ui", "ux"]),
            ("Loading state indicators", "Add loading spinners and progress bars", ["ui", "ux"]),
            ("Error state UI improvements", "Better error message display", ["ui", "ux"]),
            ("Empty state UI design", "Design for empty lists and views", ["ui", "ux"]),
            ("Tooltip additions", "Add helpful tooltips throughout", ["ui", "ux"]),
            ("Form validation feedback", "Improve form validation UX", ["ui", "ux"]),
            ("Toast notification system", "Implement notification toasts", ["ui", "ux"]),
            ("Modal dialog improvements", "Improve modal design and behavior", ["ui", "ux"]),
            ("Navigation flow improvements", "Better navigation patterns", ["ui", "ux"]),
            ("Search functionality", "Add global search feature", ["ui", "ux"]),
            ("Sorting options for lists", "Add sortable columns", ["ui", "ux"]),
            ("Pagination for large lists", "Implement efficient pagination", ["ui", "ux"]),
            ("Drag and drop support", "Add drag and drop for reordering", ["ui", "ux"]),
            ("Context menu additions", "Add right-click context menus", ["ui", "ux"]),
            ("Undo/redo functionality", "Implement action history", ["ui", "ux"]),
        ]
        for title, desc, labels in ui:
            self.add_issue(
                "[UI/UX] " + title,
                "## Description\n" + desc + "\n\n## Requirements\n- Follow consistent design patterns\n- Ensure accessibility\n- Test on different devices",
                labels, "medium"
            )
    
    def generate_devops_issues(self):
        devops = [
            ("CI/CD pipeline optimization", "Improve pipeline efficiency", ["devops"]),
            ("Automated testing in CI", "Run all tests in CI pipeline", ["devops"]),
            ("Code coverage reporting", "Add coverage reports to CI", ["devops"]),
            ("Security scanning in CI", "Automated security scans", ["devops"]),
            ("Release automation", "Automate the release process", ["devops"]),
            ("Docker image optimization", "Reduce image size and build time", ["devops"]),
            ("Dependency caching in CI", "Speed up builds with caching", ["devops"]),
            ("Build time reduction", "Optimize build performance", ["devops"]),
            ("Artifact management", "Improve artifact handling", ["devops"]),
            ("Environment configuration", "Better environment management", ["devops"]),
            ("Application monitoring setup", "Implement monitoring solution", ["devops"]),
            ("Centralized logging", "Set up log aggregation", ["devops"]),
            ("Alerting system", "Implement critical alerts", ["devops"]),
            ("Backup automation", "Automate backup procedures", ["devops"]),
            ("Disaster recovery planning", "Document and test DR", ["devops"]),
        ]
        for title, desc, labels in devops:
            self.add_issue(
                "[DevOps] " + title,
                "## Description\n" + desc + "\n\n## Requirements\n- Document all changes\n- Test before deploying",
                labels, "medium"
            )
    
    def generate_refactoring_issues(self):
        refactor = [
            ("Extract common utility functions", "Create shared utilities module", ["refactoring"]),
            ("Reduce code duplication", "Apply DRY principle across codebase", ["refactoring"]),
            ("Improve error type hierarchy", "Create specific error types", ["refactoring"]),
            ("Simplify complex functions", "Break down large functions", ["refactoring"]),
            ("Improve naming conventions", "Consistent naming throughout", ["refactoring"]),
            ("Module restructuring", "Better code organization", ["refactoring"]),
            ("Interface simplification", "Cleaner public APIs", ["refactoring"]),
            ("Remove dead code", "Clean up unused code", ["refactoring"]),
            ("Consolidate configurations", "Single source of truth for config", ["refactoring"]),
            ("Type system improvements", "Better type safety", ["refactoring"]),
        ]
        for title, desc, labels in refactor:
            self.add_issue(
                "[Refactoring] " + title,
                "## Description\n" + desc + "\n\n## Goals\n- Improve maintainability\n- Better readability\n- Reduce technical debt",
                labels, "medium"
            )
    
    def generate_compatibility_issues(self):
        compat = [
            ("Windows compatibility testing", "Test and fix Windows-specific issues", ["compatibility"]),
            ("macOS compatibility testing", "Test and fix macOS-specific issues", ["compatibility"]),
            ("Linux compatibility testing", "Test and fix Linux-specific issues", ["compatibility"]),
            ("High DPI display support", "Proper scaling on high-res displays", ["compatibility"]),
            ("Multi-monitor support", "Handle multiple displays correctly", ["compatibility"]),
            ("Older OS version support", "Define minimum OS requirements", ["compatibility"]),
            ("ARM architecture support", "Support for ARM processors", ["compatibility"]),
            ("Screen reader compatibility", "Work with assistive technologies", ["compatibility", "accessibility"]),
        ]
        for title, desc, labels in compat:
            self.add_issue(
                "[Compatibility] " + title,
                "## Description\n" + desc + "\n\n## Testing Required\n- Test on target platform\n- Document any limitations",
                labels, "medium"
            )
    
    def generate_bug_issues(self):
        bugs = [
            ("Fix race conditions in proxy rotation", "Thread safety improvements needed", ["bug", "rust"]),
            ("Fix memory leak in tab manager", "Memory not properly freed", ["bug", "performance"]),
            ("Fix connection timeout handling", "Timeouts not handled gracefully", ["bug", "network"]),
            ("Fix proxy authentication retry", "Auth failure not retried correctly", ["bug", "network"]),
            ("Fix session persistence issues", "Sessions not saved properly", ["bug", "feature"]),
            ("Fix keyboard shortcut conflicts", "Some hotkeys conflict", ["bug", "ui"]),
            ("Fix clipboard operations", "Copy/paste not working correctly", ["bug", "ui"]),
            ("Fix window state restoration", "Window position not restored", ["bug", "ui"]),
            ("Fix file dialog handling", "File selection issues", ["bug", "ui"]),
            ("Fix notification display", "Notifications not showing", ["bug", "ui"]),
            ("Fix URL validation", "Some invalid URLs accepted", ["bug", "validation"]),
            ("Fix proxy URL parsing", "Parsing errors with some URLs", ["bug", "network"]),
            ("Fix certificate error handling", "SSL errors not handled", ["bug", "security"]),
            ("Fix websocket connections", "WebSocket through proxy issues", ["bug", "network"]),
            ("Fix redirect handling", "Redirect loops possible", ["bug", "network"]),
        ]
        for title, desc, labels in bugs:
            self.add_issue(
                "[Bug] " + title,
                "## Description\n" + desc + "\n\n## Steps to Reproduce\n1. [Add steps]\n\n## Expected Behavior\n[Describe expected behavior]\n\n## Actual Behavior\n[Describe actual behavior]",
                labels, "high"
            )
    
    def generate_infrastructure_issues(self):
        infra = [
            ("Set up error tracking service", "Implement Sentry or similar", ["infrastructure"]),
            ("Implement telemetry collection", "Anonymous usage statistics", ["infrastructure"]),
            ("Set up crash reporting", "Automatic crash reports", ["infrastructure"]),
            ("Implement feature flags", "Toggle features remotely", ["infrastructure"]),
            ("Set up A/B testing framework", "Test feature variations", ["infrastructure"]),
            ("Implement analytics dashboard", "Usage analytics", ["infrastructure"]),
            ("Set up status page", "Service status monitoring", ["infrastructure"]),
            ("Implement update server", "Serve application updates", ["infrastructure"]),
            ("Set up documentation hosting", "Host docs website", ["infrastructure"]),
            ("Implement feedback collection", "In-app feedback system", ["infrastructure"]),
        ]
        for title, desc, labels in infra:
            self.add_issue(
                "[Infrastructure] " + title,
                "## Description\n" + desc + "\n\n## Requirements\n- Privacy-respecting implementation\n- Opt-out capability where applicable",
                labels, "medium"
            )
    
    def generate_accessibility_issues(self):
        a11y = [
            ("Keyboard navigation support", "Full keyboard accessibility", ["accessibility"]),
            ("Screen reader optimization", "Improve screen reader experience", ["accessibility"]),
            ("Color contrast improvements", "Meet WCAG contrast requirements", ["accessibility"]),
            ("Focus indicator visibility", "Clear focus states", ["accessibility"]),
            ("ARIA labels implementation", "Proper ARIA attributes", ["accessibility"]),
            ("Skip navigation links", "Allow skipping to content", ["accessibility"]),
            ("Form label associations", "Proper label-input associations", ["accessibility"]),
            ("Error announcement for screen readers", "Announce errors properly", ["accessibility"]),
            ("Reduced motion support", "Respect prefers-reduced-motion", ["accessibility"]),
            ("Text sizing support", "Support user text size preferences", ["accessibility"]),
        ]
        for title, desc, labels in a11y:
            self.add_issue(
                "[Accessibility] " + title,
                "## Description\n" + desc + "\n\n## WCAG Guidelines\n- Meet WCAG 2.1 AA requirements\n- Test with assistive technologies",
                labels, "medium"
            )
    
    def run_analysis(self):
        print("Analyzing Rust files...")
        self.analyze_rust_files()
        print("Analyzing TypeScript files...")
        self.analyze_typescript_files()
        print("Analyzing Svelte files...")
        self.analyze_svelte_files()
        print("Generating feature issues...")
        self.generate_feature_issues()
        print("Generating testing issues...")
        self.generate_testing_issues()
        print("Generating security issues...")
        self.generate_security_issues()
        print("Generating documentation issues...")
        self.generate_documentation_issues()
        print("Generating performance issues...")
        self.generate_performance_issues()
        print("Generating UI/UX issues...")
        self.generate_ui_issues()
        print("Generating DevOps issues...")
        self.generate_devops_issues()
        print("Generating refactoring issues...")
        self.generate_refactoring_issues()
        print("Generating compatibility issues...")
        self.generate_compatibility_issues()
        print("Generating bug issues...")
        self.generate_bug_issues()
        print("Generating infrastructure issues...")
        self.generate_infrastructure_issues()
        print("Generating accessibility issues...")
        self.generate_accessibility_issues()
        return self.issues


def create_github_issue(title, body, labels):
    cmd = ['gh', 'issue', 'create', '--title', title, '--body', body]
    if labels:
        cmd.extend(['--label', ','.join(labels)])
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode == 0:
        print("Created: " + title[:60] + "...")
        return True
    else:
        print("Failed: " + title[:60] + "... - " + result.stderr[:100])
        return False


def ensure_labels_exist(labels):
    existing = subprocess.run(['gh', 'label', 'list', '--json', 'name'], capture_output=True, text=True)
    try:
        existing_labels = {l['name'] for l in json.loads(existing.stdout)} if existing.returncode == 0 else set()
    except:
        existing_labels = set()
    
    colors = {
        "bug": "d73a4a", "enhancement": "a2eeef", "documentation": "0075ca",
        "security": "ee0701", "performance": "fbca04", "testing": "bfd4f2",
        "code-quality": "7057ff", "refactoring": "d4c5f9", "ui": "1d76db",
        "ux": "5319e7", "accessibility": "0e8a16", "todo": "fef2c0",
        "fixme": "b60205", "dependencies": "0366d6", "devops": "006b75",
        "compatibility": "c5def5", "rust": "dea584", "typescript": "3178c6",
        "svelte": "ff3e00", "feature": "a2eeef", "privacy": "5319e7",
        "error-handling": "d73a4a", "configuration": "c2e0c6", "audit": "ee0701",
        "e2e": "bfd4f2", "debugging": "d4c5f9", "infrastructure": "006b75",
        "i18n": "7057ff", "reliability": "0e8a16", "network": "0366d6",
        "auth": "5319e7", "api": "1d76db", "web": "ff3e00", "data": "fbca04",
        "logging": "c2e0c6", "automation": "a2eeef", "quality": "7057ff",
        "integration": "bfd4f2", "validation": "d73a4a",
    }
    
    for label in labels - existing_labels:
        color = colors.get(label, "ededed")
        subprocess.run(['gh', 'label', 'create', label, '--color', color, '--force'], capture_output=True, text=True)
        print("Created label: " + label)


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Comprehensive Issue Analyzer")
    parser.add_argument("--repo-path", default=".", help="Repository path")
    parser.add_argument("--create", action="store_true", help="Create issues on GitHub")
    parser.add_argument("--limit", type=int, help="Limit number of issues")
    parser.add_argument("--list", action="store_true", help="List issues only")
    parser.add_argument("--output", type=str, help="Output to JSON file")
    args = parser.parse_args()
    
    analyzer = IssueAnalyzer(args.repo_path)
    issues = analyzer.run_analysis()
    
    if args.limit:
        issues = issues[:args.limit]
    
    print("\n" + "="*60)
    print("Total issues: " + str(len(issues)))
    print("="*60 + "\n")
    
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(issues, f, indent=2)
        print("Saved to " + args.output)
    
    if args.list:
        for i, issue in enumerate(issues, 1):
            print(str(i) + ". [" + issue['priority'] + "] " + issue['title'])
    
    if args.create:
        all_labels = set()
        for issue in issues:
            all_labels.update(issue['labels'])
        print("Creating labels...")
        ensure_labels_exist(all_labels)
        print("\nCreating " + str(len(issues)) + " issues...")
        success = 0
        for issue in issues:
            if create_github_issue(issue['title'], issue['body'], issue['labels']):
                success += 1
        print("\nCreated " + str(success) + "/" + str(len(issues)) + " issues")


if __name__ == "__main__":
    main()
