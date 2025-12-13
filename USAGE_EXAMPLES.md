# Advanced Technology Focus Lock - Usage Examples

## Overview
This document provides practical examples of using the Advanced Technology Focus Lock system in both JavaScript and Python implementations.

## JavaScript Examples

### Basic Usage
```javascript
const AdvancedTechnologyFocusLock = require('./focus-lock.js');

// Create a new focus lock instance
const focusLock = new AdvancedTechnologyFocusLock();

// Check if locked
console.log(focusLock.isLocked()); // true

// Get current mode
console.log(focusLock.getCurrentMode());
// Output: "Pro Expert Advanced Technology Programming Professional Developer"

// Get status
console.log(focusLock.getStatus());
```

### Attempting to Unlock

```javascript
const focusLock = new AdvancedTechnologyFocusLock();

// Try with wrong keyword (case-sensitive)
let result = focusLock.processInput('Unlock');
console.log(result);
// Output: { action: 'continue', focusMode: true, message: '...' }
console.log(focusLock.isLocked()); // Still true

// Use correct keyword
result = focusLock.processInput('unlock');
console.log(result);
// Output: { action: 'unlock_attempt', success: true, message: 'Focus mode UNLOCKED successfully' }
console.log(focusLock.isLocked()); // Now false
```

### Direct Lock/Unlock

```javascript
const focusLock = new AdvancedTechnologyFocusLock();

// Direct unlock
const unlocked = focusLock.unlock('unlock');
console.log(unlocked); // true
console.log(focusLock.isLocked()); // false

// Re-lock
focusLock.lock();
console.log(focusLock.isLocked()); // true
```

## Python Examples

### Basic Usage
```python
from focus_lock import AdvancedTechnologyFocusLock

# Create a new focus lock instance
focus_lock = AdvancedTechnologyFocusLock()

# Check if locked
print(focus_lock.is_locked())  # True

# Get current mode
print(focus_lock.get_current_mode())
# Output: "Pro Expert Advanced Technology Programming Professional Developer"

# Get status
print(focus_lock.get_status())
```

### Attempting to Unlock

```python
focus_lock = AdvancedTechnologyFocusLock()

# Try with wrong keyword (case-sensitive)
result = focus_lock.process_input('Unlock')
print(result)
# Output: {'action': 'continue', 'focus_mode': True, 'message': '...'}
print(focus_lock.is_locked())  # Still True

# Use correct keyword
result = focus_lock.process_input('unlock')
print(result)
# Output: {'action': 'unlock_attempt', 'success': True, 'message': 'Focus mode UNLOCKED successfully'}
print(focus_lock.is_locked())  # Now False
```

### Direct Lock/Unlock

```python
focus_lock = AdvancedTechnologyFocusLock()

# Direct unlock
unlocked = focus_lock.unlock('unlock')
print(unlocked)  # True
print(focus_lock.is_locked())  # False

# Re-lock
focus_lock.lock()
print(focus_lock.is_locked())  # True
```

## Command Line Usage

### Run JavaScript Demo
```bash
node focus-lock.js
```

### Run Python Demo
```bash
python focus-lock.py
```

### Run Tests
```bash
# JavaScript tests
node test-focus-lock.js

# Python tests
python test-focus-lock.py
```

## Important Notes

### Unlock Keyword Requirements
- **Exact match required**: `unlock` (lowercase)
- **Case-sensitive**: `Unlock`, `UNLOCK`, `UnLock` will NOT work
- **No variations**: `un-lock`, `un lock` will NOT work
- **Whitespace is trimmed**: `  unlock  ` will work

### Focus Mode Characteristics
When LOCKED (default state):
- ✅ Pro Expert Advanced Technology Programming Professional Developer mode active
- ✅ Advanced Technology methodologies enforced
- ✅ Expert-level programming standards maintained
- ✅ Professional development practices applied

When UNLOCKED:
- ℹ️ Standard mode active
- ℹ️ Focus restrictions lifted
- ℹ️ Can be re-locked using `lock()` method

## Configuration

The system can be configured via `config.json`:

```json
{
  "focusMode": {
    "enabled": true,
    "type": "Advanced Technology Professional",
    "locked": true,
    "unlockKeyword": "unlock"
  },
  "expertise": {
    "level": "Pro Expert Advanced",
    "technology": "Advanced Technology",
    "experience": "Expert-level programming experience"
  }
}
```

## Integration Examples

### Web Application Integration
```javascript
const express = require('express');
const AdvancedTechnologyFocusLock = require('./focus-lock.js');

const app = express();
const focusLock = new AdvancedTechnologyFocusLock();

app.post('/api/unlock', (req, res) => {
  const result = focusLock.processInput(req.body.keyword);
  res.json(result);
});

app.get('/api/status', (req, res) => {
  res.json(focusLock.getStatus());
});
```

### CLI Tool Integration
```python
import sys
from focus_lock import AdvancedTechnologyFocusLock

def main():
    focus_lock = AdvancedTechnologyFocusLock()
    
    print("Advanced Technology Focus Lock CLI")
    print(focus_lock.get_status())
    
    while focus_lock.is_locked():
        user_input = input("\nEnter command (or 'unlock' to unlock): ")
        result = focus_lock.process_input(user_input)
        print(result['message'])
    
    print("Focus mode unlocked. Exiting...")

if __name__ == '__main__':
    main()
```

## Testing Your Implementation

Both implementations include comprehensive test suites that validate:
1. Initial locked state
2. Wrong keyword rejection
3. Correct keyword acceptance
4. Direct unlock/lock methods
5. Status information accuracy
6. Whitespace handling

Run the tests to ensure proper functionality:
```bash
# JavaScript
node test-focus-lock.js

# Python
python test-focus-lock.py
```

Expected output: All tests should pass with ✓ indicators.
