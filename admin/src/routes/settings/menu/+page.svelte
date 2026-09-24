<script>
  import { Menu, Plus, Trash2, Check, ExternalLink, ArrowRight, X, Link, HelpCircle, Compass, ArrowUp, ArrowDown, Edit2 } from 'lucide-svelte';

  export let data;
  let menuItems = data.menuItems || [];
  let categories = data.categories || [];
  let pages = data.pages || [];

  let activeTab = 'header'; // 'header' or 'footer'
  let isModalOpen = false;
  let isEditModalOpen = false;
  let editingItemId = null;
  let editLabel = '';
  let editUrl = '/';
  let editSortOrder = 1;

  let newLabel = '';
  let newUrl = '/';
  let newSortOrder = 1;
  let selectedPreset = '';
  let isSaving = false;

  $: currentItems = menuItems
    .filter(i => (i.location || 'header') === activeTab)
    .sort((a, b) => a.sort_order - b.sort_order);

  // Predefined store routes
  const standardRoutes = [
    { label: 'Home / Overview', url: '/' },
    { label: 'All Products Catalog', url: '/?category=' },
    { label: 'Track Order', url: '/track' },
    { label: 'Customer Login / Register', url: '/account/login' },
    { label: 'Shopping Cart Checkout', url: '/checkout' }
  ];

  $: categoryRoutes = categories.map(c => ({
    label: `Category: ${c.name}`,
    url: `/?category=${encodeURIComponent(c.name)}`
  }));

  $: cmsPageRoutes = pages.map(p => ({
    label: `Policy / Page: ${p.title}`,
    url: `/policies/${p.slug}`
  }));

  $: allPossibleLinks = [
    ...standardRoutes,
    ...categoryRoutes,
    ...cmsPageRoutes
  ];

  function handlePresetSelect(e) {
    const val = e.target.value;
    if (!val) return;
    const found = allPossibleLinks.find(l => l.url === val);
    if (found) {
      newLabel = found.label.replace(/^Category: |^Policy \/ Page: /, '');
      newUrl = found.url;
    }
  }

  function openCreateModal() {
    newLabel = '';
    newUrl = '/';
    selectedPreset = '';
    newSortOrder = currentItems.length + 1;
    isModalOpen = true;
  }

  async function handleCreateItem() {
    isSaving = true;
    const token = localStorage.getItem('admin_token');

    try {
      const res = await fetch('/api/v1/admin/menu', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          label: newLabel,
          url: newUrl,
          sort_order: parseInt(newSortOrder) || 1,
          is_active: true,
          location: activeTab
        })
      });

      if (res.ok) {
        await reloadMenu();
        isModalOpen = false;
      }
    } catch (e) {
      console.error('Failed to create menu item:', e);
    } finally {
      isSaving = false;
    }
  }

  async function reloadMenu() {
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch('/api/v1/admin/menu', {
        headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) }
      });
      if (res.ok) {
        menuItems = await res.json();
      }
    } catch (e) {
      console.error('Failed to reload menu:', e);
    }
  }

  async function handleDeleteItem(id) {
    if (!confirm('Are you sure you want to remove this navigation item?')) return;
    const token = localStorage.getItem('admin_token');

    try {
      const res = await fetch(`/api/v1/admin/menu/${id}`, {
        method: 'DELETE',
        headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) }
      });
      if (res.ok) {
        menuItems = menuItems.filter((i) => i.id !== id);
      }
    } catch (e) {
      console.error('Failed to delete item:', e);
    }
  }

  function openEditModal(item) {
    editingItemId = item.id;
    editLabel = item.label;
    editUrl = item.url;
    editSortOrder = item.sort_order;
    isEditModalOpen = true;
  }

  async function handleEditItem() {
    isSaving = true;
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/menu/${editingItemId}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          label: editLabel,
          url: editUrl,
          sort_order: parseInt(editSortOrder) || 1,
          is_active: true,
          location: activeTab
        })
      });
      if (res.ok) {
        await reloadMenu();
        isEditModalOpen = false;
      }
    } catch (e) {
      console.error('Failed to update menu item:', e);
    } finally {
      isSaving = false;
    }
  }

  async function moveItem(index, direction) {
    const newIndex = index + direction;
    if (newIndex < 0 || newIndex >= currentItems.length) return;
    const itemsCopy = [...currentItems];
    const temp = itemsCopy[index];
    itemsCopy[index] = itemsCopy[newIndex];
    itemsCopy[newIndex] = temp;

    const reorderedPayload = itemsCopy.map((item, idx) => ({
      id: item.id,
      sort_order: idx + 1
    }));

    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch('/api/v1/admin/menu/reorder', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({ items: reorderedPayload })
      });
      if (res.ok) {
        await reloadMenu();
      }
    } catch (e) {
      console.error('Failed to reorder menu items:', e);
    }
  }
</script>

