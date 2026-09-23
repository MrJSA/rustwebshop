<script>
  import {
    Package,
    Plus,
    Trash2,
    Edit2,
    Edit3,
    Layers,
    Download,
    Check,
    X,
    Box,
    Upload,
    Image,
    Sparkles,
    CheckCircle2,
    AlertCircle,
    FolderTree,
    ChevronRight,
    Save
  } from 'lucide-svelte';

  export let data;
  let products = data.products || [];
  let categories = data.categories || [];

  // Modals state
  let isCreateModalOpen = false;
  let isEditModalOpen = false;
  let isSaving = false;
  let isUploading = false;
  let uploadSavingsText = '';

  // Form State (Product)
  let editingProductId = null;
  let title = '';
  let category = 'Hardware';
  let subcategory = 'Keyboards';
  let description = '';
  let productType = 'physical';
  let basePriceEuros = 49.99;
  let imageUrl = '';
  let digitalDownloadUrl = '';

  // New variant in create form
  let variantSku = '';
  let variantTitle = '';
  let variantPriceEuros = 49.99;
  let variantStock = 20;

  // Parts (Bill of Materials) state for editing product
  let productParts = [];
  let newPartName = '';
  let newPartSku = '';
  let newPartQuantity = 1;
  let newPartVariantId = '';
  let newPartNotes = '';

  // Edit Part Modal State
  let isEditPartOpen = false;
  let editingPartId = null;
  let editPartName = '';
  let editPartSku = '';
  let editPartQuantity = 1;
  let editPartVariantId = '';
  let editPartNotes = '';

  // Current editing product's variants list
  let currentProductVariants = [];

  // Edit Variant Modal State
  let isEditVariantOpen = false;
  let editingVariantId = null;
  let editVariantTitle = '';
  let editVariantSku = '';
  let editVariantPriceEuros = 49.99;
  let editVariantStock = 20;
  let editVariantImageUrl = '';
  let isUploadingVariantImg = false;
  let variantUploadSavingsText = '';

  // Category Tree Modal Picker State
  let isCatPickerOpen = false;
  let catPickerTarget = 'create'; // 'create' or 'edit'

  // Compression toggle
  let compressImage = true;

  // Build tree from flat categories list
  $: rootCategories = categories
    .filter((c) => !c.parent_id)
    .sort((a, b) => a.display_order - b.display_order);

  function getSubcategories(parentId) {
    return categories
      .filter((c) => c.parent_id === parentId)
      .sort((a, b) => a.display_order - b.display_order);
  }

  function selectCategoryFromTree(cat, sub = null) {
    if (sub) {
      category = cat.name;
      subcategory = sub.name;
    } else {
      category = cat.name;
      subcategory = cat.name;
    }
    isCatPickerOpen = false;
  }

  // --- Image Upload with Client Canvas WebP Compression ---
  async function compressFileWebP(file) {
    if (!compressImage || !file.type.startsWith('image/')) {
      return { file, savingsText: '' };
    }
    const originalSize = file.size;
    const bitmap = await createImageBitmap(file);
    const canvas = document.createElement('canvas');
    const maxDim = 1400;
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
      canvas.toBlob((b) => resolve(b), 'image/webp', 0.82);
    });

    if (blob) {
      const compressedFile = new File([blob], file.name.replace(/\.[^/.]+$/, '') + '.webp', {
        type: 'image/webp'
      });
      const savedPercent = Math.round((1 - blob.size / originalSize) * 100);
      return {
        file: compressedFile,
        savingsText: `Compressed: ${(originalSize / 1024).toFixed(0)}KB → ${(blob.size / 1024).toFixed(0)}KB (-${savedPercent}%)`
      };
    }
    return { file, savingsText: '' };
  }

  async function handleFileSelect(event) {
    const file = event.target.files && event.target.files[0];
    if (!file) return;

    isUploading = true;
    uploadSavingsText = '';

    try {
      const { file: fileToUpload, savingsText } = await compressFileWebP(file);
      uploadSavingsText = savingsText;

      const formData = new FormData();
      formData.append('file', fileToUpload);

      const res = await fetch('/api/v1/admin/media/upload', {
        method: 'POST',
        headers: { 'X-Dev-Mode': 'true' },
        body: formData
      });

      if (res.ok) {
        const data = await res.json();
        imageUrl = data.url;
      }
    } catch (e) {
      console.error('Upload failed:', e);
      alert('Failed to upload image.');
    } finally {
      isUploading = false;
    }
  }

  // Variant-specific image upload
  async function handleVariantFileSelect(event) {
    const file = event.target.files && event.target.files[0];
    if (!file) return;

    isUploadingVariantImg = true;
    variantUploadSavingsText = '';

    try {
      const { file: fileToUpload, savingsText } = await compressFileWebP(file);
      variantUploadSavingsText = savingsText;

      const formData = new FormData();
      formData.append('file', fileToUpload);

      const res = await fetch('/api/v1/admin/media/upload', {
        method: 'POST',
        headers: { 'X-Dev-Mode': 'true' },
        body: formData
      });

      if (res.ok) {
        const data = await res.json();
        editVariantImageUrl = data.url;
      }
    } catch (e) {
      console.error('Variant image upload failed:', e);
      alert('Failed to upload variant image.');
    } finally {
      isUploadingVariantImg = false;
    }
  }

  // --- Modal Openers ---
  function openCreateModal() {
    editingProductId = null;
    title = '';
    category = 'Hardware';
    subcategory = 'Keyboards';
    description = '';
    productType = 'physical';
    basePriceEuros = 49.99;
    imageUrl = 'https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=800&q=80';
    digitalDownloadUrl = '';
    variantSku = 'PRD-' + Math.floor(1000 + Math.random() * 9000);
    variantTitle = 'Standard Version';
    variantPriceEuros = 49.99;
    variantStock = 25;
    uploadSavingsText = '';
    isCreateModalOpen = true;
  }

  async function openEditModal(product) {
    editingProductId = product.id;
    title = product.title;
    category = product.category;
    subcategory = product.subcategory;
    description = product.description;
    productType = product.product_type;
    basePriceEuros = (product.base_price_cents / 100);
    imageUrl = product.image_url;
    digitalDownloadUrl = product.digital_download_url || '';
    currentProductVariants = product.variants || [];
    uploadSavingsText = '';

    // Load parts for this product
    try {
      const partsRes = await fetch(`/api/v1/admin/products/${product.id}/parts`, {
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (partsRes.ok) {
        productParts = await partsRes.json();
      }
    } catch (e) {
      productParts = [];
    }

    isEditModalOpen = true;
  }

  async function reloadProducts() {
    const refresh = await fetch('/api/v1/admin/products', {
      headers: { 'X-Dev-Mode': 'true' }
    });
    if (refresh.ok) {
      products = await refresh.json();
    }
  }

  // --- Product CRUD ---
  async function handleCreateProduct() {
    isSaving = true;
    const basePriceCents = Math.round(basePriceEuros * 100);

    const payload = {
      title,
      description,
      product_type: productType,
      category,
      subcategory,
      base_price_cents: basePriceCents,
      digital_download_url: productType === 'digital' ? digitalDownloadUrl : null,
      image_url: imageUrl,
      variants: [
        {
          sku: variantSku,
          title: variantTitle,
          price_override_cents: Math.round(variantPriceEuros * 100),
          attributes: { version: variantTitle },
          stock_quantity: productType === 'digital' ? 999999 : variantStock,
          low_stock_threshold: 5,
          image_url: imageUrl
        }
      ]
    };

    try {
      const res = await fetch('/api/v1/admin/products', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        await reloadProducts();
        isCreateModalOpen = false;
      }
    } catch (e) {
      console.error('Failed to create product:', e);
    } finally {
      isSaving = false;
    }
  }

  async function handleUpdateProduct() {
    isSaving = true;
    const basePriceCents = Math.round(basePriceEuros * 100);

    try {
      const res = await fetch(`/api/v1/admin/products/${editingProductId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify({
          title,
          description,
          category,
          subcategory,
          base_price_cents: basePriceCents,
          digital_download_url: productType === 'digital' ? digitalDownloadUrl : null,
          image_url: imageUrl,
          is_active: true
        })
      });

      if (res.ok) {
        await reloadProducts();
        isEditModalOpen = false;
      }
    } catch (e) {
      console.error('Failed to update product:', e);
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete(productId) {
    if (!confirm('Are you sure you want to delete this product and all associated SKU variants?')) return;
    try {
      const res = await fetch(`/api/v1/admin/products/${productId}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        products = products.filter((p) => p.id !== productId);
      }
    } catch (e) {
      console.error('Failed to delete product:', e);
    }
  }

  // --- Variant CRUD under editing product ---
  let isAddVariantOpen = false;
  let addVariantTitle = '';
  let addVariantSku = '';
  let addVariantPriceEuros = 49.99;
  let addVariantStock = 20;

  async function handleAddVariant() {
    try {
      const res = await fetch(`/api/v1/admin/products/${editingProductId}/variants`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify({
          sku: addVariantSku,
          title: addVariantTitle,
          price_override_cents: Math.round(addVariantPriceEuros * 100),
          attributes: { version: addVariantTitle },
          stock_quantity: addVariantStock,
          low_stock_threshold: 5,
          image_url: imageUrl
        })
      });
      if (res.ok) {
        await reloadProducts();
        const updated = products.find((p) => p.id === editingProductId);
        if (updated) currentProductVariants = updated.variants || [];
        isAddVariantOpen = false;
        addVariantTitle = '';
        addVariantSku = '';
      }
    } catch (e) {
      console.error('Failed to add variant:', e);
    }
  }

  function openEditVariant(v) {
    editingVariantId = v.id;
    editVariantTitle = v.title;
    editVariantSku = v.sku;
    editVariantPriceEuros = (v.price_override_cents ? v.price_override_cents / 100 : basePriceEuros);
    editVariantStock = v.stock_quantity;
    editVariantImageUrl = v.image_url || imageUrl || '';
    variantUploadSavingsText = '';
    isEditVariantOpen = true;
  }

  async function handleSaveVariant() {
    try {
      const res = await fetch(`/api/v1/admin/variants/${editingVariantId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify({
          sku: editVariantSku,
          title: editVariantTitle,
          price_override_cents: Math.round(editVariantPriceEuros * 100),
          attributes: { version: editVariantTitle },
          stock_quantity: parseInt(editVariantStock) || 0,
          low_stock_threshold: 5,
          image_url: editVariantImageUrl
        })
      });
      if (res.ok) {
        await reloadProducts();
        const updated = products.find((p) => p.id === editingProductId);
        if (updated) currentProductVariants = updated.variants || [];
        isEditVariantOpen = false;
      }
    } catch (e) {
      console.error('Failed to update variant:', e);
    }
  }

  async function handleDeleteVariant(variantId) {
    if (!confirm('Delete this variant?')) return;
    try {
      await fetch(`/api/v1/admin/variants/${variantId}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      currentProductVariants = currentProductVariants.filter((v) => v.id !== variantId);
      await reloadProducts();
    } catch (e) {
      console.error('Failed to delete variant:', e);
    }
  }

  // --- Parts / Bill of Materials (BOM) CRUD ---
  async function handleAddPart() {
    if (!newPartName) return;
    try {
      const res = await fetch(`/api/v1/admin/products/${editingProductId}/parts`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify({
          variant_id: newPartVariantId || null,
          part_name: newPartName,
          part_sku: newPartSku || null,
          quantity: parseInt(newPartQuantity) || 1,
          notes: newPartNotes || null
        })
      });
      if (res.ok) {
        const partsRes = await fetch(`/api/v1/admin/products/${editingProductId}/parts`, {
          headers: { 'X-Dev-Mode': 'true' }
        });
        if (partsRes.ok) productParts = await partsRes.json();
        newPartName = '';
        newPartSku = '';
        newPartNotes = '';
        newPartQuantity = 1;
      }
    } catch (e) {
      console.error('Failed to add part:', e);
    }
  }

  function openEditPart(part) {
    editingPartId = part.id;
    editPartName = part.part_name;
    editPartSku = part.part_sku || '';
    editPartQuantity = part.quantity;
    editPartVariantId = part.variant_id || '';
    editPartNotes = part.notes || '';
    isEditPartOpen = true;
  }

  async function handleSavePart() {
    try {
      const res = await fetch(`/api/v1/admin/parts/${editingPartId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json', 'X-Dev-Mode': 'true' },
        body: JSON.stringify({
          variant_id: editPartVariantId || null,
          part_name: editPartName,
          part_sku: editPartSku || null,
          quantity: parseInt(editPartQuantity) || 1,
          notes: editPartNotes || null
        })
      });
      if (res.ok) {
        const partsRes = await fetch(`/api/v1/admin/products/${editingProductId}/parts`, {
          headers: { 'X-Dev-Mode': 'true' }
        });
        if (partsRes.ok) productParts = await partsRes.json();
        isEditPartOpen = false;
      }
    } catch (e) {
      console.error('Failed to update part:', e);
    }
  }

  async function handleDeletePart(partId) {
    if (!confirm('Remove this part from the BOM bundle?')) return;
    try {
      const res = await fetch(`/api/v1/admin/products/${editingProductId}/parts/${partId}`, {
        method: 'DELETE',
        headers: { 'X-Dev-Mode': 'true' }
      });
      if (res.ok) {
        productParts = productParts.filter((p) => p.id !== partId);
      }
    } catch (e) {
      console.error('Failed to delete part:', e);
    }
  }
</script>

<svelte:head>
  <title>Products, Variants & Bill of Materials | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-7xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Package size={24} class="text-orange-500" />
        Product Catalog, Variants & BOM
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Manage hardware, digital software, composite product BOMs, and variant versioning with image uploads.
      </p>
    </div>

    <button
      on:click={openCreateModal}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>New Product</span>
    </button>
  </div>

  <!-- Products Table -->
  <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden">
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs">
        <thead class="bg-slate-950/60 border-b border-slate-800 text-slate-400 uppercase tracking-wider text-[11px]">
          <tr>
            <th class="py-3.5 px-4 font-bold">Product</th>
            <th class="py-3.5 px-4 font-bold">Category Hierarchy</th>
            <th class="py-3.5 px-4 font-bold">Base Price</th>
            <th class="py-3.5 px-4 font-bold">Versions / Variants</th>
            <th class="py-3.5 px-4 font-bold">Type</th>
            <th class="py-3.5 px-4 font-bold text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#if products.length === 0}
            <tr>
              <td colspan="6" class="text-center py-12 text-slate-400">
                No products found. Click "New Product" above to create one.
              </td>
            </tr>
          {:else}
            {#each products as item}
              <tr class="hover:bg-slate-800/30 transition-colors">
                <td class="py-4 px-4">
                  <div class="flex items-center gap-3">
                    {#if item.image_url}
                      <img src={item.image_url} alt={item.title} class="w-10 h-10 rounded-lg object-cover bg-slate-950 border border-slate-800 flex-shrink-0" />
                    {:else}
                      <div class="w-10 h-10 rounded-lg bg-slate-800 flex items-center justify-center text-lg flex-shrink-0">📦</div>
                    {/if}
                    <div>
                      <div class="font-bold text-white text-sm">{item.title}</div>
                      <div class="text-[11px] text-slate-500 font-mono">/{item.slug}</div>
                    </div>
                  </div>
                </td>
                <td class="py-4 px-4">
                  <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded text-[11px] font-semibold bg-slate-800 text-orange-400 border border-slate-700">
                    <FolderTree size={12} />
                    <span>{item.category} &rsaquo; {item.subcategory || 'General'}</span>
                  </span>
                </td>
                <td class="py-4 px-4 font-mono font-bold text-white">
                  {(item.base_price_cents / 100).toFixed(2)} €
                </td>
                <td class="py-4 px-4">
                  <div class="flex items-center gap-1.5">
                    <Layers size={13} class="text-slate-400" />
                    <span class="font-bold text-white">{item.variants ? item.variants.length : 0} Versions</span>
                  </div>
                </td>
                <td class="py-4 px-4">
                  <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold {item.product_type === 'digital' ? 'bg-sky-500/10 text-sky-400 border border-sky-500/20' : 'bg-slate-800 text-slate-300 border border-slate-700'}">
                    {item.product_type}
                  </span>
                </td>
                <td class="py-4 px-4 text-right space-x-2">
                  <button
                    on:click={() => openEditModal(item)}
                    class="px-2.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-bold transition-colors border border-slate-700 inline-flex items-center gap-1"
                    title="Edit Product, Variants & Parts"
                  >
                    <Edit3 size={13} class="text-orange-400" />
                    <span>Edit / BOM</span>
                  </button>
                  <button
                    on:click={() => handleDelete(item.id)}
                    class="p-1.5 rounded-lg bg-slate-800 hover:bg-rose-950/40 text-slate-400 hover:text-rose-400 transition-colors border border-slate-700 inline-flex items-center"
                    title="Delete Product"
                  >
                    <Trash2 size={13} />
                  </button>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>

  <!-- Create Product Modal -->
  {#if isCreateModalOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div class="w-full max-w-2xl bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 sm:p-8 space-y-6">
        <div class="flex items-center justify-between border-b border-slate-800 pb-4">
          <h2 class="text-lg font-bold text-white flex items-center gap-2">
            <Package size={20} class="text-orange-500" />
            <span>Create New Product</span>
          </h2>
          <button on:click={() => isCreateModalOpen = false} class="p-1 text-slate-400 hover:text-white">✕</button>
        </div>

        <form on:submit|preventDefault={handleCreateProduct} class="space-y-4 text-xs">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Product Title</label>
            <input type="text" bind:value={title} required placeholder="e.g. RustCraft Custom Keypad" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
          </div>

          <!-- Category Tree Picker Trigger -->
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Category & Subcategory Hierarchy</label>
            <div class="flex items-center gap-2">
              <div class="flex-1 px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white flex items-center justify-between">
                <span class="font-semibold text-orange-400">{category} &rsaquo; {subcategory}</span>
                <span class="text-[11px] text-slate-500">Selected</span>
              </div>
              <button
                type="button"
                on:click={() => { catPickerTarget = 'create'; isCatPickerOpen = true; }}
                class="px-3.5 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-bold flex items-center gap-1.5 transition-colors"
              >
                <FolderTree size={15} class="text-orange-400" />
                <span>Browse Tree</span>
              </button>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Product Type</label>
              <select bind:value={productType} class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500">
                <option value="physical">Physical Hardware</option>
                <option value="digital">Digital Download</option>
              </select>
            </div>
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Base Price (EUR)</label>
              <input type="number" step="0.01" bind:value={basePriceEuros} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
            </div>
          </div>

          <!-- Image Upload / Compression Box -->
          <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3">
            <div class="flex items-center justify-between">
              <label class="text-slate-300 font-semibold flex items-center gap-1.5">
                <Upload size={14} class="text-orange-400" />
                <span>Upload Product Image onto Server</span>
              </label>
              <label class="flex items-center gap-1.5 cursor-pointer text-[11px] text-slate-300">
                <input type="checkbox" bind:checked={compressImage} class="rounded accent-orange-500" />
                <span class="font-semibold text-orange-400">Compress Image (WebP)</span>
              </label>
            </div>

            <div class="flex items-center gap-3">
              <input type="file" accept="image/*" on:change={handleFileSelect} class="text-xs text-slate-400 file:mr-3 file:py-1.5 file:px-3 file:rounded-lg file:border-0 file:text-xs file:font-semibold file:bg-slate-800 file:text-white hover:file:bg-slate-700" />
              {#if isUploading}
                <span class="text-xs text-orange-400 animate-pulse font-mono">Uploading & Compressing...</span>
              {/if}
            </div>

            {#if uploadSavingsText}
              <div class="text-[11px] text-emerald-400 font-mono flex items-center gap-1">
                <CheckCircle2 size={13} />
                <span>{uploadSavingsText}</span>
              </div>
            {/if}

            <input type="text" bind:value={imageUrl} placeholder="Image URL (e.g. /uploads/... or https://...)" class="w-full px-3.5 py-2 rounded-lg bg-slate-900 border border-slate-800 text-slate-300 font-mono text-[11px] focus:outline-none focus:border-orange-500" />
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Description</label>
            <textarea bind:value={description} rows="3" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"></textarea>
          </div>

          <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3">
            <h4 class="font-bold text-white">Initial Variant Version</h4>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-slate-400 mb-1">Version Title</label>
                <input type="text" bind:value={variantTitle} required class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
              </div>
              <div>
                <label class="block text-slate-400 mb-1">SKU</label>
                <input type="text" bind:value={variantSku} required class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono" />
              </div>
            </div>
            {#if productType === 'physical'}
              <div>
                <label class="block text-slate-400 mb-1">Initial Stock</label>
                <input type="number" bind:value={variantStock} required class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono" />
              </div>
            {/if}
          </div>

          <div class="flex items-center justify-end gap-3 pt-3">
            <button type="button" on:click={() => isCreateModalOpen = false} class="px-4 py-2.5 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
            <button type="submit" disabled={isSaving} class="px-6 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold">{isSaving ? 'Creating...' : 'Create Product'}</button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Edit Product Modal (with Variants, Variant Images & Parts BOM Manager) -->
  {#if isEditModalOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div class="w-full max-w-4xl bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 sm:p-8 space-y-6 max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between border-b border-slate-800 pb-4">
          <div>
            <h2 class="text-lg font-bold text-white flex items-center gap-2">
              <Package size={20} class="text-orange-500" />
              <span>Edit Product & Bill of Materials (BOM)</span>
            </h2>
            <p class="text-xs text-slate-400">Configure versions, subcategories, custom images per variant, and bundled parts.</p>
          </div>
          <button on:click={() => isEditModalOpen = false} class="p-1 text-slate-400 hover:text-white">
            <X size={20} />
          </button>
        </div>

        <!-- Section 1: Main Product Fields -->
        <form on:submit|preventDefault={handleUpdateProduct} class="space-y-4 text-xs">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Product Title</label>
              <input type="text" bind:value={title} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
            </div>
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Base Price (EUR)</label>
              <input type="number" step="0.01" bind:value={basePriceEuros} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
            </div>
          </div>

          <!-- Category Tree Selector -->
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Category & Subcategory Hierarchy</label>
            <div class="flex items-center gap-2">
              <div class="flex-1 px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white flex items-center justify-between">
                <span class="font-semibold text-orange-400">{category} &rsaquo; {subcategory}</span>
                <span class="text-[11px] text-slate-500">Active</span>
              </div>
              <button
                type="button"
                on:click={() => { catPickerTarget = 'edit'; isCatPickerOpen = true; }}
                class="px-3.5 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-bold flex items-center gap-1.5 transition-colors"
              >
                <FolderTree size={15} class="text-orange-400" />
                <span>Change Category</span>
              </button>
            </div>
          </div>

          <!-- Image Upload / Compress for Edit -->
          <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3">
            <div class="flex items-center justify-between">
              <label class="text-slate-300 font-semibold flex items-center gap-1.5">
                <Upload size={14} class="text-orange-400" />
                <span>Upload Master Product Image</span>
              </label>
              <label class="flex items-center gap-1.5 cursor-pointer text-[11px] text-slate-300">
                <input type="checkbox" bind:checked={compressImage} class="rounded accent-orange-500" />
                <span class="font-semibold text-orange-400">Compress Image (WebP)</span>
              </label>
            </div>

            <div class="flex items-center gap-4">
              <input type="file" accept="image/*" on:change={handleFileSelect} class="text-xs text-slate-400 file:mr-3 file:py-1.5 file:px-3 file:rounded-lg file:border-0 file:text-xs file:font-semibold file:bg-slate-800 file:text-white" />
              {#if isUploading}
                <span class="text-xs text-orange-400 animate-pulse font-mono">Uploading & Compressing...</span>
              {/if}
              {#if imageUrl}
                <img src={imageUrl} alt="Thumbnail preview" class="w-12 h-12 rounded-lg object-cover bg-slate-900 border border-slate-700" />
              {/if}
            </div>

            {#if uploadSavingsText}
              <div class="text-[11px] text-emerald-400 font-mono">{uploadSavingsText}</div>
            {/if}

            <input type="text" bind:value={imageUrl} placeholder="Image URL" class="w-full px-3.5 py-2 rounded-lg bg-slate-900 border border-slate-800 text-slate-300 font-mono text-[11px] focus:outline-none focus:border-orange-500" />
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Description</label>
            <textarea bind:value={description} rows="3" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"></textarea>
          </div>

          <div class="flex justify-end pt-2">
            <button type="submit" disabled={isSaving} class="px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold flex items-center gap-1.5 shadow-md">
              <Save size={15} />
              <span>{isSaving ? 'Saving Changes...' : 'Save Master Product Details'}</span>
            </button>
          </div>
        </form>

        <!-- Section 2: Variants & Subcategory Versions (With Variant Image Upload) -->
        <div class="border-t border-slate-800 pt-6 space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <h3 class="text-sm font-bold text-white flex items-center gap-2">
                <Box size={16} class="text-orange-400" />
                <span>Variants & Subcategory Versions</span>
              </h3>
              <p class="text-xs text-slate-400">Configure different subcategory versions, SKU overrides, and distinct variant pictures.</p>
            </div>
            <button
              type="button"
              on:click={() => isAddVariantOpen = !isAddVariantOpen}
              class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold flex items-center gap-1"
            >
              <Plus size={14} />
              <span>Add Version / Variant</span>
            </button>
          </div>

          {#if isAddVariantOpen}
            <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3 text-xs">
              <h4 class="font-bold text-orange-400">Add New Variant Version</h4>
              <div class="grid grid-cols-1 sm:grid-cols-4 gap-3">
                <div class="sm:col-span-2">
                  <label class="block text-slate-400 mb-1">Version Title</label>
                  <input type="text" bind:value={addVariantTitle} placeholder="e.g. Ferris Orange / Clicky Blue" class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
                </div>
                <div>
                  <label class="block text-slate-400 mb-1">SKU</label>
                  <input type="text" bind:value={addVariantSku} placeholder="KB-RUST-ORG" class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono" />
                </div>
                <div>
                  <label class="block text-slate-400 mb-1">Warehouse Stock</label>
                  <input type="number" bind:value={addVariantStock} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono" />
                </div>
              </div>
              <div class="flex justify-end gap-2 pt-1">
                <button type="button" on:click={() => isAddVariantOpen = false} class="px-3 py-1.5 rounded-lg bg-slate-800 text-slate-400">Cancel</button>
                <button type="button" on:click={handleAddVariant} class="px-4 py-1.5 rounded-lg bg-orange-600 text-white font-bold">Add Variant</button>
              </div>
            </div>
          {/if}

          <div class="space-y-2 max-h-56 overflow-y-auto">
            {#each currentProductVariants as v}
              <div class="p-3 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-between text-xs">
                <div class="flex items-center gap-3">
                  {#if v.image_url}
                    <img src={v.image_url} alt="" class="w-9 h-9 rounded-lg object-cover bg-slate-900 border border-slate-800 flex-shrink-0" />
                  {:else}
                    <div class="w-9 h-9 rounded-lg bg-slate-900 border border-slate-800 flex items-center justify-center text-slate-600 flex-shrink-0">
                      <Image size={16} />
                    </div>
                  {/if}
                  <div>
                    <span class="font-bold text-white">{v.title}</span>
                    <span class="text-slate-400 font-mono text-[11px] ml-2">SKU: {v.sku}</span>
                    {#if v.price_override_cents}
                      <span class="text-orange-400 font-mono font-bold text-[11px] ml-2">{(v.price_override_cents / 100).toFixed(2)} €</span>
                    {/if}
                  </div>
                </div>
                <div class="flex items-center gap-3">
                  <span class="font-mono text-slate-300 font-semibold">{v.stock_quantity} in stock</span>
                  <button
                    type="button"
                    on:click={() => openEditVariant(v)}
                    class="px-2 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 text-[11px] font-semibold flex items-center gap-1"
                  >
                    <Edit2 size={12} class="text-orange-400" />
                    <span>Edit Picture & SKU</span>
                  </button>
                  <button type="button" on:click={() => handleDeleteVariant(v.id)} class="text-slate-500 hover:text-rose-400 p-1">
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Section 3: Bill of Materials (BOM) & Bundled Parts -->
        <div class="border-t border-slate-800 pt-6 space-y-4">
          <div>
            <h3 class="text-sm font-bold text-white flex items-center gap-2">
              <Layers size={16} class="text-orange-400" />
              <span>Bill of Materials (BOM) & Component Parts</span>
            </h3>
            <p class="text-xs text-slate-400">
              Hardware components bundled with this product. Parts can be universal or assigned to specific variants.
            </p>
          </div>

          <!-- Add Part Form -->
          <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3 text-xs">
            <h4 class="font-bold text-slate-300">Attach Part to Product Bundle</h4>
            <div class="grid grid-cols-1 sm:grid-cols-4 gap-3">
              <div class="sm:col-span-2">
                <label class="block text-slate-400 mb-1">Part Name & Specs</label>
                <input type="text" bind:value={newPartName} placeholder="e.g. CNC Aluminum Case 6063" class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
              </div>
              <div>
                <label class="block text-slate-400 mb-1">Assigned Version</label>
                <select bind:value={newPartVariantId} class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white">
                  <option value="">Universal (All Versions)</option>
                  {#each currentProductVariants as v}
                    <option value={v.id}>{v.title}</option>
                  {/each}
                </select>
              </div>
              <div>
                <label class="block text-slate-400 mb-1">Quantity</label>
                <input type="number" bind:value={newPartQuantity} min="1" class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono" />
              </div>
            </div>
            <div>
              <label class="block text-slate-400 mb-1">Part Notes / Details (Optional)</label>
              <input type="text" bind:value={newPartNotes} placeholder="e.g. Pre-installed in chassis" class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white" />
            </div>
            <div class="flex justify-end pt-1">
              <button type="button" on:click={handleAddPart} class="px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold flex items-center gap-1.5">
                <Plus size={14} />
                <span>Add Part to BOM</span>
              </button>
            </div>
          </div>

          <!-- Existing Parts List -->
          {#if productParts.length === 0}
            <div class="text-xs text-slate-500 py-3 text-center">No components added yet.</div>
          {:else}
            <div class="space-y-2">
              {#each productParts as part}
                <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80 flex items-center justify-between text-xs">
                  <div class="min-w-0 pr-3">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-white">{part.part_name}</span>
                      <span class="px-2 py-0.5 rounded text-[10px] font-bold {part.variant_id ? 'bg-orange-500/10 text-orange-400 border border-orange-500/20' : 'bg-slate-800 text-slate-400'}">
                        {part.variant_id ? 'Version Specific' : 'Universal'}
                      </span>
                    </div>
                    {#if part.notes}
                      <p class="text-[11px] text-slate-400 mt-0.5">{part.notes}</p>
                    {/if}
                  </div>
                  <div class="flex items-center gap-3 flex-shrink-0">
                    <span class="font-mono text-orange-400 font-bold px-2 py-0.5 rounded bg-slate-900">
                      {part.quantity}x
                    </span>
                    <button
                      type="button"
                      on:click={() => openEditPart(part)}
                      class="px-2 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white text-[11px] flex items-center gap-1"
                    >
                      <Edit2 size={12} class="text-orange-400" />
                      <span>Edit</span>
                    </button>
                    <button type="button" on:click={() => handleDeletePart(part.id)} class="text-slate-500 hover:text-rose-400 p-1">
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Category Tree Picker Modal -->
{#if isCatPickerOpen}
  <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="w-full max-w-lg bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-2xl space-y-4 max-h-[85vh] overflow-y-auto">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-base font-bold text-white flex items-center gap-2">
          <FolderTree size={18} class="text-orange-500" />
          <span>Select from Category Tree</span>
        </h3>
        <button on:click={() => isCatPickerOpen = false} class="text-xs text-slate-400 hover:text-white">✕</button>
      </div>

      <p class="text-xs text-slate-400">Click any category or subcategory to assign it to the product:</p>

      <div class="space-y-2">
        {#each rootCategories as root}
          {@const subs = getSubcategories(root.id)}
          <div class="border border-slate-800 rounded-xl bg-slate-950 overflow-hidden">
            <div class="p-3 bg-slate-900/60 flex items-center justify-between hover:bg-slate-800/50 transition-colors">
              <span class="font-bold text-white text-xs">{root.name}</span>
              <button
                type="button"
                on:click={() => selectCategoryFromTree(root)}
                class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-orange-600 text-white text-[11px] font-bold"
              >
                Select Root
              </button>
            </div>

            {#if subs.length > 0}
              <div class="p-2.5 pl-6 bg-slate-950/60 divide-y divide-slate-800/40">
                {#each subs as sub}
                  <div class="py-2 flex items-center justify-between">
                    <span class="text-xs text-slate-300 font-semibold">&bull; {sub.name}</span>
                    <button
                      type="button"
                      on:click={() => selectCategoryFromTree(root, sub)}
                      class="px-2.5 py-1 rounded-lg bg-orange-600/20 hover:bg-orange-600 text-orange-400 hover:text-white border border-orange-500/30 text-[11px] font-bold transition-colors"
                    >
                      Select {sub.name}
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<!-- Edit Variant & Variant Image Modal -->
{#if isEditVariantOpen}
  <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-base font-bold text-white flex items-center gap-2">
          <Box size={18} class="text-orange-500" />
          <span>Edit Variant & Picture</span>
        </h3>
        <button on:click={() => isEditVariantOpen = false} class="text-xs text-slate-400 hover:text-white">✕</button>
      </div>

      <form on:submit|preventDefault={handleSaveVariant} class="space-y-4 text-xs">
        <div>
          <label class="block font-semibold text-slate-300 mb-1">Version Title</label>
          <input type="text" bind:value={editVariantTitle} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block font-semibold text-slate-300 mb-1">SKU</label>
            <input type="text" bind:value={editVariantSku} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
          </div>
          <div>
            <label class="block font-semibold text-slate-300 mb-1">Price (EUR)</label>
            <input type="number" step="0.01" bind:value={editVariantPriceEuros} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
          </div>
        </div>

        <div>
          <label class="block font-semibold text-slate-300 mb-1">Stock Quantity</label>
          <input type="number" bind:value={editVariantStock} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
        </div>

        <!-- Variant Image Upload -->
        <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800 space-y-2.5">
          <div class="flex items-center justify-between">
            <span class="font-semibold text-slate-300">Variant-Specific Picture</span>
            <label class="flex items-center gap-1 text-[10px] text-orange-400 font-semibold cursor-pointer">
              <input type="checkbox" bind:checked={compressImage} class="rounded accent-orange-500" />
              <span>WebP</span>
            </label>
          </div>

          <div class="flex items-center gap-3">
            <input type="file" accept="image/*" on:change={handleVariantFileSelect} class="text-xs text-slate-400 file:mr-2 file:py-1 file:px-2.5 file:rounded-lg file:border-0 file:text-[11px] file:bg-slate-800 file:text-white" />
            {#if editVariantImageUrl}
              <img src={editVariantImageUrl} alt="" class="w-9 h-9 rounded-lg object-cover bg-slate-900 border border-slate-700 flex-shrink-0" />
            {/if}
          </div>

          {#if variantUploadSavingsText}
            <div class="text-[10px] text-emerald-400 font-mono">{variantUploadSavingsText}</div>
          {/if}

          <input type="text" bind:value={editVariantImageUrl} placeholder="Variant image URL" class="w-full px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-300 font-mono text-[10px]" />
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-slate-800">
          <button type="button" on:click={() => isEditVariantOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300">Cancel</button>
          <button type="submit" class="px-5 py-2 rounded-xl bg-orange-600 text-white font-bold">Save Variant</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Edit BOM Part Modal -->
{#if isEditPartOpen}
  <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-base font-bold text-white flex items-center gap-2">
          <Layers size={18} class="text-orange-500" />
          <span>Edit Component Part (BOM)</span>
        </h3>
        <button on:click={() => isEditPartOpen = false} class="text-xs text-slate-400 hover:text-white">✕</button>
      </div>

      <form on:submit|preventDefault={handleSavePart} class="space-y-4 text-xs">
        <div>
          <label class="block font-semibold text-slate-300 mb-1">Part Name & Specifications</label>
          <input type="text" bind:value={editPartName} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block font-semibold text-slate-300 mb-1">Part SKU / Number</label>
            <input type="text" bind:value={editPartSku} placeholder="Optional" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
          </div>
          <div>
            <label class="block font-semibold text-slate-300 mb-1">Quantity</label>
            <input type="number" bind:value={editPartQuantity} min="1" required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
          </div>
        </div>

        <div>
          <label class="block font-semibold text-slate-300 mb-1">Assigned Variant / Version</label>
          <select bind:value={editPartVariantId} class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500">
            <option value="">Universal (Included with all versions)</option>
            {#each currentProductVariants as v}
              <option value={v.id}>{v.title}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="block font-semibold text-slate-300 mb-1">Part Notes</label>
          <input type="text" bind:value={editPartNotes} placeholder="e.g. Factory tuned stabilizers" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-slate-800">
          <button type="button" on:click={() => isEditPartOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300">Cancel</button>
          <button type="submit" class="px-5 py-2 rounded-xl bg-orange-600 text-white font-bold">Save Part</button>
        </div>
      </form>
    </div>
  </div>
{/if}
