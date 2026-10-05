<script>
  import MediaPickerModal from '$lib/components/MediaPickerModal.svelte';
  import CategoryTreeNode from '$lib/components/CategoryTreeNode.svelte';
  import CouponsManager from '$lib/components/CouponsManager.svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
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
    Save,
    Star,
    Warehouse,
    FolderPlus,
    Search,
    RefreshCw,
    AlertTriangle,
    Tag
  } from 'lucide-svelte';

  export let data;
  let products = data.products || [];
  let categories = data.categories || [];
  let featuredProductIds = data.featuredProductIds || [];
  let inventory = data.inventory || [];
  let storeSettings = data.storeSettings || {};
  let coupons = data.coupons || [];

  $: isKleingewerbe = (storeSettings?.tax_mode === 'kleingewerbe');
  $: if (isKleingewerbe) {
    createProductTaxRate = 0.0;
  }

  $: activeTab = $page.url.searchParams.get('tab') || 'catalog';
  function setTab(tab) {
    goto(`/products?tab=${tab}`, { keepFocus: true, noScroll: true, replaceState: true });
  }

  async function reloadInventory() {
    try {
      const res = await fetch('/api/v1/admin/logistics/inventory', {
        headers: {}
      });
      if (res.ok) {
        inventory = await res.json();
      }
    } catch (e) {
      console.error('Failed to reload inventory:', e);
    }
  }

  // Modals state
  let isCreateModalOpen = false;
  let isEditModalOpen = false;
  let isSaving = false;
  let isUploading = false;
  let uploadSavingsText = '';

  // Simple Create Product Form State
  let createProductName = '';
  let createProductMultiVariant = false;
  let createProductBasePrice = 49.99;
  let createProductTaxRate = 19.0;

  // Form State (Product)
  let editingProductId = null;
  let title = '';
  let subtitle = '';
  let variantSelectorLabel = 'Choose Variant / Model:';
  let shortDescription = '';
  let longDescription = '';
  let productImages = [];
  let isProductMediaPickerOpen = false;
  let isVariantMediaPickerOpen = false;
  let category = 'Hardware';
  let subcategory = 'Keyboards';
  let description = '';
  let productType = 'physical';
  let basePriceEuros = 49.99;
  let taxRatePercent = 19.0;
  let imageUrl = '';
  let digitalDownloadUrl = '';
  let digitalFiles = [];
  let isUploadingDigital = false;
  let hasMultipleVariants = false;
  let singleVariantStock = 20;
  let singleVariantSku = '';

  // New variant in create form
  let variantSku = '';
  let variantTitle = '';
  let variantPriceEuros = 49.99;
  let variantStock = 20;

  // Parts (Bill of Materials) state for editing product
  let productParts = [];
  let newPartType = 'physical'; // 'physical' or 'digital'
  let newPartName = '';
  let newPartSku = '';
  let newPartQuantity = 1;
  let newPartVariantId = '';
  let newPartNotes = '';
  let isUploadingBomFile = false;

  // Edit Part Modal State
  let isEditPartOpen = false;
  let editingPartId = null;
  let editPartType = 'physical';
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
  let editVariantImages = [];
  let isUploadingVariantImg = false;
  let variantUploadSavingsText = '';

  // Category Tree Modal Picker State
  let isCatPickerOpen = false;
  let catPickerTarget = 'create'; // 'create' or 'edit'

  // Compression toggle
  let compressImage = true;

  // Build recursive hierarchical category tree
  function buildCategoryHierarchy(pid = null, depth = 0) {
    let result = [];
    const directChildren = categories
      .filter(c => c.parent_id === pid)
      .sort((a, b) => a.display_order - b.display_order);

    for (const child of directChildren) {
      result.push({ ...child, depth });
      result = result.concat(buildCategoryHierarchy(child.id, depth + 1));
    }
    return result;
  }

  function getRootCategoryName(cat) {
    let curr = cat;
    while (curr && curr.parent_id) {
      const parent = categories.find(c => c.id === curr.parent_id);
      if (!parent) break;
      curr = parent;
    }
    return curr ? curr.name : cat.name;
  }

  function selectAnyCategory(cat) {
    if (!cat.parent_id) {
      category = cat.name;
      subcategory = cat.name;
    } else {
      category = getRootCategoryName(cat);
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
        headers: {},
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
        headers: {},
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

  // Digital product files management
  function addDigitalFile() {
    digitalFiles = [...digitalFiles, { name: '', url: '' }];
  }

  function removeDigitalFile(index) {
    digitalFiles = digitalFiles.filter((_, i) => i !== index);
  }

  async function handleDigitalFileUpload(event) {
    const file = event.target.files && event.target.files[0];
    if (!file) return;
    isUploadingDigital = true;
    try {
      const formData = new FormData();
      formData.append('file', file);
      const res = await fetch('/api/v1/admin/media/upload', {
        method: 'POST',
        headers: {},
        body: formData
      });
      if (res.ok) {
        const data = await res.json();
        digitalFiles = [...digitalFiles, { name: file.name, url: data.url }];
      } else {
        alert('File upload failed.');
      }
    } catch (e) {
      console.error('Digital file upload error:', e);
      alert('Upload failed: ' + e.message);
    } finally {
      isUploadingDigital = false;
      event.target.value = '';
    }
  }

  async function handleBomFileUpload(event) {
    const file = event.target.files && event.target.files[0];
    if (!file) return;
    isUploadingBomFile = true;
    try {
      const formData = new FormData();
      formData.append('file', file);
      const res = await fetch('/api/v1/admin/media/upload', {
        method: 'POST',
        headers: {},
        body: formData
      });
      if (res.ok) {
        const data = await res.json();
        newPartNotes = data.url;
        if (!newPartName) newPartName = file.name;
        newPartSku = 'DIGITAL_FILE';
      } else {
        alert('BOM file upload failed.');
      }
    } catch (e) {
      console.error('BOM file upload error:', e);
      alert('Upload failed: ' + e.message);
    } finally {
      isUploadingBomFile = false;
      event.target.value = '';
    }
  }

  // --- Modal Openers ---
  function openCreateModal() {
    createProductName = '';
    createProductMultiVariant = false;
    createProductBasePrice = 49.99;
    isCreateModalOpen = true;
  }

  async function openEditModal(product) {
    editingProductId = product.id;
    title = product.title;
    subtitle = product.subtitle || '';
    variantSelectorLabel = product.variant_selector_label || 'Choose Variant / Model:';
    shortDescription = product.short_description || product.description || '';
    longDescription = product.long_description || '';
    productImages = product.images && Array.isArray(product.images) && product.images.length > 0
      ? [...product.images]
      : (product.image_url ? [product.image_url] : []);
    category = product.category || 'Hardware';
    subcategory = product.subcategory || 'Keyboards';
    description = product.description || '';
    productType = product.product_type || (product.digital_download_url ? 'digital' : 'physical');
    basePriceEuros = (product.base_price_cents / 100);
    taxRatePercent = isKleingewerbe ? 0.0 : (product.tax_rate_percent !== undefined && product.tax_rate_percent !== null ? product.tax_rate_percent : 19.0);
    imageUrl = product.image_url || '';
    digitalDownloadUrl = product.digital_download_url || '';
    digitalFiles = [];
    if (digitalDownloadUrl) {
      try {
        const parsed = JSON.parse(digitalDownloadUrl);
        if (Array.isArray(parsed)) {
          digitalFiles = parsed.map(f => typeof f === 'string' ? { name: 'Download File', url: f } : { name: f.name || 'Download File', url: f.url || '' });
        } else if (typeof parsed === 'string') {
          digitalFiles = [{ name: 'Download File', url: parsed }];
        }
      } catch (e) {
        digitalFiles = [{ name: 'Download File', url: digitalDownloadUrl }];
      }
    }
    currentProductVariants = product.variants || [];
    hasMultipleVariants = !!product.has_multiple_variants;
    if (currentProductVariants.length > 0) {
      singleVariantStock = currentProductVariants[0].stock_quantity;
      singleVariantSku = currentProductVariants[0].sku;
    } else {
      singleVariantStock = productType === 'digital' ? 999999 : 20;
      singleVariantSku = 'PRD-' + (product.id ? product.id.substring(0, 4).toUpperCase() : '0000');
    }
    uploadSavingsText = '';

    // Load parts for this product
    try {
      const partsRes = await fetch(`/api/v1/admin/products/${product.id}/parts`, {
        headers: {}
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
      headers: {}
    });
    if (refresh.ok) {
      products = await refresh.json();
    }
  }

  // --- Product CRUD ---
  async function handleCreateProduct() {
    if (!createProductName.trim()) return;
    isSaving = true;
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch('/api/v1/admin/products', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          title: createProductName.trim(),
          has_multiple_variants: createProductMultiVariant,
          base_price_cents: Math.round(createProductBasePrice * 100),
          tax_rate_percent: isKleingewerbe ? 0.0 : (parseFloat(createProductTaxRate) || 0.0)
        })
      });

      if (res.ok) {
        const created = await res.json();
        await reloadProducts();
        isCreateModalOpen = false;
        const found = products.find(p => p.id === created.id || p.slug === created.slug);
        if (found) {
          await openEditModal(found);
        }
      } else {
        const err = await res.json().catch(() => ({}));
        alert('Failed to create product: ' + (err.error || res.statusText));
      }
    } catch (e) {
      console.error('Failed to create product:', e);
      alert('Error creating product: ' + e.message);
    } finally {
      isSaving = false;
    }
  }

  async function handleUpdateProduct() {
    isSaving = true;
    const basePriceCents = Math.round(basePriceEuros * 100);
    const primaryImg = productImages[0] || imageUrl || '';
    const token = localStorage.getItem('admin_token');

    let serializedDownloads = null;
    if (productType === 'digital') {
      const validFiles = digitalFiles.filter(f => f.url && f.url.trim().length > 0);
      serializedDownloads = validFiles.length > 0 ? JSON.stringify(validFiles) : null;
    }

    try {
      const res = await fetch(`/api/v1/admin/products/${editingProductId}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          title,
          subtitle: subtitle.trim() || null,
          variant_selector_label: variantSelectorLabel.trim() || 'Choose Variant / Model:',
          short_description: shortDescription.trim() || description.trim() || null,
          long_description: longDescription.trim() || null,
          description: shortDescription.trim() || description.trim(),
          category,
          subcategory,
          product_type: productType,
          base_price_cents: basePriceCents,
          tax_rate_percent: isKleingewerbe ? 0.0 : (parseFloat(taxRatePercent) || 0.0),
          digital_download_url: serializedDownloads,
          image_url: primaryImg,
          images: productImages.length > 0 ? productImages : [primaryImg],
          has_multiple_variants: hasMultipleVariants,
          is_active: true
        })
      });

      if (res.ok) {
        // If single variant product, synchronize the single variant SKU and stock
        if (!hasMultipleVariants && currentProductVariants.length > 0) {
          const v = currentProductVariants[0];
          await fetch(`/api/v1/admin/variants/${v.id}`, {
            method: 'PUT',
            headers: {
              'Content-Type': 'application/json',
              ...(token ? { Authorization: `Bearer ${token}` } : {})
            },
            body: JSON.stringify({
              sku: singleVariantSku || v.sku,
              title: 'Standard',
              price_override_cents: basePriceCents,
              stock_quantity: productType === 'digital' ? 999999 : (parseInt(singleVariantStock) || 0),
              low_stock_threshold: 5,
              image_url: primaryImg,
              images: productImages.length > 0 ? productImages : [primaryImg]
            })
          });
        }

        await reloadProducts();
        isEditModalOpen = false;
      } else {
        const err = await res.json().catch(() => ({}));
        alert('Failed to update product: ' + (err.error || res.statusText));
      }
    } catch (e) {
      console.error('Failed to update product:', e);
      alert('Error updating product: ' + e.message);
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete(productId) {
    if (!confirm('Are you sure you want to delete this product and all associated SKU variants?')) return;
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/products/${productId}`, {
        method: 'DELETE',
        headers: {
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (res.ok) {
        products = products.filter((p) => p.id !== productId);
        featuredProductIds = featuredProductIds.filter((id) => id !== productId);
      } else {
        const err = await res.json().catch(() => ({}));
        alert('Failed to delete product: ' + (err.error || res.statusText));
      }
    } catch (e) {
      console.error('Failed to delete product:', e);
      alert('Error deleting product: ' + e.message);
    }
  }

  async function toggleFeatured(productId) {
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/products/${productId}/toggle-featured`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (res.ok) {
        const data = await res.json();
        if (data.is_featured) {
          if (!featuredProductIds.includes(productId)) {
            featuredProductIds = [...featuredProductIds, productId];
          }
        } else {
          featuredProductIds = featuredProductIds.filter(id => id !== productId);
        }
      }
    } catch (e) {
      console.error('Failed to toggle featured:', e);
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
        headers: { 'Content-Type': 'application/json' },
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
    editVariantImages = v.images && Array.isArray(v.images) && v.images.length > 0
      ? [...v.images]
      : (v.image_url ? [v.image_url] : []);
    variantUploadSavingsText = '';
    isEditVariantOpen = true;
  }

  async function handleSaveVariant() {
    const primaryImg = editVariantImages[0] || editVariantImageUrl || '';
    try {
      const res = await fetch(`/api/v1/admin/variants/${editingVariantId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sku: editVariantSku,
          title: editVariantTitle,
          price_override_cents: Math.round(editVariantPriceEuros * 100),
          attributes: { version: editVariantTitle },
          stock_quantity: parseInt(editVariantStock) || 0,
          low_stock_threshold: 5,
          image_url: primaryImg,
          images: editVariantImages.length > 0 ? editVariantImages : [primaryImg]
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
        headers: {}
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
    const isDigital = newPartType === 'digital';
    try {
      const res = await fetch(`/api/v1/admin/products/${editingProductId}/parts`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          variant_id: newPartVariantId || null,
          part_name: newPartName,
          part_sku: isDigital ? 'DIGITAL_FILE' : (newPartSku || null),
          quantity: isDigital ? 1 : (parseInt(newPartQuantity) || 1),
          notes: newPartNotes || null
        })
      });
      if (res.ok) {
        const partsRes = await fetch(`/api/v1/admin/products/${editingProductId}/parts`, {
          headers: {}
        });
        if (partsRes.ok) productParts = await partsRes.json();
        newPartName = '';
        newPartSku = '';
        newPartNotes = '';
        newPartQuantity = 1;
        newPartType = 'physical';
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
    editPartType = (part.part_sku === 'DIGITAL_FILE') ? 'digital' : 'physical';
    isEditPartOpen = true;
  }

  async function handleSavePart() {
    const isDigital = editPartType === 'digital';
    try {
      const res = await fetch(`/api/v1/admin/parts/${editingPartId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          variant_id: editPartVariantId || null,
          part_name: editPartName,
          part_sku: isDigital ? 'DIGITAL_FILE' : (editPartSku || null),
          quantity: isDigital ? 1 : (parseInt(editPartQuantity) || 1),
          notes: editPartNotes || null
        })
      });
      if (res.ok) {
        const partsRes = await fetch(`/api/v1/admin/products/${editingProductId}/parts`, {
          headers: {}
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
        headers: {}
      });
      if (res.ok) {
        productParts = productParts.filter((p) => p.id !== partId);
      }
    } catch (e) {
      console.error('Failed to delete part:', e);
    }
  }

  // --- Category Tree Hierarchy State & Functions ---
  let isCatModalOpen = false;
  let catModalMode = 'create'; // 'create' or 'edit'
  let catEditId = null;
  let catParentId = null;
  let catName = '';
  let catSlug = '';
  let catDescription = '';
  let catImageUrl = '';
  let catDisplayOrder = 1;
  let catIsSaving = false;
  let catSuccessNotice = '';

  $: rootCategories = categories
    .filter((c) => !c.parent_id)
    .sort((a, b) => a.display_order - b.display_order);

  async function reloadCategories() {
    try {
      const res = await fetch('/api/v1/admin/categories', {
        headers: {}
      });
      if (res.ok) {
        categories = await res.json();
      }
    } catch (e) {
      console.error('Failed to reload categories:', e);
    }
  }

  function buildHierarchicalCategoryOptions(pid = null, prefix = '') {
    let result = [];
    const directChildren = categories
      .filter(c => c.parent_id === pid)
      .sort((a, b) => a.display_order - b.display_order);
    for (const child of directChildren) {
      if (catModalMode === 'edit' && child.id === catEditId) continue;
      result.push({ id: child.id, label: `${prefix}${prefix ? '↳ ' : ''}${child.name}` });
      result = result.concat(buildHierarchicalCategoryOptions(child.id, prefix + '  '));
    }
    return result;
  }

  function openCreateRootCat() {
    catModalMode = 'create';
    catEditId = null;
    catParentId = null;
    catName = '';
    catSlug = '';
    catDescription = '';
    catImageUrl = '';
    catDisplayOrder = (rootCategories.length + 1) * 10;
    isCatModalOpen = true;
  }

  function openCreateSubCat(parent) {
    catModalMode = 'create';
    catEditId = null;
    catParentId = parent.id;
    catName = '';
    catSlug = '';
    catDescription = '';
    catImageUrl = '';
    const siblings = categories.filter(c => c.parent_id === parent.id);
    catDisplayOrder = (siblings.length + 1) * 10;
    isCatModalOpen = true;
  }

  function openEditCat(cat) {
    catModalMode = 'edit';
    catEditId = cat.id;
    catParentId = cat.parent_id;
    catName = cat.name;
    catSlug = cat.slug;
    catDescription = cat.description || '';
    catImageUrl = cat.image_url || '';
    catDisplayOrder = cat.display_order;
    isCatModalOpen = true;
  }

  async function handleSaveCat() {
    catIsSaving = true;
    const finalSlug = catSlug.trim() || catName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
    const token = localStorage.getItem('admin_token');
    const headers = {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {})
    };
    try {
      if (catModalMode === 'create') {
        const res = await fetch('/api/v1/admin/categories', {
          method: 'POST',
          headers,
          body: JSON.stringify({
            parent_id: catParentId || null,
            name: catName,
            slug: finalSlug,
            description: catDescription,
            image_url: catImageUrl.trim() || null,
            display_order: parseInt(catDisplayOrder) || 0
          })
        });
        if (res.ok) {
          catSuccessNotice = `Category "${catName}" created successfully!`;
          isCatModalOpen = false;
          await reloadCategories();
          setTimeout(() => catSuccessNotice = '', 3500);
        }
      } else {
        const res = await fetch(`/api/v1/admin/categories/${catEditId}`, {
          method: 'PUT',
          headers,
          body: JSON.stringify({
            parent_id: catParentId || null,
            name: catName,
            slug: finalSlug,
            description: catDescription,
            image_url: catImageUrl.trim() || null,
            display_order: parseInt(catDisplayOrder) || 0
          })
        });
        if (res.ok) {
          catSuccessNotice = `Category "${catName}" updated successfully!`;
          isCatModalOpen = false;
          await reloadCategories();
          setTimeout(() => catSuccessNotice = '', 3500);
        }
      }
    } catch (e) {
      console.error('Failed to save category:', e);
    } finally {
      catIsSaving = false;
    }
  }

  async function handleDeleteCat(categoryToDelete) {
    if (!confirm(`Are you sure you want to delete category "${categoryToDelete.name}" and all its child categories?`)) return;
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/categories/${categoryToDelete.id}`, {
        method: 'DELETE',
        headers: {
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (res.ok) {
        await reloadCategories();
      }
    } catch (e) {
      console.error('Failed to delete category:', e);
    }
  }

  // --- Logistics & Warehouse Inventory State & Functions ---
  let filterLowStockOnly = false;
  let inventorySearchQuery = '';
  let updatingVariantId = null;

  $: filteredInventory = inventory.filter((item) => {
    if (filterLowStockOnly && !item.is_low_stock && !item.is_out_of_stock) {
      return false;
    }
    if (inventorySearchQuery.trim()) {
      const q = inventorySearchQuery.toLowerCase();
      return (
        item.sku.toLowerCase().includes(q) ||
        item.product_title.toLowerCase().includes(q) ||
        item.variant_title.toLowerCase().includes(q)
      );
    }
    return true;
  });

  async function adjustStock(variant_id, amount) {
    updatingVariantId = variant_id;
    try {
      const res = await fetch(`/api/v1/admin/logistics/inventory/${variant_id}/stock`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ adjustment: amount })
      });
      if (res.ok) {
        inventory = inventory.map((item) => {
          if (item.variant_id === variant_id) {
            const newQty = item.stock_quantity + amount;
            return {
              ...item,
              stock_quantity: newQty,
              is_low_stock: item.product_type === 'physical' && newQty <= item.low_stock_threshold,
              is_out_of_stock: item.product_type === 'physical' && newQty <= 0
            };
          }
          return item;
        });
      }
    } catch (e) {
      console.error('Failed to update stock:', e);
    } finally {
      updatingVariantId = null;
    }
  }
</script>

<svelte:head>
  <title>Products, Variants & Bill of Materials | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-7xl mx-auto">
  <!-- Section Tabs Navigation -->
  <div class="flex items-center gap-2 border-b border-slate-800 pb-3 overflow-x-auto">
    <button
      type="button"
      on:click={() => setTab('catalog')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'catalog' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Package size={15} />
      <span>Products & BOM</span>
      <span class="text-[10px] px-2 py-0.5 rounded-full {activeTab === 'catalog' ? 'bg-white/20 text-white' : 'bg-slate-800 text-slate-400'}">{products.length}</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('categories')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'categories' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <FolderTree size={15} />
      <span>Categories Tree</span>
      <span class="text-[10px] px-2 py-0.5 rounded-full {activeTab === 'categories' ? 'bg-white/20 text-white' : 'bg-slate-800 text-slate-400'}">{categories.length}</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('stock')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'stock' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Warehouse size={15} />
      <span>Logistics & Stock</span>
      <span class="text-[10px] px-2 py-0.5 rounded-full {activeTab === 'stock' ? 'bg-white/20 text-white' : 'bg-slate-800 text-slate-400'}">{inventory.length}</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('coupons')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'coupons' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Tag size={15} />
      <span>Promo & Discount Codes</span>
      <span class="text-[10px] px-2 py-0.5 rounded-full {activeTab === 'coupons' ? 'bg-white/20 text-white' : 'bg-slate-800 text-slate-400'}">{coupons.length}</span>
    </button>
  </div>

  {#if activeTab === 'catalog'}
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
            <th class="py-3.5 px-4 font-bold text-center w-16">Featured</th>
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
              <td colspan="7" class="text-center py-12 text-slate-400">
                No products found. Click "New Product" above to create one.
              </td>
            </tr>
          {:else}
            {#each products as item}
              <tr class="hover:bg-slate-800/30 transition-colors">
                <!-- Featured Star Column -->
                <td class="py-4 px-4 text-center">
                  <button
                    type="button"
                    on:click|stopPropagation={() => toggleFeatured(item.id)}
                    class="p-2 rounded-xl transition-all hover:scale-110 {featuredProductIds.includes(item.id) ? 'bg-amber-500/15 text-amber-400 border border-amber-500/30 shadow-md shadow-amber-500/10' : 'bg-slate-800/60 text-slate-600 hover:text-slate-400 border border-slate-800'}"
                    title={featuredProductIds.includes(item.id) ? "Featured on Storefront Carousel (Click to remove)" : "Not in Featured Carousel (Click to feature)"}
                  >
                    <Star size={15} fill={featuredProductIds.includes(item.id) ? "currentColor" : "none"} />
                  </button>
                </td>

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
                  {#if item.has_multiple_variants}
                    <div class="flex items-center gap-1.5">
                      <Layers size={13} class="text-indigo-400" />
                      <span class="font-bold text-white">{item.variants ? item.variants.length : 0} Versions</span>
                    </div>
                  {:else}
                    <div class="flex items-center gap-1.5">
                      <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-slate-800 text-slate-300 border border-slate-700">
                        Single Version
                      </span>
                      {#if item.variants && item.variants[0]}
                        <span class="text-[11px] font-mono text-emerald-400 font-semibold">({item.variants[0].stock_quantity} in stock)</span>
                      {/if}
                    </div>
                  {/if}
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
                    <span>Edit / Settings</span>
                  </button>
                  <button
                    on:click={() => handleDelete(item.id)}
                    class="p-1.5 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/20 inline-flex items-center"
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
  {:else if activeTab === 'categories'}
    <!-- Categories Tree View -->
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
            <FolderTree size={22} class="text-orange-500" />
            Category Hierarchy & Product Mapping
          </h2>
          <p class="text-xs text-slate-400 mt-1">
            Unlimited nesting. Products assigned to any 2nd, 3rd, or deeper subcategory are automatically included in every ancestor category above.
          </p>
        </div>

        <button
          type="button"
          on:click={openCreateRootCat}
          class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all self-start sm:self-auto"
        >
          <FolderPlus size={16} />
          <span>Add Root Category</span>
        </button>
      </div>

      {#if catSuccessNotice}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <CheckCircle2 size={16} />
          <span>{catSuccessNotice}</span>
        </div>
      {/if}

      <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden p-6 space-y-4">
        {#if rootCategories.length === 0}
          <div class="text-center py-16 text-xs text-slate-400">
            No categories found. Click "Add Root Category" to begin building your tree.
          </div>
        {:else}
          <div class="space-y-2">
            {#each rootCategories as root}
              <CategoryTreeNode
                node={root}
                allCategories={categories}
                products={products}
                depth={0}
                onAddChild={openCreateSubCat}
                onEdit={openEditCat}
                onDelete={handleDeleteCat}
              />
            {/each}
          </div>
        {/if}
      </div>
    </div>

  {:else if activeTab === 'stock'}
    <!-- Logistics & Stock View -->
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
            <Warehouse size={22} class="text-orange-500" />
            Logistics & Warehouse Stock Management
          </h2>
          <p class="text-xs text-slate-400 mt-1">Live tracking of physical inventory SKUs, stock thresholds, and one-click restocking adjustments.</p>
        </div>

        <!-- Filters & Search -->
        <div class="flex flex-wrap items-center gap-3">
          <div class="relative">
            <input
              type="text"
              bind:value={inventorySearchQuery}
              placeholder="Filter by SKU or title..."
              class="pl-9 pr-4 py-2 rounded-xl bg-slate-900 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500 w-52 sm:w-64"
            />
            <Search size={14} class="absolute left-3 top-3 text-slate-500 pointer-events-none" />
          </div>

          <button
            type="button"
            on:click={() => filterLowStockOnly = !filterLowStockOnly}
            class="px-3.5 py-2 rounded-xl text-xs font-bold border transition-colors flex items-center gap-1.5 {filterLowStockOnly ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' : 'bg-slate-900 text-slate-300 border-slate-800 hover:border-slate-700'}"
          >
            <AlertTriangle size={13} class={filterLowStockOnly ? 'text-amber-400' : 'text-slate-400'} />
            <span>Low Stock Only</span>
          </button>
        </div>
      </div>

      <!-- Warehouse Inventory Table -->
      <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden">
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead class="bg-slate-950/60 border-b border-slate-800 text-slate-400 uppercase tracking-wider text-[11px]">
              <tr>
                <th class="py-3.5 px-4 font-bold">SKU</th>
                <th class="py-3.5 px-4 font-bold">Product & Variant Title</th>
                <th class="py-3.5 px-4 font-bold">Type</th>
                <th class="py-3.5 px-4 font-bold text-center">Current Stock</th>
                <th class="py-3.5 px-4 font-bold text-center">Status</th>
                <th class="py-3.5 px-4 font-bold text-right">Quick Restock</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800/60">
              {#if filteredInventory.length === 0}
                <tr>
                  <td colspan="6" class="text-center py-12 text-slate-400">
                    No matching warehouse SKUs found.
                  </td>
                </tr>
              {:else}
                {#each filteredInventory as item}
                  <tr class="hover:bg-slate-800/30 transition-colors">
                    <td class="py-3 px-4 font-mono font-bold text-orange-400">
                      {item.sku}
                    </td>
                    <td class="py-3 px-4">
                      <div class="font-semibold text-white">{item.product_title}</div>
                      <div class="text-[11px] text-slate-400">Option: {item.variant_title}</div>
                    </td>
                    <td class="py-3 px-4">
                      <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider {item.product_type === 'digital' ? 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/20' : 'bg-slate-800 text-slate-300 border border-slate-700'}">
                        {item.product_type}
                      </span>
                    </td>
                    <td class="py-3 px-4 text-center">
                      {#if item.product_type === 'digital'}
                        <span class="text-slate-500 font-mono italic">&infin; (Unlimited)</span>
                      {:else}
                        <span class="text-sm font-bold font-mono {item.is_out_of_stock ? 'text-rose-400' : item.is_low_stock ? 'text-amber-400' : 'text-emerald-400'}">
                          {item.stock_quantity}
                        </span>
                      {/if}
                    </td>
                    <td class="py-3 px-4 text-center">
                      {#if item.product_type === 'digital'}
                        <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                          Digital Delivery
                        </span>
                      {:else if item.is_out_of_stock}
                        <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-rose-500/10 text-rose-400 border border-rose-500/20">
                          Out of Stock
                        </span>
                      {:else if item.is_low_stock}
                        <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20">
                          Low Stock (&le; {item.low_stock_threshold})
                        </span>
                      {:else}
                        <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                          Healthy Stock
                        </span>
                      {/if}
                    </td>
                    <td class="py-3 px-4 text-right">
                      {#if item.product_type === 'physical'}
                        <div class="flex items-center justify-end gap-1.5">
                          <button
                            type="button"
                            on:click={() => adjustStock(item.variant_id, -1)}
                            disabled={updatingVariantId === item.variant_id || item.stock_quantity <= 0}
                            class="px-2 py-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 text-white font-mono font-bold text-xs"
                            title="Subtract 1"
                          >
                            -1
                          </button>
                          <button
                            type="button"
                            on:click={() => adjustStock(item.variant_id, 1)}
                            disabled={updatingVariantId === item.variant_id}
                            class="px-2 py-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-30 text-white font-mono font-bold text-xs"
                            title="Add 1"
                          >
                            +1
                          </button>
                          <button
                            type="button"
                            on:click={() => adjustStock(item.variant_id, 10)}
                            disabled={updatingVariantId === item.variant_id}
                            class="px-2.5 py-1 rounded bg-orange-600/20 hover:bg-orange-600/30 border border-orange-500/30 disabled:opacity-30 text-orange-400 font-mono font-bold text-xs"
                            title="Restock +10"
                          >
                            +10
                          </button>
                        </div>
                      {:else}
                        <span class="text-slate-600 text-xs italic">N/A</span>
                      {/if}
                    </td>
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  {:else if activeTab === 'coupons'}
    <CouponsManager initialCoupons={coupons} />
  {/if}

  <!-- Simplified Create Product Modal -->
  {#if isCreateModalOpen}
    <div class="fixed inset-0 z-50 overflow-y-auto bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4 animate-in fade-in duration-150">
      <div class="w-full max-w-md bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl p-6 sm:p-7 space-y-5">
        <div class="flex items-center justify-between border-b border-slate-800 pb-3">
          <h2 class="text-base font-bold text-white flex items-center gap-2">
            <Package size={18} class="text-orange-500" />
            <span>Create New Product</span>
          </h2>
          <button on:click={() => isCreateModalOpen = false} class="p-1 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition-colors">
            <X size={16} />
          </button>
        </div>

        <form on:submit|preventDefault={handleCreateProduct} class="space-y-4 text-xs">
          <div>
            <label class="block text-slate-300 font-semibold mb-1.5">Product Name / Title</label>
            <input
              type="text"
              bind:value={createProductName}
              required
              autofocus
              placeholder="e.g. RustCraft Custom Keypad"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500 placeholder-slate-500"
            />
          </div>

          <div class="p-3.5 rounded-2xl bg-slate-950 border border-slate-800 flex items-center justify-between">
            <div>
              <span class="font-bold text-white text-xs">Multiple Versions / Variants?</span>
              <p class="text-[10px] text-slate-400 mt-0.5">
                Turn ON if this product has multiple colors or models. Leave OFF for a simple single product.
              </p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer ml-3 flex-shrink-0">
              <input type="checkbox" bind:checked={createProductMultiVariant} class="sr-only peer" />
              <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-600"></div>
            </label>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Base Price (EUR)</label>
              <input
                type="number"
                step="0.01"
                bind:value={createProductBasePrice}
                required
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
              />
            </div>
            <div>
              <div class="flex items-center justify-between mb-1">
                <label class="block text-slate-300 font-semibold">VAT Rate (%)</label>
                {#if isKleingewerbe}
                  <span class="text-[10px] font-bold text-amber-400 bg-amber-500/10 px-1.5 py-0.5 rounded border border-amber-500/20">
                    § 19 UStG (0% Exempt)
                  </span>
                {/if}
              </div>
              <div class="flex items-center gap-1.5">
                <input
                  type="number"
                  step="0.01"
                  min="0"
                  max="100"
                  bind:value={createProductTaxRate}
                  disabled={isKleingewerbe}
                  required
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500 disabled:opacity-50 disabled:cursor-not-allowed"
                />
                {#if !isKleingewerbe}
                  <button type="button" on:click={() => createProductTaxRate = 19.0} class="px-2 py-2.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-[10px] font-mono">19%</button>
                  <button type="button" on:click={() => createProductTaxRate = 7.0} class="px-2 py-2.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-[10px] font-mono">7%</button>
                  <button type="button" on:click={() => createProductTaxRate = 0.0} class="px-2 py-2.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-[10px] font-mono">0%</button>
                {/if}
              </div>
            </div>
          </div>

          <p class="text-[11px] text-slate-400 bg-slate-950/60 p-3 rounded-xl border border-slate-800/80">
            💡 After clicking Create, the full product editor will open automatically so you can configure pictures, descriptions, BOM parts, and inventory.
          </p>

          <div class="flex items-center justify-end gap-2 pt-2 border-t border-slate-800">
            <button type="button" on:click={() => isCreateModalOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white font-semibold">
              Cancel
            </button>
            <button type="submit" disabled={isSaving || !createProductName.trim()} class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold transition-all shadow-md shadow-orange-600/30 disabled:opacity-50">
              {isSaving ? 'Creating...' : 'Create & Open Settings'}
            </button>
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
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Product Title</label>
              <input type="text" bind:value={title} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
            </div>
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Base Price (EUR)</label>
              <input type="number" step="0.01" bind:value={basePriceEuros} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
            </div>
            <div>
              <div class="flex items-center justify-between mb-1">
                <label class="block text-slate-300 font-semibold">VAT Rate (%)</label>
                {#if isKleingewerbe}
                  <span class="text-[10px] font-bold text-amber-400 bg-amber-500/10 px-1.5 py-0.5 rounded border border-amber-500/20">
                    § 19 UStG (0% Exempt)
                  </span>
                {/if}
              </div>
              <div class="flex items-center gap-1.5">
                <input
                  type="number"
                  step="0.01"
                  min="0"
                  max="100"
                  bind:value={taxRatePercent}
                  disabled={isKleingewerbe}
                  required
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500 disabled:opacity-50 disabled:cursor-not-allowed"
                />
                {#if !isKleingewerbe}
                  <button type="button" on:click={() => taxRatePercent = 19.0} class="px-2 py-2.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-[10px] font-mono">19%</button>
                  <button type="button" on:click={() => taxRatePercent = 7.0} class="px-2 py-2.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-[10px] font-mono">7%</button>
                  <button type="button" on:click={() => taxRatePercent = 0.0} class="px-2 py-2.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-[10px] font-mono">0%</button>
                {/if}
              </div>
            </div>
          </div>

          <!-- Physical Product (OFF) vs Digital Product (ON) Toggle -->
          <div class="p-3.5 rounded-2xl {productType === 'digital' ? 'bg-cyan-950/20 border-cyan-800/40' : 'bg-slate-950 border-slate-800'} border flex items-center justify-between transition-colors">
            <div>
              <div class="font-bold text-white text-xs flex items-center gap-2">
                <span>Product Delivery Type:</span>
                <span class="px-2.5 py-0.5 rounded text-[10px] font-bold {productType === 'digital' ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/30' : 'bg-amber-500/10 text-amber-300 border border-amber-500/20'}">
                  {productType === 'digital' ? '⚡ Digital Product (Files / Download)' : '📦 Physical Product (Shipping & Warehouse Stock)'}
                </span>
              </div>
              <p class="text-[11px] text-slate-400 mt-0.5">
                {productType === 'digital'
                  ? 'Digital product: Stock tracking is disabled (unlimited downloads). Shipping cost is automatically waived (0.00 €) for digital-only orders.'
                  : 'Physical product: Warehouse stock tracking and standard physical shipping apply.'}
              </p>
            </div>
            <div class="flex items-center gap-3">
              <span class="text-xs font-semibold {productType === 'digital' ? 'text-cyan-400 font-bold' : 'text-slate-400'}">
                {productType === 'digital' ? 'Digital (ON)' : 'Physical (OFF)'}
              </span>
              <button
                type="button"
                on:click={() => productType = productType === 'digital' ? 'physical' : 'digital'}
                class="relative inline-flex h-6 w-12 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {productType === 'digital' ? 'bg-cyan-600' : 'bg-slate-800'}"
              >
                <span class="sr-only">Toggle Digital Product</span>
                <span
                  class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {productType === 'digital' ? 'translate-x-6' : 'translate-x-0'}"
                />
              </button>
            </div>
          </div>

          <!-- Multi-File Digital Product Manager (When Digital Product is ON) -->
          {#if productType === 'digital'}
            <div class="p-4 rounded-2xl bg-cyan-950/20 border border-cyan-800/40 space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <label class="text-cyan-300 font-bold flex items-center gap-1.5 text-xs">
                    <Download size={15} class="text-cyan-400" />
                    <span>Digital Download Files ({digitalFiles.length})</span>
                  </label>
                  <p class="text-[10px] text-cyan-400/70">
                    Upload multiple files (PDFs, ZIPs, firmware, 3D STLs) or add download URLs for customers to access after purchase.
                  </p>
                </div>
                <div class="flex items-center gap-2">
                  <label class="px-3 py-1.5 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-white font-bold text-xs flex items-center gap-1 cursor-pointer transition-colors shadow">
                    <Upload size={13} />
                    <span>Upload File</span>
                    <input type="file" on:change={handleDigitalFileUpload} class="hidden" />
                  </label>
                  <button
                    type="button"
                    on:click={addDigitalFile}
                    class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 font-bold text-xs flex items-center gap-1 transition-colors"
                  >
                    <Plus size={13} />
                    <span>Add Link</span>
                  </button>
                </div>
              </div>

              {#if isUploadingDigital}
                <div class="text-xs text-cyan-400 animate-pulse font-mono py-1">
                  Uploading digital asset file to server...
                </div>
              {/if}

              {#if digitalFiles.length === 0}
                <div class="p-4 rounded-xl border border-dashed border-cyan-800/50 text-center text-xs text-cyan-300/60">
                  No files added yet. Click <strong>Upload File</strong> or <strong>Add Link</strong> to attach downloadable files.
                </div>
              {:else}
                <div class="space-y-2">
                  {#each digitalFiles as file, idx}
                    <div class="p-2.5 rounded-xl bg-slate-950 border border-cyan-900/30 flex items-center gap-2 text-xs">
                      <span class="text-cyan-400 font-mono text-[10px] px-1.5 py-0.5 rounded bg-cyan-950/60 flex-shrink-0">#{idx + 1}</span>
                      <input
                        type="text"
                        bind:value={file.name}
                        placeholder="File Label (e.g. Firmware v2.1.hex or Assembly Manual.pdf)"
                        class="flex-1 px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white focus:outline-none focus:border-cyan-500 text-xs"
                      />
                      <input
                        type="text"
                        bind:value={file.url}
                        placeholder="URL or media path (/media/uploads/...)"
                        class="flex-1 px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono focus:outline-none focus:border-cyan-500 text-xs"
                      />
                      {#if file.url}
                        <a
                          href={file.url}
                          target="_blank"
                          rel="noreferrer"
                          class="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-cyan-400 hover:text-white"
                          title="Preview file"
                        >
                          <Download size={14} />
                        </a>
                      {/if}
                      <button
                        type="button"
                        on:click={() => removeDigitalFile(idx)}
                        class="p-1.5 rounded-lg bg-slate-800 hover:bg-rose-900/40 text-slate-400 hover:text-rose-400"
                        title="Remove file"
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}

          <!-- Product Structure Toggle -->
          <div class="p-3.5 rounded-2xl bg-slate-950 border border-slate-800 flex items-center justify-between">
            <div>
              <div class="font-bold text-white text-xs flex items-center gap-2">
                <span>Product Version Structure</span>
                <span class="px-2 py-0.5 rounded text-[10px] font-bold {hasMultipleVariants ? 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/20' : 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'}">
                  {hasMultipleVariants ? 'Multiple Versions / Variants' : 'Single Version Product'}
                </span>
              </div>
              <p class="text-[11px] text-slate-400 mt-0.5">
                {hasMultipleVariants ? 'Multiple models/colors with individual prices and variant matrix.' : 'Single standalone product with a direct price and inventory stock.'}
              </p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer ml-3 flex-shrink-0">
              <input type="checkbox" bind:checked={hasMultipleVariants} class="sr-only peer" />
              <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-600"></div>
            </label>
          </div>

          <!-- Single Version Direct Stock & SKU Inputs -->
          {#if !hasMultipleVariants}
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block text-slate-300 font-semibold mb-1">SKU Code (Inventory Identifier)</label>
                <input type="text" bind:value={singleVariantSku} placeholder="e.g. PRD-8910" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
              </div>
              <div>
                <div class="flex items-center justify-between mb-1">
                  <label class="block text-slate-300 font-semibold">Available Warehouse Stock</label>
                  {#if productType === 'digital'}
                    <span class="text-[10px] font-bold text-cyan-400 bg-cyan-500/10 px-1.5 py-0.5 rounded border border-cyan-500/20">
                      Stock Disabled (Virtual / Unlimited)
                    </span>
                  {/if}
                </div>
                {#if productType === 'digital'}
                  <div class="px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-cyan-400 font-mono text-xs flex items-center justify-between">
                    <span>Virtual Stock (Tracking disabled for digital)</span>
                    <span class="font-bold">∞</span>
                  </div>
                {:else}
                  <input type="number" bind:value={singleVariantStock} min="0" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
                {/if}
              </div>
            </div>
          {:else}
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Variant Selector Label</label>
              <input type="text" bind:value={variantSelectorLabel} placeholder="e.g. Choose Color or Choose Mainboard Type (Default: Choose Variant / Model:)" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
            </div>
          {/if}

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Product Subtitle (Under Title)</label>
            <input type="text" bind:value={subtitle} placeholder="e.g. Hot-swappable CNC anodized aluminum chassis with QMK/VIA firmware" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
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

          <!-- Multi-Image Picture Gallery Box -->
          <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3">
            <div class="flex items-center justify-between">
              <div>
                <label class="text-slate-300 font-semibold flex items-center gap-1.5 text-xs">
                  <Image size={14} class="text-orange-400" />
                  <span>Product Picture Gallery ({productImages.length})</span>
                </label>
                <p class="text-[10px] text-slate-400">The first picture serves as the primary storefront thumbnail.</p>
              </div>
              <button
                type="button"
                on:click={() => isProductMediaPickerOpen = true}
                class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-bold text-[11px] flex items-center gap-1"
              >
                <Upload size={12} />
                <span>Media Library</span>
              </button>
            </div>

            {#if productImages.length > 0}
              <div class="flex flex-wrap gap-2 pt-1">
                {#each productImages as img, idx}
                  <div class="relative group w-14 h-14 rounded-xl bg-slate-900 border border-slate-800 overflow-hidden">
                    <img src={img} alt={`Img ${idx + 1}`} class="w-full h-full object-cover" />
                    <button
                      type="button"
                      on:click={() => {
                        productImages = productImages.filter((_, i) => i !== idx);
                        imageUrl = productImages[0] || '';
                      }}
                      class="absolute top-0.5 right-0.5 p-0.5 rounded bg-rose-600/90 text-white opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                      <X size={10} />
                    </button>
                    {#if idx === 0}
                      <div class="absolute bottom-0 inset-x-0 bg-orange-600 text-white text-[7px] font-bold text-center">
                        PRIMARY
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}

            <div class="flex items-center gap-3 pt-1">
              <input type="file" accept="image/*" on:change={handleFileSelect} class="text-xs text-slate-400 file:mr-3 file:py-1 file:px-2.5 file:rounded-lg file:border-0 file:text-[11px] file:bg-slate-800 file:text-white hover:file:bg-slate-700" />
              {#if isUploading}
                <span class="text-xs text-orange-400 animate-pulse font-mono">Uploading & Compressing...</span>
              {/if}
            </div>
            {#if uploadSavingsText}
              <div class="text-[11px] text-emerald-400 font-mono">{uploadSavingsText}</div>
            {/if}
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Short Description (Next to Image)</label>
            <textarea bind:value={shortDescription} rows="2" placeholder="Brief summary displayed right beside the product image..." class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"></textarea>
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Long Description (Full Width Markdown)</label>
            <textarea bind:value={longDescription} rows="4" placeholder="Detailed product overview. Supports Markdown (# headers, **bold**, - lists, and blank lines for paragraphs)..." class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-xs focus:outline-none focus:border-orange-500"></textarea>
          </div>

          <div class="flex justify-end pt-2">
            <button type="submit" disabled={isSaving} class="px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold flex items-center gap-1.5 shadow-md">
              <Save size={15} />
              <span>{isSaving ? 'Saving Changes...' : 'Save Master Product Details'}</span>
            </button>
          </div>
        </form>

        <!-- Section 2: Variants & Subcategory Versions (With Variant Image Upload) -->
        {#if hasMultipleVariants}
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
        {/if}

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
            <div class="flex items-center justify-between pb-2 border-b border-slate-800/80">
              <h4 class="font-bold text-slate-300">Attach Part or Digital Asset to Product Bundle</h4>
              <div class="flex items-center gap-1.5 bg-slate-900 p-1 rounded-xl border border-slate-800">
                <button
                  type="button"
                  on:click={() => newPartType = 'physical'}
                  class="px-2.5 py-1 rounded-lg text-xs font-bold transition-colors {newPartType === 'physical' ? 'bg-orange-600 text-white' : 'text-slate-400 hover:text-white'}"
                >
                  📦 Physical Part
                </button>
                <button
                  type="button"
                  on:click={() => { newPartType = 'digital'; newPartSku = 'DIGITAL_FILE'; }}
                  class="px-2.5 py-1 rounded-lg text-xs font-bold transition-colors {newPartType === 'digital' ? 'bg-cyan-600 text-white' : 'text-slate-400 hover:text-white'}"
                >
                  ⚡ Digital File
                </button>
              </div>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-4 gap-3">
              <div class="sm:col-span-2">
                <label class="block text-slate-400 mb-1">{newPartType === 'digital' ? 'Digital Asset Name / Label' : 'Part Name & Specs'}</label>
                <input
                  type="text"
                  bind:value={newPartName}
                  placeholder={newPartType === 'digital' ? 'e.g. Firmware v2.0.hex or 3D Model STL' : 'e.g. CNC Aluminum Case 6063'}
                  class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white"
                />
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
                {#if newPartType === 'physical'}
                  <label class="block text-slate-400 mb-1">Quantity</label>
                  <input type="number" bind:value={newPartQuantity} min="1" class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono" />
                {:else}
                  <label class="block text-slate-400 mb-1">Upload Digital File</label>
                  <label class="w-full px-3 py-2 rounded-lg bg-cyan-950/60 hover:bg-cyan-900/60 border border-cyan-800/60 text-cyan-300 font-bold flex items-center justify-center gap-1.5 cursor-pointer transition-colors">
                    <Upload size={13} />
                    <span>{isUploadingBomFile ? 'Uploading...' : 'Choose File'}</span>
                    <input type="file" on:change={handleBomFileUpload} class="hidden" />
                  </label>
                {/if}
              </div>
            </div>

            <div>
              <label class="block text-slate-400 mb-1">{newPartType === 'digital' ? 'Download URL or File Path' : 'Part Notes / Details (Optional)'}</label>
              <input
                type="text"
                bind:value={newPartNotes}
                placeholder={newPartType === 'digital' ? 'https://... or /media/uploads/firmware.zip' : 'e.g. Pre-installed in chassis'}
                class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono text-xs"
              />
            </div>

            <div class="flex justify-end pt-1">
              <button
                type="button"
                on:click={handleAddPart}
                class="px-4 py-2 rounded-xl {newPartType === 'digital' ? 'bg-cyan-600 hover:bg-cyan-500' : 'bg-orange-600 hover:bg-orange-500'} text-white font-bold flex items-center gap-1.5 transition-colors"
              >
                <Plus size={14} />
                <span>{newPartType === 'digital' ? 'Add Digital Asset to BOM' : 'Add Part to BOM'}</span>
              </button>
            </div>
          </div>

          <!-- Existing Parts List -->
          {#if productParts.length === 0}
            <div class="text-xs text-slate-500 py-3 text-center">No components or digital assets added yet.</div>
          {:else}
            <div class="space-y-2">
              {#each productParts as part}
                <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80 flex items-center justify-between text-xs">
                  <div class="min-w-0 pr-3">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-white">{part.part_name}</span>
                      {#if part.part_sku === 'DIGITAL_FILE'}
                        <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-cyan-500/20 text-cyan-300 border border-cyan-500/30">
                          ⚡ DIGITAL FILE
                        </span>
                      {/if}
                      <span class="px-2 py-0.5 rounded text-[10px] font-bold {part.variant_id ? 'bg-orange-500/10 text-orange-400 border border-orange-500/20' : 'bg-slate-800 text-slate-400'}">
                        {part.variant_id ? 'Version Specific' : 'Universal'}
                      </span>
                    </div>
                    {#if part.notes}
                      <div class="text-[11px] text-slate-400 mt-0.5 flex items-center gap-1.5 truncate">
                        {#if part.part_sku === 'DIGITAL_FILE'}
                          <a href={part.notes} target="_blank" rel="noreferrer" class="text-cyan-400 hover:underline flex items-center gap-1 truncate">
                            <Download size={11} />
                            <span>{part.notes}</span>
                          </a>
                        {:else}
                          <span>{part.notes}</span>
                        {/if}
                      </div>
                    {/if}
                  </div>
                  <div class="flex items-center gap-3 flex-shrink-0">
                    {#if part.part_sku !== 'DIGITAL_FILE'}
                      <span class="font-mono text-orange-400 font-bold px-2 py-0.5 rounded bg-slate-900">
                        {part.quantity}x
                      </span>
                    {/if}
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

<!-- Category Tree Picker Modal (Arbitrary Hierarchy Depth) -->
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

      <p class="text-xs text-slate-400">Click any category, subcategory, or nested sub-sub-category to assign to this product:</p>

      <div class="space-y-1.5 max-h-96 overflow-y-auto pr-1">
        {#each buildCategoryHierarchy() as cat}
          <div
            class="p-2.5 rounded-xl border border-slate-800 bg-slate-950 flex items-center justify-between hover:bg-slate-900 transition-colors"
            style="margin-left: {cat.depth * 16}px;"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              {#if cat.image_url}
                <img src={cat.image_url} alt="" class="w-6 h-6 rounded object-cover flex-shrink-0" />
              {:else}
                <span class="text-xs">{cat.depth === 0 ? '📁' : '↳'}</span>
              {/if}
              <span class="text-xs font-semibold text-white truncate">{cat.name}</span>
              {#if cat.depth > 0}
                <span class="text-[9px] font-mono px-1 rounded bg-slate-800 text-slate-400">Level {cat.depth + 1}</span>
              {/if}
            </div>
            <button
              type="button"
              on:click={() => selectAnyCategory(cat)}
              class="px-2.5 py-1 rounded-lg bg-orange-600/20 hover:bg-orange-600 text-orange-400 hover:text-white border border-orange-500/30 text-[11px] font-bold transition-colors flex-shrink-0"
            >
              Select
            </button>
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

        <!-- Variant Pictures Multi-Gallery -->
        <div class="p-3.5 rounded-xl bg-slate-950 border border-slate-800 space-y-2.5">
          <div class="flex items-center justify-between">
            <span class="font-semibold text-slate-300">Variant-Specific Picture Gallery ({editVariantImages.length})</span>
            <button
              type="button"
              on:click={() => isVariantMediaPickerOpen = true}
              class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-white text-[10px] font-bold flex items-center gap-1"
            >
              <Upload size={10} />
              <span>Media</span>
            </button>
          </div>

          {#if editVariantImages.length > 0}
            <div class="flex flex-wrap gap-2">
              {#each editVariantImages as vImg, vIdx}
                <div class="relative group w-12 h-12 rounded-lg bg-slate-900 border border-slate-800 overflow-hidden">
                  <img src={vImg} alt="" class="w-full h-full object-cover" />
                  <button
                    type="button"
                    on:click={() => {
                      editVariantImages = editVariantImages.filter((_, i) => i !== vIdx);
                      editVariantImageUrl = editVariantImages[0] || '';
                    }}
                    class="absolute top-0.5 right-0.5 p-0.5 rounded bg-rose-600 text-white opacity-0 group-hover:opacity-100 transition-opacity"
                  >
                    <X size={9} />
                  </button>
                  {#if vIdx === 0}
                    <div class="absolute bottom-0 inset-x-0 bg-orange-600 text-white text-[6px] font-bold text-center">
                      PRIMARY
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}

          <div class="flex items-center gap-3">
            <input type="file" accept="image/*" on:change={handleVariantFileSelect} class="text-xs text-slate-400 file:mr-2 file:py-1 file:px-2.5 file:rounded-lg file:border-0 file:text-[11px] file:bg-slate-800 file:text-white" />
            {#if isUploadingVariantImg}
              <span class="text-[10px] text-orange-400 animate-pulse font-mono">Uploading...</span>
            {/if}
          </div>

          {#if variantUploadSavingsText}
            <div class="text-[10px] text-emerald-400 font-mono">{variantUploadSavingsText}</div>
          {/if}
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
        <div class="flex items-center justify-between pb-2 border-b border-slate-800">
          <span class="text-slate-400 font-semibold">Part Delivery Type:</span>
          <div class="flex items-center gap-1.5 bg-slate-950 p-1 rounded-xl border border-slate-800">
            <button
              type="button"
              on:click={() => editPartType = 'physical'}
              class="px-2.5 py-1 rounded-lg text-xs font-bold transition-colors {editPartType === 'physical' ? 'bg-orange-600 text-white' : 'text-slate-400 hover:text-white'}"
            >
              📦 Physical
            </button>
            <button
              type="button"
              on:click={() => { editPartType = 'digital'; editPartSku = 'DIGITAL_FILE'; }}
              class="px-2.5 py-1 rounded-lg text-xs font-bold transition-colors {editPartType === 'digital' ? 'bg-cyan-600 text-white' : 'text-slate-400 hover:text-white'}"
            >
              ⚡ Digital File
            </button>
          </div>
        </div>

        <div>
          <label class="block font-semibold text-slate-300 mb-1">{editPartType === 'digital' ? 'Digital Asset Name' : 'Part Name & Specifications'}</label>
          <input type="text" bind:value={editPartName} required class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
        </div>

        {#if editPartType === 'physical'}
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
        {/if}

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
          <label class="block font-semibold text-slate-300 mb-1">{editPartType === 'digital' ? 'Download URL or File Path' : 'Part Notes'}</label>
          <input type="text" bind:value={editPartNotes} placeholder={editPartType === 'digital' ? 'https://... or /media/uploads/file.zip' : 'e.g. Factory tuned stabilizers'} class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-slate-800">
          <button type="button" on:click={() => isEditPartOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300">Cancel</button>
          <button type="submit" class="px-5 py-2 rounded-xl bg-orange-600 text-white font-bold">Save Part</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Category Create / Edit Modal -->
{#if isCatModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-lg rounded-3xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-slate-800">
        <h3 class="text-sm font-bold text-white">
          {catModalMode === 'create' ? (catParentId ? 'Create Nested Subcategory' : 'Create Root Category') : 'Edit Category'}
        </h3>
        <button
          type="button"
          on:click={() => isCatModalOpen = false}
          class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <form on:submit|preventDefault={handleSaveCat} class="space-y-3.5 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Parent Category</label>
          <select
            bind:value={catParentId}
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 font-mono text-xs"
          >
            <option value={null}>None (Root Category)</option>
            {#each buildHierarchicalCategoryOptions() as opt}
              <option value={opt.id}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Category Name</label>
          <input
            type="text"
            bind:value={catName}
            required
            placeholder="e.g. Mechanical Keyboards"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">URL Slug</label>
          <input
            type="text"
            bind:value={catSlug}
            placeholder="e.g. mechanical-keyboards (Leave blank to autogenerate)"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Description (Optional)</label>
          <textarea
            bind:value={catDescription}
            rows="2"
            placeholder="Short overview of items found in this category..."
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          ></textarea>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Display Order</label>
            <input
              type="number"
              bind:value={catDisplayOrder}
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Image URL</label>
            <input
              type="text"
              bind:value={catImageUrl}
              placeholder="https://..."
              class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div class="flex items-center justify-end gap-2 pt-3 border-t border-slate-800">
          <button
            type="button"
            on:click={() => isCatModalOpen = false}
            class="px-4 py-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 transition-colors font-semibold"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={catIsSaving || !catName.trim()}
            class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white font-bold transition-all shadow-lg shadow-orange-600/30"
          >
            {catIsSaving ? 'Saving...' : 'Save Category'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Media Picker Modals for Product and Variant -->
<MediaPickerModal
  open={isProductMediaPickerOpen}
  onSelect={(url) => {
    productImages = [...productImages, url];
    if (!imageUrl) imageUrl = url;
  }}
  onClose={() => isProductMediaPickerOpen = false}
/>

<MediaPickerModal
  open={isVariantMediaPickerOpen}
  onSelect={(url) => {
    editVariantImages = [...editVariantImages, url];
    if (!editVariantImageUrl) editVariantImageUrl = url;
  }}
  onClose={() => isVariantMediaPickerOpen = false}
/>
