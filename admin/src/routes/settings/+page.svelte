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
  let logoUrl = settings.logo_url || '';

  // System Environment & Debugging Mode
  let deploymentMode = settings.deployment_mode || 'development';
  let debugMode = settings.debug_mode !== undefined ? settings.debug_mode : false;

  let isSavingIdentity = false;
  let identitySuccessNotice = '';
  let identityErrorNotice = '';
  let showLogoPicker = false;

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
        commercial_register: commercialRegister,
        odr_url: odrUrl,
        dispute_resolution_notice: disputeResolutionNotice,
        currency,
        currency_symbol: currencySymbol,
        tax_rate_percent: parseFloat(taxRatePercent) || 0,
        logo_url: logoUrl,
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
        identitySuccessNotice = 'Store Identity & Environment settings saved!';
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

  // --- Footer & Social State ---
  let footerConfig = {
    branding_mode: 'full',
    menu_layout: 'columns',
    columns: [
      {
        title: 'Customer Service',
        links: [
          { label: 'Shipping Policy & Rates', url: '/policies/shipment-policy' },
          { label: 'Return Policy', url: '/policies/return-policy' },
          { label: 'Revocation Policy & Form', url: '/policies/revocation-policy' },
          { label: 'Track Order', url: '/track' }
        ]
      },
      {
        title: 'Legal & Privacy',
        links: [
          { label: 'Legal Notice (Impressum)', url: '/policies/legal-notice' },
          { label: 'Terms and Conditions (AGB)', url: '/policies/terms-conditions' },
          { label: 'Privacy Policy (GDPR)', url: '/policies/privacy-policy' },
          { label: 'Cookie Policy', url: '/policies/cookie-policy' }
        ]
      },
      {
        title: 'Store & Support',
        links: [
          { label: 'Contact Information', url: '/policies/contact' }
        ]
      }
    ],
    social_links: {
      github: 'https://github.com',
      twitter: 'https://x.com',
      instagram: '',
      youtube: '',
      facebook: '',
      discord: 'https://discord.gg',
      whatsapp: ''
    },
    enabled_socials: ['github', 'twitter', 'discord'],
    show_socials: true,
    show_payments: true,
    copyright_format: 'standard',
    custom_copyright: '',
    ...(settings.footer_config || {})
  };

  let isSavingFooter = false;
  let footerSuccessNotice = '';
  let footerErrorNotice = '';

  function toggleSocial(platform) {
    if (footerConfig.enabled_socials.includes(platform)) {
      footerConfig.enabled_socials = footerConfig.enabled_socials.filter((p) => p !== platform);
    } else {
      footerConfig.enabled_socials = [...footerConfig.enabled_socials, platform];
    }
  }

  function addColumn() {
    if (footerConfig.columns.length < 3) {
      footerConfig.columns = [
        ...footerConfig.columns,
        { title: `Section ${footerConfig.columns.length + 1}`, links: [{ label: 'New Link', url: '/' }] }
      ];
    }
  }

  function removeColumn(idx) {
    footerConfig.columns = footerConfig.columns.filter((_, i) => i !== idx);
  }

  function addLinkToColumn(colIdx) {
    footerConfig.columns[colIdx].links = [
      ...footerConfig.columns[colIdx].links,
      { label: 'New Link', url: '/' }
    ];
  }

  function removeLinkFromColumn(colIdx, linkIdx) {
    footerConfig.columns[colIdx].links = footerConfig.columns[colIdx].links.filter((_, i) => i !== linkIdx);
  }

  async function handleSaveFooter() {
    isSavingFooter = true;
    footerSuccessNotice = '';
    footerErrorNotice = '';

    const token = localStorage.getItem('admin_token');
    const headers = {
      'Content-Type': 'application/json',
      'X-Dev-Mode': 'true',
      ...(token ? { Authorization: `Bearer ${token}` } : {})
    };

    try {
      const payload = {
        ...settings,
        footer_config: footerConfig
      };

      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers,
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        settings = payload;
        footerSuccessNotice = 'Storefront Footer & Social settings saved successfully!';
        setTimeout(() => footerSuccessNotice = '', 4500);
      } else {
        const err = await res.json();
        footerErrorNotice = err.error || 'Failed to save footer settings.';
      }
    } catch (e) {
      footerErrorNotice = 'Failed to connect to backend server.';
    } finally {
      isSavingFooter = false;
    }
  }

  // --- Export & Backups Handlers ---
  let isExportingData = false;
  let isExportingMedia = false;
  let exportSuccessNotice = '';
  let exportErrorNotice = '';

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
      on:click={() => setTab('footer')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'footer' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Layout size={15} />
      <span>Footer & Social</span>
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

        <!-- Card 2: Tax, Invoicing & Legal Notices -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
            <ShieldCheck size={18} class="text-orange-400" />
            <span>Taxation, Currency & Invoice Disclosures</span>
          </h2>

          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs">
            <div>
              <label class="block text-slate-300 font-semibold mb-1">VAT Identification Number (USt-IdNr.)</label>
              <input
                type="text"
                bind:value={vatId}
                placeholder="e.g. DE314159265"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Default Currency Code</label>
              <input
                type="text"
                bind:value={currency}
                placeholder="EUR"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
              />
            </div>

            <div>
              <label class="block text-slate-300 font-semibold mb-1">Currency Symbol</label>
              <input
                type="text"
                bind:value={currencySymbol}
                placeholder="€"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
              />
            </div>

            <div class="sm:col-span-3">
              <label class="block text-slate-300 font-semibold mb-1">
                Small Business Regulation / Tax Exemption Notice (§ 19 UStG)
              </label>
              <textarea
                bind:value={taxNotice}
                rows="2"
                placeholder="Gemäß § 19 UStG wird keine Umsatzsteuer berechnet (Kleinunternehmerstatus)."
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs leading-relaxed focus:outline-none focus:border-orange-500"
              ></textarea>
              <p class="text-[10px] text-slate-500 mt-1">
                Appears automatically in the footer of all PDF Invoices and Packing Slips, and resolves into {"{{TAX_NOTICE}}"} in Policy pages.
              </p>
            </div>

            <div class="sm:col-span-3 pt-2 border-t border-slate-800 space-y-3">
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

        <!-- Card 3: Logo & Header Brand Image -->
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

  <!-- TAB 6: Footer & Social -->
  {:else if activeTab === 'footer'}
    <div class="space-y-6">
      {#if footerSuccessNotice}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <CheckCircle2 size={16} />
          <span>{footerSuccessNotice}</span>
        </div>
      {/if}

      {#if footerErrorNotice}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={16} />
          <span>{footerErrorNotice}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleSaveFooter} class="space-y-6">
        <!-- Section 1: Left - Store Branding -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
            <Building size={18} class="text-orange-400" />
            <span>Left Section: Storefront Branding Presentation</span>
          </h2>
          <p class="text-xs text-slate-400">Choose how your brand identity appears on the left side of the storefront footer.</p>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <label class="p-4 rounded-xl border cursor-pointer transition-all {footerConfig.branding_mode === 'full' ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}">
              <div class="flex items-center gap-3">
                <input
                  type="radio"
                  name="branding_mode"
                  value="full"
                  bind:group={footerConfig.branding_mode}
                  class="accent-orange-600"
                />
                <div>
                  <div class="text-xs font-bold text-white">Full Store Identity & Details</div>
                  <div class="text-[11px] text-slate-400 mt-0.5">Logo, store name, address, support email, and phone number</div>
                </div>
              </div>
            </label>

            <label class="p-4 rounded-xl border cursor-pointer transition-all {footerConfig.branding_mode === 'logo_only' ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}">
              <div class="flex items-center gap-3">
                <input
                  type="radio"
                  name="branding_mode"
                  value="logo_only"
                  bind:group={footerConfig.branding_mode}
                  class="accent-orange-600"
                />
                <div>
                  <div class="text-xs font-bold text-white">Logo Only (Minimalist)</div>
                  <div class="text-[11px] text-slate-400 mt-0.5">Displays only the brand logo or store title without address details</div>
                </div>
              </div>
            </label>
          </div>
        </div>

        <!-- Section 2: Middle - Footer Menu (Columns vs Single Line) -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-slate-800 pb-3">
            <h2 class="text-base font-bold text-white flex items-center gap-2">
              <Layout size={18} class="text-orange-400" />
              <span>Middle Section: Footer Menu & Column Architecture</span>
            </h2>
            <div class="flex items-center gap-2">
              <label class="inline-flex items-center gap-2 text-xs text-slate-300">
                <input
                  type="radio"
                  name="menu_layout"
                  value="columns"
                  bind:group={footerConfig.menu_layout}
                  class="accent-orange-600"
                />
                <span>Up to 3 Columns</span>
              </label>
              <label class="inline-flex items-center gap-2 text-xs text-slate-300 ml-3">
                <input
                  type="radio"
                  name="menu_layout"
                  value="single_line"
                  bind:group={footerConfig.menu_layout}
                  class="accent-orange-600"
                />
                <span>Single Line Menu</span>
              </label>
            </div>
          </div>

          {#if footerConfig.menu_layout === 'columns'}
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <p class="text-xs text-slate-400">Configure up to 3 top-aligned navigation columns with customized headings.</p>
                {#if footerConfig.columns.length < 3}
                  <button
                    type="button"
                    on:click={addColumn}
                    class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold flex items-center gap-1.5 transition-colors"
                  >
                    <Plus size={13} class="text-orange-400" />
                    <span>Add Column ({footerConfig.columns.length}/3)</span>
                  </button>
                {/if}
              </div>

              <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                {#each footerConfig.columns as col, colIdx}
                  <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3">
                    <div class="flex items-center justify-between gap-2 border-b border-slate-800/80 pb-2">
                      <input
                        type="text"
                        bind:value={col.title}
                        placeholder="Column Heading"
                        class="w-full px-2.5 py-1.5 rounded-lg bg-slate-900 border border-slate-700 text-white font-bold text-xs focus:outline-none focus:border-orange-500"
                      />
                      {#if footerConfig.columns.length > 1}
                        <button
                          type="button"
                          on:click={() => removeColumn(colIdx)}
                          class="p-1.5 text-slate-500 hover:text-rose-400 transition-colors"
                          title="Remove Column"
                        >
                          <Trash2 size={14} />
                        </button>
                      {/if}
                    </div>

                    <div class="space-y-2">
                      {#each col.links as link, lIdx}
                        <div class="flex items-center gap-2">
                          <input
                            type="text"
                            bind:value={link.label}
                            placeholder="Link Title"
                            class="w-1/2 px-2 py-1 rounded bg-slate-900 border border-slate-800 text-white text-[11px] focus:outline-none focus:border-orange-500"
                          />
                          <input
                            type="text"
                            bind:value={link.url}
                            placeholder="/path"
                            class="w-1/2 px-2 py-1 rounded bg-slate-900 border border-slate-800 text-slate-300 font-mono text-[11px] focus:outline-none focus:border-orange-500"
                          />
                          <button
                            type="button"
                            on:click={() => removeLinkFromColumn(colIdx, lIdx)}
                            class="p-1 text-slate-600 hover:text-rose-400 transition-colors"
                          >
                            <Trash2 size={12} />
                          </button>
                        </div>
                      {/each}

                      <button
                        type="button"
                        on:click={() => addLinkToColumn(colIdx)}
                        class="w-full py-1.5 rounded border border-dashed border-slate-800 hover:border-orange-500 text-slate-400 hover:text-orange-400 text-[11px] font-semibold flex items-center justify-center gap-1 transition-colors mt-2"
                      >
                        <Plus size={11} />
                        <span>Add Link</span>
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {:else}
            <!-- Single Line Mode -->
            <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3">
              <p class="text-xs text-slate-400">Links will be displayed horizontally in a single bar across the middle section.</p>
              <div class="space-y-2">
                {#if footerConfig.columns[0]}
                  {#each footerConfig.columns[0].links as link, lIdx}
                    <div class="flex items-center gap-2">
                      <input
                        type="text"
                        bind:value={link.label}
                        placeholder="Link Title"
                        class="w-1/3 px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
                      />
                      <input
                        type="text"
                        bind:value={link.url}
                        placeholder="/path or https://..."
                        class="flex-1 px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-300 font-mono text-xs focus:outline-none focus:border-orange-500"
                      />
                      <button
                        type="button"
                        on:click={() => removeLinkFromColumn(0, lIdx)}
                        class="p-1.5 text-slate-600 hover:text-rose-400 transition-colors"
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  {/each}

                  <button
                    type="button"
                    on:click={() => addLinkToColumn(0)}
                    class="py-2 px-4 rounded-xl border border-dashed border-slate-800 hover:border-orange-500 text-slate-400 hover:text-orange-400 text-xs font-semibold flex items-center justify-center gap-1.5 transition-colors mt-2"
                  >
                    <Plus size={13} />
                    <span>Add Horizontal Link</span>
                  </button>
                {/if}
              </div>
            </div>
          {/if}
        </div>

        <!-- Section 3: Right - Follow Us Platforms & Supported Payment Gateways -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
          <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
            <Share2 size={18} class="text-orange-400" />
            <span>Right Section: "Follow Us" Social Channels & Payment Badges</span>
          </h2>

          <!-- Part A: Social Follow Links -->
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-xs font-bold text-white uppercase tracking-wider">Follow Us Platforms</h3>
                <p class="text-[11px] text-slate-400 mt-0.5">Toggle platforms on/off and provide your official profile URL.</p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" bind:checked={footerConfig.show_socials} class="sr-only peer" />
                <div class="w-10 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-orange-600"></div>
                <span class="ml-2 text-xs text-slate-300 font-semibold">{footerConfig.show_socials ? 'Section Enabled' : 'Disabled'}</span>
              </label>
            </div>

            {#if footerConfig.show_socials}
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                {#each [
                  { id: 'github', name: 'GitHub', placeholder: 'https://github.com/your-username' },
                  { id: 'twitter', name: 'Twitter / X', placeholder: 'https://x.com/your-handle' },
                  { id: 'instagram', name: 'Instagram', placeholder: 'https://instagram.com/your-profile' },
                  { id: 'youtube', name: 'YouTube', placeholder: 'https://youtube.com/@your-channel' },
                  { id: 'facebook', name: 'Facebook', placeholder: 'https://facebook.com/your-page' },
                  { id: 'discord', name: 'Discord', placeholder: 'https://discord.gg/your-invite' },
                  { id: 'whatsapp', name: 'WhatsApp', placeholder: 'https://wa.me/your-phone-number' }
                ] as platform}
                  <div class="p-3 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
                    <div class="flex items-center justify-between">
                      <label class="flex items-center gap-2 font-bold text-white cursor-pointer">
                        <input
                          type="checkbox"
                          checked={footerConfig.enabled_socials.includes(platform.id)}
                          on:change={() => toggleSocial(platform.id)}
                          class="accent-orange-600 rounded"
                        />
                        <span>{platform.name}</span>
                      </label>
                      <span class="text-[10px] font-mono {footerConfig.enabled_socials.includes(platform.id) ? 'text-emerald-400' : 'text-slate-600'}">
                        {footerConfig.enabled_socials.includes(platform.id) ? 'ACTIVE' : 'OFF'}
                      </span>
                    </div>
                    {#if footerConfig.enabled_socials.includes(platform.id)}
                      <input
                        type="text"
                        bind:value={footerConfig.social_links[platform.id]}
                        placeholder={platform.placeholder}
                        class="w-full px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-200 font-mono text-xs focus:outline-none focus:border-orange-500"
                      />
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Part B: Accepted Payments Badges -->
          <div class="space-y-3 pt-4 border-t border-slate-800">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-xs font-bold text-white uppercase tracking-wider">Accepted Payment Badges</h3>
                <p class="text-[11px] text-slate-400 mt-0.5">Render verified official provider logos (Stripe, PayPal, Apple Pay, Google Pay, Amazon Pay) directly beneath Follow Us.</p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" bind:checked={footerConfig.show_payments} class="sr-only peer" />
                <div class="w-10 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-orange-600"></div>
                <span class="ml-2 text-xs text-slate-300 font-semibold">{footerConfig.show_payments ? 'Visible' : 'Hidden'}</span>
              </label>
            </div>
          </div>
        </div>

        <!-- Section 4: Full-Width Bottom Copyright Bar -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
          <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
            <ShieldCheck size={18} class="text-orange-400" />
            <span>Bottom Bar: Full-Width Copyright Line</span>
          </h2>

          <div class="space-y-4 text-xs">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <label class="p-3.5 rounded-xl border cursor-pointer transition-all {footerConfig.copyright_format === 'standard' ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400'}">
                <div class="flex items-center gap-2.5">
                  <input
                    type="radio"
                    name="copyright_format"
                    value="standard"
                    bind:group={footerConfig.copyright_format}
                    class="accent-orange-600"
                  />
                  <div>
                    <div class="font-bold text-white">Dynamic Storefront Copyright</div>
                    <div class="text-[11px] text-slate-400">© 2026 {storeName || 'RustCraft'}. All rights reserved.</div>
                  </div>
                </div>
              </label>

              <label class="p-3.5 rounded-xl border cursor-pointer transition-all {footerConfig.copyright_format === 'custom' ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950 border-slate-800 text-slate-400'}">
                <div class="flex items-center gap-2.5">
                  <input
                    type="radio"
                    name="copyright_format"
                    value="custom"
                    bind:group={footerConfig.copyright_format}
                    class="accent-orange-600"
                  />
                  <div>
                    <div class="font-bold text-white">Custom Copyright Text</div>
                    <div class="text-[11px] text-slate-400">Explicit custom entity line (e.g. Copyright © 2026 Example Store...)</div>
                  </div>
                </div>
              </label>
            </div>

            {#if footerConfig.copyright_format === 'custom'}
              <div>
                <label class="block text-slate-300 font-semibold mb-1">Custom Copyright Text</label>
                <input
                  type="text"
                  bind:value={footerConfig.custom_copyright}
                  placeholder="e.g. Copyright © 2026 Example Store. All rights reserved."
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
                />
              </div>
            {/if}
          </div>
        </div>

        <!-- Submit Button -->
        <div class="flex justify-end">
          <button
            type="submit"
            disabled={isSavingFooter}
            class="px-8 py-3 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all flex items-center gap-2 disabled:opacity-50"
          >
            <Save size={16} />
            <span>{isSavingFooter ? 'Saving Footer Architecture...' : 'Save Footer & Social Settings'}</span>
          </button>
        </div>
      </form>
    </div>

  <!-- TAB 7: Export & Backups -->
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

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Card 1: Store Configuration & Content Export -->
        <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-4 flex flex-col justify-between">
          <div class="space-y-3">
            <div class="w-12 h-12 rounded-2xl bg-orange-600/15 text-orange-400 border border-orange-500/20 flex items-center justify-center">
              <Download size={24} />
            </div>
            <h3 class="text-base font-bold text-white">Export Store Settings & Catalog (JSON)</h3>
            <p class="text-xs text-slate-400 leading-relaxed">
              Generates a single comprehensive JSON file with all store configuration parameters, active/inactive products, variants, categories, CMS policy markdown revisions, header & footer navigation structures, and shipping providers.
            </p>
            <div class="text-[11px] text-slate-500 space-y-1 pt-1 font-mono">
              <div>&bull; Store Settings & Legal Disclosures</div>
              <div>&bull; Product Catalog & Variant SKUs</div>
              <div>&bull; Legal Policy CMS Revisions (AGB, Privacy, Impressum)</div>
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

        <!-- Card 2: Media Library ZIP Export -->
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
