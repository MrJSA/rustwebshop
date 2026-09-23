<script>
  import { Truck, Plus, Trash2, Globe, Box, Save, Check } from 'lucide-svelte';

  export let data;
  let shippingZones = data.shippingZones || [];

  // Add new rate form state
  let isAddRateModalOpen = false;
  let rateName = 'DHL Express Guaranteed';
  let packageType = 'express';
  let priceEuros = 14.99;
  let deliveryDays = '1-2 business days';
  let isSavingRate = false;

  async function handleAddRate() {
    isSavingRate = true;
    const priceCents = Math.round(priceEuros * 100);

    try {
      const res = await fetch('/api/v1/admin/settings/shipping/rates', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify({
          name: rateName,
          package_type: packageType,
          price_cents: priceCents,
          estimated_delivery_days: deliveryDays,
          min_weight_g: 0,
          max_weight_g: 5000
        })
      });

      if (res.ok) {
        // Refresh zones
        const refresh = await fetch('/api/v1/admin/settings/shipping', {
          headers: { 'X-Dev-Mode': 'true' }
        });
        if (refresh.ok) {
          shippingZones = await refresh.json();
        }
        isAddRateModalOpen = false;
      }
    } catch (e) {
      console.error('Failed to create rate:', e);
    } finally {
      isSavingRate = false;
    }
  }

  async function handleDeleteRate(rateId) {
    if (!confirm('Are you sure you want to delete this shipping tier?')) return;

    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/rates/${rateId}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        shippingZones = shippingZones.map((z) => ({
          ...z,
          rates: z.rates.filter((r) => r.id !== rateId)
        }));
      }
    } catch (e) {
      console.error('Failed to delete shipping rate:', e);
    }
  }
</script>

<svelte:head>
  <title>Shipping Zones & Providers | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-5xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Truck size={24} class="text-orange-500" />
        Shipping Providers & Regional Zones
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Configure country codes, courier package tiers (Standard, Express, Heavy), and dynamic rates.
      </p>
    </div>

    <button
      on:click={() => isAddRateModalOpen = true}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md shadow-orange-600/20 flex items-center gap-2 self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>Add Shipping Option</span>
    </button>
  </div>

  <!-- Zones List -->
  <div class="space-y-6">
    {#each shippingZones as zone}
      <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
        <!-- Zone Header -->
        <div class="flex items-center justify-between border-b border-slate-800 pb-4">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center text-orange-400">
              <Globe size={20} />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h3 class="text-base font-bold text-white tracking-tight">{zone.zone_name}</h3>
                {#if zone.is_default}
                  <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-orange-500/10 text-orange-400 border border-orange-500/20">
                    Default Zone
                  </span>
                {/if}
              </div>
              <div class="text-[11px] text-slate-400 mt-0.5">
                Countries: <span class="font-mono text-slate-300">{JSON.stringify(zone.country_codes)}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Rates Table -->
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead>
              <tr class="text-slate-400 text-[11px] border-b border-slate-800/80">
                <th class="py-2.5">Package Option & Courier</th>
                <th class="py-2.5">Type</th>
                <th class="py-2.5">Estimated Transit Time</th>
                <th class="py-2.5 text-right">Price</th>
                <th class="py-2.5 text-right">Action</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800/40">
              {#if !zone.rates || zone.rates.length === 0}
                <tr>
                  <td colspan="5" class="py-6 text-center text-slate-500 text-xs">
                    No rates assigned to this zone yet.
                  </td>
                </tr>
              {:else}
                {#each zone.rates as rate}
                  <tr class="hover:bg-slate-800/20 transition-colors">
                    <td class="py-3 font-bold text-white">
                      {rate.name}
                    </td>
                    <td class="py-3">
                      <span class="px-2 py-0.5 rounded text-[10px] font-semibold uppercase bg-slate-950 border border-slate-800 text-slate-300">
                        {rate.package_type}
                      </span>
                    </td>
                    <td class="py-3 text-slate-400">
                      {rate.estimated_delivery_days}
                    </td>
                    <td class="py-3 text-right font-mono font-bold text-white">
                      {(rate.price_cents / 100).toFixed(2)} €
                    </td>
                    <td class="py-3 text-right">
                      <button
                        on:click={() => handleDeleteRate(rate.id)}
                        class="p-1.5 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-slate-800 transition-colors"
                        title="Remove shipping option"
                      >
                        <Trash2 size={14} />
                      </button>
                    </td>
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
      </div>
    {/each}
  </div>

  <!-- Add Rate Modal -->
  {#if isAddRateModalOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div class="w-full max-w-lg bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 space-y-4">
        <h3 class="text-base font-bold text-white">Add Shipping Option</h3>

        <form on:submit|preventDefault={handleAddRate} class="space-y-4 text-xs">
          <div>
            <label for="admin-shipping-rate-name" class="block font-semibold text-slate-400 mb-1">Option Name / Courier</label>
            <input
              id="admin-shipping-rate-name"
              type="text"
              bind:value={rateName}
              required
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label for="admin-shipping-package-type" class="block font-semibold text-slate-400 mb-1">Package Tier</label>
              <select
                id="admin-shipping-package-type"
                bind:value={packageType}
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              >
                <option value="standard">Standard</option>
                <option value="express">Express</option>
                <option value="fragile">Fragile</option>
                <option value="heavy">Heavy Freight</option>
              </select>
            </div>

            <div>
              <label for="admin-shipping-price" class="block font-semibold text-slate-400 mb-1">Price (€)</label>
              <input
                id="admin-shipping-price"
                type="number"
                step="0.01"
                bind:value={priceEuros}
                required
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 font-mono"
              />
            </div>
          </div>

          <div>
            <label for="admin-shipping-delivery-days" class="block font-semibold text-slate-400 mb-1">Transit Time Estimate</label>
            <input
              id="admin-shipping-delivery-days"
              type="text"
              bind:value={deliveryDays}
              required
              placeholder="e.g. 1-2 business days"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>

          <div class="pt-3 flex justify-end gap-2">
            <button
              type="button"
              on:click={() => isAddRateModalOpen = false}
              class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 font-bold hover:bg-slate-700"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSavingRate}
              class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold shadow-md disabled:opacity-50"
            >
              {isSavingRate ? 'Saving...' : 'Add Option'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>
