<script>
  import { cart, isCartOpen } from "$lib/stores/cart.js";
  import {
    Search,
    ArrowRight,
    Zap,
    Download,
    Layers,
    CheckCircle2,
    AlertTriangle,
    ShieldCheck,
  } from "lucide-svelte";

  export let data;
  $: products = data.products || [];
  $: currentCategory = data.currentCategory || "";
  let searchQuery = data.currentSearch || "";

  const categories = [
    { label: "All Products", value: "" },
    { label: "Hardware", value: "Hardware" },
    { label: "Apparel", value: "Apparel" },
    { label: "Software & Books", value: "Software & Books" },
    { label: "Accessories", value: "Accessories" },
  ];

  function quickAddDefault(product) {
    if (!product.variants || product.variants.length === 0) return;
    const defaultVariant = product.variants[0];
    cart.addItem({
      variant_id: defaultVariant.id,
      sku: defaultVariant.sku,
      product_title: product.title,
      variant_title: defaultVariant.title,
      price_cents:
        defaultVariant.price_override_cents || product.base_price_cents,
      quantity: 1,
      is_digital: product.product_type === "digital",
      image_url: product.image_url,
      slug: product.slug,
    });
    isCartOpen.set(true);
  }
</script>

<svelte:head>
  <title>RustCraft | High-Performance Gear & Software</title>
</svelte:head>

<!-- Hero Section -->
<section
  class="relative overflow-hidden pt-12 pb-20 border-b border-slate-900 bg-gradient-to-b from-slate-950 via-slate-900/60 to-slate-950"
>
  <div
    class="absolute inset-0 bg-[radial-gradient(circle_at_top,_var(--tw-gradient-stops))] from-orange-600/10 via-transparent to-transparent pointer-events-none"
  ></div>

  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10 text-center">
    <div
      class="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full border border-orange-500/30 bg-orange-500/10 text-orange-400 text-xs font-semibold mb-6 shadow-inner"
    >
      <Zap size={14} class="fill-orange-400" />
      <span
        >Memory-Safe &bull; Ultra-Low Latency &bull; Zero Race Conditions</span
      >
    </div>

    <h1
      class="text-4xl sm:text-6xl font-black tracking-tight text-white max-w-4xl mx-auto leading-tight sm:leading-none"
    >
      High-Performance Gear & Architecture for <span
        class="bg-gradient-to-r from-orange-400 via-amber-400 to-orange-500 bg-clip-text text-transparent"
        >Rustaceans</span
      >
    </h1>

    <p
      class="mt-6 text-base sm:text-lg text-slate-400 max-w-2xl mx-auto leading-relaxed"
    >
      Engineered for extreme reliability. Premium physical mechanical hardware
      and digital books backed by ACID row-level inventory locks.
    </p>

    <!-- Search Box -->
    <div class="mt-8 max-w-xl mx-auto">
      <form action="/" method="GET" class="relative flex items-center">
        <input
          type="text"
          name="search"
          bind:value={searchQuery}
          placeholder="Search products by title, category, or keyword..."
          class="w-full pl-12 pr-28 py-3.5 rounded-2xl bg-slate-900/90 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500 focus:ring-2 focus:ring-orange-500/20 text-sm shadow-xl"
        />
        <Search
          size={18}
          class="absolute left-4 text-slate-500 pointer-events-none"
        />
        <button
          type="submit"
          class="absolute right-2 px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md"
        >
          Search
        </button>
      </form>
    </div>

    <!-- Category Filters -->
    <div class="mt-8 flex flex-wrap items-center justify-center gap-2">
      {#each categories as cat}
        <a
          href={cat.value ? `/?category=${encodeURIComponent(cat.value)}` : "/"}
          class="px-4 py-2 rounded-xl text-xs font-bold transition-all duration-200 border {currentCategory ===
          cat.value
            ? 'bg-orange-600 text-white border-orange-500 shadow-lg shadow-orange-600/30'
            : 'bg-slate-900/80 text-slate-400 border-slate-800 hover:text-white hover:border-slate-700'}"
        >
          {cat.label}
        </a>
      {/each}
    </div>
  </div>
</section>

