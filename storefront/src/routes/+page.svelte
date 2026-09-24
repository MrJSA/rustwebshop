<script>
  import ProductCard from "$lib/components/ProductCard.svelte";
  import {
    ArrowRight,
    ChevronLeft,
    ChevronRight,
    Sparkles
  } from "lucide-svelte";

  export let data;
  $: products = data.products || [];
  $: carousels = data.carousels || [];
  $: currentCategory = data.currentCategory || "";
  $: currentSearch = data.currentSearch || "";
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
        subtitle: "Tactile Switches & Aluminum",
        price: "189.00 €",
        show_price: true,
        image_url: "https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/rust-mechanical-keyboard",
        bg_color: "#ea580c"
      },
      {
        id: "b2",
        title: "Heavyweight Hoodies",
        subtitle: "480 GSM Organic Cotton",
        price: "79.00 €",
        show_price: true,
        image_url: "https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/rustacean-heavyweight-hoodie",
        bg_color: "#0284c7"
      },
      {
        id: "b3",
        title: "Architecture Guide",
        subtitle: "Zero-Cost Concurrency eBook",
        price: "29.00 €",
        show_price: true,
        image_url: "https://images.unsplash.com/photo-1532012164546-f432f2e37b73?auto=format&fit=crop&w=600&q=80",
        link_url: "/products/zero-cost-abstractions-guide",
        bg_color: "#16a34a"
      },
      {
        id: "b4",
        title: "Aviator Coiled Cables",
        subtitle: "Double-Sleeved USB-C",
        price: "34.00 €",
        show_price: true,
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

  function scrollCarousel(sectionId, direction) {
    const el = document.getElementById(`carousel-${sectionId}`);
    if (el) {
      const scrollAmount = 480;
      el.scrollBy({ left: direction * scrollAmount, behavior: 'smooth' });
    }
  }
</script>

<svelte:head>
  <title>RustCraft | High-Performance Gear & Software</title>
</svelte:head>

<!-- Dynamic Hero Showcase -->
{#if !currentCategory && !currentSearch && heroConfig && carouselItems.length > 0}
  <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-6 pb-6">
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
        <div class="lg:col-span-7 relative rounded-3xl overflow-hidden shadow-2xl bg-slate-950 border border-slate-800/80 min-h-[360px] lg:min-h-[440px]">
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

        <!-- Right: 40% Width 4 Featured Product Buttons (8BitMods Depth & Floating Product Style) -->
        <div class="lg:col-span-5 grid grid-cols-1 sm:grid-cols-2 gap-4">
          {#if heroConfig.featured_buttons && heroConfig.featured_buttons.length > 0}
            {#each heroConfig.featured_buttons as btn}
              <a
                href={btn.link_url || '/'}
                class="group relative rounded-2xl overflow-hidden p-4 sm:p-5 flex flex-col justify-between shadow-xl transition-all duration-300 hover:scale-[1.02] hover:shadow-2xl border border-white/15 min-h-[170px]"
                style="background-color: {btn.bg_color || '#ea580c'};"
              >
                <!-- Depth Gradient & Highlights -->
                <div class="absolute inset-0 bg-gradient-to-br from-white/15 via-transparent to-black/30 pointer-events-none"></div>

                <!-- Floating Product Image with Depth Layering -->
                {#if btn.image_url}
                  <div class="absolute right-2 top-2 sm:right-3 sm:top-3 w-24 h-24 sm:w-28 sm:h-28 z-10 transition-all duration-300 group-hover:scale-110 group-hover:-translate-y-1">
                    <img
                      src={btn.image_url}
                      alt={btn.title}
                      class="w-full h-full object-contain drop-shadow-[0_12px_16px_rgba(0,0,0,0.5)]"
                    />
                  </div>
                {/if}

                <!-- Text Header (No "Featured" badge) -->
                <div class="relative z-20 max-w-[65%]">
                  <h3 class="text-sm sm:text-base font-black text-white leading-tight drop-shadow group-hover:underline">
                    {btn.title}
                  </h3>
                  {#if btn.subtitle}
                    <p class="text-[11px] text-white/80 font-medium mt-1 leading-snug drop-shadow line-clamp-2">
                      {btn.subtitle}
                    </p>
                  {/if}
                </div>

                <!-- Bottom Row: Price & Action -->
                <div class="relative z-20 mt-4 flex items-center justify-between">
                  <div>
                    {#if btn.show_price !== false && btn.price}
                      <span class="inline-block text-[11px] sm:text-xs font-mono font-black text-white bg-black/30 backdrop-blur-sm px-2 py-0.5 rounded-md border border-white/10 shadow-sm">
                        {btn.price}
                      </span>
                    {/if}
                  </div>
                  <div class="w-7 h-7 rounded-lg bg-white/20 group-hover:bg-white text-white group-hover:text-slate-950 flex items-center justify-center transition-colors shadow">
                    <ArrowRight size={13} />
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

<!-- Homepage Mode: Dynamic 5-per-row Carousels Sections -->
{#if !currentCategory && !currentSearch && carousels && carousels.length > 0}
  <div class="space-y-6 pb-12">
    {#each carousels as section}
      {#if section.items && section.items.length > 0}
        <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div class="flex items-center justify-between mb-4">
            <div>
              <h2 class="text-xl sm:text-2xl font-black text-white tracking-tight flex items-center gap-2">
                <span class="w-2 h-5 rounded-full bg-orange-500"></span>
                {section.title}
              </h2>
              <span class="text-[11px] text-slate-400 ml-4 font-mono">{section.items.length} items</span>
            </div>

            <!-- Scroll arrows if carousel has more than 5 items -->
            {#if section.items.length > 5}
              <div class="flex items-center gap-1.5">
                <button
                  on:click={() => scrollCarousel(section.id, -1)}
                  class="p-2 rounded-xl bg-slate-900/90 hover:bg-slate-800 text-slate-300 hover:text-white border border-slate-800 transition-colors shadow-sm"
                  aria-label="Scroll left"
                >
                  <ChevronLeft size={16} />
                </button>
                <button
                  on:click={() => scrollCarousel(section.id, 1)}
                  class="p-2 rounded-xl bg-slate-900/90 hover:bg-slate-800 text-slate-300 hover:text-white border border-slate-800 transition-colors shadow-sm"
                  aria-label="Scroll right"
                >
                  <ChevronRight size={16} />
                </button>
              </div>
            {/if}
          </div>

          {#if section.items.length <= 5}
            <!-- 5 products per row, centered layout -->
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3.5 sm:gap-4 justify-center">
              {#each section.items as itemWithV}
                <ProductCard item={itemWithV.product || itemWithV} variants={itemWithV.variants} />
              {/each}
            </div>
          {:else}
            <!-- Smooth scrollable horizontal carousel -->
            <div
              id="carousel-{section.id}"
              class="flex gap-3.5 sm:gap-4 overflow-x-auto scrollbar-none snap-x snap-mandatory scroll-smooth pb-3"
            >
              {#each section.items as itemWithV}
                <div class="w-[210px] sm:w-[230px] flex-shrink-0 snap-start">
                  <ProductCard item={itemWithV.product || itemWithV} variants={itemWithV.variants} />
                </div>
              {/each}
            </div>
          {/if}
        </section>
      {/if}
    {/each}
  </div>
{:else}
  <!-- Filtered Search / Category Results Grid -->
  <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h2 class="text-xl sm:text-2xl font-black text-white tracking-tight flex items-center gap-2">
          <span class="w-2 h-5 rounded-full bg-orange-500"></span>
          {currentCategory ? currentCategory : (currentSearch ? `Search: "${currentSearch}"` : "All Products")}
        </h2>
        <p class="text-xs text-slate-400 mt-1">
          Showing {products.length} products available for immediate dispatch
        </p>
      </div>
      {#if currentCategory || currentSearch}
        <a
          href="/"
          class="text-xs text-orange-400 hover:text-orange-300 font-semibold"
        >
          &larr; Back to Overview
        </a>
      {/if}
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
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3.5 sm:gap-4">
        {#each products as itemWithV}
          <ProductCard item={itemWithV.product || itemWithV} variants={itemWithV.variants} />
        {/each}
      </div>
    {/if}
  </section>
{/if}
