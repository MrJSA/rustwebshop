<script>
  import { KeyRound, Mail, CheckCircle2, ArrowLeft } from 'lucide-svelte';

  let email = '';
  let isSubmitted = false;
  let isLoading = false;

  async function handleReset() {
    isLoading = true;
    try {
      await fetch('/api/v1/auth/customer/reset-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email })
      });
      isSubmitted = true;
    } catch (e) {
      isSubmitted = true;
    } finally {
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>Reset Password | RustCraft</title>
</svelte:head>

<div class="max-w-md mx-auto px-4 py-16">
  <div class="p-8 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-6">
    <a href="/account/login" class="inline-flex items-center gap-1.5 text-xs text-slate-400 hover:text-white transition-colors">
      <ArrowLeft size={14} />
      <span>Back to Login</span>
    </a>

    <div class="text-center">
      <div class="w-12 h-12 rounded-2xl bg-orange-500/10 border border-orange-500/20 text-orange-400 flex items-center justify-center mx-auto mb-3">
        <KeyRound size={24} />
      </div>
      <h1 class="text-xl font-bold text-white">Lost Password / Reset Password</h1>
      <p class="text-xs text-slate-400 mt-1">
        Enter your registered email address and we'll send you an instant password recovery link.
      </p>
    </div>

    {#if isSubmitted}
      <div class="p-4 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs space-y-2 text-center">
        <CheckCircle2 size={24} class="mx-auto" />
        <p class="font-bold">Reset Instructions Sent</p>
        <p class="text-slate-300">If an account exists for <span class="font-mono text-white">{email}</span>, you will receive password reset instructions shortly.</p>
      </div>
    {:else}
      <form on:submit|preventDefault={handleReset} class="space-y-4">
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

        <button
          type="submit"
          disabled={isLoading}
          class="w-full py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50"
        >
          {isLoading ? 'Sending Link...' : 'Send Password Reset Link'}
        </button>
      </form>
    {/if}
  </div>
</div>
