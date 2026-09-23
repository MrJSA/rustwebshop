<script>
  import { Package, Plus, Trash2, Edit3, Layers, Download, Check, X, Box } from 'lucide-svelte';

  export let data;
  let products = data.products || [];

  let isModalOpen = false;
  let isSaving = false;

  // Form State
  let title = '';
  let category = 'Hardware';
  let subcategory = 'Keyboards';
  let description = '';
  let productType = 'physical';
  let basePriceEuros = 49.99;
  let imageUrl = '';
  let digitalDownloadUrl = '';

  // Variant in form
  let variantSku = '';
  let variantTitle = '';
  let variantStock = 20;

  function openCreateModal() {
    title = '';
    category = 'Hardware';
    subcategory = 'Accessories';
    description = '';
    productType = 'physical';
    basePriceEuros = 49.99;
    imageUrl = 'https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=800&q=80';
    digitalDownloadUrl = '';
    variantSku = 'PRD-' + Math.floor(1000 + Math.random() * 9000);
    variantTitle = 'Standard Edition';
    variantStock = 25;
    isModalOpen = true;
  }

  async function handleCreateProduct() {
    isSaving = true;
    const basePriceCents = Math.round(basePriceEuros * 100);

    const payload = {
      title,
      description,
      product_type: productType,
      category,
      subcategory,
      base_price_cents: basePriceCents,
      digital_download_url: productType === 'digital' ? digitalDownloadUrl : null,
      image_url: imageUrl,
      variants: [
        {
          sku: variantSku,
          title: variantTitle,
          price_override_cents: null,
          attributes: { type: variantTitle },
          stock_quantity: productType === 'digital' ? 999999 : variantStock,
          low_stock_threshold: 5,
          image_url: imageUrl
        }
      ]
    };

    try {
      const res = await fetch('/api/v1/admin/products', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        // Reload list
        const refresh = await fetch('/api/v1/admin/products', {
          headers: { 'X-Dev-Mode': 'true' }
        });
        if (refresh.ok) {
          products = await refresh.json();
        }
        isModalOpen = false;
      }
    } catch (e) {
      console.error('Failed to create product:', e);
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete(productId) {
    if (!confirm('Are you sure you want to delete this product and all associated SKU variants?')) return;

    try {
      const res = await fetch(`/api/v1/admin/products/${productId}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        products = products.filter((p) => p.id !== productId);
      }
    } catch (e) {
      console.error('Failed to delete product:', e);
    }
  }
</script>

<svelte:head>
  <title>Products & Variants | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-7xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Package size={24} class="text-orange-500" />
        Product & Variant Management
      </h1>
      <p class="text-xs text-slate-400 mt-1">Create physical & digital items, multi-attribute variants, and assign individual SKUs.</p>
    </div>

    <button
      on:click={openCreateModal}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md shadow-orange-600/20 flex items-center gap-2 self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>New Product</span>
    </button>
  </div>

  <!-- Products List -->
  <div class="grid grid-cols-1 gap-6">
    {#each products as item}
      <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl flex flex-col md:flex-row gap-6 items-start justify-between">
        <!-- Thumbnail & Info -->
        <div class="flex gap-4 items-start flex-1 min-w-0">
          {#if item.image_url}
            <img src={item.image_url} alt={item.title} class="w-20 h-20 rounded-xl object-cover bg-slate-950 border border-slate-800 flex-shrink-0" />
          {:else}
            <div class="w-20 h-20 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center text-3xl flex-shrink-0">
              📦
            </div>
          {/if}

          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2 mb-1">
              <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase {item.product_type === 'digital' ? 'bg-sky-500/10 text-sky-400 border border-sky-500/20' : 'bg-slate-800 text-slate-300'}">
                {item.product_type}
              </span>
              <span class="text-xs text-slate-400">&bull; {item.category} &rsaquo; {item.subcategory}</span>
            </div>

            <h3 class="text-base font-bold text-white tracking-tight truncate">{item.title}</h3>
            <p class="text-xs text-slate-400 line-clamp-1 mt-0.5">{item.description}</p>

            <div class="mt-3 flex items-center gap-4 text-xs font-mono">
              <span class="text-white font-bold">Base: {(item.base_price_cents / 100).toFixed(2)} €</span>
              <span class="text-slate-400">/{item.slug}</span>
            </div>
          </div>
        </div>

        <!-- Variants & SKUs table -->
        <div class="w-full md:w-80 lg:w-96 p-3 rounded-xl bg-slate-950 border border-slate-800/80">
          <div class="flex items-center justify-between text-[11px] font-bold text-slate-400 uppercase tracking-wider mb-2">
            <span>Variants ({item.variants ? item.variants.length : 0})</span>
            <span>SKU / Stock</span>
          </div>

          <div class="space-y-1.5 max-h-36 overflow-y-auto pr-1">
            {#if item.variants && item.variants.length > 0}
              {#each item.variants as v}
                <div class="flex items-center justify-between text-xs py-1 px-2 rounded bg-slate-900/60 border border-slate-800/40">
                  <span class="text-slate-300 truncate max-w-[140px]">{v.title}</span>
                  <div class="text-right font-mono">
                    <span class="text-[11px] text-orange-400 font-semibold">{v.sku}</span>
                    <span class="text-slate-400 text-[10px] ml-1">({item.product_type === 'digital' ? '∞' : v.stock_quantity})</span>
                  </div>
                </div>
              {/each}
            {:else}
              <div class="text-xs text-slate-500 py-1 text-center">No variants attached</div>
            {/if}
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex md:flex-col gap-2 flex-shrink-0 self-end md:self-center">
          <a
            href="http://localhost:8080/products/{item.slug}"
            target="_blank"
            class="p-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors border border-slate-700"
            title="View on Storefront"
          >
            <Box size={16} />
          </a>
          <button
            on:click={() => handleDelete(item.id)}
            class="p-2.5 rounded-xl bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/30"
            title="Delete Product"
          >
            <Trash2 size={16} />
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Create Product Modal -->
  {#if isModalOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div class="w-full max-w-2xl bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 sm:p-8 space-y-6">
        <div class="flex items-center justify-between border-b border-slate-800 pb-4">
          <h2 class="text-lg font-bold text-white">Create New E-Commerce Product</h2>
          <button on:click={() => isModalOpen = false} class="p-1 text-slate-400 hover:text-white">
            <X size={20} />
          </button>
        </div>

        <form on:submit|preventDefault={handleCreateProduct} class="space-y-4">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div class="sm:col-span-2">
              <label class="block text-xs font-semibold text-slate-400 mb-1">Product Title</label>
              <input
                type="text"
                bind:value={title}
                required
                placeholder="e.g. Ferris Wireless Mechanical Numpad"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Category</label>
              <input
                type="text"
                bind:value={category}
                required
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Subcategory</label>
              <input
                type="text"
                bind:value={subcategory}
                required
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Product Type</label>
              <select
                bind:value={productType}
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
              >
                <option value="physical">Physical Product</option>
                <option value="digital">Digital Product (Downloadable)</option>
              </select>
            </div>

            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Base Price (€)</label>
              <input
                type="number"
                step="0.01"
                bind:value={basePriceEuros}
                required
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
              />
            </div>

            <div class="sm:col-span-2">
              <label class="block text-xs font-semibold text-slate-400 mb-1">Image URL</label>
              <input
                type="url"
                bind:value={imageUrl}
                placeholder="https://..."
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
              />
            </div>

            {#if productType === 'digital'}
              <div class="sm:col-span-2">
                <label class="block text-xs font-semibold text-sky-400 mb-1">Digital Download Asset URL</label>
                <input
                  type="url"
                  bind:value={digitalDownloadUrl}
                  required
                  placeholder="https://cdn.rustwebshop.local/assets/bundle.zip"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-sky-800 text-white text-xs focus:outline-none focus:border-sky-500"
                />
              </div>
            {/if}

            <div class="sm:col-span-2">
              <label class="block text-xs font-semibold text-slate-400 mb-1">Description</label>
              <textarea
                bind:value={description}
                rows="3"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
              ></textarea>
            </div>
          </div>

          <!-- Initial SKU Variant Definition -->
          <div class="pt-4 border-t border-slate-800">
            <h4 class="text-xs font-bold text-slate-300 uppercase tracking-wider mb-3">Primary Sellable SKU Variant</h4>
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
              <div>
                <label class="block text-[11px] text-slate-400 mb-1">SKU Identifier</label>
                <input
                  type="text"
                  bind:value={variantSku}
                  required
                  class="w-full px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-white text-xs font-mono"
                />
              </div>
              <div>
                <label class="block text-[11px] text-slate-400 mb-1">Variant Name (Color/Option)</label>
                <input
                  type="text"
                  bind:value={variantTitle}
                  required
                  class="w-full px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-white text-xs"
                />
              </div>
              <div>
                <label class="block text-[11px] text-slate-400 mb-1">Initial Stock Count</label>
                <input
                  type="number"
                  bind:value={variantStock}
                  disabled={productType === 'digital'}
                  class="w-full px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-white text-xs font-mono disabled:opacity-50"
                />
              </div>
            </div>
          </div>

          <div class="pt-4 flex justify-end gap-3">
            <button
              type="button"
              on:click={() => isModalOpen = false}
              class="px-4 py-2.5 rounded-xl bg-slate-800 text-slate-300 text-xs font-bold hover:bg-slate-700"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSaving}
              class="px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold shadow-md transition-all disabled:opacity-50"
            >
              {isSaving ? 'Creating Product...' : 'Create & Save Product'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>
