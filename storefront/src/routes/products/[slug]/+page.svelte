<script>
  import { cart, isCartOpen } from '$lib/stores/cart.js';
  import { ShieldCheck, Truck, Zap, Download, Check, AlertCircle, ArrowLeft, Plus, Minus, Box } from 'lucide-svelte';

  export let data;
  $: product = data.product || {};
  $: variants = data.variants || [];

  let selectedVariantIndex = 0;
  let quantity = 1;

  $: currentVariant = variants[selectedVariantIndex] || {};
  $: currentPriceCents = currentVariant.price_override_cents || product.base_price_cents;
  $: isDigital = product.product_type === 'digital';
  $: inStock = isDigital || (currentVariant.stock_quantity && currentVariant.stock_quantity > 0);
  $: isLowStock = !isDigital && currentVariant.stock_quantity > 0 && currentVariant.stock_quantity <= currentVariant.low_stock_threshold;

  function addToCart() {
    if (!inStock) return;
    cart.addItem({
      variant_id: currentVariant.id,
      sku: currentVariant.sku,
      product_title: product.title,
      variant_title: currentVariant.title,
      price_cents: currentPriceCents,
      quantity,
      is_digital: isDigital,
      image_url: currentVariant.image_url || product.image_url,
      slug: product.slug
    });
    isCartOpen.set(true);
  }
</script>

