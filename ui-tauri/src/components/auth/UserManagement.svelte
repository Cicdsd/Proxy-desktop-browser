<script lang="ts">
  import { onMount } from 'svelte';
  import { getContext } from 'svelte';
  import { promoteUserToAdmin, createEnterpriseUser, fetchUsers } from '../../lib/api';

  import LoadingSpinner from '../ui/LoadingSpinner.svelte';
  import type { AuthContext } from './AuthWrapper.svelte';
  import type { User } from '../../lib/types';
  
  const auth: AuthContext = getContext('auth');
  
  let users: User[] = [];
  let loading = false;
  let error = '';
  let showCreateUser = false;
  let newUsername = '';
  let newEmail = '';
  let newPassword = '';
  let newEnterpriseId = '';
  let creatingUser = false;
  
  // Fetch users on component mount
  onMount(async () => {
    loading = true;
    try {
      users = await fetchUsers();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to fetch users';
    } finally {
      loading = false;
    }
  });

  
  async function handlePromoteToAdmin(userId: string, username: string) {
    if (!confirm(`Are you sure you want to promote ${username} to admin?`)) {
      return;
    }
    
    loading = true;
    error = '';
    
    try {
      await promoteUserToAdmin(userId);
      
      // Update local state
      const user = users.find(u => u.id === userId);
      if (user) {
        user.role = 'Admin';
        users = [...users];
      }
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to promote user';
    } finally {

      loading = false;
    }
  }
  
  async function handleCreateEnterpriseUser() {
    if (!newUsername || !newEmail || !newPassword || !newEnterpriseId) {
      error = 'Please fill in all fields';
      return;
    }
    
    creatingUser = true;
    error = '';
    
    try {
      const newUser = await createEnterpriseUser(
        newUsername,
        newEmail,
        newPassword,
        newEnterpriseId
      );
      
      users = [...users, newUser];
      
      // Reset form
      newUsername = '';
      newEmail = '';
      newPassword = '';
      newEnterpriseId = '';
      showCreateUser = false;
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to create user';
    } finally {

      creatingUser = false;
    }
  }
  
  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString();
  }
  
  function getRoleBadgeClass(role: string) {
    switch (role) {
      case 'Admin':
        return 'admin';
      case 'Enterprise':
        return 'enterprise';
      default:
        return 'user';
    }
  }
</script>

