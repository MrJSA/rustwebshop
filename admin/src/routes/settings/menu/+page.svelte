<script>
  import { Menu, Plus, Trash2, Save, Check, ExternalLink, ArrowUp, ArrowDown } from 'lucide-svelte';

  export let data;
  let menuItems = data.menuItems || [];

  let isModalOpen = false;
  let newLabel = '';
  let newUrl = '/';
  let newSortOrder = menuItems.length + 1;
  let isSaving = false;

  async function handleCreateItem() {
    isSaving = true;
    try {
      const res = await fetch('/api/v1/admin/menu', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify({
          label: newLabel,
          url: newUrl,
          sort_order: parseInt(newSortOrder) || 1,
          is_active: true
        })
      });

      if (res.ok) {
        const refresh = await fetch('/api/v1/admin/menu', { headers: { 'X-Dev-Mode': 'true' } });
        if (refresh.ok) menuItems = await refresh.json();
        isModalOpen = false;
        newLabel = '';
        newUrl = '/';
      }
    } catch (e) {
      console.error('Failed to create menu item:', e);
    } finally {
      isSaving = false;
    }
  }

  async function handleDeleteItem(id) {
    if (!confirm('Are you sure you want to remove this navigation item from the storefront header?')) return;
    try {
      const res = await fetch(`/api/v1/admin/menu/${id}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        menuItems = menuItems.filter((i) => i.id !== id);
      }
    } catch (e) {
      console.error('Failed to delete item:', e);
    }
  }

  async function handleToggleActive(item) {
    try {
      await fetch(`/api/v1/admin/menu/${item.id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify({
          is_active: !item.is_active
        })
      });
      item.is_active = !item.is_active;
      menuItems = [...menuItems];
    } catch (e) {
      console.error('Failed to update item:', e);
    }
  }
</script>

<svelte:head>
  <title>Navigation Menu Customizer | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-5xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Menu size={24} class="text-orange-500" />
        Storefront Navigation Menu
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Customize the navigation menu links displayed in the customer storefront header.
      </p>
    </div>

    <button
      on:click={() => { isModalOpen = true; newSortOrder = menuItems.length + 1; }}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md shadow-orange-600/20 flex items-center gap-2 self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>Add Navigation Link</span>
    </button>
  </div>

  <!-- Items Table -->
  <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl">
    {#if menuItems.length === 0}
      <div class="py-12 text-center text-xs text-slate-400">No navigation items configured.</div>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead>
            <tr class="border-b border-slate-800 text-[11px] text-slate-400 uppercase tracking-wider">
              <th class="pb-3 font-semibold">Order</th>
              <th class="pb-3 font-semibold">Menu Label</th>
              <th class="pb-3 font-semibold">Target URL</th>
              <th class="pb-3 font-semibold text-center">Status</th>
              <th class="pb-3 font-semibold text-right">Actions</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-800/60">
            {#each menuItems as item}
              <tr class="text-slate-300">
                <td class="py-3.5 font-mono text-slate-400 font-bold">{item.sort_order}</td>
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
                  <button
                    type="button"
                    on:click={() => handleDeleteItem(item.id)}
                    class="p-2 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/20"
                    title="Delete Menu Link"
                  >
                    <Trash2 size={14} />
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>

  <!-- Create Modal -->
  {#if isModalOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 space-y-4">
        <h3 class="text-base font-bold text-white">Add Storefront Navigation Link</h3>

        <form on:submit|preventDefault={handleCreateItem} class="space-y-4 text-xs">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Display Label</label>
            <input
              type="text"
              bind:value={newLabel}
              required
              placeholder="e.g. Mechanical Keyboards"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Target URL</label>
            <input
              type="text"
              bind:value={newUrl}
              required
              placeholder="e.g. /?category=Hardware or /policies/shipment-policy"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Sort Order Position</label>
            <input
              type="number"
              bind:value={newSortOrder}
              required
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>

          <div class="flex items-center justify-end gap-3 pt-2">
            <button
              type="button"
              on:click={() => isModalOpen = false}
              class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSaving}
              class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold"
            >
              {isSaving ? 'Saving...' : 'Add Link'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>
