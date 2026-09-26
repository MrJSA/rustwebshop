<script>
  import MediaPickerModal from '$lib/components/MediaPickerModal.svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import {
    Building,
    CreditCard,
    Mail,
    Truck,
    Image,
    ShieldCheck,
    Save,
    Check,
    CheckCircle2,
    AlertCircle,
    Info,
    FolderOpen,
    Send,
    ExternalLink,
    Lock,
    Server,
    UserCheck,
    RefreshCw,
    Sparkles,
    Scale,
    Download,
    Upload,
    FileArchive,
    Layout,
    Share2,
    Globe,
    Plus,
    Trash2,
    Sliders
  } from 'lucide-svelte';
  import ShippingManager from '$lib/components/ShippingManager.svelte';
  import MediaManager from '$lib/components/MediaManager.svelte';

  export let data;
  let settings = data.settings || {};
  let paymentConfigs = data.paymentConfigs || [];
  let shippingProviders = data.shippingProviders || [];

  $: activeTab = $page.url.searchParams.get('tab') || 'identity';
  function setTab(tab) {
    goto(`/settings?tab=${tab}`, { keepFocus: true, noScroll: true, replaceState: true });
  }

  // --- Store Identity & Legal Form State ---
  let storeName = settings.store_name || 'RustCraft Store';
  let legalName = settings.legal_name || 'Max Mustermann E-Commerce';
  let storeOwner = settings.store_owner || 'Max Mustermann';
  let storeSubtitle = settings.store_subtitle || 'Rust Powered • ACID Fast';
  let companyAddress = settings.company_address || 'Musterstraße 1, 12345 Musterstadt, Germany';
  let supportEmail = settings.support_email || 'shop@example.com';
  let phone = settings.phone || '+49 123 4567890';
  let vatId = settings.vat_id || 'DE123456789';
  let taxNotice = settings.tax_notice || 'Value added tax is not collected, as small businesses according to §19 (1) UStG.';
  let commercialRegister = settings.commercial_register || '';
  let odrUrl = settings.odr_url || 'https://ec.europa.eu/odr';
  let disputeResolutionNotice = settings.dispute_resolution_notice || 'The European Commission provides a platform for the out-of-court resolution of disputes (ODR platform), which can be viewed under https://ec.europa.eu/odr. We are not willing and not obligated to enter into dispute resolution proceedings before the consumer arbitration board.';
  let currency = settings.currency || 'EUR';
  let currencySymbol = settings.currency_symbol || '€';
  let taxRatePercent = settings.tax_rate_percent !== undefined ? settings.tax_rate_percent : 19.0;
  let taxMode = settings.tax_mode || 'kleingewerbe';
  let logoUrl = settings.logo_url || '';

  // Order Number Layout & GoBD Sequence Customization
  let orderPrefixEnabled = settings.order_prefix_enabled !== undefined ? settings.order_prefix_enabled : true;
  let orderPrefix = settings.order_prefix || 'ORD';
  let orderDateEnabled = settings.order_date_enabled !== undefined ? settings.order_date_enabled : true;

  $: todayDateStr = new Date().toISOString().slice(0, 10).replace(/-/g, '');
  $: sampleOrderNumber = (() => {
    const parts = [];
    const cleanP = (orderPrefix || '').trim().toUpperCase().slice(0, 7);
    if (orderPrefixEnabled && cleanP) parts.push(cleanP);
    if (orderDateEnabled) parts.push(todayDateStr);
    parts.push('10000');
    return parts.join('-');
  })();

  // System Environment & Debugging Mode
  let deploymentMode = settings.deployment_mode || 'development';
  let debugMode = settings.debug_mode !== undefined ? settings.debug_mode : false;

  let isSavingIdentity = false;
  let identitySuccessNotice = '';
  let identityErrorNotice = '';
  let showLogoPicker = false;

  function handleTaxModeChange(mode) {
    taxMode = mode;
    if (mode === 'kleingewerbe') {
      taxNotice = 'Gemäß § 19 UStG wird keine Umsatzsteuer berechnet.';
    } else if (mode === 'included') {
      taxNotice = 'Preise verstehen sich inklusive der gesetzlichen Mehrwertsteuer.';
    } else if (mode === 'excluded') {
      taxNotice = 'Preise verstehen sich zuzüglich der gesetzlichen Mehrwertsteuer.';
    }
  }

  async function handleSaveIdentity() {
    isSavingIdentity = true;
    identitySuccessNotice = '';
    identityErrorNotice = '';

    const token = localStorage.getItem('admin_token');
    const headers = {
      'Content-Type': 'application/json',
      'X-Dev-Mode': 'true',
      ...(token ? { Authorization: `Bearer ${token}` } : {})
    };

    try {
      const payload = {
        ...settings,
        store_name: storeName,
        legal_name: legalName,
        store_owner: storeOwner,
        store_subtitle: storeSubtitle,
        company_address: companyAddress,
        support_email: supportEmail,
        phone,
        vat_id: vatId,
        tax_notice: taxNotice,
        tax_mode: taxMode,
        commercial_register: commercialRegister,
        odr_url: odrUrl,
        dispute_resolution_notice: disputeResolutionNotice,
        currency,
        currency_symbol: currencySymbol,
        tax_rate_percent: parseFloat(taxRatePercent) || 0,
        logo_url: logoUrl,
        order_prefix_enabled: orderPrefixEnabled,
        order_prefix: (orderPrefix || '').trim().toUpperCase().slice(0, 7),
        order_date_enabled: orderDateEnabled,
        deployment_mode: deploymentMode,
        debug_mode: debugMode
      };

      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers,
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        settings = payload;
        identitySuccessNotice = 'Store Identity & Taxation settings saved!';
        setTimeout(() => identitySuccessNotice = '', 4500);
      } else {
        const err = await res.json();
        identityErrorNotice = err.error || 'Failed to save store identity settings.';
      }
    } catch (e) {
      identityErrorNotice = 'Failed to connect to backend server.';
    } finally {
      isSavingIdentity = false;
    }
  }

  // --- Export & Backups Handlers ---
  let isExportingData = false;
  let isExportingMedia = false;
  let isImportingData = false;
  let importFileInput;
  let exportSuccessNotice = '';
  let exportErrorNotice = '';

  async function handleFileSelectForImport(e) {
    const file = e.target.files && e.target.files[0];
    if (!file) return;

    exportErrorNotice = '';
    exportSuccessNotice = '';

    try {
      const text = await file.text();
      const parsed = JSON.parse(text);

      // Strict schema version check to prevent data corruption
      const version = parsed.version || '';
      if (!version.startsWith('1.')) {
        exportErrorNotice = `Incompatible backup schema: version '${version || 'unknown'}'. This store requires schema version 1.x. Import was safely aborted.`;
        if (importFileInput) importFileInput.value = '';
        return;
      }

      if (!confirm(`Are you sure you want to restore store data from '${file.name}' (Schema Version: ${version})?\n\nThis will update your store settings, products, categories, CMS policy pages, navigation menus, and shipping configuration.`)) {
        if (importFileInput) importFileInput.value = '';
        return;
      }

      isImportingData = true;
      const token = localStorage.getItem('admin_token');
      const res = await fetch('/api/v1/admin/import/store-data', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: text
      });

      const resData = await res.json().catch(() => ({}));
      if (res.ok) {
        exportSuccessNotice = `Store restored successfully! Restored: ${resData.restored?.products || 0} products, ${resData.restored?.categories || 0} categories, ${resData.restored?.pages || 0} policy pages, ${resData.restored?.menu_items || 0} menu items. Reloading...`;
        setTimeout(() => window.location.reload(), 2500);
      } else {
        exportErrorNotice = resData.error || 'Failed to restore store data.';
      }
    } catch (err) {
      exportErrorNotice = 'Invalid JSON backup file or parse failure: ' + err.message;
    } finally {
      isImportingData = false;
      if (importFileInput) importFileInput.value = '';
    }
  }

  async function handleExportStoreData() {
    isExportingData = true;
    exportSuccessNotice = '';
    exportErrorNotice = '';
    try {
      const token = localStorage.getItem('admin_token');
      const res = await fetch('/api/v1/admin/export/store-data', {
        headers: {
          'X-Dev-Mode': 'true',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (!res.ok) throw new Error('Export request failed');
      const blob = await res.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `rustcraft_store_data_${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      window.URL.revokeObjectURL(url);
      exportSuccessNotice = 'Store settings, products, categories & CMS markdown pages exported to JSON!';
      setTimeout(() => exportSuccessNotice = '', 5000);
    } catch (err) {
      exportErrorNotice = 'Failed to export store data: ' + err.message;
    } finally {
      isExportingData = false;
    }
  }

  async function handleExportMedia() {
    isExportingMedia = true;
    exportSuccessNotice = '';
    exportErrorNotice = '';
    try {
      const token = localStorage.getItem('admin_token');
      const res = await fetch('/api/v1/admin/export/media', {
        headers: {
          'X-Dev-Mode': 'true',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (!res.ok) throw new Error('Media archive request failed');
      const blob = await res.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `rustcraft_media_library_${new Date().toISOString().split('T')[0]}.zip`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      window.URL.revokeObjectURL(url);
      exportSuccessNotice = 'Media library files compressed and downloaded as ZIP!';
      setTimeout(() => exportSuccessNotice = '', 5000);
    } catch (err) {
      exportErrorNotice = 'Failed to export media library: ' + err.message;
    } finally {
      isExportingMedia = false;
    }
  }

  // --- Payment Providers State ---
  let savingProvider = null;
  let paymentNotice = '';

  async function savePaymentConfig(provider) {
    savingProvider = provider.provider;
    paymentNotice = '';

    try {
      const res = await fetch(`/api/v1/admin/settings/payments/${provider.provider}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true'
        },
        body: JSON.stringify({
          display_name: provider.display_name,
          is_enabled: provider.is_enabled,
          is_sandbox: provider.is_sandbox,
          public_client_id: provider.public_client_id
        })
      });

      if (res.ok) {
        paymentNotice = `Saved settings for ${provider.display_name} successfully!`;
        setTimeout(() => paymentNotice = '', 3500);
      }
    } catch (e) {
      console.error('Failed to save payment config:', e);
    } finally {
      savingProvider = null;
    }
  }

  // --- Email & Auth State ---
  let smtpHost = settings.smtp_host || '';
  let smtpPort = settings.smtp_port || 587;
  let smtpUsername = settings.smtp_username || '';
  let smtpPassword = settings.smtp_password || '';
  let smtpEncryption = settings.smtp_encryption || 'starttls';
  let smtpFromEmail = settings.smtp_from_email || 'noreply@rustcraft.com';
  let smtpFromName = settings.smtp_from_name || 'RustCraft Gear';
  let smtpEnabled = Boolean(settings.smtp_enabled);

  let requireRegisteredCheckout = Boolean(settings.require_registered_checkout);
  let requireEmailVerification = Boolean(settings.require_email_verification);

  let testRecipient = supportEmail || 'admin@rustcraft.com';
  let isSendingTest = false;
  let testResult = null;
  let isSavingEmail = false;
  let emailNotice = '';
  let emailError = '';

  async function handleSaveEmailSettings() {
    isSavingEmail = true;
    emailNotice = '';
    emailError = '';

    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          ...settings,
          smtp_host: smtpHost,
          smtp_port: parseInt(smtpPort) || 587,
          smtp_username: smtpUsername,
          smtp_password: smtpPassword,
          smtp_encryption: smtpEncryption,
          smtp_from_email: smtpFromEmail,
          smtp_from_name: smtpFromName,
          smtp_enabled: smtpEnabled,
          require_registered_checkout: requireRegisteredCheckout,
          require_email_verification: requireEmailVerification
        })
      });

      if (res.ok) {
        emailNotice = 'Email addon settings & registration policies saved successfully!';
        setTimeout(() => emailNotice = '', 3500);
      } else {
        const err = await res.json();
        emailError = err.error || 'Failed to save settings.';
      }
    } catch (e) {
      emailError = 'Failed to connect to backend server.';
    } finally {
      isSavingEmail = false;
    }
  }

  async function handleSendTestEmail() {
    if (!testRecipient) return;
    isSendingTest = true;
    testResult = null;

    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch('/api/v1/admin/settings/email/test', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Dev-Mode': 'true',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({ recipient: testRecipient })
      });

      const data = await res.json();
      if (res.ok) {
        testResult = { success: true, message: data.message || `Test email dispatched successfully to ${testRecipient}!` };
      } else {
        testResult = { success: false, message: data.error || 'Failed to send test email. Check SMTP credentials.' };
      }
    } catch (e) {
      testResult = { success: false, message: 'Network connection failure trying to trigger email.' };
    } finally {
      isSendingTest = false;
    }
  }
