<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { loginUser } from '../../lib/api';
  import LoadingSpinner from '../ui/LoadingSpinner.svelte';
  
  export let onRegister: () => void;
  export let onLoginSuccess: (tokens: { accessToken: string; refreshToken: string }) => void;
  
  const dispatch = createEventDispatcher();
  
  let username = '';
  let password = '';
  let loading = false;
  let error = '';
  
  async function handleLogin() {
    if (!username || !password) {
      error = 'Please enter username and password';
      return;
    }
    
    loading = true;
    error = '';
    
    try {
      const tokens = await loginUser(username, password);
      onLoginSuccess(tokens);
      dispatch('loginSuccess', { tokens });
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Login failed';
    } finally {

      loading = false;
    }
  }
  
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleLogin();
    }
  }
</script>

<div class="login-form">
  <div class="form-header">
    <h2>Sign In</h2>
    <p>Access your Virtual IP Browser</p>
  </div>
  
  <form on:submit|preventDefault={handleLogin}>
    <div class="form-group">
      <label for="username">Username</label>
      <input
        id="username"
        type="text"
        bind:value={username}
        placeholder="Enter your username"
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
        placeholder="Enter your password"
        on:keydown={handleKeydown}
        disabled={loading}
        required
      />
    </div>
    
    {#if error}
      <div class="error-message">{error}</div>
    {/if}
    
    <button type="submit" class="login-btn" disabled={loading}>
      {#if loading}
        <LoadingSpinner size="small" />
        Signing In...
      {:else}
        Sign In
      {/if}
    </button>
  </form>
  
  <div class="form-footer">
    <p>Don't have an account? 
      <button type="button" class="link-btn" on:click={onRegister}>
        Sign Up
      </button>
    </p>
  </div>
</div>

<style>
  .login-form {
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
  
  .login-btn {
    width: 100%;
    padding: 12px;
    background: #3b82f6;
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
  
  .login-btn:hover:not(:disabled) {
    background: #2563eb;
  }
  
  .login-btn:disabled {
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
