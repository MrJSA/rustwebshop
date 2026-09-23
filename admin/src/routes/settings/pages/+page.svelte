<script>
  import { FileText, Save, Check, Eye, ExternalLink, Bold, Italic, Heading, List, Link, Code } from 'lucide-svelte';

  export let data;
  let pages = data.pages || [];

  let selectedSlug = pages[0] ? pages[0].slug : 'shipment-policy';
  let title = '';
  let contentMarkdown = '';
  let isSaving = false;
  let saveSuccess = false;

  $: activePage = pages.find((p) => p.slug === selectedSlug);

  $: if (activePage) {
    title = activePage.title;
    contentMarkdown = activePage.content_markdown;
  }

  function insertMarkdown(prefix, suffix = '') {
    contentMarkdown += `${prefix}${suffix}`;
  }

  function parseMarkdown(md) {
    if (!md) return '';
    let html = md
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/^### (.*$)/gim, '<h3 class="text-base font-bold text-white mt-4 mb-1">$1</h3>')
      .replace(/^## (.*$)/gim, '<h2 class="text-lg font-bold text-white mt-5 mb-2 border-b border-slate-800 pb-1">$1</h2>')
      .replace(/^# (.*$)/gim, '<h1 class="text-2xl font-black text-white mb-4">$1</h1>')
      .replace(/\*\*(.*?)\*\*/gim, '<strong class="font-bold text-white">$1</strong>')
      .replace(/\*(.*?)\*/gim, '<em class="italic text-slate-300">$1</em>')
      .replace(/`(.*?)`/gim, '<code class="px-1.5 py-0.5 rounded bg-slate-950 font-mono text-orange-400 text-xs">$1</code>')
      .replace(/\[(.*?)\]\((.*?)\)/gim, '<a href="$2" target="_blank" class="text-orange-400 underline">$1</a>')
      .replace(/^\s*-\s+(.*$)/gim, '<li class="ml-4 list-disc text-slate-300">$1</li>')
      .replace(/\n\n/gim, '</p><p class="my-3 text-slate-300 leading-relaxed text-xs">');

    return `<p class="my-3 text-slate-300 leading-relaxed text-xs">${html}</p>`;
  }

  async function handleSave() {
    isSaving = true;
    saveSuccess = false;
    try {
      const res = await fetch(`/api/v1/admin/pages/${selectedSlug}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify({
          title,
          content_markdown: contentMarkdown,
          is_published: true
        })
      });

      if (res.ok) {
        saveSuccess = true;
        pages = pages.map((p) => (p.slug === selectedSlug ? { ...p, title, content_markdown: contentMarkdown } : p));
        setTimeout(() => (saveSuccess = false), 3000);
      }
    } catch (e) {
      console.error('Failed to save page:', e);
    } finally {
      isSaving = false;
    }
  }
</script>

<svelte:head>
  <title>Policy Pages CMS (Markdown) | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-7xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <FileText size={24} class="text-orange-500" />
        Legal Notices & Store Policy CMS
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Direct Markdown content management for Shipment Policy, Terms, Privacy, Cookies, and Legal Notices.
      </p>
    </div>

    <div class="flex items-center gap-3">
      <a
        href="http://localhost:8080/policies/{selectedSlug}"
        target="_blank"
        class="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold flex items-center gap-1.5 transition-colors border border-slate-700"
      >
        <Eye size={14} />
        <span>View Live on Store</span>
        <ExternalLink size={12} class="text-orange-400" />
      </a>

      <button
        on:click={handleSave}
        disabled={isSaving}
        class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md shadow-orange-600/20 flex items-center gap-2 disabled:opacity-50"
      >
        {#if saveSuccess}
          <Check size={16} class="text-emerald-300" />
          <span>Saved Successfully!</span>
        {:else}
          <Save size={16} />
          <span>{isSaving ? 'Saving...' : 'Save Changes'}</span>
        {/if}
      </button>
    </div>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
    <!-- Page Selector Sidebar -->
    <div class="space-y-2">
      <h3 class="text-xs font-bold text-slate-400 uppercase tracking-wider px-2">Store Policies</h3>
      <div class="space-y-1">
        {#each pages as p}
          <button
            type="button"
            on:click={() => selectedSlug = p.slug}
            class="w-full text-left px-3.5 py-2.5 rounded-xl text-xs font-semibold transition-all flex items-center justify-between {selectedSlug === p.slug ? 'bg-orange-600 text-white shadow-md shadow-orange-600/30' : 'bg-slate-900/60 text-slate-300 hover:bg-slate-800 hover:text-white border border-slate-800'}"
          >
            <span>{p.title}</span>
            <span class="text-[10px] font-mono opacity-70">/{p.slug}</span>
          </button>
        {/each}
      </div>

      {#if selectedSlug === 'shipment-policy'}
        <div class="mt-4 p-4 rounded-2xl bg-orange-500/10 border border-orange-500/20 text-xs text-slate-300 space-y-2">
          <strong class="text-orange-400 block font-bold">Dynamic Shipping Rates Included:</strong>
          <p class="text-[11px] leading-relaxed text-slate-400">
            The storefront automatically appends your live carriers, zones, and rate table right below this markdown text.
          </p>
        </div>
      {/if}
    </div>

    <!-- Markdown Editor & Preview -->
    <div class="lg:col-span-3 space-y-4">
      <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-4">
        <!-- Title Input -->
        <div>
          <label class="block text-xs font-bold text-slate-400 uppercase tracking-wider mb-1">Page Title</label>
          <input
            type="text"
            bind:value={title}
            class="w-full px-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm font-bold focus:outline-none focus:border-orange-500"
          />
        </div>

        <!-- Markdown Toolbar -->
        <div class="flex items-center gap-1.5 p-1 rounded-xl bg-slate-950 border border-slate-800 flex-wrap">
          <button type="button" on:click={() => insertMarkdown('# ', 'Heading')} class="p-1.5 rounded hover:bg-slate-800 text-slate-400 hover:text-white text-xs font-bold px-2">H1</button>
          <button type="button" on:click={() => insertMarkdown('## ', 'Section')} class="p-1.5 rounded hover:bg-slate-800 text-slate-400 hover:text-white text-xs font-bold px-2">H2</button>
          <button type="button" on:click={() => insertMarkdown('### ', 'Subheader')} class="p-1.5 rounded hover:bg-slate-800 text-slate-400 hover:text-white text-xs font-bold px-2">H3</button>
          <button type="button" on:click={() => insertMarkdown('**', '**')} class="p-1.5 rounded hover:bg-slate-800 text-slate-400 hover:text-white" title="Bold"><Bold size={14} /></button>
          <button type="button" on:click={() => insertMarkdown('*', '*')} class="p-1.5 rounded hover:bg-slate-800 text-slate-400 hover:text-white" title="Italic"><Italic size={14} /></button>
          <button type="button" on:click={() => insertMarkdown('\n- ', 'List item')} class="p-1.5 rounded hover:bg-slate-800 text-slate-400 hover:text-white" title="Bullet List"><List size={14} /></button>
          <button type="button" on:click={() => insertMarkdown('`', '`')} class="p-1.5 rounded hover:bg-slate-800 text-slate-400 hover:text-white" title="Inline Code"><Code size={14} /></button>
          <button type="button" on:click={() => insertMarkdown('[Link text](', 'https://example.com)')} class="p-1.5 rounded hover:bg-slate-800 text-slate-400 hover:text-white" title="Link"><Link size={14} /></button>
        </div>

        <!-- Split Editor / Live Preview -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- Textarea -->
          <div>
            <div class="text-[11px] font-bold text-slate-400 uppercase tracking-wider mb-1.5">Markdown Source</div>
            <textarea
              bind:value={contentMarkdown}
              rows="20"
              class="w-full p-4 rounded-2xl bg-slate-950 border border-slate-800 text-white font-mono text-xs leading-relaxed focus:outline-none focus:border-orange-500 shadow-inner"
            ></textarea>
          </div>

          <!-- Live Preview -->
          <div>
            <div class="text-[11px] font-bold text-slate-400 uppercase tracking-wider mb-1.5">Live Formatted Preview</div>
            <div class="p-5 rounded-2xl bg-slate-950/70 border border-slate-800 min-h-[440px] max-h-[440px] overflow-y-auto">
              {@html parseMarkdown(contentMarkdown)}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
