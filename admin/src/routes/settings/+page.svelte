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
    Sliders,
    Box,
    Bell,
    Users,
    X
  } from 'lucide-svelte';
  import ShippingManager from '$lib/components/ShippingManager.svelte';
  import MediaManager from '$lib/components/MediaManager.svelte';
  import AdminUsersManager from '$lib/components/AdminUsersManager.svelte';
  import PaymentsManager from '$lib/components/PaymentsManager.svelte';
  import SystemManager from '$lib/components/SystemManager.svelte';
  import CloudBackupManager from '$lib/components/CloudBackupManager.svelte';
  import RestoreBackupManager from '$lib/components/RestoreBackupManager.svelte';
  import ShopFeaturesManager from '$lib/components/ShopFeaturesManager.svelte';

  export let data;
  let settings = data.settings || {};
  let paymentConfigs = data.paymentConfigs || [];
  let shippingProviders = data.shippingProviders || [];
  let adminUsers = data.adminUsers || [];

  $: activeTab = $page.url.searchParams.get('tab') || 'identity';
  function setTab(tab) {
    goto(`/settings?tab=${tab}`, { keepFocus: true, noScroll: true, replaceState: true });
  }

  // --- Store Identity & Legal Form State ---
  // Empty fields stay empty: example values only appear as placeholders, never get saved
  let storeName = settings.store_name || '';
  let legalName = settings.legal_name || '';
  let storeOwner = settings.store_owner || '';
  let storeSubtitle = settings.store_subtitle ?? '';
  let addressStreet = settings.address_street || '';
  let addressHouseNumber = settings.address_house_number || '';
  let addressExtra = settings.address_extra || '';
  let addressPostalCode = settings.address_postal_code || '';
  let addressCity = settings.address_city || '';
  let addressCountry = settings.address_country || '';
  let supportEmail = settings.support_email || '';
  let phone = settings.phone || '';
  let vatId = settings.vat_id || '';
  let taxNumber = settings.tax_number || '';
  let taxNotice = settings.tax_notice || 'Value added tax is not collected, as small businesses according to §19 (1) UStG.';
  let commercialRegister = settings.commercial_register || '';
  let disputeResolutionNotice = settings.dispute_resolution_notice || '';
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

  let stockDisplayTemplate = settings.stock_display_template || 'In Stock ({stock} units available in central warehouse)';

  let isSavingIdentity = false;
  let identitySuccessNotice = '';
  let identityErrorNotice = '';
  let showLogoPicker = false;

  function handleTaxModeChange(mode) {
    taxMode = mode;
    if (mode === 'kleingewerbe') {
      taxRatePercent = 0.0;
      taxNotice = 'According to § 19 UStG, no value-added tax is charged (small business regulation).';
    } else if (mode === 'included') {
      taxNotice = 'All prices include statutory value-added tax (VAT).';
    } else if (mode === 'excluded') {
      taxNotice = 'All prices exclude statutory value-added tax (VAT).';
    }
  }

  async function handleSaveIdentity() {
    if (data.settingsLoadFailed) {
      identityErrorNotice = 'The current settings could not be loaded. Please reload the page before saving.';
      return;
    }
    isSavingIdentity = true;
    identitySuccessNotice = '';
    identityErrorNotice = '';

    const token = localStorage.getItem('admin_token');
    const headers = {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {})
    };

    try {
      // Only this form's own fields — never resend a stale copy of other settings
      const payload = {
        store_name: storeName,
        legal_name: legalName,
        store_owner: storeOwner,
        store_subtitle: storeSubtitle,
        address_street: addressStreet,
        address_house_number: addressHouseNumber,
        address_extra: addressExtra,
        address_postal_code: addressPostalCode,
        address_city: addressCity,
        address_country: addressCountry,
        support_email: supportEmail,
        phone,
        vat_id: vatId,
        tax_number: taxNumber,
        tax_notice: taxNotice,
        tax_mode: taxMode,
        commercial_register: commercialRegister,
        dispute_resolution_notice: disputeResolutionNotice,
        currency,
        currency_symbol: currencySymbol,
        tax_rate_percent: parseFloat(taxRatePercent) || 0,
        logo_url: logoUrl,
        order_prefix_enabled: orderPrefixEnabled,
        order_prefix: (orderPrefix || '').trim().toUpperCase().slice(0, 7),
        order_date_enabled: orderDateEnabled,
        stock_display_template: stockDisplayTemplate,
        deployment_mode: deploymentMode,
        debug_mode: debugMode
      };

      const res = await fetch('/api/v1/admin/settings/system', {
        method: 'PUT',
        headers,
        body: JSON.stringify(payload)
      });

      if (res.ok) {
        settings = { ...settings, ...payload };
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
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: text
      });

      const bodyText = await res.text();
      let resData = {};
      try { resData = JSON.parse(bodyText); } catch (_) {}
      if (res.ok) {
        const r = resData.restored || {};
        exportSuccessNotice = `Store restored successfully! Restored: ${r.products || 0} products, ${r.variants || 0} variants, ${r.categories || 0} categories, ${r.pages || 0} pages, ${r.menu_items || 0} menu items, ${r.shipping_zones || 0} shipping zones, ${r.coupons || 0} coupons. Reloading...`;
        setTimeout(() => window.location.reload(), 2500);
      } else {
        exportErrorNotice = resData.error || bodyText || 'Failed to restore store data.';
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

  // --- Email & Auth State ---
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

  let testRecipient = supportEmail || '';
  let isSendingTest = false;
  let testResult = null;
  let isSavingEmail = false;
  let emailNotice = '';
  let emailError = '';

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
      emailNotice = `Email saved for ${emailModalUser.username}!`;
      setTimeout(() => (emailNotice = ''), 3500);
      emailModalUser = null;
    } catch (e) {
      quickEmailError = e.message || 'Failed to save email.';
    } finally {
      isSavingQuickEmail = false;
    }
  }

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
          low_stock_alerts_enabled: lowStockAlertsEnabled,
          low_stock_alert_recipients_mode: lowStockAlertRecipientsMode,
          low_stock_alert_custom_emails: lowStockAlertCustomEmails,
          low_stock_alert_selected_user_ids: lowStockAlertSelectedUserIds
        })
      });

      if (res.ok) {
        emailNotice = 'Email addon settings & low-stock notification policies saved successfully!';
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
  ];

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

      const data = await res.json();
      if (res.ok) {
        testResult = { success: true, message: data.message || `Test email dispatched successfully to ${testRecipient}!`, warning: data.warning };
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
      on:click={() => setTab('users')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'users' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <UserCheck size={15} />
      <span>Admin Users & Access</span>
      <span class="text-[10px] px-2 py-0.5 rounded-full {activeTab === 'users' ? 'bg-white/20 text-white' : 'bg-slate-800 text-slate-400'}">{adminUsers.length}</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('export')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'export' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Download size={15} />
      <span>Export & Backups</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('system')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'system' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Server size={15} />
      <span>System & Updates</span>
    </button>

    <button
      type="button"
      on:click={() => setTab('features')}
      class="px-4 py-2.5 rounded-xl text-xs font-bold transition-all flex items-center gap-2 {activeTab === 'features' ? 'bg-orange-600 text-white shadow-lg shadow-orange-600/25 ring-1 ring-orange-500' : 'bg-slate-900 text-slate-400 hover:text-white hover:bg-slate-800 border border-slate-800'}"
    >
      <Sparkles size={15} />
      <span>Shop Features & Legal</span>
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
              <p class="text-[10px] text-slate-500 mt-1">Authorized representative (§ 5 DDG "Vertreten durch").</p>
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

            <div class="sm:col-span-2 grid grid-cols-1 sm:grid-cols-6 gap-4">
              <div class="sm:col-span-6">
                <p class="text-slate-300 font-semibold">Registered Business Address</p>
                <p class="text-[10px] text-slate-500 mt-0.5">Used on invoices, packing slips, emails, the legal notice and all legal pages.</p>
              </div>
            <div class="sm:col-span-4">
              <label class="block text-slate-300 font-semibold mb-1">Street</label>
              <input
                type="text"
                bind:value={addressStreet}
                required
                placeholder="e.g. Musterstraße"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>
            <div class="sm:col-span-2">
              <label class="block text-slate-300 font-semibold mb-1">House Number</label>
              <input
                type="text"
                bind:value={addressHouseNumber}
                placeholder="e.g. 1a"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>
            <div class="sm:col-span-6">
              <label class="block text-slate-300 font-semibold mb-1">Address Addition (optional)</label>
              <input
                type="text"
                bind:value={addressExtra}
                placeholder="e.g. Building B, 2nd floor"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>
            <div class="sm:col-span-2">
              <label class="block text-slate-300 font-semibold mb-1">Postal Code</label>
              <input
                type="text"
                bind:value={addressPostalCode}
                required
                placeholder="e.g. 12345"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>
            <div class="sm:col-span-4">
              <label class="block text-slate-300 font-semibold mb-1">City</label>
              <input
                type="text"
                bind:value={addressCity}
                required
                placeholder="e.g. Musterstadt"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>
            <div class="sm:col-span-6">
              <label class="block text-slate-300 font-semibold mb-1">Country</label>
              <input
                type="text"
                bind:value={addressCountry}
                placeholder="e.g. Germany"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
              />
            </div>
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
                <label class="block font-semibold mb-1 {taxMode === 'kleingewerbe' ? 'text-slate-500 opacity-50' : 'text-slate-300'}">
                  Standard Store VAT Rate (%)
                  {#if taxMode === 'kleingewerbe'}
                    <span class="text-[10px] text-amber-400 font-normal ml-1">(0% § 19 UStG)</span>
                  {/if}
                </label>
                <div class="relative">
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    max="100"
                    bind:value={taxRatePercent}
                    disabled={taxMode === 'kleingewerbe'}
                    class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500 disabled:opacity-40 disabled:bg-slate-900/60 disabled:text-slate-500 disabled:border-slate-800/60 disabled:cursor-not-allowed"
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
                <label class="block text-slate-300 font-semibold mb-1">Tax number (Steuernummer)</label>
                <input
                  type="text"
                  bind:value={taxNumber}
                  placeholder="e.g. 12/345/67890"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
                />
                <p class="text-[10px] text-slate-500 mt-1">Shown on invoices. Required on invoices without a VAT ID (e.g. small businesses) and for e-invoices.</p>
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
                  placeholder="According to § 19 UStG, no value-added tax is charged (small business regulation)."
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs leading-relaxed focus:outline-none focus:border-orange-500"
                ></textarea>
                <p class="text-[10px] text-slate-500 mt-1">
                  Appears automatically in the footer of all PDF Invoices and Packing Slips, and replaces {"{{TAX_NOTICE}}"} in Policy pages.
                </p>
              </div>

              <!-- Stock Availability Text Template -->
              <div class="sm:col-span-4 p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
                <label class="block text-slate-300 font-semibold text-xs">
                  Physical Product Stock Availability Message Template
                </label>
                <input
                  type="text"
                  bind:value={stockDisplayTemplate}
                  placeholder="In Stock (25 units available in central warehouse)"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-900 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500 font-mono"
                />
                <p class="text-[11px] text-slate-400">
                  Custom text displayed on storefront product pages for available physical stock. Use <code class="text-orange-400 font-mono">&#123;stock&#125;</code> as the live quantity placeholder (e.g. <em>&#123;stock&#125; items in stock - ready to ship</em>).
                </p>
              </div>

              <div class="sm:col-span-4 pt-2 border-t border-slate-800 space-y-3">
                <div class="flex items-center gap-2 text-white font-semibold">
                  <Scale size={15} class="text-orange-400" />
                  <span>Consumer Dispute Resolution (§ 36 VSBG)</span>
                </div>
                <p class="text-[11px] text-slate-400">The EU online dispute resolution (ODR) platform was shut down on 20 July 2025, so the shop no longer links to it. State whether you take part in consumer arbitration proceedings.</p>

                <div>
                  <label class="block text-slate-300 font-semibold mb-1">Dispute Resolution Statement / Arbitration Notice</label>
                  <textarea
                    bind:value={disputeResolutionNotice}
                    rows="2"
                    placeholder="We are neither willing nor obliged to participate in dispute resolution proceedings before a consumer arbitration board (§ 36 VSBG)."
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
    <PaymentsManager configs={paymentConfigs} />

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
          {#if !smtpEnabled}
            <p class="text-[11px] text-amber-300 bg-amber-500/10 border border-amber-500/20 rounded-xl px-3 py-2">
              Email sending is switched off: the shop sends no emails at all (account verification, password reset, order confirmations). Turn it on and click Save.
            </p>
          {/if}

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
                placeholder="Leave empty to keep the saved password"
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

        <!-- BOM Inventory & Low Stock Alerts -->
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

        <!-- Submit & Test -->
        <div class="flex flex-col sm:flex-row items-center justify-between gap-4">
          <!-- Quick Send Test Email -->
          <div class="flex flex-wrap items-center gap-2 w-full sm:w-auto">
            <select
              bind:value={testEmailType}
              class="px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500 font-medium"
              title="Select email type to test"
            >
              {#each emailTypeOptions as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
            <input
              type="email"
              bind:value={testRecipient}
              placeholder="Test recipient email..."
              class="px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs w-52 focus:outline-none focus:border-orange-500 font-mono"
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
          {#if testResult.warning}
            <div class="p-3.5 rounded-xl text-xs font-semibold bg-amber-500/10 border border-amber-500/30 text-amber-300 flex items-start gap-2">
              <AlertCircle size={16} class="flex-shrink-0" />
              <span>{testResult.warning}</span>
            </div>
          {/if}
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
      <CloudBackupManager />
      <RestoreBackupManager />

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
  {:else if activeTab === 'users'}
    <AdminUsersManager initialUsers={adminUsers} />

  <!-- System, updates & domains -->
  {:else if activeTab === 'system'}
    <SystemManager />

  <!-- Optional features & legal requirements -->
  {:else if activeTab === 'features'}
    <ShopFeaturesManager {settings} />
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