<!-- Product Catalog Grid -->
<section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
  <div class="flex items-center justify-between mb-8">
    <div>
      <h2 class="text-2xl font-bold text-white tracking-tight">
        {currentCategory ? currentCategory : "Featured Collection"}
      </h2>
      <p class="text-xs text-slate-400 mt-1">
        Showing {products.length} products available for immediate dispatch
      </p>
    </div>
  </div>

  {#if products.length === 0}
    <div
      class="text-center py-20 bg-slate-900/40 rounded-3xl border border-slate-800/80"
    >
      <div class="text-4xl mb-3">🔍</div>
      <h3 class="text-lg font-bold text-white">
        No products match your filter
      </h3>
      <p class="text-xs text-slate-400 mt-1">
        Try resetting the category filter or searching for another term.
      </p>
      <a
        href="/"
        class="inline-block mt-4 px-5 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold"
      >
        Reset Filters
      </a>
    </div>
  {:else}
    <div
      class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6"
    >
      {#each products as item}
        <div
          class="group relative rounded-2xl bg-slate-900/60 border border-slate-800/80 hover:border-orange-500/40 transition-all duration-300 flex flex-col overflow-hidden hover:shadow-2xl hover:shadow-orange-950/20"
        >
          <!-- Thumbnail Container -->
          <a
            href="/products/{item.slug}"
            class="relative aspect-square overflow-hidden bg-slate-950 block"
          >
            {#if item.image_url}
              <img
                src={item.image_url}
                alt={item.title}
                class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500 ease-out"
                loading="lazy"
              />
            {:else}
              <div
                class="w-full h-full flex items-center justify-center text-4xl text-slate-700"
              >
                📦
              </div>
            {/if}

            <!-- Badges -->
            <div class="absolute top-3 left-3 flex flex-col gap-1.5 z-10">
              {#if item.product_type === "digital"}
                <span
                  class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-[10px] font-bold bg-sky-500/90 text-white backdrop-blur-md shadow-sm"
                >
                  <Download size={11} /> Digital Download
                </span>
              {:else}
                <span
                  class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-[10px] font-bold bg-slate-900/80 text-slate-300 border border-slate-700 backdrop-blur-md shadow-sm"
                >
                  Physical Unit
                </span>
              {/if}

              {#if item.variants && item.variants.some((v) => v.stock_quantity > 0 && v.stock_quantity <= v.low_stock_threshold && item.product_type === "physical")}
                <span
                  class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-[10px] font-bold bg-amber-500 text-slate-950 shadow-sm animate-pulse"
                >
                  <AlertTriangle size={11} /> Low Stock
                </span>
              {/if}
            </div>

            <!-- Category Pill -->
            <div class="absolute bottom-3 left-3 z-10">
              <span
                class="px-2 py-0.5 rounded text-[11px] font-semibold bg-slate-950/80 text-orange-400 border border-slate-800"
              >
                {item.subcategory || item.category}
              </span>
            </div>
          </a>

          <!-- Details & Options -->
          <div class="p-5 flex-1 flex flex-col justify-between">
            <div>
              <a
                href="/products/{item.slug}"
                class="block group-hover:text-orange-400 transition-colors"
              >
                <h3
                  class="text-base font-bold text-white leading-snug line-clamp-2"
                >
                  {item.title}
                </h3>
              </a>

              <p
                class="text-xs text-slate-400 mt-2 line-clamp-2 leading-relaxed"
              >
                {item.description}
              </p>

              <!-- Variants Pill info -->
              {#if item.variants && item.variants.length > 0}
                <div
                  class="mt-3 flex items-center gap-2 text-xs text-slate-400"
                >
                  <Layers size={13} class="text-slate-500" />
                  <span
                    >{item.variants.length}
                    {item.variants.length === 1
                      ? "Option"
                      : "Options & Colors"}</span
                  >
                </div>
              {/if}
            </div>

            <div
              class="mt-5 pt-4 border-t border-slate-800/80 flex items-center justify-between"
            >
              <div>
                <span
                  class="text-[10px] text-slate-400 block uppercase font-mono tracking-wider"
                  >From</span
                >
                <span class="text-lg font-black text-white font-mono">
                  {(item.base_price_cents / 100).toFixed(2)} €
                </span>
              </div>

              <div class="flex items-center gap-2">
                <a
                  href="/products/{item.slug}"
                  class="px-3.5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md shadow-orange-600/20 flex items-center gap-1.5"
                >
                  <span>Select</span>
                  <ArrowRight size={13} />
                </a>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</section>
