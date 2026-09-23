<script>
  import { cart, isCartOpen } from "$lib/stores/cart.js";
  import {
    Search,
    ArrowRight,
    Download,
    Layers,
    AlertTriangle,
    ChevronLeft,
    ChevronRight,
    Sparkles
  } from "lucide-svelte";

  export let data;
  $: products = data.products || [];
  $: currentCategory = data.currentCategory || "";
  $: heroConfig = data.heroConfig || {
    layout: "split",
    carousel_items: [
      {
        id: "c1",
        title: "RustCraft Pro 75% Mechanical Keyboard",
        subtitle: "Hot-swappable CNC anodized aluminum chassis with QMK/VIA firmware",
        image_url: "https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=1400&q=80",
        link_url: "/products/rust-mechanical-keyboard",
        button_text: "Discover Precision"
      },
      {
        id: "c2",
        title: "Rustacean Heavyweight Hoodie",
        subtitle: "480 GSM French Terry organic cotton with embroidered Ferris insignia",
        image_url: "https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=1400&q=80",
        link_url: "/products/rustacean-heavyweight-hoodie",
        button_text: "Gear Up"
      }
    ],
    featured_buttons: [
      {
        id: "b1",
        title: "Mechanical Keyboards",
        subtitle: "From 189.00 €",
        image_url: "https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/rust-mechanical-keyboard",
        bg_color: "#ea580c"
      },
      {
        id: "b2",
        title: "Heavyweight Hoodies",
        subtitle: "From 79.00 €",
        image_url: "https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/rustacean-heavyweight-hoodie",
        bg_color: "#0284c7"
      },
      {
        id: "b3",
        title: "Architecture Guide",
        subtitle: "From 29.00 €",
        image_url: "https://images.unsplash.com/photo-1532012164546-f432f2e37b73?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/zero-cost-abstractions-guide",
        bg_color: "#16a34a"
      },
      {
        id: "b4",
        title: "Aviator Coiled Cables",
        subtitle: "From 34.00 €",
        image_url: "https://images.unsplash.com/photo-1544716278-ca5e3f4abd8c?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/rust-mechanical-keyboard",
        bg_color: "#7c3aed"
      }
    ]
  };

  let activeSlide = 0;
  $: carouselItems = (heroConfig && heroConfig.carousel_items && heroConfig.carousel_items.length > 0)
    ? heroConfig.carousel_items
    : [];

  function nextSlide() {
    if (carouselItems.length === 0) return;
    activeSlide = (activeSlide + 1) % carouselItems.length;
  }

  function prevSlide() {
    if (carouselItems.length === 0) return;
    activeSlide = (activeSlide - 1 + carouselItems.length) % carouselItems.length;
  }
</script>

<svelte:head>
  <title>RustCraft | High-Performance Gear & Software</title>
</svelte:head>

