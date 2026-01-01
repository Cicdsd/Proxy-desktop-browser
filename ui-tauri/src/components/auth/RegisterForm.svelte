<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { registerUser } from '../../lib/api';
  import LoadingSpinner from '../ui/LoadingSpinner.svelte';
  
  export let onLogin: () => void;
  export let onRegisterSuccess: () => void;
  
  const dispatch = createEventDispatcher();
  
  let username = '';
  let email = '';
  let password = '';
  let confirmPassword = '';
  let loading = false;
  let error = '';
  
  async function handleRegister() {
    // Validation
    if (!username || !email || !password || !confirmPassword) {
      error = 'Please fill in all fields';
      return;
    }
    
    if (username.length < 3) {
      error = 'Username must be at least 3 characters';
      return;
    }
    
    if (!email.includes('@') || !email.includes('.')) {
      error = 'Please enter a valid email address';
      return;
    }
    
    if (password.length < 8) {
      error = 'Password must be at least 8 characters';
      return;
    }
    
    if (password !== confirmPassword) {
      error = 'Passwords do not match';
      return;
    }
    
    loading = true;
    error = '';
    
    try {
      await registerUser(username, email, password);
      onRegisterSuccess();
      dispatch('registerSuccess');
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Registration failed';
    } finally {

      loading = false;
    }
  }
  
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleRegister();
    }
  }
</script>

<div class="register-form">
  <div class="form-header">
    <h2>Create Account</h2>
    <p>Join Virtual IP Browser</p>
  </div>
  
  <form on:submit|preventDefault={handleRegister}>
    <div class="form-group">
      <label for="username">Username</label>
      <input
        id="username"
        type="text"
        bind:value={username}
        placeholder="Choose a username"
        on:keydown={handleKeydown}
        disabled={loading}
        required
        minlength="3"
      />
    </div>
    
    <div class="form-group">
      <label for="email">Email</label>
      <input
        id="email"
        type="email"
        bind:value={email}
        placeholder="Enter your email"
        on:keydown={handleKeydown}
        disabled={loading}
        required
      />
    </div>
    
    <div class="form-group">
      <label for="password">Password</label>
      <input
        id="password"
        type="password"
        bind:value={password}
        placeholder="Create a strong password"
        on:keydown={handleKeydown}
        disabled={loading}
        required
        minlength="8"
      />
    </div>
    
    <div class="form-group">
      <label for="confirmPassword">Confirm Password</label>
      <input
        id="confirmPassword"
        type="password"
        bind:value={confirmPassword}
        placeholder="Confirm your password"
        on:keydown={handleKeydown}
        disabled={loading}
        required
        minlength="8"
      />
    </div>
    
    {#if error}
      <div class="error-message">{error}</div>
    {/if}
    
    <button type="submit" class="register-btn" disabled={loading}>
      {#if loading}
        <LoadingSpinner size="small" />
        Creating Account...
      {:else}
        Create Account
      {/if}
    </button>
  </form>
  
  <div class="form-footer">
    <p>Already have an account? 
      <button type="button" class="link-btn" on:click={onLogin}>
        Sign In
      </button>
    </p>
  </div>
</div>

<style>
  .register-form {
    background: #151d2e;
    border: 1px solid #1f2a45;
    border-radius: 12px;
    padding: 32px;
    width: 100%;
    max-width: 400px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }
  
  .form-header {
    text-align: center;
    margin-bottom: 24px;
  }
  
  .form-header h2 {
    color: #e0e7f5;
    font-size: 24px;
    font-weight: 600;
    margin: 0 0 8px;
  }
  
  .form-header p {
    color: #9fb0ce;
    font-size: 14px;
    margin: 0;
  }
  
  .form-group {
    margin-bottom: 20px;
  }
  
  .form-group label {
    display: block;
    color: #c7d4ec;
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 8px;
  }
  
  .form-group input {
    width: 100%;
    padding: 12px 16px;
    background: #0c1120;
    border: 1px solid #2a3750;
    border-radius: 8px;
    color: #e0e7f5;
    font-size: 14px;
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  
  .form-group input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }
  
  .form-group input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .error-message {
    background: #3a1a2c;
    border: 1px solid #ff5c8a;
    color: #ffb3c8;
    padding: 12px;
    border-radius: 8px;
    font-size: 13px;
    margin-bottom: 20px;
  }
  
  .register-btn {
    width: 100%;
    padding: 12px;
    background: #10b981;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }
  
  .register-btn:hover:not(:disabled) {
    background: #059669;
  }
  
  .register-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }
  
  .form-footer {
    text-align: center;
    margin-top: 24px;
    padding-top: 24px;
    border-top: 1px solid #2a3750;
  }
  
  .form-footer p {
    color: #9fb0ce;
    font-size: 14px;
    margin: 0;
  }
  
  .link-btn {
    background: none;
    border: none;
    color: #3b82f6;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    text-decoration: underline;
    padding: 0;
    margin-left: 4px;
  }
  
  .link-btn:hover {
    color: #2563eb;
  }
</style>
