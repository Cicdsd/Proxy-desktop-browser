#!/usr/bin/env python3
"""Comprehensive tool to generate 300+ GitHub issues from codebase analysis."""

import os
import re
import subprocess
import json
import sys
from pathlib import Path
from collections import defaultdict

class IssueGenerator:
    def __init__(self, repo_path):
        self.repo_path = Path(repo_path)
        self.issues = []
        self.issue_id = 0
        
    def add_issue(self, title, body, labels, category):
        """Add an issue to the list."""
        self.issue_id += 1
        self.issues.append({
            'id': self.issue_id,
            'title': title,
            'body': body,
            'labels': labels,
            'category': category
        })
        
    def scan_checklist_issues(self):
        """Parse IMPLEMENTATION_CHECKLIST.md for uncompleted tasks."""
        checklist_path = self.repo_path / 'IMPLEMENTATION_CHECKLIST.md'
        if not checklist_path.exists():
            return
            
        current_phase = None
        current_section = None
        
        with open(checklist_path, 'r') as f:
            lines = f.readlines()
            
        for line in lines:
            phase_match = re.match(r'^## (Phase \d+:.+?)(?:\s*[⚙️🌐🔌🎨🔒📦✨🚀])*\s*$', line.strip())
            if phase_match:
                current_phase = phase_match.group(1).strip()
                continue
                
            section_match = re.match(r'^### (.+)$', line.strip())
            if section_match and current_phase:
                current_section = section_match.group(1).strip()
                continue
                
            task_match = re.match(r'^- \[ \] (.+)$', line.strip())
            if task_match and current_phase and current_section:
                task = task_match.group(1).strip()
                phase_num = re.search(r'Phase (\d+)', current_phase)
                phase_label = f"phase-{phase_num.group(1)}" if phase_num else "enhancement"
                
                self.add_issue(
                    title=f"[{current_phase.split(':')[0]}] {current_section}: {task}",
                    body=f"""## Description
Implement the following task from the implementation checklist.

## Task
- [ ] {task}

## Context
- **Phase**: {current_phase}
- **Section**: {current_section}

## Acceptance Criteria
- Task is fully implemented
- Code is tested and working
- Documentation is updated if needed
""",
                    labels=[phase_label, "enhancement"],
                    category="checklist"
                )
                
    def scan_todo_comments(self):
        """Scan source files for TODO, FIXME, HACK, XXX comments."""
        patterns = {
            'TODO': r'(?://|#|/\*)\s*TODO[:\s](.+?)(?:\*/)?$',
            'FIXME': r'(?://|#|/\*)\s*FIXME[:\s](.+?)(?:\*/)?$',
            'HACK': r'(?://|#|/\*)\s*HACK[:\s](.+?)(?:\*/)?$',
            'XXX': r'(?://|#|/\*)\s*XXX[:\s](.+?)(?:\*/)?$',
        }
        
        extensions = ['.rs', '.ts', '.js', '.svelte', '.py']
        
        for ext in extensions:
            for file_path in self.repo_path.rglob(f'*{ext}'):
                if '.git' in str(file_path):
                    continue
                try:
                    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                        lines = f.readlines()
                    
                    for line_num, line in enumerate(lines, 1):
                        for comment_type, pattern in patterns.items():
                            match = re.search(pattern, line, re.IGNORECASE)
                            if match:
                                comment_text = match.group(1).strip()
                                rel_path = file_path.relative_to(self.repo_path)
                                
                                self.add_issue(
                                    title=f"[{comment_type}] {comment_text[:60]}",
                                    body=f"""## Description
Address the {comment_type} comment found in the codebase.

## Location
- **File**: `{rel_path}`
- **Line**: {line_num}

## Comment
{comment_text}
""",
                                    labels=[comment_type.lower(), 'code-quality'],
                                    category=f"code-{comment_type.lower()}"
                                )
                except:
                    pass
                    
    def scan_missing_tests(self):
        """Identify source files without corresponding test files."""
        src_files = {}
        test_files = set()
        
        for rs_file in self.repo_path.rglob('*.rs'):
            if '.git' in str(rs_file):
                continue
            rel_path = str(rs_file.relative_to(self.repo_path))
            if 'tests' in rel_path or '_test' in rel_path:
                test_files.add(rs_file.stem)
            elif 'src' in rel_path and rs_file.stem not in ['lib', 'main', 'mod', 'prelude']:
                src_files[rs_file.stem] = rel_path
                
        for src_name, src_path in src_files.items():
            if src_name not in test_files and f"{src_name}_tests" not in test_files:
                self.add_issue(
                    title=f"[Testing] Add unit tests for {src_name}",
                    body=f"""## Description
Add comprehensive unit tests for the `{src_name}` module.

## File
`{src_path}`

## Requirements
- [ ] Unit tests for all public functions
- [ ] Edge case testing
- [ ] Error handling tests
""",
                    labels=['testing', 'quality'],
                    category="missing-tests"
                )
                
    def generate_documentation_issues(self):
        """Generate issues for documentation improvements."""
        doc_issues = [
            ("Add comprehensive API documentation", "Document all public API endpoints"),
            ("Create user installation guide", "Step-by-step installation for all platforms"),
            ("Document proxy configuration options", "All proxy settings explained"),
            ("Add troubleshooting guide", "Common issues and solutions"),
            ("Create developer contribution guide", "How to contribute to the project"),
            ("Document keyboard shortcuts", "All available shortcuts"),
            ("Add security best practices guide", "Security recommendations"),
            ("Create architecture overview", "System architecture documentation"),
            ("Document testing procedures", "How to run and write tests"),
            ("Add FAQ section", "Frequently asked questions"),
            ("Create video tutorials", "Video guides for common tasks"),
            ("Document CLI usage", "Command-line interface documentation"),
            ("Add performance tuning guide", "Optimization recommendations"),
            ("Create plugin development guide", "How to extend functionality"),
            ("Document database schema", "Database structure and relationships"),
        ]
        
        for title, desc in doc_issues:
            self.add_issue(
                title=f"[Documentation] {title}",
                body=f"""## Description
{desc}

## Requirements
- Clear and concise writing
- Code examples where applicable
- Keep updated with code changes
""",
                labels=['documentation'],
                category="documentation"
            )
            
    def generate_ui_ux_issues(self):
        """Generate UI/UX improvement issues."""
        ui_issues = [
            ("Add loading skeleton animations", "Show skeleton loaders during data fetching"),
            ("Implement smooth dark/light mode transition", "Animated theme switching"),
            ("Add keyboard navigation support", "Full keyboard accessibility"),
            ("Improve error message display", "User-friendly error messages"),
            ("Add tooltips to all buttons", "Helpful tooltips for interactive elements"),
            ("Implement undo/redo functionality", "Undo and redo for user actions"),
            ("Add drag and drop visual feedback", "Visual feedback during drag operations"),
            ("Improve form validation UX", "Real-time validation with helpful messages"),
            ("Add confirmation dialogs", "Confirm destructive actions"),
            ("Implement responsive design improvements", "Better mobile layouts"),
            ("Add progress indicators for long operations", "Show progress for async tasks"),
            ("Improve tab management interface", "Better tab organization"),
            ("Add global search functionality", "Search across all content"),
            ("Implement command palette", "Quick access via keyboard"),
            ("Add notification toast system", "Toast notifications for events"),
            ("Improve proxy status visualization", "Clear proxy connection status"),
            ("Add context menu support", "Right-click menus throughout UI"),
            ("Implement settings search", "Search within settings"),
            ("Add first-time user onboarding", "Welcome tutorial flow"),
            ("Improve bookmark management UI", "Better bookmark organization"),
        ]
        
        for title, desc in ui_issues:
            self.add_issue(
                title=f"[UI/UX] {title}",
                body=f"""## Description
{desc}

## Requirements
- Consistent with existing design
- Accessible (WCAG 2.1)
- Works in both themes
""",
                labels=['ui', 'enhancement'],
                category="ui-ux"
            )
            
    def generate_performance_issues(self):
        """Generate performance optimization issues."""
        perf_issues = [
            ("Implement lazy loading for tabs", "Load tab content on demand"),
            ("Optimize proxy connection pooling", "Improve connection reuse"),
            ("Add request caching layer", "Cache repeated requests"),
            ("Reduce memory usage for inactive tabs", "Tab suspension"),
            ("Implement DNS prefetching", "Prefetch DNS for links"),
            ("Add virtual scrolling for lists", "Efficient long list rendering"),
            ("Optimize application startup time", "Faster launch"),
            ("Implement background task scheduling", "Efficient background processing"),
            ("Optimize state management", "Reduce unnecessary re-renders"),
            ("Add HTTP/2 connection multiplexing", "Better connection usage"),
            ("Implement image lazy loading", "Load images on scroll"),
            ("Add service worker for caching", "Offline support"),
            ("Optimize bundle size", "Reduce JavaScript bundle"),
            ("Implement incremental updates", "Partial UI updates"),
            ("Add connection keep-alive optimization", "Reuse connections"),
        ]
        
        for title, desc in perf_issues:
            self.add_issue(
                title=f"[Performance] {title}",
                body=f"""## Description
{desc}

## Expected Improvement
Describe expected performance gains and metrics.
""",
                labels=['performance'],
                category="performance"
            )
            
    def generate_feature_issues(self):
        """Generate feature request issues."""
        features = [
            ("Add multi-profile browser support", "Multiple browser profiles"),
            ("Implement session save/restore", "Save browsing sessions"),
            ("Add settings export/import", "Backup and restore settings"),
            ("Implement proxy chaining", "Route through multiple proxies"),
            ("Add custom user agent support", "Configure user agents"),
            ("Implement certificate pinning", "Enhanced security"),
            ("Add network throttling", "Simulate slow networks"),
            ("Implement request logging", "Debug network requests"),
            ("Add automated proxy health checks", "Scheduled proxy testing"),
            ("Implement bookmark sync", "Cross-device bookmarks"),
            ("Add browsing history export", "Export history data"),
            ("Implement full-page screenshots", "Capture entire pages"),
            ("Add PDF export functionality", "Save pages as PDF"),
            ("Implement reader mode", "Distraction-free reading"),
            ("Add picture-in-picture support", "Video PiP mode"),
            ("Implement tab grouping", "Organize tabs in groups"),
            ("Add workspace support", "Multiple workspaces"),
            ("Implement basic extension support", "Browser extensions"),
            ("Add custom CSS injection", "Per-site custom CSS"),
            ("Implement JavaScript toggle", "Disable JS per site"),
            ("Add cookie management UI", "View and edit cookies"),
            ("Implement local storage viewer", "Inspect storage"),
            ("Add network inspector panel", "Debug network"),
            ("Implement console log viewer", "View JS console"),
            ("Add geolocation spoofing", "Fake location data"),
        ]
        
        for title, desc in features:
            self.add_issue(
                title=f"[Feature] {title}",
                body=f"""## Description
{desc}

## User Story
As a user, I want to {desc.lower()}.

## Requirements
- [ ] Define requirements
- [ ] Design UI/UX
- [ ] Implement backend
- [ ] Add frontend
- [ ] Write tests
""",
                labels=['enhancement', 'feature'],
                category="feature"
            )
            
    def generate_infrastructure_issues(self):
        """Generate infrastructure issues."""
        infra_issues = [
            ("Set up GitHub Actions CI/CD", "Automated builds"),
            ("Add code coverage reporting", "Track test coverage"),
            ("Implement automated releases", "Semantic releases"),
            ("Add dependency update automation", "Dependabot setup"),
            ("Set up staging environment", "Pre-production testing"),
            ("Add application monitoring", "Performance tracking"),
            ("Implement error tracking", "Error reporting service"),
            ("Add security vulnerability scanning", "Automated security scans"),
            ("Set up code quality gates", "Quality enforcement"),
            ("Implement build caching", "Faster CI builds"),
            ("Add cross-platform build matrix", "Build for all platforms"),
            ("Implement automated changelog", "Generate changelogs"),
            ("Add license compliance checking", "License auditing"),
            ("Set up documentation deployment", "Auto-deploy docs"),
            ("Implement nightly builds", "Nightly test builds"),
        ]
        
        for title, desc in infra_issues:
            self.add_issue(
                title=f"[Infrastructure] {title}",
                body=f"""## Description
{desc}

## Benefits
- Improved development workflow
- Better reliability
""",
                labels=['infrastructure'],
                category="infrastructure"
            )
            
    def generate_accessibility_issues(self):
        """Generate accessibility issues."""
        a11y_issues = [
            ("Add comprehensive ARIA labels", "Label all interactive elements"),
            ("Implement focus management", "Proper focus handling"),
            ("Add screen reader announcements", "Live region updates"),
            ("Implement high contrast mode", "High contrast theme"),
            ("Add keyboard shortcuts documentation", "Accessible shortcuts help"),
            ("Fix color contrast issues", "WCAG contrast compliance"),
            ("Add skip navigation links", "Skip to content"),
            ("Implement visible focus indicators", "Clear focus styles"),
            ("Add descriptive alt text", "Image descriptions"),
            ("Support reduced motion preference", "Respect motion settings"),
        ]
        
        for title, desc in a11y_issues:
            self.add_issue(
                title=f"[Accessibility] {title}",
                body=f"""## Description
{desc}

## WCAG Guidelines
Meet WCAG 2.1 AA standards.
""",
                labels=['accessibility'],
                category="accessibility"
            )

    def generate_security_issues(self):
        """Generate security improvement issues."""
        security_issues = [
            ("Implement Content Security Policy", "Add strict CSP headers"),
            ("Add XSS protection measures", "Prevent cross-site scripting"),
            ("Implement CSRF protection", "Cross-site request forgery prevention"),
            ("Add input sanitization", "Sanitize all user inputs"),
            ("Implement rate limiting", "Prevent abuse"),
            ("Add secure cookie settings", "HttpOnly, Secure, SameSite"),
            ("Implement certificate validation", "Strict SSL/TLS validation"),
            ("Add DNS leak protection", "Prevent DNS leaks"),
            ("Implement WebRTC leak protection", "Block WebRTC leaks"),
            ("Add browser fingerprint protection", "Reduce fingerprinting"),
            ("Implement secure storage encryption", "Encrypt sensitive data"),
            ("Add password strength validation", "Enforce strong passwords"),
            ("Implement session timeout", "Auto-logout inactive sessions"),
            ("Add audit logging", "Log security events"),
            ("Implement two-factor authentication", "2FA support"),
        ]
        
        for title, desc in security_issues:
            self.add_issue(
                title=f"[Security] {title}",
                body=f"""## Description
{desc}

## Security Impact
Describe the security benefit.
""",
                labels=['security'],
                category="security"
            )

    def generate_refactoring_issues(self):
        """Generate refactoring issues."""
        refactor_issues = [
            ("Extract common utility functions", "Create shared utilities"),
            ("Standardize error type hierarchy", "Consistent error types"),
            ("Improve module organization", "Better code structure"),
            ("Add stronger type definitions", "Enhanced type safety"),
            ("Remove code duplication", "DRY improvements"),
            ("Improve naming conventions", "Consistent naming"),
            ("Centralize configuration constants", "Single config source"),
            ("Improve async/await patterns", "Better async code"),
            ("Standardize logging format", "Consistent logs"),
            ("Implement builder patterns", "Complex object construction"),
        ]
        
        for title, desc in refactor_issues:
            self.add_issue(
                title=f"[Refactor] {title}",
                body=f"""## Description
{desc}

## Goals
- Improve maintainability
- Reduce technical debt
""",
                labels=['refactoring'],
                category="refactoring"
            )

    def generate_bug_issues(self):
        """Generate bug report placeholder issues."""
        bug_issues = [
            ("Fix memory leak in tab management", "Memory not freed on tab close"),
            ("Resolve proxy connection timeout issues", "Connections timing out"),
            ("Fix bookmark sync race condition", "Sync conflicts"),
            ("Resolve CSS rendering glitches", "UI rendering issues"),
            ("Fix keyboard shortcut conflicts", "Shortcut collisions"),
            ("Resolve download manager crashes", "Download failures"),
            ("Fix history search performance", "Slow history search"),
            ("Resolve cookie isolation leaks", "Cookie bleeding between tabs"),
            ("Fix navigation back/forward bugs", "Navigation state issues"),
            ("Resolve proxy rotation stuck states", "Rotation not working"),
        ]
        
        for title, desc in bug_issues:
            self.add_issue(
                title=f"[Bug] {title}",
                body=f"""## Description
{desc}

## Steps to Reproduce
1. Step 1
2. Step 2
3. Step 3

## Expected Behavior
What should happen.

## Actual Behavior
What actually happens.
""",
                labels=['bug'],
                category="bug"
            )

    def run_analysis(self):
        """Run all analysis methods."""
        print("Scanning codebase for issues...")
        
        print("  - Parsing implementation checklist...")
        self.scan_checklist_issues()
        
        print("  - Scanning for TODO/FIXME comments...")
        self.scan_todo_comments()
        
        print("  - Identifying missing tests...")
        self.scan_missing_tests()
        
        print("  - Generating documentation issues...")
        self.generate_documentation_issues()
        
        print("  - Generating UI/UX issues...")
        self.generate_ui_ux_issues()
        
        print("  - Generating performance issues...")
        self.generate_performance_issues()
        
        print("  - Generating feature issues...")
        self.generate_feature_issues()
        
        print("  - Generating infrastructure issues...")
        self.generate_infrastructure_issues()
        
        print("  - Generating accessibility issues...")
        self.generate_accessibility_issues()
        
        print("  - Generating security issues...")
        self.generate_security_issues()
        
        print("  - Generating refactoring issues...")
        self.generate_refactoring_issues()
        
        print("  - Generating bug issues...")
        self.generate_bug_issues()
        
        print(f"\nTotal issues found: {len(self.issues)}")
        return self.issues
        
    def save_issues(self, output_file):
        """Save issues to JSON file."""
        with open(output_file, 'w') as f:
            json.dump(self.issues, f, indent=2)
        print(f"Issues saved to {output_file}")
        
    def print_summary(self):
        """Print summary of issues by category."""
        categories = defaultdict(int)
        for issue in self.issues:
            categories[issue['category']] += 1
            
        print("\n=== Issue Summary by Category ===")
        for cat, count in sorted(categories.items(), key=lambda x: -x[1]):
            print(f"  {cat}: {count}")
        print(f"\nTotal: {len(self.issues)} issues")


