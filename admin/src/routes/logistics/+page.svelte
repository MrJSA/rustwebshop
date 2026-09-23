<script>
  import { Warehouse, AlertTriangle, CheckCircle, Search, RefreshCw, Plus, Package } from 'lucide-svelte';

  export let data;
  let inventory = data.inventory || [];
  let filterLowStockOnly = false;
  let searchQuery = '';
  let updatingVariantId = null;

  $: filteredItems = inventory.filter((item) => {
    if (filterLowStockOnly && !item.is_low_stock && !item.is_out_of_stock) {
      return false;
    }
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      return (
        item.sku.toLowerCase().includes(q) ||
        item.product_title.toLowerCase().includes(q) ||
        item.variant_title.toLowerCase().includes(q)
      );
    }
    return true;
  });

  async function adjustStock(variant_id, amount) {
    updatingVariantId = variant_id;
    try {
      const res = await fetch(`/api/v1/admin/logistics/inventory/${variant_id}/stock`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify({ adjustment: amount })
      });

      if (res.ok) {
        // Update local state reactively
        inventory = inventory.map((item) => {
          if (item.variant_id === variant_id) {
            const newQty = item.stock_quantity + amount;
            return {
              ...item,
              stock_quantity: newQty,
              is_low_stock: item.product_type === 'physical' && newQty <= item.low_stock_threshold,
              is_out_of_stock: item.product_type === 'physical' && newQty <= 0
            };
          }
          return item;
        });
      }
    } catch (e) {
      console.error('Failed to update stock:', e);
    } finally {
      updatingVariantId = null;
    }
  }
</script>

<svelte:head>
  <title>Logistics & Inventory | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-7xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Warehouse size={24} class="text-orange-500" />
        Logistics & Warehouse Management
      </h1>
      <p class="text-xs text-slate-400 mt-1">Live tracking of physical inventory SKUs, stock thresholds, and one-click restocking.</p>
    </div>

    <!-- Filters & Search -->
    <div class="flex flex-wrap items-center gap-3">
      <div class="relative">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter by SKU or title..."
          class="pl-9 pr-4 py-2 rounded-xl bg-slate-900 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500 w-52 sm:w-64"
        />
        <Search size={14} class="absolute left-3 top-3 text-slate-500 pointer-events-none" />
      </div>

      <button
        on:click={() => filterLowStockOnly = !filterLowStockOnly}
        class="px-3.5 py-2 rounded-xl text-xs font-bold border transition-colors flex items-center gap-1.5 {filterLowStockOnly ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' : 'bg-slate-900 text-slate-300 border-slate-800 hover:border-slate-700'}"
      >
        <AlertTriangle size={13} class={filterLowStockOnly ? 'text-amber-400' : 'text-slate-400'} />
        <span>Low Stock Only</span>
      </button>
    </div>
  </div>

  <!-- Warehouse Inventory Table -->
  <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden">
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs">
        <thead class="bg-slate-950/60 border-b border-slate-800 text-slate-400 uppercase tracking-wider text-[11px]">
          <tr>
            <th class="py-3.5 px-4 font-bold">SKU Code</th>
            <th class="py-3.5 px-4 font-bold">Product / Variant</th>
            <th class="py-3.5 px-4 font-bold">Category</th>
            <th class="py-3.5 px-4 font-bold text-center">Type</th>
            <th class="py-3.5 px-4 font-bold text-center">In Stock</th>
            <th class="py-3.5 px-4 font-bold text-center">Threshold</th>
            <th class="py-3.5 px-4 font-bold text-center">Stock Health</th>
            <th class="py-3.5 px-4 font-bold text-right">Quick Restock</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#if filteredItems.length === 0}
            <tr>
              <td colspan="8" class="text-center py-12 text-slate-400">
                No inventory matches the active filter.
              </td>
            </tr>
          {:else}
            {#each filteredItems as item}
              <tr class="hover:bg-slate-800/30 transition-colors">
                <!-- SKU -->
                <td class="py-4 px-4 font-mono font-bold text-orange-400">
                  {item.sku}
                </td>

                <!-- Product & Variant Title -->
                <td class="py-4 px-4">
                  <div class="font-bold text-white text-sm">{item.product_title}</div>
                  <div class="text-slate-400 text-xs mt-0.5">{item.variant_title}</div>
                </td>

                <!-- Category -->
                <td class="py-4 px-4 text-slate-300">
                  {item.category}
                </td>

                <!-- Physical vs Digital -->
                <td class="py-4 px-4 text-center">
                  {#if item.product_type === 'digital'}
                    <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-sky-500/10 text-sky-400 border border-sky-500/20">
                      Digital
                    </span>
                  {:else}
                    <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-slate-800 text-slate-300 border border-slate-700">
                      Physical
                    </span>
                  {/if}
                </td>

                <!-- In Stock Count -->
                <td class="py-4 px-4 text-center font-mono font-bold text-base {item.is_out_of_stock ? 'text-rose-400' : item.is_low_stock ? 'text-amber-400' : 'text-emerald-400'}">
                  {item.product_type === 'digital' ? '∞' : item.stock_quantity}
                </td>

                <!-- Threshold -->
                <td class="py-4 px-4 text-center font-mono text-slate-400">
                  {item.product_type === 'digital' ? 'N/A' : item.low_stock_threshold}
                </td>

                <!-- Status Badge -->
                <td class="py-4 px-4 text-center">
                  {#if item.product_type === 'digital'}
                    <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-sky-500/10 text-sky-400 border border-sky-500/20">
                      Unlimited
                    </span>
                  {:else if item.is_out_of_stock}
                    <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-rose-500/10 text-rose-400 border border-rose-500/20">
                      Depleted
                    </span>
                  {:else if item.is_low_stock}
                    <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 animate-pulse">
                      Low Stock
                    </span>
                  {:else}
                    <span class="px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                      Healthy
                    </span>
                  {/if}
                </td>

                <!-- Quick Restock Actions -->
                <td class="py-4 px-4 text-right">
                  {#if item.product_type === 'physical'}
                    <div class="inline-flex items-center gap-1.5">
                      <button
                        on:click={() => adjustStock(item.variant_id, 5)}
                        disabled={updatingVariantId === item.variant_id}
                        class="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold font-mono border border-slate-700 transition-colors"
                        title="Add 5 units to warehouse"
                      >
                        +5
                      </button>
                      <button
                        on:click={() => adjustStock(item.variant_id, 25)}
                        disabled={updatingVariantId === item.variant_id}
                        class="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold font-mono border border-slate-700 transition-colors"
                        title="Add 25 units to warehouse"
                      >
                        +25
                      </button>
                      <button
                        on:click={() => adjustStock(item.variant_id, 100)}
                        disabled={updatingVariantId === item.variant_id}
                        class="px-2.5 py-1 rounded bg-orange-600 hover:bg-orange-500 text-white text-xs font-semibold font-mono shadow-sm transition-colors"
                        title="Add 100 units to warehouse"
                      >
                        +100
                      </button>
                    </div>
                  {:else}
                    <span class="text-[11px] text-slate-500">Asset Hosted</span>
                  {/if}
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>
