<script>
  import { CreditCard, ShieldCheck, Check, AlertCircle, Save, ExternalLink, Sparkles } from 'lucide-svelte';

  export let data;
  let paymentConfigs = data.paymentConfigs || [];
  let savingProvider = null;
  let successNotice = '';

  async function saveConfig(provider) {
    savingProvider = provider.provider;
    successNotice = '';

    try {
      const res = await fetch(`/api/v1/admin/settings/payments/${provider.provider}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify({
          display_name: provider.display_name,
          is_enabled: provider.is_enabled,
          is_sandbox: provider.is_sandbox,
          public_client_id: provider.public_client_id
        })
      });

      if (res.ok) {
        successNotice = `Saved settings for ${provider.display_name} successfully!`;
        setTimeout(() => successNotice = '', 3500);
      }
    } catch (e) {
      console.error('Failed to save payment config:', e);
    } finally {
      savingProvider = null;
    }
  }
</script>

<svelte:head>
  <title>Payment Provider Settings | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-5xl mx-auto">
  <!-- Header -->
  <div>
    <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
      <CreditCard size={24} class="text-orange-500" />
      Payment Gateway Configuration
    </h1>
    <p class="text-xs text-slate-400 mt-1">
      Manage active payment methods. Stripe and PayPal are primary, Apple Pay, Google Pay, and Amazon Pay are configurable.
    </p>
  </div>

  {#if successNotice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <Check size={16} />
      <span>{successNotice}</span>
    </div>
  {/if}

  <!-- Gateways List -->
  <div class="space-y-5">
    {#each paymentConfigs as provider}
      {@const isStripe = provider.provider === 'stripe'}
      <div class="p-6 rounded-3xl border shadow-xl space-y-4 {isStripe ? 'bg-gradient-to-br from-slate-900 via-slate-900 to-orange-950/20 border-orange-500/50 shadow-orange-950/20 ring-1 ring-orange-500/20' : 'bg-slate-900 border-slate-800'}">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-4">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center font-black text-orange-400">
              {#if provider.provider === 'stripe'}S
              {:else if provider.provider === 'paypal'}P
              {:else if provider.provider === 'apple_pay'}🍎
              {:else if provider.provider === 'google_pay'}G
              {:else if provider.provider === 'amazon_pay'}a
              {:else}💳{/if}
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h3 class="text-base font-bold text-white tracking-tight">{provider.display_name}</h3>
                {#if isStripe}
                  <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-orange-500/15 text-orange-400 border border-orange-500/30 flex items-center gap-1">
                    <Sparkles size={11} /> Primary Gateway
                  </span>
                {/if}
              </div>
              <div class="text-[11px] font-mono text-slate-400 uppercase">Provider ID: {provider.provider}</div>
            </div>
          </div>

          <!-- Toggles for Enabled and Sandbox -->
          <div class="flex items-center gap-4">
            <!-- Sandbox Toggle -->
            <label class="flex items-center gap-2 cursor-pointer text-xs">
              <input
                type="checkbox"
                bind:checked={provider.is_sandbox}
                class="sr-only peer"
              />
              <div class="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-amber-500"></div>
              <span class="text-xs font-semibold {provider.is_sandbox ? 'text-amber-400' : 'text-slate-400'}">
                {provider.is_sandbox ? 'Sandbox Mode' : 'Live Mode'}
              </span>
            </label>

            <!-- Enabled Toggle -->
            <label class="flex items-center gap-2 cursor-pointer text-xs">
              <input
                type="checkbox"
                bind:checked={provider.is_enabled}
                class="sr-only peer"
              />
              <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-600"></div>
              <span class="text-xs font-bold {provider.is_enabled ? 'text-white' : 'text-slate-500'}">
                {provider.is_enabled ? 'Active' : 'Disabled'}
              </span>
            </label>
          </div>
        </div>

        {#if isStripe}
          <!-- Stripe Terminal & Integration Helper Banner -->
          <div class="p-3.5 rounded-2xl bg-orange-950/20 border border-orange-500/30 flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-xs">
            <div class="space-y-0.5">
              <div class="font-bold text-orange-400 flex items-center gap-1.5">
                <CreditCard size={14} />
                <span>Stripe Terminal & Payment Intents Support</span>
              </div>
              <p class="text-[11px] text-slate-300">
                Collect in-person card payments and online checkouts seamlessly with Stripe Terminal and Elements.
              </p>
            </div>
            <a
              href="https://docs.stripe.com/terminal/payments/collect-card-payment"
              target="_blank"
              rel="noopener noreferrer"
              class="px-3 py-1.5 rounded-xl bg-orange-600/30 hover:bg-orange-600 text-orange-300 hover:text-white border border-orange-500/40 text-[11px] font-bold transition-all flex items-center gap-1.5 self-start sm:self-auto flex-shrink-0"
            >
              <span>Stripe Terminal Docs</span>
              <ExternalLink size={12} />
            </a>
          </div>
        {/if}

        <!-- Credentials Inputs -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
          <div>
            <label class="block font-semibold text-slate-400 mb-1">Display Label in Checkout</label>
            <input
              type="text"
              bind:value={provider.display_name}
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
            />
          </div>

          <div>
            <label class="block font-semibold text-slate-400 mb-1">Public Key / Merchant Client ID</label>
            <input
              type="text"
              bind:value={provider.public_client_id}
              placeholder="e.g. pk_test_..."
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div class="flex justify-end pt-2">
          <button
            on:click={() => saveConfig(provider)}
            disabled={savingProvider === provider.provider}
            class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-orange-600 text-white text-xs font-bold transition-all flex items-center gap-1.5 shadow-sm disabled:opacity-50"
          >
            <Save size={14} />
            <span>{savingProvider === provider.provider ? 'Saving...' : 'Save Configuration'}</span>
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>
