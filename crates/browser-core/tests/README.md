# Browser Core Tests

This directory contains comprehensive tests for the `browser-core` crate, including unit tests, integration tests, and end-to-end tests with real browser instances.

## Test Structure

```
tests/
├── common/                    # Shared test utilities
│   └── mod.rs                # Helper functions and macros
├── chromium_config_tests.rs   # Configuration struct tests
├── chromium_tab_tests.rs      # Tab operation tests
├── chromium_engine_tests.rs   # Engine lifecycle tests
├── engine_manager_tests.rs    # Engine manager tests
├── chromium_integration_tests.rs  # Integration tests (browser required)
└── README.md                  # This file
```

## Test Categories

### 1. Unit Tests (140 tests)
Fast tests that don't require a real browser. Run automatically in CI/CD.

**Files:**
- `chromium_config_tests.rs` - 18 tests (Configuration validation)
- `chromium_tab_tests.rs` - 17 tests (Tab operations)
- `chromium_engine_tests.rs` - 34 tests (Engine lifecycle)
- `engine_manager_tests.rs` - 17 tests (Engine management)
- `chromium_features_tests.rs` - 9 tests (Feature validation)
- `network_privacy_tests.rs` - 20 tests (Network and privacy)
- `security_tests.rs` - 18 tests (Security features)
- `stability_tests.rs` - 13 tests (Stability and recovery)
- `tab_lifecycle.rs` - 4 tests (Tab lifecycle states)

**Run unit tests:**
```bash
cargo test --lib
cargo test --test chromium_config_tests
cargo test --test chromium_tab_tests
cargo test --test chromium_engine_tests
cargo test --test engine_manager_tests
cargo test --test chromium_features_tests
cargo test --test network_privacy_tests
cargo test --test security_tests
cargo test --test stability_tests
cargo test --test tab_lifecycle
```

### 2. Integration Tests (16 tests)
Tests that launch a real Chromium browser. Marked with `#[ignore]` to avoid CI failures.

**File:** `chromium_integration_tests.rs`

**Requirements:**
- Chrome or Chromium must be installed on your system
- Should be run with `--test-threads=1` to avoid resource conflicts

**Run integration tests:**
```bash
# Run all integration tests
cargo test --test chromium_integration_tests -- --ignored --test-threads=1

# Run specific integration test
cargo test --test chromium_integration_tests test_browser_launch_and_shutdown -- --ignored

# Run with verbose output
cargo test --test chromium_integration_tests -- --ignored --test-threads=1 --nocapture
```

## Running All Tests

```bash
# Run all unit tests
cargo test --lib --tests

# Run all tests including integration tests (requires Chrome)
cargo test --tests -- --include-ignored --test-threads=1
```

## Environment Variables

### Chrome/Chromium Path
If Chrome/Chromium is not in a standard location, you can set:
```bash
# Not currently implemented, but can be added if needed
export CHROME_PATH=/path/to/chrome
```

### Test Timeouts
```bash
# Not currently implemented, but can be added if needed
export BROWSER_LAUNCH_TIMEOUT=60  # seconds
```

## Chrome/Chromium Installation

### Windows
- Download from: https://www.google.com/chrome/
- Or install Chromium via Chocolatey: `choco install chromium`

### macOS
- Download from: https://www.google.com/chrome/
- Or install via Homebrew: `brew install --cask google-chrome`

### Linux (Ubuntu/Debian)
```bash
# Chrome
wget https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
sudo dpkg -i google-chrome-stable_current_amd64.deb

# Or Chromium
sudo apt-get install chromium-browser
```

### Linux (Fedora/RHEL)
```bash
sudo dnf install chromium
```

## Test Coverage

### Configuration Tests (18 tests)
- ✅ Enum defaults and variants
- ✅ Network condition calculations
- ✅ Fingerprint configuration
- ✅ Proxy authentication
- ✅ Serialization/deserialization

### Tab Operation Tests (17 tests)
- ✅ Tab creation and management
- ✅ State tracking and updates
- ✅ Proxy assignment per tab
- ✅ Unicode and special URL support

### Engine Tests (34 tests)
- ✅ Engine initialization and lifecycle
- ✅ Configuration management
- ✅ Error handling
- ✅ Concurrent operations
- ✅ All feature configurations

### Engine Manager Tests (17 tests)
- ✅ Engine type switching
- ✅ Configuration updates
- ✅ Proxy management
- ✅ Concurrent config operations

### Integration Tests (19 tests)
- ✅ Real browser launch/shutdown
- ✅ Tab creation and navigation
- ✅ Multiple tabs management
- ✅ Custom viewport and user agent
- ✅ Stealth mode and WebRTC protection
- ✅ DNS over HTTPS
- ✅ Concurrent operations

## Troubleshooting

### Integration tests fail with "Chrome not found"
**Solution:** Install Chrome/Chromium (see installation section above)

### Integration tests timeout
**Possible causes:**
1. System is slow or resource-constrained
2. Firewall blocking browser connections
3. Antivirus interfering with browser launch

**Solutions:**
- Increase timeout in test code (edit `launch_browser_with_timeout`)
- Disable antivirus temporarily
- Run with fewer concurrent tests: `--test-threads=1`

### Tests fail with "Address already in use"
**Cause:** Multiple browser instances trying to use same debugging port

**Solution:** Run with `--test-threads=1` to serialize test execution

### Headless mode issues on Linux
**Cause:** Missing dependencies for headless Chrome

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install -y \
    libnss3 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libdrm2 \
    libxkbcommon0 \
    libxcomposite1 \
    libxdamage1 \
    libxrandr2 \
    libgbm1 \
    libasound2
```

## Writing New Tests

### Unit Test Example
```rust
#[tokio::test]
async fn test_my_feature() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    // Test without launching browser
    assert!(!engine.is_running().await);
}
```

### Integration Test Example
```rust
#[tokio::test]
#[ignore] // Don't run in CI
async fn test_my_browser_feature() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Test with real browser
    // ...
    
    shutdown_browser(engine).await.unwrap();
}
```

## CI/CD Configuration

**Unit tests** run automatically in CI/CD pipelines.

**Integration tests** are skipped by default (marked with `#[ignore]`). To run them in CI:

```yaml
# GitHub Actions example
- name: Run integration tests
  run: cargo test --test chromium_integration_tests -- --ignored --test-threads=1
  # Only run if Chrome is available
  if: steps.check-chrome.outputs.available == 'true'
```

## Performance Considerations

- **Unit tests:** Very fast (~1-2 seconds total)
- **Integration tests:** Slower (~30-60 seconds per test due to browser launch)
- **Recommendation:** Run unit tests frequently, integration tests before commits

## Test Maintenance

When adding new features to `ChromiumEngine`:

1. **Add unit tests** in appropriate test file
2. **Add integration test** if feature requires real browser
3. **Update this README** if new test categories are added
4. **Keep tests isolated** - each test should clean up after itself

## Code Coverage

To generate code coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage (unit tests only)
cargo tarpaulin --out Html --output-dir coverage

# Generate coverage including integration tests
cargo tarpaulin --out Html --output-dir coverage --run-ignored -- --test-threads=1
```

## Contact

For questions or issues with tests, please open an issue in the repository.