<!-- Dynamic Hero Showcase -->
{#if heroConfig && carouselItems.length > 0}
  <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-6 pb-8">
    {#if heroConfig.layout === 'carousel'}
      <!-- Option A: Full-Width Widescreen Carousel (8BitDo style) -->
      <div class="relative rounded-3xl overflow-hidden shadow-2xl bg-slate-950 border border-slate-800/80 aspect-[21/9] min-h-[380px] max-h-[560px]">
        {#each carouselItems as item, idx}
          <div
            class="absolute inset-0 transition-opacity duration-700 ease-in-out {idx === activeSlide ? 'opacity-100 z-10' : 'opacity-0 z-0 pointer-events-none'}"
          >
            <!-- Background Image -->
            <img
              src={item.image_url}
              alt={item.title}
              class="w-full h-full object-cover object-center"
            />
            <!-- Gradient Overlay -->
            <div class="absolute inset-0 bg-gradient-to-r from-slate-950/95 via-slate-950/70 to-transparent"></div>

            <!-- Content -->
            <div class="absolute inset-0 flex flex-col justify-center px-8 sm:px-16 max-w-2xl z-20">
              <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-orange-600/20 text-orange-400 border border-orange-500/30 text-xs font-bold uppercase tracking-wider mb-4 w-fit">
                <Sparkles size={13} /> Featured Hardware
              </span>
              <h1 class="text-3xl sm:text-5xl font-black text-white tracking-tight leading-tight drop-shadow-md">
                {item.title}
              </h1>
              <p class="text-xs sm:text-sm text-slate-300 mt-3 leading-relaxed drop-shadow line-clamp-3">
                {item.subtitle}
              </p>
              <div class="mt-6">
                <a
                  href={item.link_url || '/'}
                  class="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs sm:text-sm shadow-xl shadow-orange-600/30 transition-all hover:scale-105"
                >
                  <span>{item.button_text || 'Explore Now'}</span>
                  <ArrowRight size={16} />
                </a>
              </div>
            </div>
          </div>
        {/each}

        <!-- Arrows -->
        {#if carouselItems.length > 1}
          <button
            on:click={prevSlide}
            aria-label="Previous Slide"
            class="absolute left-4 top-1/2 -translate-y-1/2 z-30 p-2.5 rounded-full bg-slate-950/70 hover:bg-orange-600 text-white transition-all backdrop-blur-md border border-slate-700/60"
          >
            <ChevronLeft size={20} />
          </button>
          <button
            on:click={nextSlide}
            aria-label="Next Slide"
            class="absolute right-4 top-1/2 -translate-y-1/2 z-30 p-2.5 rounded-full bg-slate-950/70 hover:bg-orange-600 text-white transition-all backdrop-blur-md border border-slate-700/60"
          >
            <ChevronRight size={20} />
          </button>

          <!-- Dot Indicators -->
          <div class="absolute bottom-5 left-1/2 -translate-x-1/2 z-30 flex items-center gap-2">
            {#each carouselItems as _, idx}
              <button
                on:click={() => activeSlide = idx}
                aria-label="Slide {idx + 1}"
                class="h-2 rounded-full transition-all {idx === activeSlide ? 'w-8 bg-orange-500' : 'w-2 bg-white/40 hover:bg-white/70'}"
              ></button>
            {/each}
          </div>
        {/if}
      </div>

    {:else}
      <!-- Option B: Split Hero (60% Carousel + 40% 4 Featured Product Buttons, 8BitMods style) -->
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-5 items-stretch">
        <!-- Left: 60% Width Carousel Slider (7 cols on lg) -->
        <div class="lg:col-span-7 relative rounded-3xl overflow-hidden shadow-2xl bg-slate-950 border border-slate-800/80 min-h-[380px] lg:min-h-[460px]">
          {#each carouselItems as item, idx}
            <div
              class="absolute inset-0 transition-opacity duration-700 ease-in-out {idx === activeSlide ? 'opacity-100 z-10' : 'opacity-0 z-0 pointer-events-none'}"
            >
              <img
                src={item.image_url}
                alt={item.title}
                class="w-full h-full object-cover object-center"
              />
              <div class="absolute inset-0 bg-gradient-to-t from-slate-950 via-slate-950/60 to-transparent"></div>

              <div class="absolute inset-x-0 bottom-0 p-6 sm:p-8 z-20">
                <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-orange-600/30 text-orange-400 border border-orange-500/40 text-[10px] font-bold uppercase tracking-wider mb-2">
                  <Sparkles size={11} /> Featured Release
                </span>
                <h2 class="text-xl sm:text-3xl font-black text-white tracking-tight leading-snug drop-shadow-md">
                  {item.title}
                </h2>
                <p class="text-xs text-slate-300 mt-2 line-clamp-2 drop-shadow">
                  {item.subtitle}
                </p>
                <div class="mt-4">
                  <a
                    href={item.link_url || '/'}
                    class="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all hover:scale-105"
                  >
                    <span>{item.button_text || 'View Product'}</span>
                    <ArrowRight size={14} />
                  </a>
                </div>
              </div>
            </div>
          {/each}

          <!-- Arrows & Dots -->
          {#if carouselItems.length > 1}
            <button
              on:click={prevSlide}
              aria-label="Previous Slide"
              class="absolute left-3 top-1/2 -translate-y-1/2 z-30 p-2 rounded-full bg-slate-950/70 hover:bg-orange-600 text-white transition-all backdrop-blur-md border border-slate-700/60"
            >
              <ChevronLeft size={16} />
            </button>
            <button
              on:click={nextSlide}
              aria-label="Next Slide"
              class="absolute right-3 top-1/2 -translate-y-1/2 z-30 p-2 rounded-full bg-slate-950/70 hover:bg-orange-600 text-white transition-all backdrop-blur-md border border-slate-700/60"
            >
              <ChevronRight size={16} />
            </button>
            <div class="absolute top-4 right-4 z-30 flex items-center gap-1.5">
              {#each carouselItems as _, idx}
                <button
                  on:click={() => activeSlide = idx}
                  aria-label="Slide {idx + 1}"
                  class="h-1.5 rounded-full transition-all {idx === activeSlide ? 'w-6 bg-orange-500' : 'w-2 bg-white/40'}"
                ></button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Right: 40% Width 4 Featured Product Buttons (5 cols on lg) -->
        <div class="lg:col-span-5 grid grid-cols-1 sm:grid-cols-2 gap-4">
          {#if heroConfig.featured_buttons && heroConfig.featured_buttons.length > 0}
            {#each heroConfig.featured_buttons as btn}
              <a
                href={btn.link_url || '/'}
                class="group relative rounded-2xl overflow-hidden p-5 flex flex-col justify-between shadow-xl transition-all duration-300 hover:scale-[1.02] hover:shadow-2xl border border-white/10"
                style="background-color: {btn.bg_color || '#ea580c'};"
              >
                <!-- Image Accent -->
                {#if btn.image_url}
                  <div class="absolute right-[-10px] bottom-[-10px] w-28 h-28 opacity-30 group-hover:opacity-45 group-hover:scale-110 transition-all duration-500 overflow-hidden pointer-events-none rounded-xl">
                    <img src={btn.image_url} alt="" class="w-full h-full object-cover" />
                  </div>
                {/if}

                <div class="relative z-10">
                  <span class="text-[10px] uppercase font-bold tracking-widest text-white/80 block">
                    Featured
                  </span>
                  <h3 class="text-base font-extrabold text-white leading-tight mt-1 group-hover:underline">
                    {btn.title}
                  </h3>
                </div>

                <div class="relative z-10 mt-6 flex items-center justify-between">
                  <span class="text-xs font-mono font-bold text-white/90">
                    {btn.subtitle}
                  </span>
                  <div class="w-7 h-7 rounded-lg bg-white/20 group-hover:bg-white text-white group-hover:text-slate-950 flex items-center justify-center transition-colors">
                    <ArrowRight size={14} />
                  </div>
                </div>
              </a>
            {/each}
          {/if}
        </div>
      </div>
    {/if}
  </section>
{/if}

<!-- Product Catalog Grid -->
<section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
  <div class="flex items-center justify-between mb-8">
    <div>
      <h2 class="text-2xl font-bold text-white tracking-tight">
        {currentCategory ? currentCategory : "Product Catalog"}
      </h2>
      <p class="text-xs text-slate-400 mt-1">
        Showing {products.length} products available for immediate dispatch
      </p>
    </div>
  </div>

  {#if products.length === 0}
    <div class="text-center py-20 bg-slate-900/40 rounded-3xl border border-slate-800/80">
      <div class="text-4xl mb-3">🔍</div>
      <h3 class="text-lg font-bold text-white">No products match your filter</h3>
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
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
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
              <div class="w-full h-full flex items-center justify-center text-4xl text-slate-700">
                📦
              </div>
            {/if}

            <!-- Badges -->
            <div class="absolute top-3 left-3 flex flex-col gap-1.5 z-10">
              {#if item.product_type === "digital"}
                <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-[10px] font-bold bg-sky-500/90 text-white backdrop-blur-md shadow-sm">
                  <Download size={11} /> Digital Download
                </span>
              {:else}
                <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-[10px] font-bold bg-slate-900/80 text-slate-300 border border-slate-700 backdrop-blur-md shadow-sm">
                  Physical Unit
                </span>
              {/if}

              {#if item.variants && item.variants.some((v) => v.stock_quantity > 0 && v.stock_quantity <= v.low_stock_threshold && item.product_type === "physical")}
                <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-[10px] font-bold bg-amber-500 text-slate-950 shadow-sm animate-pulse">
                  <AlertTriangle size={11} /> Low Stock
                </span>
              {/if}
            </div>

            <!-- Category Pill -->
            <div class="absolute bottom-3 left-3 z-10">
              <span class="px-2 py-0.5 rounded text-[11px] font-semibold bg-slate-950/80 text-orange-400 border border-slate-800">
                {item.subcategory || item.category}
              </span>
            </div>
          </a>

          <!-- Details & Options -->
          <div class="p-5 flex-1 flex flex-col justify-between">
            <div>
              <a href="/products/{item.slug}" class="block group-hover:text-orange-400 transition-colors">
                <h3 class="text-base font-bold text-white leading-snug line-clamp-2">
                  {item.title}
                </h3>
              </a>

              <p class="text-xs text-slate-400 mt-2 line-clamp-2 leading-relaxed">
                {item.description}
              </p>

              {#if item.variants && item.variants.length > 0}
                <div class="mt-3 flex items-center gap-2 text-xs text-slate-400">
                  <Layers size={13} class="text-slate-500" />
                  <span>
                    {item.variants.length} {item.variants.length === 1 ? "Option" : "Options & Colors"}
                  </span>
                </div>
              {/if}
            </div>

            <div class="mt-5 pt-4 border-t border-slate-800/80 flex items-center justify-between">
              <div>
                <span class="text-[10px] text-slate-400 block uppercase font-mono tracking-wider">From</span>
                <span class="text-lg font-black text-white font-mono">
                  {(item.base_price_cents / 100).toFixed(2)} €
                </span>
              </div>

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
      {/each}
    </div>
  {/if}
</section>
