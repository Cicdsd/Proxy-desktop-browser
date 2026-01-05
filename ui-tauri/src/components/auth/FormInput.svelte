<script lang="ts">
  // Props
  let {
    type = 'text',
    name,
    label,
    value = $bindable(''),
    placeholder = '',
    required = false,
    disabled = false,
    error = '',
    autocomplete = 'off',
    minlength,
    maxlength,
    pattern,
    onInput,
    onBlur
  }: {
    type?: 'text' | 'password' | 'email' | 'number' | 'tel' | 'url';
    name: string;
    label: string;
    value?: string;
    placeholder?: string;
    required?: boolean;
    disabled?: boolean;
    error?: string;
    autocomplete?: string;
    minlength?: number;
    maxlength?: number;
    pattern?: string;
    onInput?: (value: string) => void;
    onBlur?: () => void;
  } = $props();

  // Local state
  let showPassword = $state(false);
  let isFocused = $state(false);

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    value = target.value;
    if (onInput) onInput(value);
  }

  function handleBlur() {
    isFocused = false;
    if (onBlur) onBlur();
  }

  function togglePasswordVisibility() {
    showPassword = !showPassword;
  }

  let inputType = $derived(() => {
    if (type === 'password' && showPassword) return 'text';
    return type;
  });
</script>

<div class="form-input" class:has-error={!!error} class:is-focused={isFocused}>
  <label for={name}>{label}{#if required}<span class="required">*</span>{/if}</label>
  
  <div class="input-wrapper">
    <input
      id={name}
      {name}
      type={inputType()}
      {placeholder}
      {required}
      {disabled}
      {autocomplete}
      {minlength}
      {maxlength}
      {pattern}
      bind:value
      oninput={handleInput}
      onblur={handleBlur}
      onfocus={() => isFocused = true}
    />
    
    {#if type === 'password'}
      <button 
        type="button" 
        class="toggle-password" 
        onclick={togglePasswordVisibility}
        tabindex="-1"
      >
        {showPassword ? '👁️' : '👁️‍🗨️'}
      </button>
    {/if}
  </div>
  
  {#if error}
    <span class="error-message">{error}</span>
  {/if}
</div>

<style>
  .form-input {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 16px;
  }

  label {
    font-size: 13px;
    font-weight: 500;
    color: #e0e0e0;
  }

  .required {
    color: #ef4444;
    margin-left: 2px;
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  input {
    width: 100%;
    padding: 10px 12px;
    background: #0d0d1a;
    border: 1px solid #2a2a45;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 14px;
    transition: all 0.2s;
  }

  input:focus {
    outline: none;
    border-color: #4f46e5;
    box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.1);
  }

  input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  input::placeholder {
    color: #808090;
  }

  .form-input.has-error input {
    border-color: #ef4444;
  }

  .form-input.is-focused input {
    border-color: #4f46e5;
  }

  .toggle-password {
    position: absolute;
    right: 10px;
    background: transparent;
    border: none;
    color: #808090;
    cursor: pointer;
    padding: 4px;
    font-size: 14px;
  }

  .toggle-password:hover {
    color: #e0e0e0;
  }

  .error-message {
    font-size: 12px;
    color: #ef4444;
  }
</style>
