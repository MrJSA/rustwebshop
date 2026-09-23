<script>
  import { customer } from '$lib/stores/customer.js';
  import { goto } from '$app/navigation';
  import { LogIn, UserPlus, Lock, Mail, User, AlertCircle, CheckCircle2 } from 'lucide-svelte';

  let activeTab = 'login'; // 'login' | 'register'
  let email = '';
  let password = '';
  let fullName = '';
  let errorMsg = '';
  let successMsg = '';
  let isLoading = false;

  async function handleLogin() {
    errorMsg = '';
    isLoading = true;
    try {
      const res = await fetch('/api/v1/auth/customer/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
      });
      const data = await res.json();
      if (!res.ok) {
        errorMsg = data.error || 'Invalid email or password';
      } else {
        customer.login(data.token, data.email, data.full_name);
        goto('/');
      }
    } catch (e) {
      errorMsg = 'Network error. Please try again.';
    } finally {
      isLoading = false;
    }
  }

  async function handleRegister() {
    errorMsg = '';
    isLoading = true;
    try {
      const res = await fetch('/api/v1/auth/customer/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password, full_name: fullName })
      });
      const data = await res.json();
      if (!res.ok) {
        errorMsg = data.error || 'Registration failed. Email may already be in use.';
      } else {
        customer.login(data.token, data.email, data.full_name);
        goto('/');
      }
    } catch (e) {
      errorMsg = 'Network error. Please try again.';
    } finally {
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>Customer Login & Register | RustCraft</title>
</svelte:head>

<div class="max-w-md mx-auto px-4 py-16">
  <div class="p-8 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-6">
    <!-- Header Tabs -->
    <div class="flex items-center rounded-xl bg-slate-950 p-1 border border-slate-800">
      <button
        type="button"
        on:click={() => { activeTab = 'login'; errorMsg = ''; }}
        class="flex-1 py-2 rounded-lg text-xs font-bold transition-all flex items-center justify-center gap-1.5 {activeTab === 'login' ? 'bg-orange-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
      >
        <LogIn size={14} />
        <span>Login</span>
      </button>
      <button
        type="button"
        on:click={() => { activeTab = 'register'; errorMsg = ''; }}
        class="flex-1 py-2 rounded-lg text-xs font-bold transition-all flex items-center justify-center gap-1.5 {activeTab === 'register' ? 'bg-orange-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
      >
        <UserPlus size={14} />
        <span>Register</span>
      </button>
    </div>

    <!-- Error Alert -->
    {#if errorMsg}
      <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2">
        <AlertCircle size={16} class="flex-shrink-0" />
        <span>{errorMsg}</span>
      </div>
    {/if}

    {#if activeTab === 'login'}
      <!-- Login Form -->
      <form on:submit|preventDefault={handleLogin} class="space-y-4">
        <div>
          <label class="block text-xs font-semibold text-slate-300 mb-1">Email Address</label>
          <div class="relative">
            <input
              type="email"
              bind:value={email}
              required
              placeholder="customer@example.com"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <Mail size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="block text-xs font-semibold text-slate-300">Password</label>
            <a href="/account/reset-password" class="text-[11px] text-orange-400 hover:underline">
              Lost password?
            </a>
          </div>
          <div class="relative">
            <input
              type="password"
              bind:value={password}
              required
              placeholder="••••••••"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <Lock size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <button
          type="submit"
          disabled={isLoading}
          class="w-full py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50"
        >
          {isLoading ? 'Signing In...' : 'Sign In to Account'}
        </button>
      </form>
    {:else}
      <!-- Register Form -->
      <form on:submit|preventDefault={handleRegister} class="space-y-4">
        <div>
          <label class="block text-xs font-semibold text-slate-300 mb-1">Full Name</label>
          <div class="relative">
            <input
              type="text"
              bind:value={fullName}
              required
              placeholder="Joshua Rust"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <User size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <div>
          <label class="block text-xs font-semibold text-slate-300 mb-1">Email Address</label>
          <div class="relative">
            <input
              type="email"
              bind:value={email}
              required
              placeholder="customer@example.com"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <Mail size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <div>
          <label class="block text-xs font-semibold text-slate-300 mb-1">Create Password</label>
          <div class="relative">
            <input
              type="password"
              bind:value={password}
              required
              minlength="6"
              placeholder="••••••••"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <Lock size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <button
          type="submit"
          disabled={isLoading}
          class="w-full py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50"
        >
          {isLoading ? 'Creating Account...' : 'Create Customer Account'}
        </button>
      </form>
    {/if}
  </div>
</div>
