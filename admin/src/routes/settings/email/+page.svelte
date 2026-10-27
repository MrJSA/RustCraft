<script>
  import { Mail, Send, ShieldCheck, CheckCircle2, AlertCircle, Lock, Server, UserCheck, RefreshCw, Box, Bell, Users, Check, X, FileText } from 'lucide-svelte';

  export let data;
  let settings = data.settings || {};
  let adminUsers = data.adminUsers || [];

  let emailModalUser = null;
  let quickEmailValue = '';
  let quickEmailError = '';
  let isSavingQuickEmail = false;

  function openEmailModal(user) {
    emailModalUser = user;
    quickEmailValue = user.email || '';
    quickEmailError = '';
  }

  async function handleSaveQuickEmail() {
    if (!emailModalUser) return;
    quickEmailError = '';
    isSavingQuickEmail = true;

    try {
      const res = await fetch(`/api/v1/admin/users/${emailModalUser.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: quickEmailValue.trim() })
      });

      const resText = await res.text().catch(() => '');
      let resData = {};
      try { resData = JSON.parse(resText); } catch { resData = { error: resText }; }

      if (!res.ok) {
        quickEmailError = resData.error || resData.message || 'Could not save email address.';
        return;
      }

      const updatedEmail = resData.email !== undefined ? resData.email : (quickEmailValue.trim() || null);
      adminUsers = adminUsers.map((u) => (u.id === emailModalUser.id ? { ...u, email: updatedEmail } : u));
      saveNotice = `Email saved for ${emailModalUser.username}!`;
      setTimeout(() => (saveNotice = ''), 3500);
      emailModalUser = null;
    } catch (e) {
      quickEmailError = e.message || 'Failed to save email.';
    } finally {
      isSavingQuickEmail = false;
    }
  }

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

  // Invoices sent with the payment confirmation
  let emailInvoicePdf = settings.email_invoice_pdf ?? true;
  let einvoiceEnabled = Boolean(settings.einvoice_enabled);
  const hasTaxId = Boolean((settings.vat_id || '').trim() || (settings.tax_number || '').trim());

  // BOM Low Stock Alerts Configuration
  let lowStockAlertsEnabled = settings.low_stock_alerts_enabled ?? true;
  let lowStockAlertRecipientsMode = settings.low_stock_alert_recipients_mode || 'stock_managers';
  let lowStockAlertCustomEmails = settings.low_stock_alert_custom_emails || '';
  let lowStockAlertSelectedUserIds = Array.isArray(settings.low_stock_alert_selected_user_ids)
    ? [...settings.low_stock_alert_selected_user_ids]
    : [];

  function toggleSelectedUser(userId) {
    if (lowStockAlertSelectedUserIds.includes(userId)) {
      lowStockAlertSelectedUserIds = lowStockAlertSelectedUserIds.filter(id => id !== userId);
    } else {
      lowStockAlertSelectedUserIds = [...lowStockAlertSelectedUserIds, userId];
    }
  }

  function hasStockAccess(user) {
    if (user.role === 'superadmin') return true;
    if (user.permissions && user.permissions.products) return true;
    return false;
  }

  let testRecipient = settings.support_email || '';
  let testEmailType = 'test';
  const emailTypeOptions = [
    { value: 'test', label: 'SMTP Connection Test' },
    { value: 'verification', label: 'Account Verification' },
    { value: 'order_created', label: 'Order Confirmation' },
    { value: 'payment_received', label: 'Payment Confirmed' },
    { value: 'order_shipped', label: 'Order Shipped & Tracking' },
    { value: 'back_in_stock', label: 'Back in Stock Alert' },
    { value: 'password_reset', label: 'Password Reset' },
    { value: 'low_stock_alert', label: 'BOM Low Stock Inventory Alert' },
    { value: 'backup_failed', label: 'Automatic Backup Failed' },
    { value: 'refund', label: 'Refund with Credit Note' },
    { value: 'newsletter_confirm', label: 'Newsletter: Confirm Subscription (Double Opt-in)' },
    { value: 'newsletter', label: 'Newsletter: Sample Issue' },
    { value: 'withdrawal_confirmation', label: 'Withdrawal: Confirmation to Customer' },
    { value: 'withdrawal_notification', label: 'Withdrawal: Notification to Shop' },
  ];
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
        // Only the email & low stock fields — a stale copy of other settings must never be resent
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
          require_email_verification: requireEmailVerification,
          email_invoice_pdf: emailInvoicePdf,
          einvoice_enabled: einvoiceEnabled,
          low_stock_alerts_enabled: lowStockAlertsEnabled,
          low_stock_alert_recipients_mode: lowStockAlertRecipientsMode,
          low_stock_alert_custom_emails: lowStockAlertCustomEmails,
          low_stock_alert_selected_user_ids: lowStockAlertSelectedUserIds
        })
      });

      if (res.ok) {
        saveNotice = 'Email addon settings & low-stock notification policies saved successfully!';
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
          email_type: testEmailType,
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

  <!-- Section 2a: Invoices attached to payment confirmations -->
  <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-5">
    <div class="flex items-center gap-2.5 text-white font-bold text-sm uppercase tracking-wider pb-3 border-b border-slate-800">
      <FileText size={18} class="text-emerald-400" />
      <span>Invoices</span>
    </div>

    <label class="flex items-start gap-3 cursor-pointer">
      <input type="checkbox" bind:checked={emailInvoicePdf} class="w-4 h-4 mt-0.5 accent-emerald-600" />
      <span class="text-xs">
        <span class="font-bold text-white block">Attach the invoice as PDF to the payment confirmation</span>
        <span class="text-slate-400">Customers receive their invoice automatically as soon as the payment is confirmed. You can open every invoice in the order details.</span>
      </span>
    </label>

    <label class="flex items-start gap-3 cursor-pointer">
      <input type="checkbox" bind:checked={einvoiceEnabled} class="w-4 h-4 mt-0.5 accent-emerald-600" />
      <span class="text-xs">
        <span class="font-bold text-white block">E-invoice: ZUGFeRD / Factur-X (EN 16931)</span>
        <span class="text-slate-400">
          The invoice PDF additionally contains the invoice as structured XML (EN 16931, the European e-invoicing standard). People see a normal PDF,
          accounting software and business customers can import it automatically. Recommended if you sell to businesses (B2B e-invoicing in Germany since 2025).
        </span>
      </span>
    </label>
    {#if einvoiceEnabled && !hasTaxId}
      <div class="p-3 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs">
        E-invoices need your VAT ID or tax number (Steuernummer). Enter one under Settings → Store Identity & Legal first.
      </div>
    {/if}
  </div>

  <!-- Section 2b: BOM Inventory & Low Stock Alerts -->
  <div class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-xl space-y-5">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-slate-800">
      <div class="flex items-center gap-2.5 text-white font-bold text-sm uppercase tracking-wider">
        <Box size={18} class="text-amber-400" />
        <span>BOM Inventory & Low-Stock Email Alerts</span>
      </div>
      <label class="relative inline-flex items-center cursor-pointer">
        <input type="checkbox" bind:checked={lowStockAlertsEnabled} class="sr-only peer" />
        <div class="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-amber-600"></div>
        <span class="ml-3 text-xs font-semibold {lowStockAlertsEnabled ? 'text-amber-400' : 'text-slate-500'}">
          {lowStockAlertsEnabled ? 'Alerts Enabled' : 'Alerts Disabled'}
        </span>
      </label>
    </div>

    <p class="text-xs text-slate-400 leading-relaxed">
      Automatically dispatch email notifications when any BOM part inventory falls to or below its minimum threshold.
      Stock quantities are strictly kept in sync across all parts sharing the same SKU.
    </p>

    {#if lowStockAlertsEnabled}
      <div class="space-y-4 pt-1">
        <!-- Recipient Target Selection -->
        <div>
          <label class="block text-slate-300 font-semibold text-xs mb-2 flex items-center gap-1.5">
            <Users size={14} class="text-amber-400" />
            <span>Alert Recipients Mode</span>
          </label>
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <button
              type="button"
              on:click={() => lowStockAlertRecipientsMode = 'stock_managers'}
              class="p-3.5 rounded-xl text-left border transition-all text-xs {lowStockAlertRecipientsMode === 'stock_managers' ? 'bg-amber-500/10 border-amber-500/40 text-amber-300 ring-1 ring-amber-500/30' : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-white hover:bg-slate-850'}"
            >
              <div class="font-bold text-white mb-1 flex items-center justify-between">
                <span>Stock Managers</span>
                {#if lowStockAlertRecipientsMode === 'stock_managers'}
                  <Check size={14} class="text-amber-400" />
                {/if}
              </div>
              <p class="text-[11px] text-slate-400 leading-snug">
                All administrators & users with access to Products & Stock.
              </p>
            </button>

            <button
              type="button"
              on:click={() => lowStockAlertRecipientsMode = 'selected_users'}
              class="p-3.5 rounded-xl text-left border transition-all text-xs {lowStockAlertRecipientsMode === 'selected_users' ? 'bg-amber-500/10 border-amber-500/40 text-amber-300 ring-1 ring-amber-500/30' : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-white hover:bg-slate-850'}"
            >
              <div class="font-bold text-white mb-1 flex items-center justify-between">
                <span>Specific Selected Users</span>
                {#if lowStockAlertRecipientsMode === 'selected_users'}
                  <Check size={14} class="text-amber-400" />
                {/if}
              </div>
              <p class="text-[11px] text-slate-400 leading-snug">
                Only the designated admin accounts checked in the list below.
              </p>
            </button>

            <button
              type="button"
              on:click={() => lowStockAlertRecipientsMode = 'both'}
              class="p-3.5 rounded-xl text-left border transition-all text-xs {lowStockAlertRecipientsMode === 'both' ? 'bg-amber-500/10 border-amber-500/40 text-amber-300 ring-1 ring-amber-500/30' : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-white hover:bg-slate-850'}"
            >
              <div class="font-bold text-white mb-1 flex items-center justify-between">
                <span>Both (Managers & Selected)</span>
                {#if lowStockAlertRecipientsMode === 'both'}
                  <Check size={14} class="text-amber-400" />
                {/if}
              </div>
              <p class="text-[11px] text-slate-400 leading-snug">
                Combines stock managers with specifically selected accounts.
              </p>
            </button>
          </div>
        </div>

        <!-- User Selection Table -->
        {#if lowStockAlertRecipientsMode !== 'stock_managers'}
          <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3">
            <span class="text-xs font-bold text-slate-200 block">Select Specific Accounts to Receive Alerts:</span>
            {#if adminUsers.length === 0}
              <p class="text-xs text-slate-500">No admin accounts found or accounts loading.</p>
            {:else}
              <div class="space-y-2 max-h-56 overflow-y-auto pr-1">
                {#each adminUsers as user}
                  {@const isChecked = lowStockAlertSelectedUserIds.includes(user.id)}
                  {@const hasStock = hasStockAccess(user)}
                  <div class="flex items-center justify-between p-2.5 rounded-xl border transition-colors {isChecked ? 'bg-amber-500/10 border-amber-500/30 text-white' : 'bg-slate-900 border-slate-850 text-slate-300 hover:bg-slate-850'}">
                    <label class="flex items-center gap-3 cursor-pointer flex-1">
                      <input
                        type="checkbox"
                        checked={isChecked}
                        on:change={() => toggleSelectedUser(user.id)}
                        class="accent-amber-500 w-4 h-4 rounded cursor-pointer"
                      />
                      <div>
                        <div class="flex items-center gap-2">
                          <span class="font-bold text-xs">{user.username}</span>
                          <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-semibold">{user.role}</span>
                          {#if hasStock}
                            <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">Stock Access</span>
                          {/if}
                        </div>
                        <span class="text-[11px] {user.email ? 'text-slate-400' : 'text-amber-400/90 font-semibold'} font-mono">
                          {user.email || '(No email set on account)'}
                        </span>
                      </div>
                    </label>

                    <button
                      type="button"
                      on:click|stopPropagation={() => openEmailModal(user)}
                      class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white text-[11px] font-semibold border border-slate-700 flex items-center gap-1.5 transition-all ml-2 flex-shrink-0"
                      title="Set or update email for this account"
                    >
                      <Mail size={12} class="text-orange-400" />
                      <span>{user.email ? 'Change Email' : 'Set Email'}</span>
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}

        <!-- Custom Additional Emails Input -->
        <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-2">
          <label class="block text-slate-300 font-semibold text-xs">
            Additional Email Addresses (Optional)
          </label>
          <input
            type="text"
            bind:value={lowStockAlertCustomEmails}
            placeholder="warehouse@company.com, alerts@supplier.de"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-900 border border-slate-800 text-white font-mono text-xs focus:outline-none focus:border-amber-500"
          />
          <p class="text-[11px] text-slate-400">
            Comma- or newline-separated list of extra email addresses that will receive alerts directly, without needing an admin user account.
          </p>
        </div>
      </div>
    {/if}
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
      <select
        bind:value={testEmailType}
        class="px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500 font-medium"
        title="Select email template to test"
      >
        {#each emailTypeOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
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
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3 text-xs">
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
        <strong class="text-orange-400 block mb-1">1. Order Confirmation</strong>
        <p class="text-slate-400 text-[11px]">Sent immediately upon order creation with items & total breakdown.</p>
      </div>
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
        <strong class="text-emerald-400 block mb-1">2. Payment Receipt</strong>
        <p class="text-slate-400 text-[11px]">Sent once the payment is confirmed — with the invoice PDF attached.</p>
      </div>
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
        <strong class="text-sky-400 block mb-1">3. Shipment & Tracking</strong>
        <p class="text-slate-400 text-[11px]">Sent when admin changes order status to Shipped with tracking URL.</p>
      </div>
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
        <strong class="text-purple-400 block mb-1">4. Back-in-Stock Alert</strong>
        <p class="text-slate-400 text-[11px]">Sent automatically to waitlist customers when inventory is replenished.</p>
      </div>
      <div class="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
        <strong class="text-amber-400 block mb-1">5. Low Stock Alert</strong>
        <p class="text-slate-400 text-[11px]">Sent automatically to managers when BOM part stock falls below threshold.</p>
      </div>
    </div>
  </div>
</div>

{#if emailModalUser}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-sm rounded-2xl bg-slate-900 border border-slate-800 shadow-2xl p-5 space-y-4">
      <div class="flex items-center justify-between border-b border-slate-800 pb-2.5">
        <h3 class="text-sm font-bold text-white flex items-center gap-2">
          <Mail size={16} class="text-orange-400" />
          <span>Save Email for {emailModalUser.username}</span>
        </h3>
        <button type="button" on:click={() => (emailModalUser = null)} class="text-slate-400 hover:text-white">
          <X size={15} />
        </button>
      </div>

      {#if quickEmailError}
        <div class="p-2.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold">
          {quickEmailError}
        </div>
      {/if}

      <div class="space-y-3 text-xs">
        <div>
          <label class="block font-semibold text-slate-300 mb-1">Email Address</label>
          <input
            type="email"
            bind:value={quickEmailValue}
            placeholder="e.g. user@yourdomain.com"
            class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 font-mono"
          />
          <p class="text-[10px] text-slate-500 mt-1">This user will receive low-stock notifications and alert emails at this address.</p>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 pt-2 border-t border-slate-800">
        <button
          type="button"
          on:click={() => (emailModalUser = null)}
          class="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold"
        >
          Cancel
        </button>
        <button
          type="button"
          disabled={isSavingQuickEmail}
          on:click={handleSaveQuickEmail}
          class="px-4 py-1.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold shadow-md shadow-orange-600/30 transition-all disabled:opacity-50"
        >
          {isSavingQuickEmail ? 'Saving...' : 'Save Email'}
        </button>
      </div>
    </div>
  </div>
{/if}
