<script>
  import { Folder, Plus, Edit2, Trash2, ChevronRight, ChevronDown } from 'lucide-svelte';

  export let node;
  export let allCategories = [];
  export let depth = 0;
  export let onAddChild = (parent) => {};
  export let onEdit = (cat) => {};
  export let onDelete = (cat) => {};

  let isExpanded = true;

  $: children = allCategories
    .filter((c) => c.parent_id === node.id)
    .sort((a, b) => a.display_order - b.display_order);
</script>

<div class="border border-slate-800/80 rounded-2xl bg-slate-950/40 overflow-hidden mb-2">
  <!-- Node Header Row -->
  <div class="p-3.5 bg-slate-900/60 flex items-center justify-between gap-3 hover:bg-slate-900/90 transition-colors">
    <div class="flex items-center gap-2.5 min-w-0">
      <!-- Expand/Collapse toggle if has children -->
      {#if children.length > 0}
        <button
          type="button"
          on:click={() => isExpanded = !isExpanded}
          class="p-1 rounded text-slate-400 hover:text-white transition-colors"
          aria-label="Toggle children"
        >
          {#if isExpanded}
            <ChevronDown size={14} />
          {:else}
            <ChevronRight size={14} />
          {/if}
        </button>
      {:else}
        <div class="w-5"></div>
      {/if}

      <!-- Icon & Depth indicator -->
      <div class="p-1.5 rounded-lg {depth === 0 ? 'bg-orange-500/10 text-orange-400 border border-orange-500/20' : depth === 1 ? 'bg-sky-500/10 text-sky-400 border border-sky-500/20' : 'bg-purple-500/10 text-purple-400 border border-purple-500/20'}">
        <Folder size={15} />
      </div>

      <div class="min-w-0">
        <div class="flex items-center gap-2 flex-wrap">
          <span class="text-xs sm:text-sm font-bold text-white truncate">{node.name}</span>
          <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-800 text-slate-300 border border-slate-700">
            /{node.slug}
          </span>
          <span class="text-[10px] font-mono text-slate-500">
            Order: {node.display_order}
          </span>
          {#if depth > 0}
            <span class="text-[9px] font-mono px-1 rounded bg-slate-900 text-slate-400">
              Level {depth + 1}
            </span>
          {/if}
        </div>
        {#if node.description}
          <p class="text-[11px] text-slate-400 truncate mt-0.5">{node.description}</p>
        {/if}
      </div>
    </div>

    <!-- Actions: Add Child, Edit, Delete -->
    <div class="flex items-center gap-1.5 flex-shrink-0">
      <button
        type="button"
        on:click={() => onAddChild(node)}
        class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] font-semibold flex items-center gap-1 transition-colors border border-slate-700"
        title="Add subcategory under {node.name}"
      >
        <Plus size={12} class="text-orange-400" />
        <span class="hidden sm:inline">Add Child</span>
      </button>

      <button
        type="button"
        on:click={() => onEdit(node)}
        class="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors"
        title="Edit"
      >
        <Edit2 size={13} />
      </button>

      <button
        type="button"
        on:click={() => onDelete(node)}
        class="p-1.5 rounded-lg bg-slate-800 hover:bg-rose-950/40 text-slate-400 hover:text-rose-400 transition-colors"
        title="Delete"
      >
        <Trash2 size={13} />
      </button>
    </div>
  </div>

  <!-- Recursive Child Branches -->
  {#if children.length > 0 && isExpanded}
    <div class="pl-4 sm:pl-8 pr-2 py-2 border-t border-slate-800/60 bg-slate-950/30">
      {#each children as child}
        <svelte:self
          node={child}
          allCategories={allCategories}
          depth={depth + 1}
          {onAddChild}
          {onEdit}
          {onDelete}
        />
      {/each}
    </div>
  {/if}
</div>
