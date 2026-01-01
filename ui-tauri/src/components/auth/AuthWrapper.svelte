<script lang="ts">
  import { setContext } from 'svelte';
  import type { User } from '../../lib/types';

  // Login credentials type
  interface LoginCredentials {
    username: string;
    password: string;
  }

  export interface AuthContext {
    isAuthenticated: boolean;
    user: User | null;
    login: (credentials: LoginCredentials) => Promise<void>;
    logout: () => void;
  }

  // Create a stub auth context for demo purposes
  const authContext: AuthContext = {
    isAuthenticated: true,
    user: { 
      id: 'demo', 
      username: 'Demo User',
      email: 'demo@example.com',
      role: 'User',
      created_at: new Date().toISOString(),
      last_login: new Date().toISOString(),
      enterprise_id: null
    },
    login: async (_credentials: LoginCredentials) => {
      // In production, this would call the actual login API
    },
    logout: () => {
      // In production, this would clear auth state and tokens
    }
  };

  setContext('auth', authContext);
</script>

<slot />
