<script lang="ts">
  // Props
  let {
    password = ''
  }: {
    password: string;
  } = $props();

  // Calculate password strength
  let strength = $derived(() => {
    if (!password) return { score: 0, label: '', color: '' };
    
    let score = 0;
    
    // Length check
    if (password.length >= 8) score += 1;
    if (password.length >= 12) score += 1;
    if (password.length >= 16) score += 1;
    
    // Character variety checks
    if (/[a-z]/.test(password)) score += 1;
    if (/[A-Z]/.test(password)) score += 1;
    if (/[0-9]/.test(password)) score += 1;
    if (/[^a-zA-Z0-9]/.test(password)) score += 1;
    
    // Determine label and color based on score
    if (score <= 2) {
      return { score: 1, label: 'Weak', color: '#ef4444' };
    } else if (score <= 4) {
      return { score: 2, label: 'Fair', color: '#f59e0b' };
    } else if (score <= 6) {
      return { score: 3, label: 'Good', color: '#10b981' };
    } else {
      return { score: 4, label: 'Strong', color: '#06b6d4' };
    }
  });

  let barWidth = $derived(() => (strength().score / 4) * 100);
</script>

{#if password}
  <div class="password-strength">
    <div class="strength-bar">
      <div 
        class="strength-fill" 
        style="width: {barWidth()}%; background-color: {strength().color}"
      ></div>
    </div>
    <span class="strength-label" style="color: {strength().color}">
      {strength().label}
    </span>
  </div>
{/if}

<style>
  .password-strength {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 4px;
  }

  .strength-bar {
    flex: 1;
    height: 4px;
    background: #2a2a45;
    border-radius: 2px;
    overflow: hidden;
  }

  .strength-fill {
    height: 100%;
    border-radius: 2px;
    transition: all 0.3s ease;
  }

  .strength-label {
    font-size: 11px;
    font-weight: 500;
    min-width: 50px;
  }
</style>
