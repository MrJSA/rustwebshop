<script>
  import { FolderTree, Plus, Edit2, Trash2, ChevronRight, ChevronDown, Folder, FolderPlus, Save, CheckCircle2 } from 'lucide-svelte';

  export let data;
  let categories = data.categories || [];
  let isSaving = false;
  let successNotice = '';

  // Modal State
  let isModalOpen = false;
  let modalMode = 'create'; // 'create' or 'edit'
  let editId = null;
  let parentId = null;
  let name = '';
  let slug = '';
  let description = '';
  let displayOrder = 1;

  // Build tree from flat categories array
  $: rootCategories = categories
    .filter((c) => !c.parent_id)
    .sort((a, b) => a.display_order - b.display_order);

  function getChildren(pid) {
    return categories
      .filter((c) => c.parent_id === pid)
      .sort((a, b) => a.display_order - b.display_order);
  }

  // Open Create Root Category
  function openCreateRoot() {
    modalMode = 'create';
    editId = null;
    parentId = null;
    name = '';
    slug = '';
    description = '';
    displayOrder = (rootCategories.length + 1) * 10;
    isModalOpen = true;
  }

  // Open Create Subcategory
  function openCreateSub(parent) {
    modalMode = 'create';
    editId = null;
    parentId = parent.id;
    name = '';
    slug = '';
    description = '';
    displayOrder = (getChildren(parent.id).length + 1) * 10;
    isModalOpen = true;
  }

  // Open Edit Category
  function openEdit(category) {
    modalMode = 'edit';
    editId = category.id;
    parentId = category.parent_id;
    name = category.name;
    slug = category.slug;
    description = category.description || '';
    displayOrder = category.display_order;
    isModalOpen = true;
  }

  async function reloadCategories() {
    try {
      const res = await fetch('/api/v1/admin/categories', {
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        categories = await res.json();
      }
    } catch (e) {
      console.error('Failed to reload categories:', e);
    }
  }

  async function handleSave() {
    isSaving = true;
    const finalSlug = slug.trim() || name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');

    try {
      if (modalMode === 'create') {
        const res = await fetch('/api/v1/admin/categories', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
          body: JSON.stringify({
            parent_id: parentId,
            name,
            slug: finalSlug,
            description,
            display_order: parseInt(displayOrder) || 0
          })
        });
        if (res.ok) {
          successNotice = `Category "${name}" created successfully!`;
          isModalOpen = false;
          await reloadCategories();
          setTimeout(() => successNotice = '', 3500);
        }
      } else {
        const res = await fetch(`/api/v1/admin/categories/${editId}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
          body: JSON.stringify({
            parent_id: parentId,
            name,
            slug: finalSlug,
            description,
            display_order: parseInt(displayOrder) || 0
          })
        });
        if (res.ok) {
          successNotice = `Category "${name}" updated successfully!`;
          isModalOpen = false;
          await reloadCategories();
          setTimeout(() => successNotice = '', 3500);
        }
      }
    } catch (e) {
      console.error('Failed to save category:', e);
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete(category) {
    if (!confirm(`Are you sure you want to delete category "${category.name}" and any child categories?`)) return;
    try {
      const res = await fetch(`/api/v1/admin/categories/${category.id}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        await reloadCategories();
      }
    } catch (e) {
      console.error('Failed to delete category:', e);
    }
  }
</script>

<svelte:head>
  <title>Category Hierarchy Tree | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-5xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <FolderTree size={24} class="text-orange-500" />
        Categories & Hierarchy Tree
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Define nested categories for the storefront navigation, catalog filters, and product assignments.
      </p>
    </div>

    <button
      on:click={openCreateRoot}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all self-start sm:self-auto"
    >
      <FolderPlus size={16} />
      <span>Add Root Category</span>
    </button>
  </div>

  {#if successNotice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <CheckCircle2 size={16} />
      <span>{successNotice}</span>
    </div>
  {/if}

  <!-- Category Tree Card -->
  <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden p-6 space-y-4">
    {#if rootCategories.length === 0}
      <div class="text-center py-16 text-xs text-slate-400">
        No categories found. Click "Add Root Category" to begin building your tree.
      </div>
    {:else}
      <div class="space-y-3">
        {#each rootCategories as root}
          {@const children = getChildren(root.id)}
          <div class="border border-slate-800/80 rounded-2xl bg-slate-950/60 overflow-hidden">
            <!-- Root Category Header -->
            <div class="p-4 bg-slate-900/60 flex items-center justify-between gap-3">
              <div class="flex items-center gap-3">
                <div class="p-2 rounded-xl bg-orange-500/10 border border-orange-500/20 text-orange-400">
                  <Folder size={18} />
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-bold text-white">{root.name}</span>
                    <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-slate-800 text-slate-300 border border-slate-700">
                      /{root.slug}
                    </span>
                    <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-900 text-slate-500">
                      Order: {root.display_order}
                    </span>
                  </div>
                  {#if root.description}
                    <p class="text-xs text-slate-400 mt-0.5">{root.description}</p>
                  {/if}
                </div>
              </div>

              <!-- Actions -->
              <div class="flex items-center gap-2">
                <button
                  on:click={() => openCreateSub(root)}
                  class="px-2.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold flex items-center gap-1.5 transition-colors border border-slate-700"
                  title="Add child category under {root.name}"
                >
                  <Plus size={13} class="text-orange-400" />
                  <span>Subcategory</span>
                </button>
                <button
                  on:click={() => openEdit(root)}
                  class="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors"
                  title="Edit category"
                >
                  <Edit2 size={14} />
                </button>
                <button
                  on:click={() => handleDelete(root)}
                  class="p-1.5 rounded-lg bg-slate-800 hover:bg-rose-950/40 text-slate-400 hover:text-rose-400 transition-colors"
                  title="Delete category"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>

            <!-- Subcategories Child List -->
            {#if children.length > 0}
              <div class="p-3 pl-8 sm:pl-12 bg-slate-950/40 divide-y divide-slate-800/40 border-t border-slate-800/60">
                {#each children as sub}
                  <div class="py-2.5 flex items-center justify-between gap-3 group">
                    <div class="flex items-center gap-2.5">
                      <div class="w-1.5 h-1.5 rounded-full bg-orange-500"></div>
                      <span class="text-xs font-semibold text-slate-200 group-hover:text-white transition-colors">
                        {sub.name}
                      </span>
                      <span class="text-[10px] font-mono text-slate-500">/{sub.slug}</span>
                      <span class="text-[10px] font-mono text-slate-600">Order: {sub.display_order}</span>
                    </div>

                    <div class="flex items-center gap-1.5">
                      <button
                        on:click={() => openEdit(sub)}
                        class="p-1 rounded bg-slate-900 hover:bg-slate-800 text-slate-400 hover:text-white transition-colors"
                        title="Edit subcategory"
                      >
                        <Edit2 size={13} />
                      </button>
                      <button
                        on:click={() => handleDelete(sub)}
                        class="p-1 rounded bg-slate-900 hover:bg-rose-950/40 text-slate-500 hover:text-rose-400 transition-colors"
                        title="Delete subcategory"
                      >
                        <Trash2 size={13} />
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Category Modal -->
{#if isModalOpen}
  <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-base font-bold text-white flex items-center gap-2">
          <FolderTree size={18} class="text-orange-500" />
          <span>{modalMode === 'create' ? (parentId ? 'New Subcategory' : 'New Root Category') : 'Edit Category'}</span>
        </h3>
        <button on:click={() => isModalOpen = false} class="text-xs text-slate-400 hover:text-white">
          ✕
        </button>
      </div>

      <form on:submit|preventDefault={handleSave} class="space-y-4 text-xs">
        <div>
          <label class="block font-semibold text-slate-300 mb-1">Parent Category</label>
          <select bind:value={parentId} class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500">
            <option value={null}>None (Root Category)</option>
            {#each rootCategories as r}
              <option value={r.id}>{r.name}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="block font-semibold text-slate-300 mb-1">Category Name</label>
          <input
            type="text"
            bind:value={name}
            required
            placeholder="e.g. Mechanical Keyboards"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block font-semibold text-slate-300 mb-1">URL Slug (Optional)</label>
          <input
            type="text"
            bind:value={slug}
            placeholder="auto-generated from name if empty"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block font-semibold text-slate-300 mb-1">Description</label>
          <textarea
            bind:value={description}
            rows="2"
            placeholder="Optional summary for catalog metadata"
            class="w-full px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          ></textarea>
        </div>

        <div>
          <label class="block font-semibold text-slate-300 mb-1">Display Sort Order</label>
          <input
            type="number"
            bind:value={displayOrder}
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="flex items-center justify-end gap-3 pt-3 border-t border-slate-800">
          <button type="button" on:click={() => isModalOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">
            Cancel
          </button>
          <button type="submit" disabled={isSaving} class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold transition-all shadow-md">
            {isSaving ? 'Saving...' : 'Save Category'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