<div class="user-management">
  <div class="header">
    <h2>User Management</h2>
    <button class="create-btn" on:click={() => showCreateUser = true}>
      + Create Enterprise User
    </button>
  </div>
  
  {#if error}
    <div class="error-message">{error}</div>
  {/if}
  
  {#if showCreateUser}
    <div class="create-user-form">
      <h3>Create Enterprise User</h3>
      <form on:submit|preventDefault={handleCreateEnterpriseUser}>
        <div class="form-row">
          <div class="form-group">
            <label>Username</label>
            <input type="text" bind:value={newUsername} required />
          </div>
          <div class="form-group">
            <label>Email</label>
            <input type="email" bind:value={newEmail} required />
          </div>
        </div>
        <div class="form-row">
          <div class="form-group">
            <label>Password</label>
            <input type="password" bind:value={newPassword} required minlength="8" />
          </div>
          <div class="form-group">
            <label>Enterprise ID</label>
            <input type="text" bind:value={newEnterpriseId} required />
          </div>
        </div>
        <div class="form-actions">
          <button type="submit" class="submit-btn" disabled={creatingUser}>
            {#if creatingUser}
              <LoadingSpinner size="small" />
              Creating...
            {:else}
              Create User
            {/if}
          </button>
          <button type="button" class="cancel-btn" on:click={() => showCreateUser = false}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  {/if}
  
  <div class="users-table">
    <table>
      <thead>
        <tr>
          <th>Username</th>
          <th>Email</th>
          <th>Role</th>
          <th>Enterprise</th>
          <th>Created</th>
          <th>Last Login</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each users as user (user.id)}
          <tr>
            <td>{user.username}</td>
            <td>{user.email}</td>
            <td>
              <span class="role-badge {getRoleBadgeClass(user.role)}">
                {user.role}
              </span>
            </td>
            <td>{user.enterprise_id || '-'}</td>
            <td>{formatDate(user.created_at)}</td>
            <td>{user.last_login ? formatDate(user.last_login) : 'Never'}</td>
            <td>
              {#if user.role === 'User'}
                <button 
                  class="action-btn promote"
                  on:click={() => handlePromoteToAdmin(user.id, user.username)}
                  disabled={loading}
                >
                  Promote to Admin
                </button>
              {:else if user.role === 'Enterprise'}
                <button 
                  class="action-btn promote"
                  on:click={() => handlePromoteToAdmin(user.id, user.username)}
                  disabled={loading}
                >
                  Promote to Admin
                </button>
              {:else}
                <span class="action-text">Admin</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    
    {#if users.length === 0}
      <div class="empty-state">
        <p>No users found</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .user-management {
    padding: 24px;
    background: #0c1120;
    border-radius: 12px;
    height: 100%;
    overflow-y: auto;
  }
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
  }
  
  .header h2 {
    color: #e0e7f5;
    font-size: 20px;
    font-weight: 600;
    margin: 0;
  }
  
  .create-btn {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .create-btn:hover {
    background: #2563eb;
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
  
  .create-user-form {
    background: #151d2e;
    border: 1px solid #2a3750;
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 24px;
  }
  
  .create-user-form h3 {
    color: #e0e7f5;
    font-size: 16px;
    font-weight: 600;
    margin: 0 0 16px;
  }
  
  .form-row {
    display: flex;
    gap: 16px;
    margin-bottom: 16px;
  }
  
  .form-group {
    flex: 1;
  }
  
  .form-group label {
    display: block;
    color: #c7d4ec;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 6px;
  }
  
  .form-group input {
    width: 100%;
    padding: 8px 12px;
    background: #0c1120;
    border: 1px solid #2a3750;
    border-radius: 6px;
    color: #e0e7f5;
    font-size: 13px;
  }
  
  .form-group input:focus {
    outline: none;
    border-color: #3b82f6;
  }
  
  .form-actions {
    display: flex;
    gap: 12px;
    margin-top: 20px;
  }
  
  .submit-btn {
    background: #10b981;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .submit-btn:hover:not(:disabled) {
    background: #059669;
  }
  
  .submit-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }
  
  .cancel-btn {
    background: transparent;
    color: #9fb0ce;
    border: 1px solid #2a3750;
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
  }
  
  .cancel-btn:hover {
    background: #2a3750;
    color: #e0e7f5;
  }
  
  .users-table {
    background: #151d2e;
    border: 1px solid #2a3750;
    border-radius: 8px;
    overflow: hidden;
  }
  
  table {
    width: 100%;
    border-collapse: collapse;
  }
  
  th {
    background: #1a2332;
    padding: 12px;
    text-align: left;
    color: #9fb0ce;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  td {
    padding: 12px;
    border-top: 1px solid #2a3750;
    color: #e0e7f5;
    font-size: 13px;
  }
  
  tr:hover {
    background: #1a2332;
  }
  
  .role-badge {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
  }
  
  .role-badge.admin {
    background: #3b82f620;
    color: #60a5fa;
  }
  
  .role-badge.enterprise {
    background: #8b5cf620;
    color: #a78bfa;
  }
  
  .role-badge.user {
    background: #6b728020;
    color: #9ca3af;
  }
  
  .action-btn {
    background: transparent;
    border: 1px solid #2a3750;
    color: #9fb0ce;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .action-btn:hover:not(:disabled) {
    background: #2a3750;
    color: #e0e7f5;
  }
  
  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .action-text {
    color: #6b7a9a;
    font-size: 11px;
    font-style: italic;
  }
  
  .empty-state {
    padding: 40px;
    text-align: center;
    color: #6b7a9a;
  }
</style>
