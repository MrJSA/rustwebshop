<script>
  import MediaPickerModal from '$lib/components/MediaPickerModal.svelte';
  import {
    Sliders,
    Save,
    Check,
    Terminal,
    Upload,
    Image,
    Sparkles,
    CheckCircle2,
    Layout,
    Layers,
    Plus,
    Trash2,
    Phone,
    Mail,
    Building,
    Eye,
    FolderOpen,
    ArrowUp,
    ArrowDown
  } from 'lucide-svelte';

  export let data;
  let settings = data.settings || {};
  let products = data.products || [];

  let isSaving = false;
  let successNotice = '';
  let showMediaPicker = false;
  let mediaPickerTarget = null; // callback or field identifier

  // Branding toggles
  let showStoreTitle = settings.show_store_title !== undefined ? Boolean(settings.show_store_title) : true;
  let showStoreSubtitle = settings.show_store_subtitle !== undefined ? Boolean(settings.show_store_subtitle) : true;
  let storeSubtitle = settings.store_subtitle || 'Rust Powered • ACID Fast';

  // Cookie Consent Banner settings
  let cookieBannerEnabled = settings.cookie_banner_enabled !== undefined ? Boolean(settings.cookie_banner_enabled) : true;
  let cookieBannerTitle = settings.cookie_banner_title || 'We respect your privacy';
  let cookieBannerDescription = settings.cookie_banner_description || 'We use cookies and similar technologies to ensure our website works safely and properly, analyze usage patterns, and improve your shopping experience under GDPR regulations.';
  let cookiePolicyUrl = settings.cookie_policy_url || '/policies/cookie-policy';
  let cookieBannerAcceptText = settings.cookie_banner_accept_text || 'Accept All';
  let cookieBannerDeclineText = settings.cookie_banner_decline_text || 'Decline Optional';
  let cookieBannerPreferencesText = settings.cookie_banner_preferences_text || 'Cookie Preferences';

  // Ensure hero_config has valid structure
  let heroConfig = settings.hero_config && typeof settings.hero_config === 'object'
    ? settings.hero_config
    : {
        layout: 'split',
        carousel_items: [],
        featured_buttons: []
      };

  if (!heroConfig.layout) heroConfig.layout = 'split';
  if (!heroConfig.carousel_items) heroConfig.carousel_items = [];
  if (!heroConfig.featured_buttons) heroConfig.featured_buttons = [];

  // Ensure carousels_config
  let carouselsConfig = settings.carousels_config && typeof settings.carousels_config === 'object' && settings.carousels_config.sections
    ? settings.carousels_config
    : {
        sections: [
          { id: 'featured', title: 'Featured Gear', enabled: true, product_ids: [] },
          { id: 'new', title: 'New Arrivals', enabled: true, days: 30 },
          { id: 'bestsellers', title: 'Best Sellers', enabled: true, limit: 10 },
          { id: 'catalog', title: 'In Stock Hardware & Gear', enabled: true }
        ]
      };

  function openPicker(target) {
    mediaPickerTarget = target;
    showMediaPicker = true;
  }

  function handleMediaSelected(url) {
    if (mediaPickerTarget === 'logo') {
      settings.logo_url = url;
    } else if (typeof mediaPickerTarget === 'function') {
      mediaPickerTarget(url);
    }
  }

  function addCarouselSlide() {
    heroConfig.carousel_items = [
      ...heroConfig.carousel_items,
      {
        id: 'c_' + Date.now(),
        title: 'New Highlight Product',
        subtitle: 'Engineered for extreme performance and precision',
        image_url: 'https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=1200&q=80',
        link_url: '/products/rust-mechanical-keyboard',
        button_text: 'Explore Hardware'
      }
    ];
  }

  function removeCarouselSlide(idx) {
    heroConfig.carousel_items = heroConfig.carousel_items.filter((_, i) => i !== idx);
  }

  function applyProductToSlide(idx, productId) {
    const prod = products.find((p) => p.id === productId);
    if (!prod) return;
    heroConfig.carousel_items[idx].title = prod.title;
    heroConfig.carousel_items[idx].subtitle = prod.description || 'Precision crafted';
    heroConfig.carousel_items[idx].image_url = prod.image_url;
    heroConfig.carousel_items[idx].link_url = `/products/${prod.slug}`;
  }

  function applyProductToButton(idx, productId) {
    const prod = products.find((p) => p.id === productId);
    if (!prod) return;
    heroConfig.featured_buttons[idx].title = prod.title;
    heroConfig.featured_buttons[idx].subtitle = prod.description ? prod.description.substring(0, 36) + '...' : 'Precision engineered';
    heroConfig.featured_buttons[idx].price = `${(prod.base_price_cents / 100).toFixed(2)} €`;
    heroConfig.featured_buttons[idx].show_price = true;
    heroConfig.featured_buttons[idx].image_url = prod.image_url;
    heroConfig.featured_buttons[idx].link_url = `/products/${prod.slug}`;
  }

  function moveCarouselSection(index, direction) {
    const targetIdx = index + direction;
    if (targetIdx < 0 || targetIdx >= carouselsConfig.sections.length) return;
    const temp = carouselsConfig.sections[index];
    carouselsConfig.sections[index] = carouselsConfig.sections[targetIdx];
    carouselsConfig.sections[targetIdx] = temp;
    carouselsConfig = { ...carouselsConfig };
  }

  async function handleSaveSettings() {
    isSaving = true;
    successNotice = '';
    const token = localStorage.getItem('admin_token');

    try {
      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          ...settings,
          show_store_title: showStoreTitle,
          show_store_subtitle: showStoreSubtitle,
          store_subtitle: storeSubtitle,
          cookie_banner_enabled: cookieBannerEnabled,
          cookie_banner_title: cookieBannerTitle,
          cookie_banner_description: cookieBannerDescription,
          cookie_policy_url: cookiePolicyUrl,
          cookie_banner_accept_text: cookieBannerAcceptText,
          cookie_banner_decline_text: cookieBannerDeclineText,
          cookie_banner_preferences_text: cookieBannerPreferencesText,
          hero_config: heroConfig,
          carousels_config: carouselsConfig
        })
      });

      if (res.ok) {
        successNotice = 'Shop identity, logo, cookie banner & carousels saved successfully!';
        setTimeout(() => successNotice = '', 4000);
      }
    } catch (e) {
      console.error('Failed to update system settings:', e);
    } finally {
      isSaving = false;
    }
  }
