/**
 * Test Suite for Advanced Technology Focus Lock System
 * Validates the Pro Expert Advanced Technology Programming Professional Developer Mode
 */

const AdvancedTechnologyFocusLock = require('./focus-lock.js');

// Test helper
function assert(condition, message) {
  if (!condition) {
    throw new Error(`Assertion failed: ${message}`);
  }
  console.log(`✓ ${message}`);
}

// Run tests
function runTests() {
  console.log('=== Advanced Technology Focus Lock Test Suite ===\n');

  // Test 1: Initial state should be locked
  console.log('Test 1: Initial State');
  const lock1 = new AdvancedTechnologyFocusLock();
  assert(lock1.isLocked() === true, 'Initial state should be locked');
  assert(lock1.getCurrentMode() === 'Pro Expert Advanced Technology Programming Professional Developer', 
         'Mode should be Pro Expert Advanced Technology Programming Professional Developer');
  assert(lock1.getTechnologyLevel() === 'Advanced Technology', 
         'Technology level should be Advanced Technology');
  assert(lock1.getExpertiseLevel() === 'Expert', 
         'Expertise level should be Expert');

  // Test 2: Wrong unlock keyword should fail
  console.log('\nTest 2: Wrong Unlock Keyword');
  const lock2 = new AdvancedTechnologyFocusLock();
  const result1 = lock2.processInput('Unlock');
  assert(result1.action === 'continue', 'Wrong keyword should continue in locked mode');
  assert(lock2.isLocked() === true, 'Should remain locked with wrong keyword');
  
  const result2 = lock2.processInput('UNLOCK');
  assert(lock2.isLocked() === true, 'Should remain locked with uppercase keyword');

  // Test 3: Correct unlock keyword should work
  console.log('\nTest 3: Correct Unlock Keyword');
  const lock3 = new AdvancedTechnologyFocusLock();
  const result3 = lock3.processInput('unlock');
  assert(result3.action === 'unlock_attempt', 'Should attempt unlock');
  assert(result3.success === true, 'Should successfully unlock');
  assert(lock3.isLocked() === false, 'Should be unlocked after correct keyword');

  // Test 4: Direct unlock method
  console.log('\nTest 4: Direct Unlock Method');
  const lock4 = new AdvancedTechnologyFocusLock();
  const unlocked = lock4.unlock('unlock');
  assert(unlocked === true, 'Direct unlock with correct keyword should return true');
  assert(lock4.isLocked() === false, 'Should be unlocked after direct unlock');

  // Test 5: Lock after unlock
  console.log('\nTest 5: Re-lock After Unlock');
  const lock5 = new AdvancedTechnologyFocusLock();
  lock5.unlock('unlock');
  assert(lock5.isLocked() === false, 'Should be unlocked');
  lock5.lock();
  assert(lock5.isLocked() === true, 'Should be locked again after lock()');

  // Test 6: Status information
  console.log('\nTest 6: Status Information');
  const lock6 = new AdvancedTechnologyFocusLock();
  const status = lock6.getStatus();
  assert(status.locked === true, 'Status should show locked');
  assert(status.mode === 'Pro Expert Advanced Technology Programming Professional Developer', 
         'Status should contain correct mode');
  assert(status.technologyLevel === 'Advanced Technology', 
         'Status should contain correct technology level');
  assert(status.unlockKeyword === 'unlock', 
         'Status should contain unlock keyword');

  // Test 7: Input with whitespace
  console.log('\nTest 7: Input with Whitespace');
  const lock7 = new AdvancedTechnologyFocusLock();
  const result7 = lock7.processInput('  unlock  ');
  assert(result7.success === true, 'Should unlock with whitespace around keyword');
  assert(lock7.isLocked() === false, 'Should be unlocked with whitespace trimmed');

  console.log('\n=== All Tests Passed ✓ ===');
}

// Run the test suite
try {
  runTests();
  process.exit(0);
} catch (error) {
  console.error(`\n✗ Test failed: ${error.message}`);
  process.exit(1);
}
