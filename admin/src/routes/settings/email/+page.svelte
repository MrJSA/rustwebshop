<script>
  import { Mail, Send, ShieldCheck, CheckCircle2, AlertCircle, Lock, Server, UserCheck, RefreshCw } from 'lucide-svelte';

  export let data;
  let settings = data.settings || {};

  let smtpHost = settings.smtp_host || '';
  let smtpPort = settings.smtp_port || 587;
  let smtpUsername = settings.smtp_username || '';
  let smtpPassword = settings.smtp_password || '';
  let smtpEncryption = settings.smtp_encryption || 'starttls';
  let smtpFromEmail = settings.smtp_from_email || '';
  let smtpFromName = settings.smtp_from_name || settings.store_name || '';
  let smtpEnabled = Boolean(settings.smtp_enabled);

  let requireRegisteredCheckout = Boolean(settings.require_registered_checkout);
  let requireEmailVerification = Boolean(settings.require_email_verification);

  let testRecipient = settings.support_email || '';
  let isSendingTest = false;
  let testResult = null; // { success: boolean, message: string }
  let isSavingSettings = false;
  let saveNotice = '';
  let saveError = '';

  async function handleSaveSettings() {
    isSavingSettings = true;
    saveNotice = '';
    saveError = '';

    const token = localStorage.getItem('admin_token');

    try {
      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        // Only the email fields — a stale copy of other settings must never be resent
        body: JSON.stringify({
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
        saveNotice = 'Email addon settings & registration policies saved successfully!';
        setTimeout(() => saveNotice = '', 3500);
      } else {
        const err = await res.json();
        saveError = err.error || 'Failed to save settings.';
      }
    } catch (e) {
      saveError = 'Failed to connect to backend server.';
    } finally {
      isSavingSettings = false;
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
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        // Test the values currently in the form (an empty password keeps the saved one)
        body: JSON.stringify({
          recipient_email: testRecipient,
          smtp: {
            host: smtpHost,
            port: parseInt(smtpPort) || 587,
            username: smtpUsername,
            password: smtpPassword,
            encryption: smtpEncryption,
            from_email: smtpFromEmail,
            from_name: smtpFromName
          }
        })
      });

      const resData = await res.json();
      if (res.ok) {
        testResult = {
          success: true,
          message: resData.message || `Test email dispatched successfully to ${testRecipient}!`,
          warning: resData.warning
        };
      } else {
        testResult = {
          success: false,
          message: resData.error || 'SMTP server rejected test dispatch. Check host, port, or credentials.'
        };
      }
    } catch (e) {
      testResult = {
        success: false,
        message: 'Network error communicating with email server.'
      };
    } finally {
      isSendingTest = false;
    }
  }
</script>

<svelte:head>
  <title>Email Addon & Auth Policies | RustCraft Admin</title>
</svelte:head>