</script>

<svelte:head>
  <title>System & Shop Identity Settings | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-5xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Sliders size={24} class="text-orange-500" />
        Shop Identity, Branding & Showcase Layouts
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Configure shop branding, logo uploads, 8BitDo/8BitMods-style hero layouts, and 5-per-row product carousels.
      </p>
    </div>

    <button
      on:click={handleSaveSettings}
      disabled={isSaving}
      class="px-6 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all flex items-center gap-2 disabled:opacity-50 self-start sm:self-auto"
    >
      <Save size={16} />
      <span>{isSaving ? 'Saving Changes...' : 'Save All Settings'}</span>
    </button>
  </div>

  {#if successNotice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <Check size={16} />
      <span>{successNotice}</span>
    </div>
  {/if}

  <form on:submit|preventDefault={handleSaveSettings} class="space-y-6">
    <!-- Card 1: Shop Logo & Header Identity -->
    <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-5">
      <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
        <Building size={18} class="text-orange-400" />
        <span>Shop Logo & Header Branding</span>
      </h2>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-6 items-center">
        <!-- Logo Upload / Picker Box -->
        <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3 text-xs">
          <span class="font-semibold text-slate-300 block">Shop Logo Image</span>

          <div class="flex items-center gap-2">
            <button
              type="button"
              on:click={() => openPicker('logo')}
              class="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold flex items-center gap-1.5 transition-colors border border-slate-700"
            >
              <FolderOpen size={14} class="text-orange-400" />
              <span>Choose from Media Library</span>
            </button>
          </div>

          <div>
            <label class="block text-slate-400 mb-1 text-[11px]">Logo URL / Path</label>
            <input
              type="text"
              bind:value={settings.logo_url}
              placeholder="/uploads/... or https://..."
              class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-slate-300 font-mono text-xs focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <!-- Logo Live Preview -->
        <div class="p-6 rounded-xl bg-slate-950 border border-slate-800 flex flex-col items-center justify-center space-y-2">
          <span class="text-xs font-semibold text-slate-400">Header Preview</span>
          {#if settings.logo_url}
            <img src={settings.logo_url} alt="Shop Logo Preview" class="h-14 max-w-[220px] object-contain rounded-lg p-1 bg-slate-900 border border-slate-800 shadow" />
          {:else}
            <div class="h-12 px-4 rounded-xl bg-slate-900 border border-slate-800 flex items-center gap-2 text-slate-400 text-xs font-bold">
              <span>🦀</span>
              <span>{settings.store_name || 'RustCraft'}</span>
            </div>
          {/if}
          <span class="text-[10px] text-slate-500">Rendered in header navigation and order PDF documents</span>
        </div>
      </div>

      <!-- Header Title & Subtitle Toggles -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4 pt-4 border-t border-slate-800 text-xs">
        <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800/80 flex items-center justify-between">
          <div>
            <span class="font-bold text-white block">Display Store Name in Header</span>
            <p class="text-[11px] text-slate-400">If disabled, logo is enlarged and menu centers.</p>
          </div>
          <input type="checkbox" bind:checked={showStoreTitle} class="accent-orange-500 w-4 h-4 ml-2" />
        </div>

        <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800/80 flex items-center justify-between">
          <div>
            <span class="font-bold text-white block">Display Subtitle in Header</span>
            <p class="text-[11px] text-slate-400">Toggle the header badge tagline.</p>
          </div>
          <input type="checkbox" bind:checked={showStoreSubtitle} class="accent-orange-500 w-4 h-4 ml-2" />
        </div>

        <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800/80">
          <label class="font-bold text-white block mb-1">Custom Header Subtitle</label>
          <input
            type="text"
            bind:value={storeSubtitle}
            placeholder="Rust Powered • ACID Fast"
            class="w-full px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
          />
        </div>
      </div>
    </div>

    <!-- Card 2: Homepage Hero Showcase Layout (8BitDo vs 8BitMods Style) -->
    <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
      <div>
        <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
          <Layout size={18} class="text-orange-400" />
          <span>Homepage Hero Showcase Layout</span>
        </h2>
        <p class="text-xs text-slate-400 mt-2">
          Choose between full-width widescreen item carousel or split hero with floating product buttons:
        </p>
      </div>

      <!-- Layout Choice Radio -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <!-- Option A: Full-Width Carousel -->
        <label
          class="p-4 rounded-2xl border-2 transition-all cursor-pointer flex flex-col justify-between {heroConfig.layout === 'carousel' ? 'border-orange-500 bg-orange-500/10' : 'border-slate-800 bg-slate-950/60 hover:border-slate-700'}"
        >
          <div class="flex items-center gap-3 mb-2">
            <input
              type="radio"
              bind:group={heroConfig.layout}
              value="carousel"
              class="accent-orange-500 w-4 h-4"
            />
            <span class="font-bold text-white text-sm">Full-Width Item Carousel</span>
          </div>
          <p class="text-xs text-slate-400 leading-relaxed">
            Widescreen dynamic slider spanning full screen width with product slides and headlines (similar to <strong>8bitdo.com</strong>).
          </p>
        </label>

        <!-- Option B: Split 60/40 Hero -->
        <label
          class="p-4 rounded-2xl border-2 transition-all cursor-pointer flex flex-col justify-between {heroConfig.layout === 'split' ? 'border-orange-500 bg-orange-500/10' : 'border-slate-800 bg-slate-950/60 hover:border-slate-700'}"
        >
          <div class="flex items-center gap-3 mb-2">
            <input
              type="radio"
              bind:group={heroConfig.layout}
              value="split"
              class="accent-orange-500 w-4 h-4"
            />
            <span class="font-bold text-white text-sm">Split Hero (60% Slider + 40% 4 Featured Buttons)</span>
          </div>
          <p class="text-xs text-slate-400 leading-relaxed">
            60% width carousel on the left + 40% width grid of 4 product feature buttons on the right with depth and floating images (similar to <strong>8bitmods.com</strong>).
          </p>
        </label>
      </div>

      <!-- Carousel Slides Configuration -->
      <div class="space-y-4 pt-4 border-t border-slate-800">
        <div class="flex items-center justify-between">
          <div>
            <h3 class="text-sm font-bold text-white flex items-center gap-2">
              <Sparkles size={16} class="text-orange-400" />
              <span>Carousel Slides</span>
            </h3>
            <p class="text-xs text-slate-400">Configure slides shown in the hero carousel:</p>
          </div>
          <button
            type="button"
            on:click={addCarouselSlide}
            class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold flex items-center gap-1"
          >
            <Plus size={14} />
            <span>Add Slide</span>
          </button>
        </div>

        <div class="space-y-4">
          {#each heroConfig.carousel_items as slide, idx}
            <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3 text-xs">
              <div class="flex items-center justify-between">
                <span class="font-bold text-orange-400">Slide #{idx + 1}</span>
                <div class="flex items-center gap-2">
                  <select
                    on:change={(e) => applyProductToSlide(idx, e.target.value)}
                    class="px-2.5 py-1 rounded bg-slate-900 border border-slate-800 text-slate-300 text-[11px]"
                  >
                    <option value="">Auto-fill from Product...</option>
                    {#each products as prod}
                      <option value={prod.id}>{prod.title}</option>
                    {/each}
                  </select>
                  <button
                    type="button"
                    on:click={() => removeCarouselSlide(idx)}
                    class="p-1 rounded text-slate-400 hover:text-rose-400"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label class="block text-slate-400 mb-1">Headline Title</label>
                  <input type="text" bind:value={slide.title} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
                </div>
                <div>
                  <label class="block text-slate-400 mb-1">Target Link URL</label>
                  <input type="text" bind:value={slide.link_url} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono text-[11px]" />
                </div>
                <div class="sm:col-span-2">
                  <label class="block text-slate-400 mb-1">Slide Subtitle</label>
                  <input type="text" bind:value={slide.subtitle} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
                </div>
                <div class="sm:col-span-2 flex items-center gap-2">
                  <input type="text" bind:value={slide.image_url} placeholder="Image URL" class="flex-1 px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono text-[11px]" />
                  <button
                    type="button"
                    on:click={() => openPicker((url) => slide.image_url = url)}
                    class="px-3 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold"
                  >
                    Pick
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Option B: 4 Featured Buttons Configuration -->
      {#if heroConfig.layout === 'split'}
        <div class="space-y-4 pt-4 border-t border-slate-800">
          <div class="flex items-center justify-between">
            <div>
              <h3 class="text-sm font-bold text-white flex items-center gap-2">
                <Layers size={16} class="text-orange-400" />
                <span>8BitMods-Style 4 Featured Product Buttons</span>
              </h3>
              <p class="text-xs text-slate-400">Configure the 4 buttons on the right with floating depth images:</p>
            </div>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {#each heroConfig.featured_buttons as btn, idx}
              <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3 text-xs">
                <div class="flex items-center justify-between">
                  <span class="font-bold text-orange-400">Button #{idx + 1}</span>
                  <select
                    on:change={(e) => applyProductToButton(idx, e.target.value)}
                    class="px-2 py-1 rounded bg-slate-900 border border-slate-800 text-slate-300 text-[10px]"
                  >
                    <option value="">Auto-fill...</option>
                    {#each products as prod}
                      <option value={prod.id}>{prod.title}</option>
                    {/each}
                  </select>
                </div>

                <div class="grid grid-cols-2 gap-2">
                  <div>
                    <label class="block text-slate-400 mb-1">Title</label>
                    <input type="text" bind:value={btn.title} class="w-full px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white" />
                  </div>
                  <div>
                    <label class="block text-slate-400 mb-1">Subtitle</label>
                    <input type="text" bind:value={btn.subtitle} class="w-full px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white" />
                  </div>
                  <div>
                    <label class="block text-slate-400 mb-1">Price Text</label>
                    <input type="text" bind:value={btn.price} placeholder="189.00 €" class="w-full px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono" />
                  </div>
                  <div>
                    <label class="block text-slate-400 mb-1">Button Color</label>
                    <div class="flex items-center gap-2">
                      <input type="color" bind:value={btn.bg_color} class="w-8 h-8 rounded border-0 cursor-pointer bg-transparent" />
                      <input type="text" bind:value={btn.bg_color} class="w-full px-2 py-1 rounded bg-slate-900 border border-slate-800 text-white font-mono text-[11px]" />
                    </div>
                  </div>
                  <div class="col-span-2">
                    <label class="flex items-center gap-1.5 cursor-pointer text-slate-300 text-[11px]">
                      <input type="checkbox" bind:checked={btn.show_price} class="accent-orange-500" />
                      <span>Show Price Badge on Button</span>
                    </label>
                  </div>
                  <div class="col-span-2 flex items-center gap-2">
                    <input type="text" bind:value={btn.image_url} placeholder="Floating product image URL" class="flex-1 px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono text-[11px]" />
                    <button
                      type="button"
                      on:click={() => openPicker((url) => btn.image_url = url)}
                      class="px-2.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold"
                    >
                      Pick
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <!-- Card 3: 5-Item Responsive Product Carousels Ordering & Config -->
    <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-slate-800">
        <div>
          <h2 class="text-base font-bold text-white flex items-center gap-2">
            <Layers size={18} class="text-orange-400" />
            <span>Storefront Product Carousels Ordering & Rules</span>
          </h2>
          <p class="text-xs text-slate-400 mt-1">
            Display 5 products per row (centered if &le; 5, smooth horizontal carousel if &gt; 5). Reorder or toggle sections:
          </p>
        </div>
      </div>

      <div class="space-y-3">
        {#each carouselsConfig.sections as sec, idx}
          <div class="p-4 rounded-xl bg-slate-950 border border-slate-800/80 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 text-xs">
            <div class="flex items-center gap-3">
              <div class="flex flex-col gap-1">
                <button
                  type="button"
                  disabled={idx === 0}
                  on:click={() => moveCarouselSection(idx, -1)}
                  class="p-1 rounded bg-slate-900 hover:bg-slate-800 text-slate-400 hover:text-white disabled:opacity-20"
                >
                  <ArrowUp size={12} />
                </button>
                <button
                  type="button"
                  disabled={idx === carouselsConfig.sections.length - 1}
                  on:click={() => moveCarouselSection(idx, 1)}
                  class="p-1 rounded bg-slate-900 hover:bg-slate-800 text-slate-400 hover:text-white disabled:opacity-20"
                >
                  <ArrowDown size={12} />
                </button>
              </div>

              <div>
                <div class="flex items-center gap-2">
                  <span class="font-bold text-white text-sm">{sec.title}</span>
                  <span class="px-2 py-0.5 rounded bg-slate-900 font-mono text-[10px] text-orange-400">ID: {sec.id}</span>
                </div>
                {#if sec.id === 'new'}
                  <div class="mt-1 flex items-center gap-2 text-slate-400">
                    <span>Products added within last</span>
                    <input type="number" bind:value={sec.days} class="w-16 px-2 py-0.5 rounded bg-slate-900 border border-slate-800 text-white font-mono" />
                    <span>days</span>
                  </div>
                {:else if sec.id === 'bestsellers'}
                  <div class="mt-1 flex items-center gap-2 text-slate-400">
                    <span>Limit to top</span>
                    <input type="number" bind:value={sec.limit} class="w-16 px-2 py-0.5 rounded bg-slate-900 border border-slate-800 text-white font-mono" />
                    <span>best-selling products</span>
                  </div>
                {:else if sec.id === 'catalog'}
                  <p class="text-[11px] text-slate-400 mt-1">Automatically shows all in-stock products.</p>
                {/if}
              </div>
            </div>

            <div class="flex items-center gap-4">
              <label class="flex items-center gap-2 cursor-pointer font-semibold text-slate-300">
                <span>Enabled</span>
                <input type="checkbox" bind:checked={sec.enabled} class="accent-orange-500 w-4 h-4" />
              </label>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- EU-Conform Cookie Consent Banner Configuration Card -->
    <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between pb-4 border-b border-slate-800 gap-4">
        <div>
          <h2 class="text-base font-bold text-white flex items-center gap-2">
            <span class="text-xl">🍪</span>
            <span>EU-Conform Cookie Consent Banner & GDPR Settings</span>
          </h2>
          <p class="text-xs text-slate-400 mt-0.5">
            Configure the customer storefront cookie banner, consent categories (Necessary, Analytics, Marketing), and privacy policy references.
          </p>
        </div>

        <label class="flex items-center gap-2.5 cursor-pointer bg-slate-950 px-4 py-2 rounded-xl border border-slate-800 self-start sm:self-auto">
          <input
            type="checkbox"
            bind:checked={cookieBannerEnabled}
            class="accent-orange-500 w-4 h-4 rounded"
          />
          <span class="text-xs font-bold {cookieBannerEnabled ? 'text-emerald-400' : 'text-slate-400'}">
            {cookieBannerEnabled ? 'Banner Enabled' : 'Banner Disabled'}
          </span>
        </label>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Banner Title</label>
          <input
            type="text"
            bind:value={cookieBannerTitle}
            placeholder="e.g. We respect your privacy"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Cookie Policy Link URL</label>
          <input
            type="text"
            bind:value={cookiePolicyUrl}
            placeholder="/policies/cookie-policy"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="sm:col-span-2">
          <label class="block text-slate-300 font-semibold mb-1">Banner Description Text</label>
          <textarea
            bind:value={cookieBannerDescription}
            rows="2"
            placeholder="Explain to visitors how cookies and local storage are used for necessary operation, analytics, and marketing..."
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white leading-relaxed focus:outline-none focus:border-orange-500"
          ></textarea>
        </div>

        <div class="sm:col-span-2 grid grid-cols-1 sm:grid-cols-3 gap-3 pt-2 border-t border-slate-800/80">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Accept All Button Label</label>
            <input
              type="text"
              bind:value={cookieBannerAcceptText}
              placeholder="Accept All"
              class="w-full px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Decline Optional Button Label</label>
            <input
              type="text"
              bind:value={cookieBannerDeclineText}
              placeholder="Decline Optional"
              class="w-full px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Preferences Modal Button Label</label>
            <input
              type="text"
              bind:value={cookieBannerPreferencesText}
              placeholder="Cookie Preferences"
              class="w-full px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Submit Action -->
    <div class="flex justify-end">
      <button
        type="submit"
        disabled={isSaving}
        class="px-8 py-3 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all flex items-center gap-2 disabled:opacity-50"
      >
        <Save size={16} />
        <span>{isSaving ? 'Updating Settings...' : 'Save All Settings'}</span>
      </button>
    </div>
  </form>
</div>

<!-- Reusable Media Picker Modal -->
<MediaPickerModal
  open={showMediaPicker}
  onSelect={handleMediaSelected}
  onClose={() => showMediaPicker = false}
/>