</script>

<svelte:head>
  <title>Store Settings & System Administration | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-6xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <ShieldCheck size={24} class="text-orange-500" />
        Store Settings & Infrastructure
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Configure master store identity, payment processors, automated transactional email, and auth policies.
      </p>
    </div>
  </div>

  <!-- Tabs Navigation Bar -->
  <div class="flex items-center gap-2 border-b border-slate-800 pb-3 overflow-x-auto">
    <button
      type="button"
      on:click={() => setTab('identity')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'identity' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Building size={15} />
      <span>Store Identity & Legal</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('payments')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'payments' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <CreditCard size={15} />
      <span>Payment Providers</span>
      <span class="text-[10px] px-2 py-0.5 rounded-full {activeTab === 'payments' ? 'bg-white/20 text-white' : 'bg-slate-800 text-slate-400'}">{paymentConfigs.length}</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('email')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'email' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Mail size={15} />
      <span>Email & Auth Policies</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('shipping')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'shipping' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Truck size={15} />
      <span>Shipping & Delivery</span>
      <span class="text-[10px] px-2 py-0.5 rounded-full {activeTab === 'shipping' ? 'bg-white/20 text-white' : 'bg-slate-800 text-slate-400'}">{shippingProviders.length}</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('media')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'media' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Image size={15} />
      <span>Media Library</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('export')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'export' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Download size={15} />
      <span>Export & Backups</span>
    </button>
  </div>

  <!-- TAB 1: Store Identity & Legal -->
  {#if activeTab === 'identity'}
    <div class="space-y-6">
      <!-- Universal Identity Propagation Banner -->
      <div class="p-4 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 flex items-start gap-3">
        <Info size={18} class="text-indigo-400 flex-shrink-0 mt-0.5" />
        <div class="text-xs text-slate-300 space-y-1">
          <p class="font-bold text-white">Single Source of Truth for Store Identity & Legal Disclosures:</p>
          <p class="text-slate-400 leading-relaxed">
            Changing your Name, Address, Email, Phone, VAT ID, or Small Business Tax Notice (§19 UStG) here updates
            <strong>every page of the shop</strong>:
            it is automatically placed on all generated <strong>PDF Invoices</strong>, <strong>Packing Slips</strong>,
            and interpolated into all <strong>Policy CMS pages</strong> (Terms, Shipment, Privacy, Legal Notice).
          </p>
        </div>
      </div>

      {#if identitySuccessNotice}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <CheckCircle2 size={16} />
          <span>{identitySuccessNotice}</span>
        </div>
      {/if}

      {#if identityErrorNotice}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={16} />
          <span>{identityErrorNotice}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleSaveIdentity} class="space-y-6">
        <!-- Card 1: Core Company Profile -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
            <Building size={18} class="text-orange-400" />
            <span>Store Brand & Legal Entity Details</span>
          </h2>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Store Display / Trade Name</label>
              <input
                type="text"
                bind:value={storeName}
                required
                placeholder="e.g. RustCraft Store"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
              <p class="text-[10px] text-slate-500 mt-1">Displayed in storefront header, title tags, and brand badges.</p>
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Legal Company / Business Name</label>
              <input
                type="text"
                bind:value={legalName}
                required
                placeholder="e.g. Max Mustermann E-Commerce"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
              <p class="text-[10px] text-slate-500 mt-1">Official registered name used in Invoices, Impressum, and AGB.</p>
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Store Owner / Represented By</label>
              <input
                type="text"
                bind:value={storeOwner}
                required
                placeholder="e.g. Max Mustermann"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
              <p class="text-[10px] text-slate-500 mt-1">Authorized representative (§ 5 TMG Vertreten durch).</p>
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Header Subtitle / Tagline</label>
              <input
                type="text"
                bind:value={storeSubtitle}
                placeholder="e.g. Rust Powered • ACID Fast"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>

            <div class="sm:col-span-2">
              <label class="block text-slate-300 font-semibold mb-1">Company Registered Legal Address</label>
              <input
                type="text"
                bind:value={companyAddress}
                required
                placeholder="Street address, Postal code, City, Country"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
              <p class="text-[10px] text-slate-500 mt-1">Printed in invoice sender header and legal notice disclosures.</p>
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Commercial Register (Handelsregister, optional)</label>
              <input
                type="text"
                bind:value={commercialRegister}
                placeholder="e.g. Amtsgericht Köln, HRB 12345 (leave blank if unregistered)"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Customer Support Email</label>
              <input
                type="email"
                bind:value={supportEmail}
                required
                placeholder="support@example.com"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Contact Phone Number</label>
              <input
                type="text"
                bind:value={phone}
                placeholder="+49 (0) 30 123456-78"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>
          </div>
        </div>

        <!-- Card 2: Taxation & VAT Configuration -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-5">
          <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center justify-between">
            <span class="flex items-center gap-2">
              <ShieldCheck size={18} class="text-orange-400" />
              <span>Taxation & VAT Configuration (Umsatzsteuer-Modus)</span>
            </span>
            <span class="text-[11px] font-mono px-2.5 py-0.5 rounded-full {taxMode === 'kleingewerbe' ? 'bg-amber-500/15 text-amber-400 border border-amber-500/20' : 'bg-orange-500/15 text-orange-400 border border-orange-500/20'}">
              {taxMode === 'kleingewerbe' ? '§ 19 UStG Active' : taxMode === 'included' ? 'Gross (Brutto) VAT' : 'Net (Netto) VAT'}
            </span>
          </h2>

          <div class="space-y-4 text-xs">
            <div>
              <label class="block text-slate-300 font-semibold mb-2">Select Shop VAT / Tax Collection Regime:</label>
              <div class="grid grid-cols-1 md:grid-cols-3 gap-3.5">
                <!-- Option 1: Kleingewerbe -->
                <button
                  type="button"
                  on:click={() => handleTaxModeChange('kleingewerbe')}
                  class="p-4 rounded-xl border text-left transition-all flex flex-col justify-between {taxMode === 'kleingewerbe' ? 'bg-orange-600/15 border-orange-500 ring-1 ring-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}"
                >
                  <div class="space-y-1.5">
                    <div class="flex items-center justify-between">
                      <span class="font-bold text-white text-xs">German Kleingewerbe</span>
                      <input type="radio" name="tax_mode_radio" checked={taxMode === 'kleingewerbe'} class="accent-orange-500" />
                    </div>
                    <div class="text-[11px] text-amber-400 font-medium">Without VAT (§ 19 UStG)</div>
                    <p class="text-[11px] text-slate-400 leading-relaxed">
                      No VAT is collected or added during checkout or payment. Official § 19 UStG exemption text is printed on invoices and packing slips.
                    </p>
                  </div>
                </button>

                <!-- Option 2: Included VAT (Brutto) -->
                <button
                  type="button"
                  on:click={() => handleTaxModeChange('included')}
                  class="p-4 rounded-xl border text-left transition-all flex flex-col justify-between {taxMode === 'included' ? 'bg-orange-600/15 border-orange-500 ring-1 ring-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}"
                >
                  <div class="space-y-1.5">
                    <div class="flex items-center justify-between">
                      <span class="font-bold text-white text-xs">Included VAT (Brutto)</span>
                      <input type="radio" name="tax_mode_radio" checked={taxMode === 'included'} class="accent-orange-500" />
                    </div>
                    <div class="text-[11px] text-emerald-400 font-medium">VAT inside product price (B2C)</div>
                    <p class="text-[11px] text-slate-400 leading-relaxed">
                      Display prices include VAT. Product tax percentage is extracted and shown as "Included VAT" on checkout, orders, and PDF invoices.
                    </p>
                  </div>
                </button>

                <!-- Option 3: Excluded VAT (Netto) -->
                <button
                  type="button"
                  on:click={() => handleTaxModeChange('excluded')}
                  class="p-4 rounded-xl border text-left transition-all flex flex-col justify-between {taxMode === 'excluded' ? 'bg-orange-600/15 border-orange-500 ring-1 ring-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}"
                >
                  <div class="space-y-1.5">
                    <div class="flex items-center justify-between">
                      <span class="font-bold text-white text-xs">Excluded VAT (Netto)</span>
                      <input type="radio" name="tax_mode_radio" checked={taxMode === 'excluded'} class="accent-orange-500" />
                    </div>
                    <div class="text-[11px] text-sky-400 font-medium">VAT added in cart & checkout (B2B)</div>
                    <p class="text-[11px] text-slate-400 leading-relaxed">
                      Catalog prices are net. Applicable product VAT percentages are calculated and added on top in the shopping cart and checkout total.
                    </p>
                  </div>
                </button>
              </div>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-4 gap-4 pt-2">
              <div>
                <label class="block text-slate-300 font-semibold mb-1">Standard Store VAT Rate (%)</label>
                <div class="relative">
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    max="100"
                    bind:value={taxRatePercent}
                    disabled={taxMode === 'kleingewerbe'}
                    class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500 disabled:opacity-40"
                  />
                  <span class="absolute right-3 top-2.5 text-slate-500 font-mono">%</span>
                </div>
                <p class="text-[10px] text-slate-500 mt-1">Default rate for new products (e.g. 19% or 7%).</p>
              </div>

              <div>
                <label class="block text-slate-300 font-semibold mb-1">VAT ID (USt-IdNr.)</label>
                <input
                  type="text"
                  bind:value={vatId}
                  placeholder="e.g. DE314159265"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
                />
                <p class="text-[10px] text-slate-500 mt-1">Leave blank if unregistered.</p>
              </div>

              <div>
                <label class="block text-slate-300 font-semibold mb-1">Currency Code</label>
                <input
                  type="text"
                  bind:value={currency}
                  placeholder="EUR"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
                />
                <p class="text-[10px] text-slate-500 mt-1">ISO 4217 (EUR, USD, GBP).</p>
              </div>

              <div>
                <label class="block text-slate-300 font-semibold mb-1">Currency Symbol</label>
                <input
                  type="text"
                  bind:value={currencySymbol}
                  placeholder="€"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
                />
                <p class="text-[10px] text-slate-500 mt-1">Formatted across storefront.</p>
              </div>

              <div class="sm:col-span-4">
                <label class="block text-slate-300 font-semibold mb-1">
                  Tax Exemption / Statutory Invoice Notice (§ 19 UStG Notice)
                </label>
                <textarea
                  bind:value={taxNotice}
                  rows="2"
                  placeholder="Gemäß § 19 UStG wird keine Umsatzsteuer berechnet."
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs leading-relaxed focus:outline-none focus:border-orange-500"
                ></textarea>
                <p class="text-[10px] text-slate-500 mt-1">
                  Appears automatically in the footer of all PDF Invoices and Packing Slips, and replaces {"{{TAX_NOTICE}}"} in Policy pages.
                </p>
              </div>

              <div class="sm:col-span-4 pt-2 border-t border-slate-800 space-y-3">
                <div class="flex items-center gap-2 text-white font-semibold">
                  <Scale size={15} class="text-orange-400" />
                  <span>EU Consumer Dispute Resolution (ODR Platform)</span>
                </div>

                <div>
                  <label class="block text-slate-300 font-semibold mb-1">Online Dispute Resolution Platform URL</label>
                  <input
                    type="text"
                    bind:value={odrUrl}
                    placeholder="https://ec.europa.eu/odr"
                    class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
                  />
                </div>

                <div>
                  <label class="block text-slate-300 font-semibold mb-1">Dispute Resolution Statement / Arbitration Notice</label>
                  <textarea
                    bind:value={disputeResolutionNotice}
                    rows="2"
                    placeholder="The European Commission provides a platform for out-of-court resolution of disputes..."
                    class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs leading-relaxed focus:outline-none focus:border-orange-500"
                  ></textarea>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Card 3: Order Number Format & Sequential Layout (GoBD-konform) -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-5">
          <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center justify-between">
            <span class="flex items-center gap-2">
              <Sliders size={18} class="text-orange-400" />
              <span>Order Number Layout & Sequential Counter (GoBD)</span>
            </span>
            <span class="text-[11px] font-mono px-2.5 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400 border border-emerald-500/20 font-bold">
              Ascending Numbers Active
            </span>
          </h2>

          <div class="space-y-4 text-xs">
            <p class="text-slate-400 leading-relaxed">
              In accordance with German tax regulations (GoBD & § 14 UStG), every generated order receives a strictly ascending, consecutive counter starting at 10000. Customize whether to include a store prefix or daily date code in the final number string.
            </p>

            <!-- Live Order Number Preview Badge -->
            <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div>
                <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block">Live Next Order Number Preview:</span>
                <span class="text-xl font-black font-mono text-orange-400 tracking-wider mt-0.5 block">{sampleOrderNumber}</span>
              </div>
              <div class="text-[11px] text-slate-500 font-mono space-y-0.5">
                <div>Prefix: <span class="text-white">{orderPrefixEnabled ? (orderPrefix.trim().toUpperCase() || 'NONE') : 'OFF'}</span></div>
                <div>Date Stamp: <span class="text-white">{orderDateEnabled ? todayDateStr : 'OFF'}</span></div>
                <div>Sequence Counter: <span class="text-emerald-400 font-bold">10000+ (Ascending)</span></div>
              </div>
            </div>

            <!-- Controls Grid -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-5 pt-1">
              <!-- Prefix Settings -->
              <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3">
                <div class="flex items-center justify-between">
                  <span class="font-bold text-white">Custom Brand Prefix</span>
                  <label class="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" bind:checked={orderPrefixEnabled} class="sr-only peer" />
                    <div class="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-orange-600"></div>
                  </label>
                </div>
                <p class="text-[11px] text-slate-400 leading-snug">
                  Toggle on/off an alphanumeric brand prefix (up to 7 characters, e.g. <code class="text-orange-400 font-mono">ORD</code> or <code class="text-orange-400 font-mono">ABCDEFG</code>).
                </p>
                <div>
                  <label class="block text-slate-400 mb-1 text-[11px]">Prefix Code (Max 7 Letters)</label>
                  <input
                    type="text"
                    bind:value={orderPrefix}
                    maxlength="7"
                    disabled={!orderPrefixEnabled}
                    placeholder="e.g. ORD"
                    class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-white font-mono uppercase text-xs focus:outline-none focus:border-orange-500 disabled:opacity-40"
                  />
                </div>
              </div>

              <!-- Date Code Settings -->
              <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3">
                <div class="flex items-center justify-between">
                  <span class="font-bold text-white">Date Code (YYYYMMDD)</span>
                  <label class="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" bind:checked={orderDateEnabled} class="sr-only peer" />
                    <div class="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-orange-600"></div>
                  </label>
                </div>
                <p class="text-[11px] text-slate-400 leading-snug">
                  Include or exclude the UTC date timestamp (<code class="text-orange-400 font-mono">{todayDateStr}</code>) in the order string.
                </p>
                <div class="pt-2">
                  <div class="text-[11px] text-slate-400">
                    Resulting Pattern: <code class="text-white font-mono">{orderPrefixEnabled ? '[PREFIX]-' : ''}{orderDateEnabled ? '[DATE]-' : ''}[10000]</code>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Card 4: Logo & Header Brand Image -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
            <Image size={18} class="text-orange-400" />
            <span>Store Logo & Media Asset</span>
          </h2>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-6 items-center">
            <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3 text-xs">
              <span class="font-semibold text-slate-300 block">Brand Logo File</span>
              <button
                type="button"
                on:click={() => showLogoPicker = true}
                class="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold flex items-center gap-1.5 transition-colors border border-slate-700"
              >
                <FolderOpen size={14} class="text-orange-400" />
                <span>Choose from Media Library</span>
              </button>

              <div>
                <label class="block text-slate-400 mb-1 text-[11px]">Logo URL / Image Path</label>
                <input
                  type="text"
                  bind:value={logoUrl}
                  placeholder="/uploads/... or https://..."
                  class="w-full px-3 py-2 rounded-lg bg-slate-900 border border-slate-800 text-slate-300 font-mono text-xs focus:outline-none focus:border-orange-500"
                />
              </div>
            </div>

            <div class="p-6 rounded-xl bg-slate-950 border border-slate-800 flex flex-col items-center justify-center space-y-2">
              <span class="text-xs font-semibold text-slate-400">Preview</span>
              {#if logoUrl}
                <img src={logoUrl} alt="Store Logo" class="h-14 max-w-[220px] object-contain rounded-lg p-1 bg-slate-900 border border-slate-800 shadow" />
              {:else}
                <div class="h-12 px-4 rounded-xl bg-slate-900 border border-slate-800 flex items-center gap-2 text-slate-400 text-xs font-bold">
                  <span>🦀</span>
                  <span>{storeName || 'RustCraft'}</span>
                </div>
              {/if}
              <span class="text-[10px] text-slate-500">Used on Storefront Header, Invoices, and Slips</span>
            </div>
          </div>
        </div>

        <!-- Card 4: System Deployment & Debugging Environment -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
            <Sliders size={18} class="text-orange-400" />
            <span>Storefront Operating Mode & Diagnostics</span>
          </h2>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-6 text-xs">
            <div>
              <label class="block text-slate-300 font-semibold mb-1">Deployment & Processing Mode</label>
              <select
                bind:value={deploymentMode}
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-medium focus:outline-none focus:border-orange-500"
              >
                <option value="production">Live / Production (Real Gateway Transactions & Webhooks)</option>
                <option value="development">Development / Sandbox (Simulated Transactions, Test Cards)</option>
                <option value="staging">Staging / Pre-Release Testing</option>
              </select>
              <p class="text-[10px] text-slate-500 mt-1.5">
                Switch to <strong>Live / Production</strong> when you are ready to accept real payments. Development mode enables test-mode bypasses.
              </p>
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Developer Debugging Mode</label>
              <div class="mt-2 flex items-center gap-3">
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" bind:checked={debugMode} class="sr-only peer" />
                  <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-600"></div>
                </label>
                <span class="text-xs font-semibold {debugMode ? 'text-orange-400' : 'text-slate-400'}">
                  {debugMode ? 'Debug Mode Active (Verbose Logs & Timing Headers)' : 'Debug Mode Off'}
                </span>
              </div>
              <p class="text-[10px] text-slate-500 mt-2">
                Enables deep tracing and developer diagnostic payloads in API responses.
              </p>
            </div>
          </div>
        </div>

        <!-- Submit Button -->
        <div class="flex justify-end">
          <button
            type="submit"
            disabled={isSavingIdentity}
            class="px-8 py-3 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all flex items-center gap-2 disabled:opacity-50"
          >
            <Save size={16} />
            <span>{isSavingIdentity ? 'Saving Store Identity...' : 'Save Store Identity & Environment'}</span>
          </button>
        </div>
      </form>
    </div>

  <!-- TAB 2: Payment Providers -->
  {:else if activeTab === 'payments'}
    <div class="space-y-6">
      <div>
        <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
          <CreditCard size={22} class="text-orange-500" />
          Payment Gateway Processors
        </h2>
        <p class="text-xs text-slate-400 mt-1">
          Manage active payment methods. Stripe and PayPal are primary; Apple Pay, Google Pay, and Crypto are configurable.
        </p>
      </div>

      {#if paymentNotice}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <Check size={16} />
          <span>{paymentNotice}</span>
        </div>
      {/if}

      <div class="space-y-5">
        {#each paymentConfigs as provider}
          {@const isStripe = provider.provider === 'stripe'}
          <div class="p-6 rounded-3xl border shadow-xl space-y-4 {isStripe ? 'bg-gradient-to-br from-slate-900 via-slate-900 to-orange-950/20 border-orange-500/50 shadow-orange-950/20 ring-1 ring-orange-500/20' : 'bg-slate-900 border-slate-800'}">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-4">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center font-black text-orange-400">
                  {#if provider.provider === 'stripe'}S
                  {:else if provider.provider === 'paypal'}P
                  {:else if provider.provider === 'apple_pay'}
                  {:else if provider.provider === 'google_pay'}G
                  {:else if provider.provider === 'crypto'}₿
                  {:else}M{/if}
                </div>
                <div>
                  <h3 class="text-sm font-bold text-white flex items-center gap-2">
                    <span>{provider.display_name}</span>
                    {#if isStripe}
                      <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-orange-500/20 text-orange-400 border border-orange-500/30">Primary</span>
                    {/if}
                  </h3>
                  <span class="text-xs text-slate-500 font-mono">provider: {provider.provider}</span>
                </div>
              </div>

              <!-- Toggles: Enabled & Sandbox -->
              <div class="flex items-center gap-6 text-xs">
                <label class="flex items-center gap-2 cursor-pointer font-semibold text-slate-300">
                  <input type="checkbox" bind:checked={provider.is_sandbox} class="accent-orange-500 w-4 h-4" />
                  <span>Sandbox / Test Mode</span>
                </label>

                <label class="flex items-center gap-2 cursor-pointer font-semibold text-slate-300">
                  <input type="checkbox" bind:checked={provider.is_enabled} class="accent-orange-500 w-4 h-4" />
                  <span>Enabled</span>
                </label>
              </div>
            </div>

            <!-- Provider Configuration Inputs -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
              <div>
                <label class="block text-slate-400 mb-1 font-semibold">Storefront Display Title</label>
                <input
                  type="text"
                  bind:value={provider.display_name}
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
                />
              </div>

              <div>
                <label class="block text-slate-400 mb-1 font-semibold">Public Key / Merchant Client ID</label>
                <input
                  type="text"
                  bind:value={provider.public_client_id}
                  placeholder={provider.is_sandbox ? 'pk_test_...' : 'pk_live_...'}
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
                />
              </div>
            </div>

            <!-- Footer Save & Webhook Info -->
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3 border-t border-slate-800/80 text-xs">
              <span class="text-slate-500 font-mono text-[11px]">
                Webhook Endpoint: <code class="text-orange-400">/api/v1/payments/{provider.provider}/webhook</code>
              </span>

              <button
                type="button"
                on:click={() => savePaymentConfig(provider)}
                disabled={savingProvider === provider.provider}
                class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white font-bold text-xs shadow-md shadow-orange-600/30 flex items-center gap-1.5 self-start sm:self-auto transition-all"
              >
                <Save size={14} />
                <span>{savingProvider === provider.provider ? 'Saving...' : 'Save Provider'}</span>
              </button>
            </div>
          </div>
        {/each}
      </div>
    </div>

  <!-- TAB 3: Email & Auth Policies -->
  {:else if activeTab === 'email'}
    <div class="space-y-6">
      <div>
        <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
          <Mail size={22} class="text-orange-500" />
          Transactional Email & Authentication Policies
        </h2>
        <p class="text-xs text-slate-400 mt-1">
          Configure outgoing SMTP email for order receipts, tracking confirmations, and customer registration policies.
        </p>
      </div>

      {#if emailNotice}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <CheckCircle2 size={16} />
          <span>{emailNotice}</span>
        </div>
      {/if}

      {#if emailError}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={16} />
          <span>{emailError}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleSaveEmailSettings} class="space-y-6">
        <!-- SMTP Server Configuration -->
        <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-5">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between pb-3 border-b border-slate-800 gap-4">
            <h3 class="text-sm font-bold text-white flex items-center gap-2">
              <Server size={18} class="text-orange-400" />
              <span>Outgoing SMTP Server (lettre / TLS)</span>
            </h3>

            <label class="flex items-center gap-2 cursor-pointer bg-slate-950 px-3.5 py-1.5 rounded-xl border border-slate-800 self-start sm:self-auto text-xs">
              <input type="checkbox" bind:checked={smtpEnabled} class="accent-orange-500 w-4 h-4 rounded" />
              <span class="font-bold {smtpEnabled ? 'text-emerald-400' : 'text-slate-400'}">
                {smtpEnabled ? 'SMTP Enabled' : 'SMTP Disabled'}
              </span>
            </label>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs">
            <div class="sm:col-span-2">
              <label class="block text-slate-300 font-semibold mb-1">SMTP Host / Server</label>
              <input
                type="text"
                bind:value={smtpHost}
                placeholder="e.g. smtp.sendgrid.net, mail.yourdomain.com"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">SMTP Port</label>
              <input
                type="number"
                bind:value={smtpPort}
                placeholder="587"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">SMTP Username</label>
              <input
                type="text"
                bind:value={smtpUsername}
                placeholder="apikey, or user@domain.com"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">SMTP Password</label>
              <input
                type="password"
                bind:value={smtpPassword}
                placeholder="••••••••••••"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Encryption Mode</label>
              <select
                bind:value={smtpEncryption}
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              >
                <option value="starttls">STARTTLS (Port 587 - Recommended)</option>
                <option value="tls">Direct TLS / SSL (Port 465)</option>
                <option value="none">None (Insecure / Local dev)</option>
              </select>
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Sender Email Address</label>
              <input
                type="email"
                bind:value={smtpFromEmail}
                placeholder="noreply@rustcraft.com"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>

            <div class="sm:col-span-2">
              <label class="block text-slate-300 font-semibold mb-1">Sender Display Name</label>
              <input
                type="text"
                bind:value={smtpFromName}
                placeholder="RustCraft Gear Store"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>
          </div>
        </div>

        <!-- Auth & Checkout Policies -->
        <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <h3 class="text-sm font-bold text-white flex items-center gap-2 border-b border-slate-800 pb-3">
            <UserCheck size={18} class="text-orange-400" />
            <span>Store Customer Authentication & Checkout Policies</span>
          </h3>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
            <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 flex items-center justify-between">
              <div>
                <span class="font-bold text-white block">Require Account at Checkout</span>
                <p class="text-[11px] text-slate-400 mt-0.5">Disables guest checkout; forces login or registration.</p>
              </div>
              <input type="checkbox" bind:checked={requireRegisteredCheckout} class="accent-orange-500 w-4 h-4 ml-3" />
            </div>

            <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 flex items-center justify-between">
              <div>
                <span class="font-bold text-white block">Require Email Verification</span>
                <p class="text-[11px] text-slate-400 mt-0.5">Sends confirmation links before customer can log in.</p>
              </div>
              <input type="checkbox" bind:checked={requireEmailVerification} class="accent-orange-500 w-4 h-4 ml-3" />
            </div>
          </div>
        </div>

        <!-- Submit & Test -->
        <div class="flex flex-col sm:flex-row items-center justify-between gap-4">
          <!-- Quick Send Test Email -->
          <div class="flex items-center gap-2 w-full sm:w-auto">
            <input
              type="email"
              bind:value={testRecipient}
              placeholder="Test recipient email..."
              class="px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs w-56 focus:outline-none focus:border-orange-500"
            />
            <button
              type="button"
              on:click={handleSendTestEmail}
              disabled={isSendingTest || !testRecipient}
              class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 disabled:opacity-50 text-slate-200 text-xs font-semibold flex items-center gap-1.5 transition-colors border border-slate-700"
            >
              <Send size={13} class="text-orange-400" />
              <span>{isSendingTest ? 'Sending...' : 'Send Test'}</span>
            </button>
          </div>

          <button
            type="submit"
            disabled={isSavingEmail}
            class="px-8 py-3 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all flex items-center gap-2 disabled:opacity-50"
          >
            <Save size={16} />
            <span>{isSavingEmail ? 'Saving...' : 'Save Email & Auth Policies'}</span>
          </button>
        </div>

        {#if testResult}
          <div class="p-3.5 rounded-xl text-xs font-semibold flex items-center gap-2 {testResult.success ? 'bg-emerald-500/10 border border-emerald-500/20 text-emerald-400' : 'bg-rose-500/10 border border-rose-500/20 text-rose-400'}">
            {#if testResult.success}
              <CheckCircle2 size={16} />
            {:else}
              <AlertCircle size={16} />
            {/if}
            <span>{testResult.message}</span>
          </div>
        {/if}
      </form>
    </div>

  <!-- TAB 4: Shipping & Delivery (Full Inline Management) -->
  {:else if activeTab === 'shipping'}
    <ShippingManager initialProviders={shippingProviders} />

  <!-- TAB 5: Media Library (Fully Integrated) -->
  {:else if activeTab === 'media'}
    <MediaManager />

  <!-- TAB 6: Export & Backups -->
  {:else if activeTab === 'export'}
    <div class="space-y-6">
      <div class="p-4 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 flex items-start gap-3">
        <Info size={18} class="text-indigo-400 flex-shrink-0 mt-0.5" />
        <div class="text-xs text-slate-300 space-y-1">
          <p class="font-bold text-white">Full Store Data & Media Archive Exports:</p>
          <p class="text-slate-400 leading-relaxed">
            Download your store database snapshot (settings, products, variants, SKU inventory, categories, custom legal policy markdown pages, navigation menus, and shipping carrier setups) as JSON, or export your complete media library files as a compressed ZIP.
          </p>
        </div>
      </div>

      {#if exportSuccessNotice}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <CheckCircle2 size={16} />
          <span>{exportSuccessNotice}</span>
        </div>
      {/if}

      {#if exportErrorNotice}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={16} />
          <span>{exportErrorNotice}</span>
        </div>
      {/if}

      <!-- Hidden file input for schema-verified import -->
      <input
        type="file"
        bind:this={importFileInput}
        on:change={handleFileSelectForImport}
        accept=".json,application/json"
        class="hidden"
      />

      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <!-- Card 1: Store Configuration & Content Export -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4 flex flex-col justify-between">
          <div class="space-y-3">
            <div class="w-12 h-12 rounded-2xl bg-orange-600/15 text-orange-400 border border-orange-500/20 flex items-center justify-center">
              <Download size={24} />
            </div>
            <h3 class="text-base font-bold text-white">Export Store Settings & Catalog (JSON)</h3>
            <p class="text-xs text-slate-400 leading-relaxed">
              Generates a single comprehensive JSON file with all store configuration parameters, active products, variants, categories, CMS policy revisions, header & footer navigation structures, and shipping providers.
            </p>
            <div class="text-[11px] text-slate-500 space-y-1 pt-1 font-mono">
              <div>&bull; Store Settings & Legal Disclosures</div>
              <div>&bull; Product Catalog & Variant SKUs</div>
              <div>&bull; Legal Policy CMS Revisions</div>
              <div>&bull; Shipping Zones & Carrier Profiles</div>
            </div>
          </div>

          <div class="pt-4 border-t border-slate-800">
            <button
              type="button"
              on:click={handleExportStoreData}
              disabled={isExportingData}
              class="w-full py-3 px-4 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-xs shadow-lg shadow-orange-600/20 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
            >
              <Download size={15} class={isExportingData ? 'animate-bounce' : ''} />
              <span>{isExportingData ? 'Generating Store Export...' : 'Download Store Data (JSON)'}</span>
            </button>
          </div>
        </div>

        <!-- Card 2: Import Store Settings & Catalog -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4 flex flex-col justify-between">
          <div class="space-y-3">
            <div class="w-12 h-12 rounded-2xl bg-emerald-600/15 text-emerald-400 border border-emerald-500/20 flex items-center justify-center">
              <Upload size={24} />
            </div>
            <div class="flex items-center justify-between">
              <h3 class="text-base font-bold text-white">Import Store Settings & Data</h3>
              <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-bold">v1.x Schema Check</span>
            </div>
            <p class="text-xs text-slate-400 leading-relaxed">
              Restore your store data from an exported JSON backup. An automated schema version check ensures incompatible backups cannot corrupt your database.
            </p>
            <div class="text-[11px] text-slate-500 space-y-1 pt-1 font-mono">
              <div>&bull; Strict Schema Version Verification (1.x)</div>
              <div>&bull; Restores Settings, Products & SKUs</div>
              <div>&bull; Restores Categories & Navigation Menus</div>
              <div>&bull; Restores Legal Markdown Revisions</div>
            </div>
          </div>

          <div class="pt-4 border-t border-slate-800">
            <button
              type="button"
              on:click={() => importFileInput && importFileInput.click()}
              disabled={isImportingData}
              class="w-full py-3 px-4 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs shadow-lg shadow-emerald-600/20 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
            >
              {#if isImportingData}
                <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                <span>Restoring Store Data...</span>
              {:else}
                <Upload size={15} />
                <span>Upload & Import JSON Backup</span>
              {/if}
            </button>
          </div>
        </div>

        <!-- Card 3: Media Library ZIP Export -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4 flex flex-col justify-between">
          <div class="space-y-3">
            <div class="w-12 h-12 rounded-2xl bg-indigo-600/15 text-indigo-400 border border-indigo-500/20 flex items-center justify-center">
              <FileArchive size={24} />
            </div>
            <h3 class="text-base font-bold text-white">Export Media Library (ZIP Archive)</h3>
            <p class="text-xs text-slate-400 leading-relaxed">
              Streams and compresses all uploaded images, product photos, brand logos, banners, and digital assets stored in the persistent uploads media volume into a downloadable ZIP archive.
            </p>
            <div class="text-[11px] text-slate-500 space-y-1 pt-1 font-mono">
              <div>&bull; Uploaded Product Images</div>
              <div>&bull; Brand Logos & Banners</div>
              <div>&bull; Hero Carousel Media</div>
              <div>&bull; Standard Deflate ZIP Packaging</div>
            </div>
          </div>

          <div class="pt-4 border-t border-slate-800">
            <button
              type="button"
              on:click={handleExportMedia}
              disabled={isExportingMedia}
              class="w-full py-3 px-4 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs border border-slate-700 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
            >
              <FileArchive size={15} class={isExportingMedia ? 'animate-spin' : ''} />
              <span>{isExportingMedia ? 'Compressing Media Library...' : 'Download Media Library (.ZIP)'}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Logo Media Picker Modal -->
<MediaPickerModal
  open={showLogoPicker}
  onSelect={(url) => {
    logoUrl = url;
    showLogoPicker = false;
  }}
  onClose={() => showLogoPicker = false}
/>