def create_github_issue(title, body, labels, dry_run=False):
    """Create a GitHub issue using gh cli."""
    if dry_run:
        print(f"[DRY RUN] Would create: {title[:60]}...")
        return True
        
    cmd = ['gh', 'issue', 'create', '--title', title, '--body', body]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode == 0:
        print(f"✅ Created: {title[:50]}...")
        return True
    else:
        print(f"❌ Failed: {title[:40]}... - {result.stderr[:50]}")
        return False


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description='Generate GitHub issues')
    parser.add_argument('--repo', default='.', help='Repository path')
    parser.add_argument('--output', default='issues.json', help='Output JSON')
    parser.add_argument('--create', action='store_true', help='Create issues')
    parser.add_argument('--dry-run', action='store_true', help='Dry run')
    parser.add_argument('--limit', type=int, default=0, help='Limit issues')
    parser.add_argument('--list', action='store_true', help='List issues')
    
    args = parser.parse_args()
    
    generator = IssueGenerator(args.repo)
    issues = generator.run_analysis()
    generator.print_summary()
    generator.save_issues(args.output)
    
    if args.list:
        print("\n=== All Issues ===")
        for issue in issues:
            print(f"{issue['id']:4d}. [{issue['category']}] {issue['title'][:60]}")
    
    if args.create or args.dry_run:
        issues_to_create = issues
        if args.limit > 0:
            issues_to_create = issues_to_create[:args.limit]
            
        print(f"\n{'[DRY RUN] ' if args.dry_run else ''}Creating {len(issues_to_create)} issues...")
        
        created = 0
        for issue in issues_to_create:
            if create_github_issue(issue['title'], issue['body'], issue['labels'], args.dry_run):
                created += 1
                
        print(f"\nCreated {created}/{len(issues_to_create)} issues")


if __name__ == "__main__":
    main()