<svelte:head>
  <title>Navigation Menus Customizer | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-6xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Menu size={24} class="text-orange-500" />
        Navigation Menus & Page Selector
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Configure separate menus for the Header navigation bar and the Footer links.
      </p>
    </div>

    <button
      on:click={openCreateModal}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md shadow-orange-600/20 flex items-center gap-2 self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>Add Link to {activeTab === 'header' ? 'Header' : 'Footer'}</span>
    </button>
  </div>

  <!-- Menu Location Tabs -->
  <div class="flex gap-2 border-b border-slate-800 pb-3">
    <button
      type="button"
      on:click={() => activeTab = 'header'}
      class="px-5 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'header' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/30' : 'bg-slate-900 text-slate-400 hover:text-white'}"
    >
      <Compass size={15} />
      <span>Header Navigation Menu ({menuItems.filter(i => (i.location || 'header') === 'header').length})</span>
    </button>

    <button
      type="button"
      on:click={() => activeTab = 'footer'}
      class="px-5 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'footer' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/30' : 'bg-slate-900 text-slate-400 hover:text-white'}"
    >
      <Link size={15} />
      <span>Footer Links Menu ({menuItems.filter(i => i.location === 'footer').length})</span>
    </button>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
    <!-- Items Table (8 cols on lg) -->
    <div class="lg:col-span-8 p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-sm font-bold text-white uppercase tracking-wider">
          {activeTab === 'header' ? 'Storefront Header Links' : 'Storefront Footer Links'}
        </h3>
        <span class="text-xs text-slate-400 font-mono">{currentItems.length} active links</span>
      </div>

      {#if currentItems.length === 0}
        <div class="py-16 text-center text-xs text-slate-400">
          No navigation items configured for the {activeTab} menu. Click "Add Link" above.
        </div>
      {:else}
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead>
              <tr class="border-b border-slate-800 text-[11px] text-slate-400 uppercase tracking-wider">
                <th class="pb-3 font-semibold">Order</th>
                <th class="pb-3 font-semibold">Label</th>
                <th class="pb-3 font-semibold">Target URL</th>
                <th class="pb-3 font-semibold text-center">Status</th>
                <th class="pb-3 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800/60">
              {#each currentItems as item, idx}
                <tr class="text-slate-300">
                  <td class="py-3.5 font-mono text-slate-400 font-bold">
                    <div class="flex items-center gap-1.5">
                      <span>{item.sort_order}</span>
                      <div class="flex flex-col gap-0.5">
                        <button
                          type="button"
                          disabled={idx === 0}
                          on:click={() => moveItem(idx, -1)}
                          class="p-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 disabled:hover:bg-slate-800 text-slate-300 hover:text-white transition-colors"
                          title="Move Up"
                        >
                          <ArrowUp size={11} />
                        </button>
                        <button
                          type="button"
                          disabled={idx === currentItems.length - 1}
                          on:click={() => moveItem(idx, 1)}
                          class="p-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 disabled:hover:bg-slate-800 text-slate-300 hover:text-white transition-colors"
                          title="Move Down"
                        >
                          <ArrowDown size={11} />
                        </button>
                      </div>
                    </div>
                  </td>
                  <td class="py-3.5 font-bold text-white text-sm">{item.label}</td>
                  <td class="py-3.5 font-mono text-orange-400 text-xs">{item.url}</td>
                  <td class="py-3.5 text-center">
                    <button
                      type="button"
                      on:click={() => handleToggleActive(item)}
                      class="px-2.5 py-1 rounded-md text-[11px] font-bold transition-colors {item.is_active ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-slate-800 text-slate-500'}"
                    >
                      {item.is_active ? 'Active' : 'Hidden'}
                    </button>
                  </td>
                  <td class="py-3.5 text-right">
                    <div class="inline-flex items-center gap-1.5">
                      <button
                        type="button"
                        on:click={() => openEditModal(item)}
                        class="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors border border-slate-700"
                        title="Edit Link & Label"
                      >
                        <Edit2 size={13} class="text-orange-400" />
                      </button>
                      <button
                        type="button"
                        on:click={() => handleDeleteItem(item.id)}
                        class="p-2 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/20"
                        title="Remove Link"
                      >
                        <Trash2 size={13} />
                      </button>
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

    <!-- Available Pages Overview Panel (4 cols on lg) -->
    <div class="lg:col-span-4 p-5 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-4">
      <div class="flex items-center gap-2 text-white font-bold text-xs uppercase tracking-wider pb-3 border-b border-slate-800">
        <HelpCircle size={15} class="text-orange-400" />
        <span>Available Target Pages Overview</span>
      </div>

      <div class="space-y-4 max-h-[500px] overflow-y-auto text-xs pr-1">
        <!-- Standard Routes -->
        <div class="space-y-1.5">
          <p class="text-[11px] font-bold text-slate-400 uppercase tracking-wider">Core Store Routes</p>
          {#each standardRoutes as r}
            <div class="p-2 rounded-xl bg-slate-950/80 border border-slate-800/80 flex items-center justify-between">
              <span class="text-slate-200">{r.label}</span>
              <code class="text-[10px] text-orange-400 font-mono">{r.url}</code>
            </div>
          {/each}
        </div>

        <!-- Categories -->
        {#if categories.length > 0}
          <div class="space-y-1.5 pt-2 border-t border-slate-800/80">
            <p class="text-[11px] font-bold text-slate-400 uppercase tracking-wider">Product Categories ({categories.length})</p>
            {#each categories as c}
              <div class="p-2 rounded-xl bg-slate-950/80 border border-slate-800/80 flex items-center justify-between">
                <span class="text-slate-200 truncate pr-2">{c.name}</span>
                <code class="text-[10px] text-sky-400 font-mono truncate">/?category={c.name}</code>
              </div>
            {/each}
          </div>
        {/if}

        <!-- CMS Pages -->
        {#if pages.length > 0}
          <div class="space-y-1.5 pt-2 border-t border-slate-800/80">
            <p class="text-[11px] font-bold text-slate-400 uppercase tracking-wider">CMS Policy Pages ({pages.length})</p>
            {#each pages as p}
              <div class="p-2 rounded-xl bg-slate-950/80 border border-slate-800/80 flex items-center justify-between">
                <span class="text-slate-200 truncate pr-2">{p.title}</span>
                <code class="text-[10px] text-emerald-400 font-mono truncate">/policies/{p.slug}</code>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<!-- Add Navigation Item Modal -->
{#if isModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-3xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-slate-800">
        <h3 class="text-sm font-bold text-white">Add Link to {activeTab === 'header' ? 'Header' : 'Footer'} Menu</h3>
        <button
          on:click={() => isModalOpen = false}
          class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <form on:submit|preventDefault={handleCreateItem} class="space-y-4 text-xs">
        <!-- Quick Select Preset Page Dropdown -->
        <div>
          <label class="block text-slate-300 font-semibold mb-1">
            Choose from Existing Shop Pages (Auto-Fills Link & URL)
          </label>
          <select
            on:change={handlePresetSelect}
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 font-mono text-xs"
          >
            <option value="">-- Or select an existing page below --</option>
            <optgroup label="Core Store Pages">
              {#each standardRoutes as r}
                <option value={r.url}>{r.label} ({r.url})</option>
              {/each}
            </optgroup>
            {#if categories.length > 0}
              <optgroup label="Categories">
                {#each categoryRoutes as c}
                  <option value={c.url}>{c.label}</option>
                {/each}
              </optgroup>
            {/if}
            {#if pages.length > 0}
              <optgroup label="Policy & CMS Pages">
                {#each cmsPageRoutes as p}
                  <option value={p.url}>{p.label}</option>
                {/each}
              </optgroup>
            {/if}
          </select>
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Menu Label / Link Title</label>
          <input
            type="text"
            bind:value={newLabel}
            required
            placeholder="e.g. Mechanical Hardware, Shipping Policy"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Target URL</label>
          <input
            type="text"
            bind:value={newUrl}
            required
            placeholder="e.g. /policies/shipment-policy or /?category=Hardware"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Menu Placement</label>
            <select
              bind:value={activeTab}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            >
              <option value="header">Header Menu</option>
              <option value="footer">Footer Menu</option>
            </select>
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Sort Order</label>
            <input
              type="number"
              bind:value={newSortOrder}
              min="1"
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div class="pt-2 flex justify-end gap-2 border-t border-slate-800">
          <button
            type="button"
            on:click={() => isModalOpen = false}
            class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-semibold"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSaving}
            class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold transition-all shadow-md shadow-orange-600/30 disabled:opacity-50"
          >
            {isSaving ? 'Adding...' : 'Add Menu Item'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Edit Navigation Item Modal -->
{#if isEditModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-3xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-slate-800">
        <h3 class="text-sm font-bold text-white flex items-center gap-2">
          <Edit2 size={16} class="text-orange-400" />
          <span>Edit Navigation Link</span>
        </h3>
        <button
          on:click={() => isEditModalOpen = false}
          class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <form on:submit|preventDefault={handleEditItem} class="space-y-4 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Menu Label / Link Title</label>
          <input
            type="text"
            bind:value={editLabel}
            required
            placeholder="e.g. Keyboards, Shipping Policy"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Target URL</label>
          <input
            type="text"
            bind:value={editUrl}
            required
            placeholder="e.g. /policies/shipment-policy or /?category=Hardware"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono placeholder-slate-500 focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Sort Order Position</label>
          <input
            type="number"
            bind:value={editSortOrder}
            min="1"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="pt-2 flex justify-end gap-2 border-t border-slate-800">
          <button
            type="button"
            on:click={() => isEditModalOpen = false}
            class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-semibold"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSaving}
            class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold transition-all shadow-md shadow-orange-600/30 disabled:opacity-50"
          >
            {isSaving ? 'Saving Changes...' : 'Save Changes'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