<svelte:head>
  <title>{product.title} | RustCraft Gear</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
  <!-- Breadcrumb -->
  <nav class="flex items-center gap-2 text-xs text-slate-400 mb-8">
    <a href="/" class="hover:text-orange-400 flex items-center gap-1 transition-colors">
      <ArrowLeft size={14} /> Back to Catalog
    </a>
    <span>/</span>
    <span class="text-slate-400">{product.category}</span>
    <span>/</span>
    <span class="text-white font-medium truncate">{product.title}</span>
  </nav>

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-12 lg:gap-16">
    <!-- Media Column -->
    <div class="space-y-4">
      <div class="aspect-square rounded-3xl overflow-hidden bg-slate-900 border border-slate-800 shadow-2xl relative">
        {#if currentVariant.image_url || product.image_url}
          <img
            src={currentVariant.image_url || product.image_url}
            alt={product.title}
            class="w-full h-full object-cover"
          />
        {:else}
          <div class="w-full h-full flex items-center justify-center text-7xl text-slate-700">
            📦
          </div>
        {/if}

        {#if isDigital}
          <div class="absolute top-4 left-4 px-3 py-1 rounded-lg bg-sky-500 text-white text-xs font-bold flex items-center gap-1.5 shadow-md">
            <Download size={13} /> Digital Masterclass & Asset
          </div>
        {/if}
      </div>

      <!-- Trust Badges -->
      <div class="grid grid-cols-3 gap-3 pt-4">
        <div class="p-3 rounded-xl bg-slate-900/50 border border-slate-800/80 text-center">
          <ShieldCheck size={20} class="mx-auto text-emerald-400 mb-1" />
          <div class="text-[11px] font-bold text-white">ACID Protected</div>
          <div class="text-[10px] text-slate-400">Zero overselling</div>
        </div>
        <div class="p-3 rounded-xl bg-slate-900/50 border border-slate-800/80 text-center">
          <Truck size={20} class="mx-auto text-orange-400 mb-1" />
          <div class="text-[11px] font-bold text-white">Zone Dispatch</div>
          <div class="text-[10px] text-slate-400">DHL & Express</div>
        </div>
        <div class="p-3 rounded-xl bg-slate-900/50 border border-slate-800/80 text-center">
          <Zap size={20} class="mx-auto text-amber-400 mb-1" />
          <div class="text-[11px] font-bold text-white">Rust Speed</div>
          <div class="text-[10px] text-slate-400">Sub-ms responses</div>
        </div>
      </div>
    </div>

    <!-- Product Details Column -->
    <div class="flex flex-col justify-between">
      <div>
        <div class="flex items-center gap-2 mb-2">
          <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-orange-500/10 text-orange-400 border border-orange-500/20">
            {product.category} &bull; {product.subcategory}
          </span>
          {#if isLowStock}
            <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-500/10 text-amber-400 border border-amber-500/20">
              Low Stock Alert
            </span>
          {/if}
        </div>

        <h1 class="text-3xl sm:text-4xl font-black text-white tracking-tight">
          {product.title}
        </h1>

        <div class="mt-4 flex items-baseline gap-3">
          <span class="text-3xl font-black text-white font-mono">
            {(currentPriceCents / 100).toFixed(2)} €
          </span>
          <span class="text-xs text-slate-400 font-mono">Incl. VAT / Taxes</span>
        </div>

        <div class="mt-6 prose prose-invert text-sm text-slate-300 leading-relaxed border-t border-b border-slate-800/80 py-5">
          {product.description}
        </div>

        <!-- Variants Selection -->
        {#if variants.length > 0}
          <div class="mt-6 space-y-3">
            <div class="flex justify-between items-center text-xs">
              <span class="font-bold text-slate-300 uppercase tracking-wider">Choose Variant / Model:</span>
              <span class="font-mono text-orange-400 font-semibold">SKU: {currentVariant.sku}</span>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
              {#each variants as variant, i}
                <button
                  type="button"
                  on:click={() => selectedVariantIndex = i}
                  class="p-3.5 rounded-xl border text-left transition-all duration-200 flex items-center justify-between {selectedVariantIndex === i ? 'bg-orange-600/15 border-orange-500 ring-1 ring-orange-500 text-white' : 'bg-slate-900/60 border-slate-800 hover:border-slate-700 text-slate-300'}"
                >
                  <div class="min-w-0 pr-2">
                    <div class="text-xs font-bold truncate">{variant.title}</div>
                    <div class="text-[11px] font-mono text-slate-400 mt-0.5">{variant.sku}</div>
                  </div>
                  <div class="text-right flex-shrink-0">
                    <div class="text-xs font-mono font-bold">
                      {((variant.price_override_cents || product.base_price_cents) / 100).toFixed(2)} €
                    </div>
                    {#if !isDigital}
                      <div class="text-[10px] {variant.stock_quantity <= 0 ? 'text-rose-400' : variant.stock_quantity <= variant.low_stock_threshold ? 'text-amber-400 font-bold' : 'text-slate-400'}">
                        {variant.stock_quantity <= 0 ? 'Out of stock' : `${variant.stock_quantity} in stock`}
                      </div>
                    {/if}
                  </div>
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Stock Availability Indicator -->
        <div class="mt-6 p-3.5 rounded-xl bg-slate-900/80 border border-slate-800 flex items-center justify-between text-xs">
          <div class="flex items-center gap-2">
            {#if isDigital}
              <div class="w-2.5 h-2.5 rounded-full bg-sky-400 animate-pulse"></div>
              <span class="font-semibold text-slate-200">Instant Digital Delivery link generated upon payment.</span>
            {:else if inStock}
              <div class="w-2.5 h-2.5 rounded-full bg-emerald-400"></div>
              <span class="font-semibold text-slate-200">
                In Stock ({currentVariant.stock_quantity} units available in central warehouse)
              </span>
            {:else}
              <div class="w-2.5 h-2.5 rounded-full bg-rose-500"></div>
              <span class="font-semibold text-rose-400">Currently out of stock</span>
            {/if}
          </div>
        </div>
      </div>

      <!-- Add to Cart Action -->
      <div class="mt-8 pt-6 border-t border-slate-800 flex flex-col sm:flex-row items-center gap-4">
        <!-- Quantity counter -->
        {#if !isDigital}
          <div class="flex items-center border border-slate-700 rounded-xl bg-slate-900 px-2 py-1 w-full sm:w-auto justify-between sm:justify-start">
            <button
              on:click={() => quantity = Math.max(1, quantity - 1)}
              class="p-2 text-slate-400 hover:text-white transition-colors"
              aria-label="Decrease quantity"
            >
              <Minus size={14} />
            </button>
            <span class="px-4 text-sm font-bold text-white font-mono">{quantity}</span>
            <button
              on:click={() => quantity = Math.min(currentVariant.stock_quantity || 10, quantity + 1)}
              class="p-2 text-slate-400 hover:text-white transition-colors"
              aria-label="Increase quantity"
            >
              <Plus size={14} />
            </button>
          </div>
        {/if}

        <button
          id="add-to-cart-btn"
          on:click={addToCart}
          disabled={!inStock}
          class="flex-1 w-full py-4 px-6 rounded-xl font-bold text-sm flex items-center justify-center gap-2 shadow-xl transition-all duration-200 {!inStock ? 'bg-slate-800 text-slate-500 cursor-not-allowed' : 'bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white shadow-orange-600/30 hover:scale-[1.01]'}"
        >
          {#if isDigital}
            <Download size={18} />
            <span>Add Digital License &bull; {(currentPriceCents / 100).toFixed(2)} €</span>
          {:else if inStock}
            <Box size={18} />
            <span>Add to Cart &bull; {((currentPriceCents * quantity) / 100).toFixed(2)} €</span>
          {:else}
            <span>Out of Stock</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>
