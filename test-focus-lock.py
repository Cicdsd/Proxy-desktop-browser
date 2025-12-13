"""
Test Suite for Advanced Technology Focus Lock System
Validates the Pro Expert Advanced Technology Programming Professional Developer Mode
"""

import sys
import os

# Add current directory to path for importing focus-lock module
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Import with hyphenated filename
focus_lock_module = __import__('focus-lock')
AdvancedTechnologyFocusLock = focus_lock_module.AdvancedTechnologyFocusLock


def assert_equal(actual, expected, message):
    """Test assertion helper."""
    if actual != expected:
        raise AssertionError(f"{message}\n  Expected: {expected}\n  Got: {actual}")
    print(f"✓ {message}")


def run_tests():
    """Run the test suite."""
    print('=== Advanced Technology Focus Lock Test Suite ===\n')

    # Test 1: Initial state should be locked
    print('Test 1: Initial State')
    lock1 = AdvancedTechnologyFocusLock()
    assert_equal(lock1.is_locked(), True, 'Initial state should be locked')
    assert_equal(lock1.get_current_mode(), 
                 'Pro Expert Advanced Technology Programming Professional Developer',
                 'Mode should be Pro Expert Advanced Technology Programming Professional Developer')
    assert_equal(lock1.get_technology_level(), 'Advanced Technology',
                 'Technology level should be Advanced Technology')
    assert_equal(lock1.get_expertise_level(), 'Expert',
                 'Expertise level should be Expert')

    # Test 2: Wrong unlock keyword should fail
    print('\nTest 2: Wrong Unlock Keyword')
    lock2 = AdvancedTechnologyFocusLock()
    result1 = lock2.process_input('Unlock')
    assert_equal(result1['action'], 'continue', 
                 'Wrong keyword should continue in locked mode')
    assert_equal(lock2.is_locked(), True, 
                 'Should remain locked with wrong keyword')
    
    result2 = lock2.process_input('UNLOCK')
    assert_equal(lock2.is_locked(), True, 
                 'Should remain locked with uppercase keyword')

    # Test 3: Correct unlock keyword should work
    print('\nTest 3: Correct Unlock Keyword')
    lock3 = AdvancedTechnologyFocusLock()
    result3 = lock3.process_input('unlock')
    assert_equal(result3['action'], 'unlock_attempt', 'Should attempt unlock')
    assert_equal(result3['success'], True, 'Should successfully unlock')
    assert_equal(lock3.is_locked(), False, 
                 'Should be unlocked after correct keyword')

    # Test 4: Direct unlock method
    print('\nTest 4: Direct Unlock Method')
    lock4 = AdvancedTechnologyFocusLock()
    unlocked = lock4.unlock('unlock')
    assert_equal(unlocked, True, 
                 'Direct unlock with correct keyword should return true')
    assert_equal(lock4.is_locked(), False, 
                 'Should be unlocked after direct unlock')

    # Test 5: Lock after unlock
    print('\nTest 5: Re-lock After Unlock')
    lock5 = AdvancedTechnologyFocusLock()
    lock5.unlock('unlock')
    assert_equal(lock5.is_locked(), False, 'Should be unlocked')
    lock5.lock()
    assert_equal(lock5.is_locked(), True, 'Should be locked again after lock()')

    # Test 6: Status information
    print('\nTest 6: Status Information')
    lock6 = AdvancedTechnologyFocusLock()
    status = lock6.get_status()
    assert_equal(status['locked'], True, 'Status should show locked')
    assert_equal(status['mode'], 
                 'Pro Expert Advanced Technology Programming Professional Developer',
                 'Status should contain correct mode')
    assert_equal(status['technology_level'], 'Advanced Technology',
                 'Status should contain correct technology level')
    assert_equal(status['unlock_keyword'], 'unlock',
                 'Status should contain unlock keyword')

    # Test 7: Input with whitespace
    print('\nTest 7: Input with Whitespace')
    lock7 = AdvancedTechnologyFocusLock()
    result7 = lock7.process_input('  unlock  ')
    assert_equal(result7['success'], True, 
                 'Should unlock with whitespace around keyword')
    assert_equal(lock7.is_locked(), False, 
                 'Should be unlocked with whitespace trimmed')

    print('\n=== All Tests Passed ✓ ===')


if __name__ == '__main__':
    try:
        run_tests()
        sys.exit(0)
    except AssertionError as error:
        print(f'\n✗ Test failed: {error}')
        sys.exit(1)
