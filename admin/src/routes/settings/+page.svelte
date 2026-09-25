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
    Sparkles
  } from 'lucide-svelte';

  export let data;
  let settings = data.settings || {};
  let paymentConfigs = data.paymentConfigs || [];
  let shippingProviders = data.shippingProviders || [];

  $: activeTab = $page.url.searchParams.get('tab') || 'identity';
  function setTab(tab) {
    goto(`/settings?tab=${tab}`, { keepFocus: true, noScroll: true, replaceState: true });
  }

  // --- Store Identity Form State ---
  let storeName = settings.store_name || 'RustCraft Gear & Software';
  let storeSubtitle = settings.store_subtitle || 'Rust Powered • ACID Fast';
  let companyAddress = settings.company_address || 'Rustacean Way 42, 10115 Berlin, Germany';
  let supportEmail = settings.support_email || 'support@rustwebshop.local';
  let phone = settings.phone || '+49 (0) 30 123456-78';
  let vatId = settings.vat_id || 'DE314159265';
  let taxNotice = settings.tax_notice || 'Gemäß § 19 UStG wird keine Umsatzsteuer berechnet (Kleinunternehmerstatus). / Small business exemption applies according to §19 UStG.';
  let currency = settings.currency || 'EUR';
  let currencySymbol = settings.currency_symbol || '€';
  let taxRatePercent = settings.tax_rate_percent !== undefined ? settings.tax_rate_percent : 19.0;
  let logoUrl = settings.logo_url || '';

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
        store_subtitle: storeSubtitle,
        company_address: companyAddress,
        support_email: supportEmail,
        phone,
        vat_id: vatId,
        tax_notice: taxNotice,
        currency,
        currency_symbol: currencySymbol,
        tax_rate_percent: parseFloat(taxRatePercent) || 0,
        logo_url: logoUrl
      };

      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers,
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        settings = payload;
        identitySuccessNotice = 'Store Identity saved! Updated dynamically across PDF Invoices, Packing Slips, and Policy CMS pages.';
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

    <a
      href="/settings/media"
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800"
    >
      <Image size={15} />
      <span>Media Library</span>
    </a>
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
              <label class="block text-slate-300 font-semibold mb-1">Store / Business Name</label>
              <input
                type="text"
                bind:value={storeName}
                required
                placeholder="e.g. RustCraft Gear & Software"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
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

        <!-- Submit Button -->
        <div class="flex justify-end">
          <button
            type="submit"
            disabled={isSavingIdentity}
            class="px-8 py-3 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all flex items-center gap-2 disabled:opacity-50"
          >
            <Save size={16} />
            <span>{isSavingIdentity ? 'Saving Store Identity...' : 'Save Store Identity (All Pages)'}</span>
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

  <!-- TAB 4: Shipping & Delivery -->
  {:else if activeTab === 'shipping'}
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
            <Truck size={22} class="text-orange-500" />
            Shipping Carriers & Delivery Zones
          </h2>
          <p class="text-xs text-slate-400 mt-1">
            Configure logistics providers (DHL, UPS, FedEx), country zones, and weight-based rate pricing.
          </p>
        </div>

        <a
          href="/settings/shipping"
          class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-1.5 transition-all self-start sm:self-auto"
        >
          <span>Extended Shipping Editor</span>
          <ExternalLink size={13} />
        </a>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        {#each shippingProviders as prov}
          <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 space-y-3">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2.5">
                <div class="w-8 h-8 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-center font-bold text-orange-400 text-xs">
                  {prov.code.toUpperCase().slice(0, 3)}
                </div>
                <div>
                  <h4 class="font-bold text-white text-xs">{prov.name}</h4>
                  <span class="text-[10px] text-slate-500 font-mono">{prov.code}</span>
                </div>
              </div>
              <span class="px-2 py-0.5 rounded text-[10px] font-bold {prov.is_active ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-slate-800 text-slate-500'}">
                {prov.is_active ? 'Active' : 'Inactive'}
              </span>
            </div>

            <p class="text-[11px] font-mono text-slate-400 truncate">
              {prov.tracking_url_template || 'No tracking template'}
            </p>
          </div>
        {/each}
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