<div class="space-y-6 max-w-5xl mx-auto">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Mail size={24} class="text-orange-500" />
        <span>Email Addon & Customer Verification Policies</span>
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Configure SMTP connection for automatic order confirmations, shipping updates, back-in-stock alerts, and email verification.
      </p>
    </div>

    <button
      on:click={handleSaveSettings}
      disabled={isSavingSettings}
      class="px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all disabled:opacity-50 self-start sm:self-auto"
    >
      <ShieldCheck size={16} />
      <span>{isSavingSettings ? 'Saving...' : 'Save Configuration'}</span>
    </button>
  </div>

  {#if saveNotice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <CheckCircle2 size={16} />
      <span>{saveNotice}</span>
    </div>
  {/if}

  {#if saveError}
    <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
      <AlertCircle size={16} />
      <span>{saveError}</span>
    </div>
  {/if}

  <!-- Section 1: Customer Registration & Checkout Policy -->
  <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-5">
    <div class="flex items-center gap-2 text-white font-bold text-sm uppercase tracking-wider pb-3 border-b border-slate-800">
      <UserCheck size={18} class="text-orange-400" />
      <span>Registration & Checkout Security Policies</span>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
      <!-- Policy Toggle 1 -->
      <div class="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80 flex items-start justify-between gap-4">
        <div>
          <h4 class="text-sm font-bold text-white">Require Registered Customer for Checkout</h4>
          <p class="text-xs text-slate-400 mt-1 leading-relaxed">
            When enabled, guest checkouts are disabled. Customers must create an account or sign in to complete a purchase.
          </p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer flex-shrink-0 mt-1">
          <input type="checkbox" bind:checked={requireRegisteredCheckout} class="sr-only peer" />
          <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-600"></div>
        </label>
      </div>

      <!-- Policy Toggle 2 -->
      <div class="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80 flex items-start justify-between gap-4">
        <div>
          <h4 class="text-sm font-bold text-white">Mandatory Customer Email Verification</h4>
          <p class="text-xs text-slate-400 mt-1 leading-relaxed">
            When enabled, new registrants receive an activation email with a verification link and must verify before placing orders.
          </p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer flex-shrink-0 mt-1">
          <input type="checkbox" bind:checked={requireEmailVerification} class="sr-only peer" />
          <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-600"></div>
        </label>
      </div>
    </div>
  </div>

  <!-- Section 2: SMTP Connection Settings -->
  <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
    <div class="flex items-center justify-between pb-3 border-b border-slate-800">
      <div class="flex items-center gap-2 text-white font-bold text-sm uppercase tracking-wider">
        <Server size={18} class="text-orange-400" />
        <span>SMTP Email Server Connection</span>
      </div>

      <label class="inline-flex items-center gap-2 cursor-pointer">
        <span class="text-xs font-bold text-slate-300">Enable SMTP Dispatch</span>
        <input type="checkbox" bind:checked={smtpEnabled} class="sr-only peer" />
        <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-600"></div>
      </label>
    </div>

    {#if !smtpEnabled}
      <div class="p-3.5 rounded-2xl bg-amber-500/10 border border-amber-500/30 text-amber-300 text-xs">
        ℹ️ SMTP dispatch is currently disabled. Emails will be simulated and logged in backend container logs without contacting an external mail server.
      </div>
    {/if}

    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs">
      <div class="md:col-span-2">
        <label class="block text-slate-300 font-semibold mb-1">SMTP Server Host</label>
        <input
          type="text"
          bind:value={smtpHost}
          placeholder="e.g. smtp.sendgrid.net, smtp.mailgun.org, smtp.gmail.com"
          class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500 font-mono"
        />
      </div>

      <div>
        <label class="block text-slate-300 font-semibold mb-1">Port</label>
        <input
          type="number"
          bind:value={smtpPort}
          placeholder="587"
          class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
        />
      </div>

      <div>
        <label class="block text-slate-300 font-semibold mb-1">Encryption Protocol</label>
        <select
          bind:value={smtpEncryption}
          class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
        >
          <option value="starttls">STARTTLS (Port 587 - Recommended)</option>
          <option value="tls">Direct TLS / SSL (Port 465)</option>
          <option value="none">Plain / None (Port 25 or 2525)</option>
        </select>
      </div>

      <div>
        <label class="block text-slate-300 font-semibold mb-1">SMTP Username</label>
        <input
          type="text"
          bind:value={smtpUsername}
          placeholder="apikey or username@domain.com"
          class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
        />
      </div>

      <div>
        <label class="block text-slate-300 font-semibold mb-1">SMTP Password / API Key</label>
        <input
          type="password"
          bind:value={smtpPassword}
          placeholder="Leave empty to keep the saved password"
          class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
        />
      </div>

      <div class="md:col-span-2">
        <label class="block text-slate-300 font-semibold mb-1">Sender Email ("From" Address)</label>
        <input
          type="email"
          bind:value={smtpFromEmail}
          placeholder="orders@yourdomain.com"
          class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500 font-mono"
        />
      </div>

      <div>
        <label class="block text-slate-300 font-semibold mb-1">Sender Display Name</label>
        <input
          type="text"
          bind:value={smtpFromName}
          placeholder="RustCraft Gear"
          class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 focus:outline-none focus:border-orange-500"
        />
      </div>
    </div>
  </div>

  <!-- Section 3: Test Email Dispatcher -->
  <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-4">
    <div class="flex items-center gap-2 text-white font-bold text-sm uppercase tracking-wider pb-3 border-b border-slate-800">
      <Send size={18} class="text-orange-400" />
      <span>Test Connection & Disptach Verification</span>
    </div>

    <p class="text-xs text-slate-400">
      Test your SMTP connection by sending a live test email directly to your inbox.
    </p>

    <div class="flex flex-col sm:flex-row gap-3">
      <input
        type="email"
        bind:value={testRecipient}
        placeholder="recipient@example.com"
        class="flex-1 px-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs placeholder-slate-500 focus:outline-none focus:border-orange-500 font-mono"
      />
      <button
        type="button"
        on:click={handleSendTestEmail}
        disabled={isSendingTest}
        class="px-5 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs flex items-center justify-center gap-2 transition-colors border border-slate-700 disabled:opacity-50"
      >
        <Send size={14} class="text-orange-400" />
        <span>{isSendingTest ? 'Sending...' : 'Send Test Email'}</span>
      </button>
    </div>

    {#if testResult}
      <div class="p-3.5 rounded-xl text-xs font-semibold flex items-center gap-2 {testResult.success ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-400 border border-rose-500/20'}">
        {#if testResult.success}
          <CheckCircle2 size={16} />
        {:else}
          <AlertCircle size={16} />
        {/if}
        <span>{testResult.message}</span>
      </div>
      {#if testResult.warning}
        <div class="p-3.5 rounded-xl text-xs font-semibold bg-amber-500/10 border border-amber-500/30 text-amber-300 flex items-start gap-2">
          <AlertCircle size={16} class="flex-shrink-0" />
          <span>{testResult.warning}</span>
        </div>
      {/if}
    {/if}
  </div>

  <!-- Automated Notifications Inventory -->
  <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-3">
    <h3 class="text-sm font-bold text-white uppercase tracking-wider mb-2">Automated Notifications Active</h3>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 text-xs">
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
        <strong class="text-orange-400 block mb-1">1. Order Confirmation</strong>
        <p class="text-slate-400 text-[11px]">Sent immediately upon order creation with items & total breakdown.</p>
      </div>
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
        <strong class="text-emerald-400 block mb-1">2. Payment Receipt</strong>
        <p class="text-slate-400 text-[11px]">Sent once Stripe/PayPal or gateway marks order as authorized or paid.</p>
      </div>
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
        <strong class="text-sky-400 block mb-1">3. Shipment & Tracking</strong>
        <p class="text-slate-400 text-[11px]">Sent when admin changes order status to Shipped with tracking URL.</p>
      </div>
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
        <strong class="text-purple-400 block mb-1">4. Back-in-Stock Alert</strong>
        <p class="text-slate-400 text-[11px]">Sent automatically to waitlist customers when inventory is replenished.</p>
      </div>
    </div>
  </div>
</div>
