<script>
  import { Truck, Plus, Trash2, Globe, Box, Save, Check, Edit3, X, ChevronRight, Layers } from 'lucide-svelte';

  export let data;
  let providers = data.providers || [];

  // Modals state
  let isAddProviderOpen = false;
  let isAddZoneOpen = false;
  let isAddRateOpen = false;
  let isEditZoneOpen = false;

  // Selected entities for actions
  let selectedProviderId = null;
  let selectedZoneId = null;
  let editingZone = null;

  // Provider Form State
  let providerName = '';
  let providerCode = '';
  let trackingUrlTemplate = 'https://www.dhl.com/track?id={tracking_number}';

  // Zone Form State
  let zoneName = '';
  let countryCodesStr = 'DE, AT, CH';

  // Rate Form State
  let rateName = 'Standard Paket (bis 5kg)';
  let packageType = 'standard';
  let priceEuros = 4.99;
  let deliveryDays = '1-2 business days';

  async function reloadProviders() {
    try {
      const res = await fetch('/api/v1/admin/settings/shipping/providers', {
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        providers = await res.json();
      }
    } catch (e) {
      console.error('Failed to reload providers:', e);
    }
  }

  // --- Provider Handlers ---
  async function handleCreateProvider() {
    try {
      const res = await fetch('/api/v1/admin/settings/shipping/providers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify({
          name: providerName,
          code: providerCode.toLowerCase().trim(),
          tracking_url_template: trackingUrlTemplate,
          is_active: true
        })
      });
      if (res.ok) {
        isAddProviderOpen = false;
        providerName = '';
        providerCode = '';
        reloadProviders();
      }
    } catch (e) {
      console.error('Failed to create provider:', e);
    }
  }

  async function handleDeleteProvider(id) {
    if (!confirm('Are you sure you want to delete this shipping carrier and all its associated zones and pricing categories?')) return;
    try {
      await fetch(`/api/v1/admin/settings/shipping/providers/${id}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      reloadProviders();
    } catch (e) {
      console.error('Failed to delete provider:', e);
    }
  }

  // --- Zone Handlers ---
  function openAddZone(providerId) {
    selectedProviderId = providerId;
    zoneName = '';
    countryCodesStr = 'DE, AT';
    isAddZoneOpen = true;
  }

  async function handleCreateZone() {
    const countryCodes = countryCodesStr.split(',').map((s) => s.trim().toUpperCase()).filter(Boolean);
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/providers/${selectedProviderId}/zones`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify({
          zone_name: zoneName,
          country_codes: countryCodes,
          is_default: false
        })
      });
      if (res.ok) {
        isAddZoneOpen = false;
        reloadProviders();
      }
    } catch (e) {
      console.error('Failed to create zone:', e);
    }
  }

  function openEditZone(zone) {
    editingZone = zone;
    zoneName = zone.zone_name;
    countryCodesStr = Array.isArray(zone.country_codes) ? zone.country_codes.join(', ') : '';
    isEditZoneOpen = true;
  }

  async function handleUpdateZone() {
    const countryCodes = countryCodesStr.split(',').map((s) => s.trim().toUpperCase()).filter(Boolean);
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/zones/${editingZone.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify({
          zone_name: zoneName,
          country_codes: countryCodes,
          is_default: editingZone.is_default
        })
      });
      if (res.ok) {
        isEditZoneOpen = false;
        reloadProviders();
      }
    } catch (e) {
      console.error('Failed to update zone:', e);
    }
  }

  async function handleDeleteZone(zoneId) {
    if (!confirm('Are you sure you want to delete this zone and all its price tiers?')) return;
    try {
      await fetch(`/api/v1/admin/settings/shipping/zones/${zoneId}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      reloadProviders();
    } catch (e) {
      console.error('Failed to delete zone:', e);
    }
  }

  // --- Rate / Price Category Handlers ---
  function openAddRate(zoneId) {
    selectedZoneId = zoneId;
    rateName = 'Standard Paket (bis 5kg)';
    priceEuros = 4.99;
    packageType = 'standard';
    deliveryDays = '1-2 business days';
    isAddRateOpen = true;
  }

  async function handleCreateRate() {
    const priceCents = Math.round(priceEuros * 100);
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/zones/${selectedZoneId}/rates`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
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
        isAddRateOpen = false;
        reloadProviders();
      }
    } catch (e) {
      console.error('Failed to create rate:', e);
    }
  }

  async function handleDeleteRate(rateId) {
    if (!confirm('Are you sure you want to delete this price category?')) return;
    try {
      await fetch(`/api/v1/admin/settings/shipping/rates/${rateId}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      reloadProviders();
    } catch (e) {
      console.error('Failed to delete rate:', e);
    }
  }
</script>

<svelte:head>
  <title>Shipping Providers, Zones & Price Categories | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-6xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Truck size={24} class="text-orange-500" />
        Shipping Carriers, Regional Zones & Pricing
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Full 3-tier hierarchy: Create Provider (DHL/Hermes/UPS) &rarr; Assign Zones & Countries &rarr; Price Categories.
      </p>
    </div>

    <button
      on:click={() => isAddProviderOpen = true}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md shadow-orange-600/20 flex items-center gap-2 self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>Add Shipping Provider</span>
    </button>
  </div>

  <!-- Providers List -->
  <div class="space-y-8">
    {#if providers.length === 0}
      <div class="p-12 text-center rounded-3xl bg-slate-900 border border-slate-800 text-xs text-slate-400">
        No shipping carriers configured yet. Click "Add Shipping Provider" to start.
      </div>
    {:else}
      {#each providers as prov}
        <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-6">
          <!-- Provider Top Header -->
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-5">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-xl bg-orange-500/10 border border-orange-500/20 text-orange-400 flex items-center justify-center font-bold">
                <Truck size={20} />
              </div>
              <div>
                <div class="flex items-center gap-2">
                  <h2 class="text-base font-bold text-white">{prov.name}</h2>
                  <span class="px-2 py-0.5 rounded text-[10px] font-mono uppercase bg-slate-800 text-orange-400 border border-slate-700">
                    {prov.code}
                  </span>
                </div>
                <div class="text-[11px] text-slate-400 font-mono mt-0.5 truncate max-w-md">
                  Tracking URL: {prov.tracking_url_template}
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <button
                type="button"
                on:click={() => openAddZone(prov.id)}
                class="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold flex items-center gap-1.5 transition-colors border border-slate-700"
              >
                <Plus size={14} />
                <span>Add Zone to {prov.name}</span>
              </button>
              <button
                type="button"
                on:click={() => handleDeleteProvider(prov.id)}
                class="p-2 rounded-xl bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/20"
                title="Delete Provider"
              >
                <Trash2 size={15} />
              </button>
            </div>
          </div>

          <!-- Zones Container -->
          <div class="space-y-4">
            {#if !prov.zones || prov.zones.length === 0}
              <div class="p-6 rounded-2xl bg-slate-950/60 border border-slate-800/60 text-center text-xs text-slate-500">
                No zones created under this carrier yet. Click "Add Zone" above to define regions & country codes.
              </div>
            {:else}
              {#each prov.zones as zone}
                <div class="p-5 rounded-2xl bg-slate-950 border border-slate-800/80 space-y-4">
                  <!-- Zone Title & Country Codes -->
                  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                      <div class="flex items-center gap-2">
                        <Globe size={15} class="text-orange-400" />
                        <h3 class="text-sm font-bold text-white">{zone.zone_name}</h3>
                        {#if zone.is_default}
                          <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20">
                            Default Zone
                          </span>
                        {/if}
                      </div>

                      <!-- Country Code Pills -->
                      <div class="flex flex-wrap gap-1 mt-2">
                        {#if Array.isArray(zone.country_codes)}
                          {#each zone.country_codes as cc}
                            <span class="px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-slate-900 border border-slate-700 text-slate-300">
                              {cc}
                            </span>
                          {/each}
                        {/if}
                      </div>
                    </div>

                    <div class="flex items-center gap-2">
                      <button
                        type="button"
                        on:click={() => openAddRate(zone.id)}
                        class="px-2.5 py-1.5 rounded-lg bg-orange-600/15 hover:bg-orange-600/30 text-orange-400 text-xs font-bold transition-colors flex items-center gap-1 border border-orange-500/30"
                      >
                        <Plus size={13} />
                        <span>Add Price Category</span>
                      </button>
                      <button
                        type="button"
                        on:click={() => openEditZone(zone)}
                        class="p-1.5 rounded-lg bg-slate-900 hover:bg-slate-800 text-slate-300 hover:text-white transition-colors border border-slate-800"
                        title="Edit Zone Countries"
                      >
                        <Edit3 size={14} />
                      </button>
                      <button
                        type="button"
                        on:click={() => handleDeleteZone(zone.id)}
                        class="p-1.5 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/20"
                        title="Delete Zone"
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </div>

                  <!-- Rates Table -->
                  {#if !zone.rates || zone.rates.length === 0}
                    <div class="text-xs text-slate-500 py-2 border-t border-slate-900">
                      No price categories created in this zone. Click "Add Price Category" to configure pricing.
                    </div>
                  {:else}
                    <div class="overflow-x-auto border-t border-slate-900 pt-3">
                      <table class="w-full text-left text-xs">
                        <thead>
                          <tr class="text-[10px] text-slate-500 uppercase tracking-wider">
                            <th class="pb-2 font-semibold">Tier Category Name</th>
                            <th class="pb-2 font-semibold">Package Type</th>
                            <th class="pb-2 font-semibold">Delivery Days</th>
                            <th class="pb-2 font-semibold text-right">Price</th>
                            <th class="pb-2 font-semibold text-right">Action</th>
                          </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-900">
                          {#each zone.rates as rate}
                            <tr class="text-slate-300">
                              <td class="py-2.5 font-bold text-white">{rate.name}</td>
                              <td class="py-2.5 capitalize text-slate-400 font-mono text-[11px]">{rate.package_type}</td>
                              <td class="py-2.5 text-slate-400">{rate.estimated_delivery_days}</td>
                              <td class="py-2.5 text-right font-mono font-bold text-emerald-400">
                                {(rate.price_cents / 100).toFixed(2)} €
                              </td>
                              <td class="py-2.5 text-right">
                                <button
                                  type="button"
                                  on:click={() => handleDeleteRate(rate.id)}
                                  class="p-1 text-slate-500 hover:text-rose-400 transition-colors"
                                  title="Delete price rate"
                                >
                                  <Trash2 size={13} />
                                </button>
                              </td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    </div>
                  {/if}
                </div>
              {/each}
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Create Provider Modal -->
  {#if isAddProviderOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 space-y-4">
        <h3 class="text-base font-bold text-white">Add Shipping Carrier / Provider</h3>
        <form on:submit|preventDefault={handleCreateProvider} class="space-y-4 text-xs">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Carrier Name</label>
            <input type="text" bind:value={providerName} required placeholder="e.g. Hermes Germany" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
          </div>
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Identifier Code (unique)</label>
            <input type="text" bind:value={providerCode} required placeholder="e.g. hermes" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
          </div>
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Tracking URL Template</label>
            <input type="text" bind:value={trackingUrlTemplate} required placeholder="https://tracking.com?id={tracking_number}" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-[11px] focus:outline-none focus:border-orange-500" />
          </div>
          <div class="flex items-center justify-end gap-3 pt-2">
            <button type="button" on:click={() => isAddProviderOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
            <button type="submit" class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold">Add Carrier</button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Create Zone Modal -->
  {#if isAddZoneOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 space-y-4">
        <h3 class="text-base font-bold text-white">Create Shipping Zone</h3>
        <form on:submit|preventDefault={handleCreateZone} class="space-y-4 text-xs">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Zone Name</label>
            <input type="text" bind:value={zoneName} required placeholder="e.g. European Union (Zone 1)" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
          </div>
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Assigned Countries (Comma separated ISO-2 codes)</label>
            <input type="text" bind:value={countryCodesStr} required placeholder="DE, FR, IT, ES, NL" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
            <p class="text-[10px] text-slate-500 mt-1">Example: DE, AT, CH, FR, NL, BE, US</p>
          </div>
          <div class="flex items-center justify-end gap-3 pt-2">
            <button type="button" on:click={() => isAddZoneOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
            <button type="submit" class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold">Create Zone</button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Edit Zone Modal -->
  {#if isEditZoneOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 space-y-4">
        <h3 class="text-base font-bold text-white">Edit Shipping Zone</h3>
        <form on:submit|preventDefault={handleUpdateZone} class="space-y-4 text-xs">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Zone Name</label>
            <input type="text" bind:value={zoneName} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
          </div>
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Assigned Countries (Comma separated)</label>
            <input type="text" bind:value={countryCodesStr} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
          </div>
          <div class="flex items-center justify-end gap-3 pt-2">
            <button type="button" on:click={() => isEditZoneOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
            <button type="submit" class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold">Update Zone</button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Create Rate Modal -->
  {#if isAddRateOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 space-y-4">
        <h3 class="text-base font-bold text-white">Add Price Category / Rate</h3>
        <form on:submit|preventDefault={handleCreateRate} class="space-y-4 text-xs">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Tier / Category Label</label>
            <input type="text" bind:value={rateName} required placeholder="e.g. Hermes Päckchen S (<3kg)" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Price (EUR)</label>
              <input type="number" step="0.01" bind:value={priceEuros} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
            </div>
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Package Type</label>
              <select bind:value={packageType} class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500">
                <option value="standard">Standard</option>
                <option value="express">Express</option>
                <option value="fragile">Fragile</option>
                <option value="heavy">Heavy Cargo</option>
              </select>
            </div>
          </div>
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Estimated Delivery Days</label>
            <input type="text" bind:value={deliveryDays} required placeholder="1-2 business days" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
          </div>
          <div class="flex items-center justify-end gap-3 pt-2">
            <button type="button" on:click={() => isAddRateOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
            <button type="submit" class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold">Add Price Tier</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>
