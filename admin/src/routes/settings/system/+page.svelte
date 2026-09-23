<script>
  import { Sliders, Save, Check, ShieldCheck, Terminal, AlertCircle } from 'lucide-svelte';

  export let data;
  let settings = data.settings || {};
  let isSaving = false;
  let successNotice = '';

  async function handleSaveSettings() {
    isSaving = true;
    successNotice = '';

    try {
      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify({
          store_name: settings.store_name,
          deployment_mode: settings.deployment_mode,
          debug_mode: settings.debug_mode,
          currency: settings.currency,
          currency_symbol: settings.currency_symbol,
          tax_rate_percent: Number(settings.tax_rate_percent),
          support_email: settings.support_email,
          company_address: settings.company_address,
          vat_id: settings.vat_id
        })
      });

      if (res.ok) {
        successNotice = 'System and deployment configuration updated successfully!';
        setTimeout(() => successNotice = '', 3500);
      }
    } catch (e) {
      console.error('Failed to update system settings:', e);
    } finally {
      isSaving = false;
    }
  }
</script>

<svelte:head>
  <title>System & Deployment Settings | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-4xl mx-auto">
  <!-- Header -->
  <div>
    <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
      <Sliders size={24} class="text-orange-500" />
      System, Deployment & Debug Engine
    </h1>
    <p class="text-xs text-slate-400 mt-1">
      Toggle deployment modes and debug engines on or off to preview and test shop changes in real-time.
    </p>
  </div>

  {#if successNotice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <Check size={16} />
      <span>{successNotice}</span>
    </div>
  {/if}

  <form on:submit|preventDefault={handleSaveSettings} class="space-y-6">
    <!-- Environment & Modes Card -->
    <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
      <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
        <Terminal size={18} class="text-orange-400" />
        Operational Environment & Mode Toggles
      </h2>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
        <!-- Deployment Mode Selector -->
        <div>
          <label for="admin-deployment-mode" class="block text-xs font-bold text-slate-300 uppercase tracking-wider mb-2">
            Deployment Mode
          </label>
          <select
            id="admin-deployment-mode"
            bind:value={settings.deployment_mode}
            class="w-full px-4 py-3 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-xs focus:outline-none focus:border-orange-500"
          >
            <option value="development">development (Local Testing)</option>
            <option value="staging">staging (Pre-Production Sandbox)</option>
            <option value="demo">demo (Showcase Mode)</option>
            <option value="production">production (Live Operational Store)</option>
          </select>
          <p class="text-[11px] text-slate-500 mt-1.5 leading-relaxed">
            Controls security strictness, customer notice banners, and telemetry logging levels.
          </p>
        </div>

        <!-- Debug Mode Toggle -->
        <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 flex flex-col justify-between">
          <div>
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-white uppercase tracking-wider">Debug Engine Mode</span>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={settings.debug_mode}
                  class="sr-only peer"
                />
                <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-600"></div>
              </label>
            </div>
            <p class="text-[11px] text-slate-400 mt-2 leading-relaxed">
              When enabled, customer checkout displays sandbox payment helpers and debug notice banners across the storefront.
            </p>
          </div>
          <div class="mt-3 text-[11px] font-mono font-semibold {settings.debug_mode ? 'text-emerald-400' : 'text-slate-500'}">
            Status: {settings.debug_mode ? 'ACTIVE (Testing Enabled)' : 'OFF (Strict Live Handling)'}
          </div>
        </div>
      </div>
    </div>

    <!-- Store Profile & Legal Information Card -->
    <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
      <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3">
        Legal Identity & Tax Invoice Settings
      </h2>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
        <div>
          <label for="admin-store-name" class="block font-semibold text-slate-400 mb-1">Store / Legal Entity Name</label>
          <input
            id="admin-store-name"
            type="text"
            bind:value={settings.store_name}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="admin-support-email" class="block font-semibold text-slate-400 mb-1">Support Email</label>
          <input
            id="admin-support-email"
            type="email"
            bind:value={settings.support_email}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="admin-vat-id" class="block font-semibold text-slate-400 mb-1">Company VAT Registration ID</label>
          <input
            id="admin-vat-id"
            type="text"
            bind:value={settings.vat_id}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="admin-tax-rate" class="block font-semibold text-slate-400 mb-1">Standard Sales Tax / VAT (%)</label>
          <input
            id="admin-tax-rate"
            type="number"
            step="0.01"
            bind:value={settings.tax_rate_percent}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="sm:col-span-2">
          <label for="admin-company-address" class="block font-semibold text-slate-400 mb-1">Official Company Address (Rendered on Invoices)</label>
          <input
            id="admin-company-address"
            type="text"
            bind:value={settings.company_address}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>
      </div>
    </div>

    <!-- Submit Action -->
    <div class="flex justify-end">
      <button
        type="submit"
        disabled={isSaving}
        class="px-6 py-3 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all flex items-center gap-2 disabled:opacity-50"
      >
        <Save size={16} />
        <span>{isSaving ? 'Updating Settings...' : 'Save System Settings'}</span>
      </button>
    </div>
  </form>
</div>
