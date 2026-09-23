<script>
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
    Eye
  } from 'lucide-svelte';

  export let data;
  let settings = data.settings || {};
  let products = data.products || [];

  let isSaving = false;
  let successNotice = '';
  let isUploadingLogo = false;
  let logoUploadSavings = '';

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

  // Compression toggle
  let compressImage = true;

  async function handleLogoUpload(event) {
    const file = event.target.files && event.target.files[0];
    if (!file) return;

    isUploadingLogo = true;
    logoUploadSavings = '';

    try {
      let fileToUpload = file;
      const originalSize = file.size;

      if (compressImage && file.type.startsWith('image/')) {
        const bitmap = await createImageBitmap(file);
        const canvas = document.createElement('canvas');
        const maxDim = 800;
        let width = bitmap.width;
        let height = bitmap.height;

        if (width > maxDim || height > maxDim) {
          if (width > height) {
            height = Math.round((height * maxDim) / width);
            width = maxDim;
          } else {
            width = Math.round((width * maxDim) / height);
            height = maxDim;
          }
        }

        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext('2d');
        ctx.drawImage(bitmap, 0, 0, width, height);

        const blob = await new Promise((resolve) => {
          canvas.toBlob((b) => resolve(b), 'image/webp', 0.85);
        });

        if (blob) {
          fileToUpload = new File([blob], 'shop_logo.webp', { type: 'image/webp' });
          const savedPercent = Math.round((1 - blob.size / originalSize) * 100);
          logoUploadSavings = `Compressed: ${(originalSize / 1024).toFixed(0)}KB → ${(blob.size / 1024).toFixed(0)}KB (-${savedPercent}%)`;
        }
      }

      const formData = new FormData();
      formData.append('file', fileToUpload);

      const res = await fetch('/api/v1/admin/media/upload', {
        method: 'POST',
        headers: { 'X-Dev-Mode': 'true' },
        body: formData
      });

      if (res.ok) {
        const data = await res.json();
        settings.logo_url = data.url;
      }
    } catch (e) {
      console.error('Logo upload failed:', e);
      alert('Failed to upload logo.');
    } finally {
      isUploadingLogo = false;
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
    heroConfig.featured_buttons[idx].subtitle = `From ${(prod.base_price_cents / 100).toFixed(2)} €`;
    heroConfig.featured_buttons[idx].image_url = prod.image_url;
    heroConfig.featured_buttons[idx].link_url = `/products/${prod.slug}`;
  }

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
          phone: settings.phone,
          company_address: settings.company_address,
          vat_id: settings.vat_id,
          logo_url: settings.logo_url,
          hero_config: heroConfig
        })
      });

      if (res.ok) {
        successNotice = 'Shop identity, logo, hero showcase & system settings saved successfully!';
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
        Shop Identity, Hero Showcase & System
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Configure shop branding, logo uploads, 8BitDo/8BitMods-style homepage showcase layouts, and company legal info.
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
    <!-- Card 1: Shop Logo & Branding -->
    <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
      <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
        <Building size={18} class="text-orange-400" />
        <span>Shop Logo & Header Identity</span>
      </h2>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-6 items-center">
        <!-- Logo Upload Box -->
        <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3 text-xs">
          <div class="flex items-center justify-between">
            <span class="font-semibold text-slate-300">Upload Shop Logo (Image onto Server)</span>
            <label class="flex items-center gap-1.5 cursor-pointer text-[11px] text-orange-400 font-semibold">
              <input type="checkbox" bind:checked={compressImage} class="rounded accent-orange-500" />
              <span>Compress (WebP)</span>
            </label>
          </div>

          <div class="flex items-center gap-3">
            <input type="file" accept="image/*" on:change={handleLogoUpload} class="text-xs text-slate-400 file:mr-3 file:py-1.5 file:px-3 file:rounded-lg file:border-0 file:text-xs file:bg-slate-800 file:text-white" />
            {#if isUploadingLogo}
              <span class="text-xs text-orange-400 animate-pulse font-mono">Uploading...</span>
            {/if}
          </div>

          {#if logoUploadSavings}
            <div class="text-[11px] text-emerald-400 font-mono">{logoUploadSavings}</div>
          {/if}

          <div>
            <label class="block text-slate-400 mb-1 text-[11px]">Logo URL</label>
            <input type="text" bind:value={settings.logo_url} placeholder="/uploads/... or https://..." class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-slate-300 font-mono text-xs focus:outline-none focus:border-orange-500" />
          </div>
        </div>

        <!-- Logo Live Preview -->
        <div class="p-6 rounded-xl bg-slate-950 border border-slate-800 flex flex-col items-center justify-center space-y-2">
          <span class="text-xs font-semibold text-slate-400">Header Preview</span>
          {#if settings.logo_url}
            <img src={settings.logo_url} alt="Shop Logo Preview" class="h-12 max-w-[200px] object-contain rounded-lg p-1 bg-slate-900 border border-slate-800 shadow" />
          {:else}
            <div class="h-12 px-4 rounded-xl bg-slate-900 border border-slate-800 flex items-center gap-2 text-slate-400 text-xs font-bold">
              <span>🦀</span>
              <span>{settings.store_name || 'RustCraft'}</span>
            </div>
          {/if}
          <span class="text-[10px] text-slate-500">Displayed in storefront top navigation and invoices</span>
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
          Choose the hero layout displayed at the top of the user store front:
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
            60% width carousel on the left + 40% width grid of 4 product feature buttons on the right with custom colors (similar to <strong>8bitmods.com</strong>).
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
            <p class="text-xs text-slate-400">Configure slides shown in the carousel:</p>
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
                  <!-- Auto-fill from Product -->
                  <select
                    on:change={(e) => applyProductToSlide(idx, e.target.value)}
                    class="px-2 py-1 rounded bg-slate-900 border border-slate-800 text-slate-300 text-[11px]"
                  >
                    <option value="">-- Copy from Product --</option>
                    {#each products as p}
                      <option value={p.id}>{p.title}</option>
                    {/each}
                  </select>
                  <button
                    type="button"
                    on:click={() => removeCarouselSlide(idx)}
                    class="p-1 text-slate-500 hover:text-rose-400"
                    title="Remove Slide"
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
                  <label class="block text-slate-400 mb-1">Button Text</label>
                  <input type="text" bind:value={slide.button_text} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
                </div>
              </div>

              <div>
                <label class="block text-slate-400 mb-1">Subtitle / Summary</label>
                <input type="text" bind:value={slide.subtitle} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
              </div>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label class="block text-slate-400 mb-1">Image URL</label>
                  <input type="text" bind:value={slide.image_url} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono text-[11px]" />
                </div>
                <div>
                  <label class="block text-slate-400 mb-1">Target Link URL</label>
                  <input type="text" bind:value={slide.link_url} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono text-[11px]" />
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Option B: 4 Featured Buttons Configuration (Visible if Split mode) -->
      {#if heroConfig.layout === 'split'}
        <div class="space-y-4 pt-4 border-t border-slate-800">
          <div>
            <h3 class="text-sm font-bold text-white flex items-center gap-2">
              <Layers size={16} class="text-sky-400" />
              <span>4 Featured Product Buttons (8BitMods style)</span>
            </h3>
            <p class="text-xs text-slate-400">
              Customize the 4 interactive product feature buttons and their specific background colors:
            </p>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {#each heroConfig.featured_buttons as btn, idx}
              <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3 text-xs">
                <div class="flex items-center justify-between">
                  <span class="font-bold text-white">Button #{idx + 1}</span>
                  <!-- Quick Product Selector -->
                  <select
                    on:change={(e) => applyProductToButton(idx, e.target.value)}
                    class="px-2 py-1 rounded bg-slate-900 border border-slate-800 text-slate-300 text-[11px]"
                  >
                    <option value="">-- Auto-fill Product --</option>
                    {#each products as p}
                      <option value={p.id}>{p.title}</option>
                    {/each}
                  </select>
                </div>

                <div class="grid grid-cols-2 gap-2">
                  <div>
                    <label class="block text-slate-400 mb-1">Title</label>
                    <input type="text" bind:value={btn.title} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
                  </div>
                  <div>
                    <label class="block text-slate-400 mb-1">Subtitle / Price</label>
                    <input type="text" bind:value={btn.subtitle} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-2 items-center">
                  <div>
                    <label class="block text-slate-400 mb-1">Button Color</label>
                    <div class="flex items-center gap-2">
                      <input type="color" bind:value={btn.bg_color} class="w-8 h-8 rounded-lg cursor-pointer bg-transparent border border-slate-700" />
                      <input type="text" bind:value={btn.bg_color} class="w-full px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono text-[11px]" />
                    </div>
                  </div>
                  <div>
                    <label class="block text-slate-400 mb-1">Target Link</label>
                    <input type="text" bind:value={btn.link_url} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono text-[11px]" />
                  </div>
                </div>

                <div>
                  <label class="block text-slate-400 mb-1">Picture URL</label>
                  <input type="text" bind:value={btn.image_url} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono text-[11px]" />
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <!-- Card 3: Store Profile & Legal Information -->
    <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
      <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
        <Building size={18} class="text-orange-400" />
        <span>Legal Entity, Address, Email & Tax Settings</span>
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
          <label for="admin-support-email" class="block font-semibold text-slate-400 mb-1">Official Support Email</label>
          <input
            id="admin-support-email"
            type="email"
            bind:value={settings.support_email}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="admin-phone" class="block font-semibold text-slate-400 mb-1">Customer Service Telephone</label>
          <input
            id="admin-phone"
            type="text"
            bind:value={settings.phone}
            placeholder="+49 (0) 30 123456-78"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
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

        <div>
          <label for="admin-currency" class="block font-semibold text-slate-400 mb-1">Default Base Currency</label>
          <select bind:value={settings.currency} class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-bold focus:outline-none focus:border-orange-500">
            <option value="EUR">EUR (€)</option>
            <option value="USD">USD ($)</option>
            <option value="GBP">GBP (£)</option>
            <option value="CHF">CHF (Fr.)</option>
          </select>
        </div>

        <div class="sm:col-span-2">
          <label for="admin-company-address" class="block font-semibold text-slate-400 mb-1">Official Company Address (Rendered on Packing Slips & Tax Invoices)</label>
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

    <!-- Card 4: Operational Environment & Mode Toggles -->
    <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
      <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
        <Terminal size={18} class="text-orange-400" />
        <span>Operational Environment & Mode Toggles</span>
      </h2>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
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
        </div>

        <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 flex flex-col justify-between">
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold text-white uppercase tracking-wider">Debug Engine Mode</span>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" bind:checked={settings.debug_mode} class="sr-only peer" />
              <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-600"></div>
            </label>
          </div>
          <div class="mt-3 text-[11px] font-mono font-semibold {settings.debug_mode ? 'text-emerald-400' : 'text-slate-500'}">
            Status: {settings.debug_mode ? 'ACTIVE (Testing Enabled)' : 'OFF (Strict Live Handling)'}
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
