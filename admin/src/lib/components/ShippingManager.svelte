<script>
  import { onMount } from 'svelte';
  import CountryPicker from '$lib/components/CountryPicker.svelte';
  import { countryName } from '$lib/countries.js';
  import { 
    Truck, Plus, Trash2, Globe, Box, Save, Check, Edit3, X, 
    ChevronRight, Layers, AlertCircle, CheckCircle2, ExternalLink, Copy
  } from 'lucide-svelte';

  export let initialProviders = [];

  let providers = Array.isArray(initialProviders) ? initialProviders : [];
  $: if (Array.isArray(initialProviders) && initialProviders.length > 0 && providers.length === 0) {
    providers = initialProviders;
  }
  let loading = false;
  let notice = '';
  let errorMsg = '';
  let modalError = '';
  let isSubmitting = false;

  // Modals state
  let isAddProviderOpen = false;
  let isEditProviderOpen = false;
  let isAddZoneOpen = false;
  let isEditZoneOpen = false;
  let isAddRateOpen = false;
  let isEditRateOpen = false;

  // Selected entities for editing/adding
  let selectedProviderId = null;
  let editingProvider = null;
  let selectedZoneId = null;
  let editingZone = null;
  let editingRate = null;

  // Provider Form State
  let providerName = '';
  let providerCode = '';
  let trackingUrlTemplate = 'https://www.dhl.com/track?id={tracking_number}';
  let providerIsActive = true;
  let providerSortOrder = 0;

  // Zone Form State
  let zoneName = '';
  let zoneCodes = ['DE'];
  let zoneIsDefault = false;

  // Rate Form State
  let rateName = 'Standard Paket (bis 5kg)';
  let packageType = 'standard';
  let priceEuros = 5.99;
  let deliveryDays = '2-4 business days';
  let minWeightG = 0;
  let maxWeightG = 5000;
  // Maximum package size in cm (empty = no size limit)
  let maxLengthCm = '';
  let maxWidthCm = '';
  let maxHeightCm = '';
  const toMm = (cm) => (cm === '' || cm === null || isNaN(parseFloat(cm)) ? 0 : Math.round(parseFloat(cm) * 10));
  const toCm = (mm) => (mm ? String(mm / 10) : '');
  function rateSizeFields() {
    return { max_length_mm: toMm(maxLengthCm), max_width_mm: toMm(maxWidthCm), max_height_mm: toMm(maxHeightCm) };
  }

  // Copy dialogs
  let isCopyZoneOpen = false;
  let copyZoneSource = null;
  let copyTargetProviderId = '';
  let copyZoneName = '';
  let isCopyRatesOpen = false;
  let copyRatesTarget = null;
  let copySourceZoneId = '';
  let copySelectedRateIds = [];

  $: allZones = (providers || []).flatMap((p) => (p.zones || []).map((z) => ({ ...z, providerName: p.name })));
  $: copySourceZone = allZones.find((z) => z.id === copySourceZoneId);

  onMount(() => {
    reloadProviders();
  });

  function getHeaders() {
    const token = localStorage.getItem('admin_token');
    return {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {})
    };
  }

  function showNotice(msg) {
    notice = msg;
    errorMsg = '';
    setTimeout(() => { if (notice === msg) notice = ''; }, 4000);
  }

  function showError(msg) {
    errorMsg = msg;
    notice = '';
    setTimeout(() => { if (errorMsg === msg) errorMsg = ''; }, 6000);
  }

  export async function reloadProviders() {
    loading = true;
    try {
      const res = await fetch('/api/v1/admin/settings/shipping/providers', {
        headers: getHeaders()
      });
      if (res.ok) {
        providers = await res.json();
      } else {
        const err = await res.json().catch(() => ({}));
        showError(err.error || 'Failed to load shipping carriers.');
      }
    } catch (e) {
      showError('Network failure fetching shipping providers.');
    } finally {
      loading = false;
    }
  }

  // --- Provider Handlers ---
  function openAddProvider() {
    providerName = '';
    providerCode = '';
    trackingUrlTemplate = 'https://www.dhl.com/track?id={tracking_number}';
    providerIsActive = true;
    providerSortOrder = (providers || []).length;
    modalError = '';
    isSubmitting = false;
    isAddProviderOpen = true;
  }

  async function handleCreateProvider() {
    modalError = '';
    if (!providerName.trim()) {
      modalError = 'Please provide a carrier name.';
      return;
    }
    let code = providerCode.toLowerCase().trim().replace(/[^a-z0-9_-]/g, '_');
    if (!code) {
      code = providerName.toLowerCase().trim().replace(/[^a-z0-9_-]/g, '_');
    }
    if (providers.some((p) => p.code.toLowerCase() === code)) {
      modalError = `Carrier code "${code}" already exists. Please choose a different code.`;
      return;
    }

    isSubmitting = true;
    try {
      const res = await fetch('/api/v1/admin/settings/shipping/providers', {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({
          name: providerName.trim(),
          code,
          tracking_url_template: trackingUrlTemplate.trim(),
          is_active: providerIsActive,
          sort_order: parseInt(providerSortOrder) || 0
        })
      });
      if (res.ok) {
        isAddProviderOpen = false;
        showNotice(`Shipping carrier "${providerName}" added successfully.`);
        await reloadProviders();
      } else {
        const err = await res.text();
        modalError = `Failed to create carrier: ${err}`;
      }
    } catch (e) {
      modalError = 'Failed to connect to backend server.';
    } finally {
      isSubmitting = false;
    }
  }

  function openEditProvider(prov) {
    editingProvider = prov;
    providerName = prov.name;
    providerCode = prov.code;
    trackingUrlTemplate = prov.tracking_url_template || '';
    providerIsActive = Boolean(prov.is_active);
    providerSortOrder = prov.sort_order || 0;
    modalError = '';
    isSubmitting = false;
    isEditProviderOpen = true;
  }

  async function handleUpdateProvider() {
    if (!editingProvider) return;
    modalError = '';
    if (!providerName.trim()) {
      modalError = 'Please provide a carrier name.';
      return;
    }
    let code = providerCode.toLowerCase().trim().replace(/[^a-z0-9_-]/g, '_');
    isSubmitting = true;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/providers/${editingProvider.id}`, {
        method: 'PUT',
        headers: getHeaders(),
        body: JSON.stringify({
          name: providerName.trim(),
          code,
          tracking_url_template: trackingUrlTemplate.trim(),
          is_active: providerIsActive,
          sort_order: parseInt(providerSortOrder) || 0
        })
      });
      if (res.ok) {
        isEditProviderOpen = false;
        editingProvider = null;
        showNotice(`Carrier "${providerName}" updated successfully.`);
        await reloadProviders();
      } else {
        const err = await res.text();
        modalError = `Failed to update carrier: ${err}`;
      }
    } catch (e) {
      modalError = 'Failed to update carrier.';
    } finally {
      isSubmitting = false;
    }
  }

  async function handleDeleteProvider(prov) {
    if (!confirm(`Are you sure you want to delete "${prov.name}" and all its zones and price categories?`)) return;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/providers/${prov.id}`, {
        method: 'DELETE',
        headers: getHeaders()
      });
      if (res.ok) {
        showNotice(`Carrier "${prov.name}" removed.`);
        await reloadProviders();
      } else {
        showError('Failed to delete carrier.');
      }
    } catch (e) {
      showError('Error deleting carrier.');
    }
  }

  // --- Zone Handlers ---
  function openAddZone(providerId) {
    selectedProviderId = providerId;
    zoneName = '';
    zoneCodes = ['DE'];
    zoneIsDefault = false;
    modalError = '';
    isSubmitting = false;
    isAddZoneOpen = true;
  }

  async function handleCreateZone() {
    modalError = '';
    const countryCodes = [...zoneCodes];
    if (!zoneName.trim() || countryCodes.length === 0) {
      modalError = 'Please provide a zone name and at least one country.';
      return;
    }
    isSubmitting = true;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/providers/${selectedProviderId}/zones`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({
          zone_name: zoneName.trim(),
          country_codes: countryCodes,
          is_default: zoneIsDefault
        })
      });
      if (res.ok) {
        isAddZoneOpen = false;
        showNotice(`Delivery zone "${zoneName}" created.`);
        await reloadProviders();
      } else {
        const err = await res.text();
        modalError = `Failed to create zone: ${err}`;
      }
    } catch (e) {
      modalError = 'Failed to create zone.';
    } finally {
      isSubmitting = false;
    }
  }

  function openEditZone(zone) {
    editingZone = zone;
    zoneName = zone.zone_name;
    zoneCodes = Array.isArray(zone.country_codes) ? [...zone.country_codes] : [];
    zoneIsDefault = Boolean(zone.is_default);
    modalError = '';
    isSubmitting = false;
    isEditZoneOpen = true;
  }

  async function handleUpdateZone() {
    if (!editingZone) return;
    modalError = '';
    const countryCodes = [...zoneCodes];
    if (countryCodes.length === 0 && !zoneIsDefault) {
      modalError = 'Please add at least one country.';
      return;
    }
    isSubmitting = true;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/zones/${editingZone.id}`, {
        method: 'PUT',
        headers: getHeaders(),
        body: JSON.stringify({
          zone_name: zoneName.trim(),
          country_codes: countryCodes,
          is_default: zoneIsDefault
        })
      });
      if (res.ok) {
        isEditZoneOpen = false;
        editingZone = null;
        showNotice(`Zone "${zoneName}" updated.`);
        await reloadProviders();
      } else {
        const err = await res.text();
        modalError = `Failed to update zone: ${err}`;
      }
    } catch (e) {
      modalError = 'Failed to update zone.';
    } finally {
      isSubmitting = false;
    }
  }

  async function handleDeleteZone(zone) {
    if (!confirm(`Are you sure you want to delete delivery zone "${zone.zone_name}"?`)) return;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/zones/${zone.id}`, {
        method: 'DELETE',
        headers: getHeaders()
      });
      if (res.ok) {
        showNotice(`Zone "${zone.zone_name}" deleted.`);
        await reloadProviders();
      } else {
        showError('Failed to delete zone.');
      }
    } catch (e) {
      showError('Error deleting zone.');
    }
  }

  // --- Rate Handlers ---
  function openAddRate(zoneId) {
    selectedZoneId = zoneId;
    rateName = 'Standard Paket';
    packageType = 'standard';
    priceEuros = 6.99;
    deliveryDays = '2-4 business days';
    maxLengthCm = maxWidthCm = maxHeightCm = '';
    minWeightG = 0;
    maxWeightG = 5000;
    modalError = '';
    isSubmitting = false;
    isAddRateOpen = true;
  }

  async function handleCreateRate() {
    modalError = '';
    const priceCents = Math.round(Number(priceEuros) * 100);
    isSubmitting = true;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/zones/${selectedZoneId}/rates`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({
          name: rateName.trim(),
          package_type: packageType,
          price_cents: priceCents,
          estimated_delivery_days: deliveryDays.trim(),
          min_weight_g: parseInt(minWeightG) || 0,
          max_weight_g: parseInt(maxWeightG) || 0,
          ...rateSizeFields()
        })
      });
      if (res.ok) {
        isAddRateOpen = false;
        showNotice(`Shipping rate tier "${rateName}" created.`);
        await reloadProviders();
      } else {
        const err = await res.text();
        modalError = `Failed to create rate: ${err}`;
      }
    } catch (e) {
      modalError = 'Failed to create rate.';
    } finally {
      isSubmitting = false;
    }
  }

  function openEditRate(rate) {
    editingRate = rate;
    rateName = rate.name;
    packageType = rate.package_type || 'standard';
    priceEuros = ((rate.price_cents || 0) / 100).toFixed(2);
    deliveryDays = rate.estimated_delivery_days || '';
    minWeightG = rate.min_weight_g || 0;
    maxWeightG = rate.max_weight_g || 0;
    maxLengthCm = toCm(rate.max_length_mm);
    maxWidthCm = toCm(rate.max_width_mm);
    maxHeightCm = toCm(rate.max_height_mm);
    modalError = '';
    isSubmitting = false;
    isEditRateOpen = true;
  }

  async function handleUpdateRate() {
    if (!editingRate) return;
    modalError = '';
    const priceCents = Math.round(Number(priceEuros) * 100);
    isSubmitting = true;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/rates/${editingRate.id}`, {
        method: 'PUT',
        headers: getHeaders(),
        body: JSON.stringify({
          name: rateName.trim(),
          package_type: packageType,
          price_cents: priceCents,
          estimated_delivery_days: deliveryDays.trim(),
          min_weight_g: parseInt(minWeightG) || 0,
          max_weight_g: parseInt(maxWeightG) || 0,
          ...rateSizeFields()
        })
      });
      if (res.ok) {
        isEditRateOpen = false;
        editingRate = null;
        showNotice(`Rate "${rateName}" updated.`);
        await reloadProviders();
      } else {
        const err = await res.text();
        modalError = `Failed to update rate: ${err}`;
      }
    } catch (e) {
      modalError = 'Failed to update rate.';
    } finally {
      isSubmitting = false;
    }
  }

  // --- Copy zones and rates ---
  function openCopyZone(zone, providerId) {
    copyZoneSource = zone;
    copyTargetProviderId = (providers.find((p) => p.id !== providerId) || providers.find((p) => p.id === providerId) || {}).id || '';
    copyZoneName = zone.zone_name;
    modalError = '';
    isSubmitting = false;
    isCopyZoneOpen = true;
  }

  async function handleCopyZone() {
    modalError = '';
    isSubmitting = true;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/zones/${copyZoneSource.id}/copy`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({ provider_id: copyTargetProviderId || null, zone_name: copyZoneName.trim() || null })
      });
      const body = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(body.error || 'Copying the zone failed');
      isCopyZoneOpen = false;
      showNotice(`Zone "${body.zone_name}" created with ${body.rates_copied} rate(s).`);
      await reloadProviders();
    } catch (e) {
      modalError = e.message;
    } finally {
      isSubmitting = false;
    }
  }

  function openCopyRates(zone) {
    copyRatesTarget = zone;
    copySourceZoneId = (allZones.find((z) => z.id !== zone.id && (z.rates || []).length > 0) || {}).id || '';
    copySelectedRateIds = [];
    modalError = '';
    isSubmitting = false;
    isCopyRatesOpen = true;
  }
  // Selecting another source zone selects all of its rates
  $: if (copySourceZone) copySelectedRateIds = (copySourceZone.rates || []).map((r) => r.id);

  async function handleCopyRates() {
    modalError = '';
    if (copySelectedRateIds.length === 0) {
      modalError = 'Select at least one rate to copy.';
      return;
    }
    isSubmitting = true;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/zones/${copyRatesTarget.id}/copy-rates`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({ source_zone_id: copySourceZoneId, rate_ids: copySelectedRateIds })
      });
      const body = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(body.error || 'Copying the rates failed');
      isCopyRatesOpen = false;
      showNotice(`${body.rates_copied} rate(s) copied into "${copyRatesTarget.zone_name}".`);
      await reloadProviders();
    } catch (e) {
      modalError = e.message;
    } finally {
      isSubmitting = false;
    }
  }

  async function handleDeleteRate(rate) {
    if (!confirm(`Delete price tier "${rate.name}"?`)) return;
    try {
      const res = await fetch(`/api/v1/admin/settings/shipping/rates/${rate.id}`, {
        method: 'DELETE',
        headers: getHeaders()
      });
      if (res.ok) {
        showNotice(`Price tier "${rate.name}" deleted.`);
        await reloadProviders();
      } else {
        showError('Failed to delete rate.');
      }
    } catch (e) {
      showError('Error deleting rate.');
    }
  }
</script>

<div class="space-y-6">
  <!-- Notifications -->
  {#if notice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <CheckCircle2 size={16} />
      <span>{notice}</span>
    </div>
  {/if}

  {#if errorMsg}
    <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
      <AlertCircle size={16} />
      <span>{errorMsg}</span>
    </div>
  {/if}

  <!-- Header with Direct Add Button -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-4">
    <div>
      <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Truck size={22} class="text-orange-500" />
        Shipping Carriers & Delivery Rates
      </h2>
      <p class="text-xs text-slate-400 mt-1">
        Manage logistics providers (DHL, Hermes, UPS), country zones, and weight/package pricing tiers directly.
        At checkout the weights of the products in the cart (Products → weight) are added up and only rates whose weight range fits are offered.
      </p>
    </div>

    <button
      type="button"
      on:click={openAddProvider}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>Add Shipping Carrier</span>
    </button>
  </div>

  <!-- Carriers / Providers List -->
  {#if providers.length === 0}
    <div class="p-12 text-center rounded-2xl bg-slate-900 border border-slate-800 space-y-4">
      <Truck size={36} class="mx-auto text-slate-600" />
      <div class="text-sm font-bold text-white">No shipping carriers configured yet</div>
      <p class="text-xs text-slate-400 max-w-md mx-auto">
        Set up DHL, Hermes, UPS or custom delivery carriers, assign destinations, and set up your delivery pricing tiers.
      </p>
      <button
        type="button"
        on:click={openAddProvider}
        class="px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold inline-flex items-center gap-1.5 transition-colors"
      >
        <Plus size={14} />
        <span>Add Your First Carrier</span>
      </button>
    </div>
  {:else}
    <div class="space-y-6">
      {#each providers as prov}
        <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden">
          <!-- Carrier Header -->
          <div class="p-5 bg-slate-900/90 border-b border-slate-800/80 flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center font-black text-orange-400 text-sm shadow-inner">
                {prov.code.toUpperCase().slice(0, 3)}
              </div>
              <div>
                <div class="flex items-center gap-2.5">
                  <h3 class="text-base font-extrabold text-white">{prov.name}</h3>
                  <span class="px-2 py-0.5 rounded text-[10px] font-bold {prov.is_active ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-slate-800 text-slate-500'}">
                    {prov.is_active ? 'Active' : 'Inactive'}
                  </span>
                </div>
                <div class="text-xs text-slate-400 font-mono flex items-center gap-2 mt-0.5">
                  <span>code: <strong class="text-slate-200">{prov.code}</strong></span>
                  {#if prov.tracking_url_template}
                    <span class="text-slate-600">•</span>
                    <span class="truncate max-w-xs text-slate-500" title={prov.tracking_url_template}>
                      {prov.tracking_url_template}
                    </span>
                  {/if}
                </div>
              </div>
            </div>

            <!-- Carrier Actions -->
            <div class="flex items-center gap-2 self-end md:self-auto">
              <button
                type="button"
                on:click={() => openAddZone(prov.id)}
                class="px-3 py-1.5 rounded-lg bg-orange-600/15 hover:bg-orange-600/25 text-orange-400 font-bold text-xs border border-orange-500/20 flex items-center gap-1.5 transition-colors"
              >
                <Plus size={13} />
                <span>Add Zone</span>
              </button>
              <button
                type="button"
                on:click={() => openEditProvider(prov)}
                class="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors border border-slate-700"
                title="Edit Carrier Details"
              >
                <Edit3 size={14} class="text-orange-400" />
              </button>
              <button
                type="button"
                on:click={() => handleDeleteProvider(prov)}
                class="p-2 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors border border-rose-500/20"
                title="Delete Carrier"
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>

          <!-- Carrier Zones List -->
          <div class="p-5 space-y-4">
            {#if !prov.zones || prov.zones.length === 0}
              <div class="py-6 text-center text-xs text-slate-500 border border-dashed border-slate-800 rounded-xl">
                No delivery zones configured for this carrier. Click "+ Add Zone" above to define country rates.
              </div>
            {:else}
              <div class="grid grid-cols-1 gap-4">
                {#each prov.zones as zone}
                  <div class="p-4 rounded-xl bg-slate-950/60 border border-slate-800/80 space-y-3">
                    <!-- Zone Header -->
                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-800/50 pb-2.5">
                      <div class="flex items-center gap-2">
                        <Globe size={15} class="text-blue-400 flex-shrink-0" />
                        <span class="font-bold text-white text-xs">{zone.zone_name}</span>
                        {#if zone.is_default}
                          <span class="px-1.5 py-0.5 rounded text-[9px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20">Default Catch-All</span>
                        {/if}
                        <div class="flex flex-wrap gap-1 ml-2">
                          {#if Array.isArray(zone.country_codes)}
                            <!-- At most 3 countries here; the full list is in the edit dialog -->
                            {#each zone.country_codes.slice(0, 3) as cc}
                              <span class="px-1.5 py-0.5 rounded text-[10px] font-mono font-bold bg-slate-800 text-slate-300 border border-slate-700" title={countryName(cc)}>
                                {cc}
                              </span>
                            {/each}
                            {#if zone.country_codes.length > 3}
                              <span class="px-1.5 py-0.5 text-[10px] font-bold text-slate-500" title="{zone.country_codes.length} countries — edit the zone to see all">… ({zone.country_codes.length})</span>
                            {/if}
                          {/if}
                        </div>
                      </div>

                      <div class="flex items-center gap-1.5 self-end sm:self-auto">
                        <button
                          type="button"
                          on:click={() => openCopyRates(zone)}
                          class="px-2.5 py-1 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white text-[11px] font-semibold flex items-center gap-1 border border-slate-700 transition-colors"
                          title="Copy rates from another zone into this zone"
                        >
                          <Copy size={11} class="text-sky-400" />
                          <span>Copy rates in</span>
                        </button>
                        <button
                          type="button"
                          on:click={() => openCopyZone(zone, prov.id)}
                          class="p-1 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white border border-slate-700 transition-colors"
                          title="Copy this zone (with its rates) to a provider"
                        >
                          <Copy size={12} class="text-emerald-400" />
                        </button>
                        <button
                          type="button"
                          on:click={() => openAddRate(zone.id)}
                          class="px-2.5 py-1 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white text-[11px] font-semibold flex items-center gap-1 border border-slate-700 transition-colors"
                        >
                          <Plus size={11} class="text-orange-400" />
                          <span>Add Rate</span>
                        </button>
                        <button
                          type="button"
                          on:click={() => openEditZone(zone)}
                          class="p-1 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white border border-slate-700 transition-colors"
                          title="Edit Zone"
                        >
                          <Edit3 size={12} class="text-orange-400" />
                        </button>
                        <button
                          type="button"
                          on:click={() => handleDeleteZone(zone)}
                          class="p-1 rounded-md bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 border border-rose-500/20 transition-colors"
                          title="Delete Zone"
                        >
                          <Trash2 size={12} />
                        </button>
                      </div>
                    </div>

                    <!-- Rates Table -->
                    {#if !zone.rates || zone.rates.length === 0}
                      <div class="py-2 text-center text-[11px] text-slate-500 italic">
                        No pricing tiers added to this zone yet.
                      </div>
                    {:else}
                      <div class="overflow-x-auto">
                        <table class="w-full text-left text-xs">
                          <thead>
                            <tr class="text-[10px] font-mono text-slate-500 uppercase border-b border-slate-800/40">
                              <th class="pb-1.5 font-semibold">Tier / Category</th>
                              <th class="pb-1.5 font-semibold">Type</th>
                              <th class="pb-1.5 font-semibold">Delivery Time</th>
                              <th class="pb-1.5 font-semibold">Weight Range</th>
                              <th class="pb-1.5 font-semibold text-right">Price</th>
                              <th class="pb-1.5 font-semibold text-right">Actions</th>
                            </tr>
                          </thead>
                          <tbody class="divide-y divide-slate-800/30">
                            {#each zone.rates as rate}
                              <tr class="text-slate-300 hover:bg-slate-900/40">
                                <td class="py-2 font-medium text-white">{rate.name}</td>
                                <td class="py-2">
                                  <span class="px-1.5 py-0.5 rounded text-[10px] font-mono uppercase bg-slate-800 text-orange-300">
                                    {rate.package_type}
                                  </span>
                                </td>
                                <td class="py-2 text-slate-400">{rate.estimated_delivery_days || '—'}</td>
                                <td class="py-2 font-mono text-[11px] text-slate-400">
                                  {(rate.min_weight_g / 1000).toFixed(1)}–{rate.max_weight_g > 0 ? `${(rate.max_weight_g / 1000).toFixed(1)} kg` : '∞'}
                                  {#if rate.max_length_mm > 0 && rate.max_width_mm > 0 && rate.max_height_mm > 0}
                                    <span class="block text-[10px] text-slate-500">max {rate.max_length_mm / 10} × {rate.max_width_mm / 10} × {rate.max_height_mm / 10} cm</span>
                                  {/if}
                                </td>
                                <td class="py-2 text-right font-bold text-emerald-400 font-mono">
                                  {((rate.price_cents || 0) / 100).toFixed(2)} €
                                </td>
                                <td class="py-2 text-right">
                                  <div class="inline-flex items-center gap-1">
                                    <button
                                      type="button"
                                      on:click={() => openEditRate(rate)}
                                      class="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white"
                                      title="Edit Rate"
                                    >
                                      <Edit3 size={11} class="text-orange-400" />
                                    </button>
                                    <button
                                      type="button"
                                      on:click={() => handleDeleteRate(rate)}
                                      class="p-1 rounded bg-rose-500/10 hover:bg-rose-500/20 text-rose-400"
                                      title="Delete Rate"
                                    >
                                      <Trash2 size={11} />
                                    </button>
                                  </div>
                                </td>
                              </tr>
                            {/each}
                          </tbody>
                        </table>
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- MODAL 1: Add Shipping Carrier -->
{#if isAddProviderOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-2xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-sm font-bold text-white flex items-center gap-2">
          <Truck size={16} class="text-orange-500" />
          <span>Add Shipping Carrier / Provider</span>
        </h3>
        <button on:click={() => isAddProviderOpen = false} class="text-slate-400 hover:text-white">
          <X size={16} />
        </button>
      </div>

      {#if modalError}
        <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2">
          <AlertCircle size={15} class="flex-shrink-0" />
          <span>{modalError}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleCreateProvider} class="space-y-4 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Carrier Name</label>
          <input
            type="text"
            bind:value={providerName}
            required
            placeholder="e.g. DHL Express, Hermes, UPS, DPD"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">System Code (unique identifier)</label>
          <input
            type="text"
            bind:value={providerCode}
            placeholder="e.g. dhl, hermes, ups (auto-generated if empty)"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Tracking URL Template</label>
          <input
            type="text"
            bind:value={trackingUrlTemplate}
            placeholder="https://www.dhl.com/track?id=TRACKING_ID"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-[11px] focus:outline-none focus:border-orange-500"
          />
          <p class="text-[10px] text-slate-500 mt-1">Use <code class="bg-black/30 px-1 py-0.5 rounded text-orange-400">&#123;tracking_number&#125;</code> as placeholder</p>
        </div>

        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <input type="checkbox" id="add-prov-active" bind:checked={providerIsActive} class="accent-orange-500 w-4 h-4" />
            <label for="add-prov-active" class="text-slate-300 font-medium cursor-pointer">Active Carrier</label>
          </div>
          <div class="flex items-center gap-2">
            <label class="text-slate-400">Sort:</label>
            <input type="number" bind:value={providerSortOrder} class="w-16 px-2 py-1 rounded bg-slate-950 border border-slate-800 text-white text-center font-mono" />
          </div>
        </div>

        <div class="flex items-center justify-end gap-2 pt-3 border-t border-slate-800">
          <button type="button" on:click={() => isAddProviderOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
          <button type="submit" disabled={isSubmitting} class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white font-bold shadow-lg shadow-orange-600/25">
            {isSubmitting ? 'Saving...' : 'Add Carrier'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- MODAL 2: Edit Shipping Carrier -->
{#if isEditProviderOpen && editingProvider}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-2xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-sm font-bold text-white flex items-center gap-2">
          <Edit3 size={16} class="text-orange-500" />
          <span>Edit Shipping Carrier: {editingProvider.name}</span>
        </h3>
        <button on:click={() => isEditProviderOpen = false} class="text-slate-400 hover:text-white">
          <X size={16} />
        </button>
      </div>

      {#if modalError}
        <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2">
          <AlertCircle size={15} class="flex-shrink-0" />
          <span>{modalError}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleUpdateProvider} class="space-y-4 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Carrier Name</label>
          <input
            type="text"
            bind:value={providerName}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Carrier Code</label>
          <input
            type="text"
            bind:value={providerCode}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Tracking URL Template</label>
          <input
            type="text"
            bind:value={trackingUrlTemplate}
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono text-[11px] focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <input type="checkbox" id="edit-prov-active" bind:checked={providerIsActive} class="accent-orange-500 w-4 h-4" />
            <label for="edit-prov-active" class="text-slate-300 font-medium cursor-pointer">Active Carrier</label>
          </div>
          <div class="flex items-center gap-2">
            <label class="text-slate-400">Sort:</label>
            <input type="number" bind:value={providerSortOrder} class="w-16 px-2 py-1 rounded bg-slate-950 border border-slate-800 text-white text-center font-mono" />
          </div>
        </div>

        <div class="flex items-center justify-end gap-2 pt-3 border-t border-slate-800">
          <button type="button" on:click={() => isEditProviderOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
          <button type="submit" disabled={isSubmitting} class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white font-bold shadow-lg shadow-orange-600/25">
            {isSubmitting ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- MODAL 3: Add Zone -->
{#if isAddZoneOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-2xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-sm font-bold text-white flex items-center gap-2">
          <Globe size={16} class="text-blue-400" />
          <span>Add Delivery Zone</span>
        </h3>
        <button on:click={() => isAddZoneOpen = false} class="text-slate-400 hover:text-white">
          <X size={16} />
        </button>
      </div>

      {#if modalError}
        <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2">
          <AlertCircle size={15} class="flex-shrink-0" />
          <span>{modalError}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleCreateZone} class="space-y-4 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Zone Name</label>
          <input
            type="text"
            bind:value={zoneName}
            required
            placeholder="e.g. Germany & DACH, European Union, Worldwide"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="add-zone-countries" class="block text-slate-300 font-semibold mb-1">Countries</label>
          <CountryPicker bind:codes={zoneCodes} id="add-zone-countries" />
        </div>

        <div class="flex items-center gap-2">
          <input type="checkbox" id="add-zone-def" bind:checked={zoneIsDefault} class="accent-orange-500 w-4 h-4" />
          <label for="add-zone-def" class="text-slate-300 font-medium cursor-pointer">Default / Fallback Zone for unlisted countries</label>
        </div>

        <div class="flex items-center justify-end gap-2 pt-3 border-t border-slate-800">
          <button type="button" on:click={() => isAddZoneOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
          <button type="submit" disabled={isSubmitting} class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white font-bold shadow-lg shadow-orange-600/25">
            {isSubmitting ? 'Creating...' : 'Create Zone'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- MODAL 4: Edit Zone -->
{#if isEditZoneOpen && editingZone}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-2xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-sm font-bold text-white flex items-center gap-2">
          <Edit3 size={16} class="text-blue-400" />
          <span>Edit Delivery Zone: {editingZone.zone_name}</span>
        </h3>
        <button on:click={() => isEditZoneOpen = false} class="text-slate-400 hover:text-white">
          <X size={16} />
        </button>
      </div>

      {#if modalError}
        <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2">
          <AlertCircle size={15} class="flex-shrink-0" />
          <span>{modalError}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleUpdateZone} class="space-y-4 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Zone Name</label>
          <input
            type="text"
            bind:value={zoneName}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="edit-zone-countries" class="block text-slate-300 font-semibold mb-1">Countries</label>
          <CountryPicker bind:codes={zoneCodes} id="edit-zone-countries" />
        </div>

        <div class="flex items-center gap-2">
          <input type="checkbox" id="edit-zone-def" bind:checked={zoneIsDefault} class="accent-orange-500 w-4 h-4" />
          <label for="edit-zone-def" class="text-slate-300 font-medium cursor-pointer">Default / Fallback Zone</label>
        </div>

        <div class="flex items-center justify-end gap-2 pt-3 border-t border-slate-800">
          <button type="button" on:click={() => isEditZoneOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
          <button type="submit" disabled={isSubmitting} class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white font-bold shadow-lg shadow-orange-600/25">
            {isSubmitting ? 'Saving...' : 'Save Zone'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- MODAL 5: Add Rate -->
{#if isAddRateOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-2xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-sm font-bold text-white flex items-center gap-2">
          <Box size={16} class="text-orange-500" />
          <span>Add Pricing Rate Tier</span>
        </h3>
        <button on:click={() => isAddRateOpen = false} class="text-slate-400 hover:text-white">
          <X size={16} />
        </button>
      </div>

      <form on:submit|preventDefault={handleCreateRate} class="space-y-4 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Tier / Rate Name</label>
          <input
            type="text"
            bind:value={rateName}
            required
            placeholder="e.g. Standard Paket (bis 5kg), Package M, Package L"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Package Type</label>
            <select
              bind:value={packageType}
              class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            >
              <option value="standard">Standard Paket</option>
              <option value="express">Express</option>
              <option value="medium">Package M</option>
              <option value="large">Package L</option>
              <option value="xlarge">Package XL</option>
              <option value="heavy">Heavy / Freight</option>
            </select>
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Price (EUR €)</label>
            <input
              type="number"
              step="0.01"
              bind:value={priceEuros}
              required
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Estimated Delivery Time</label>
          <input
            type="text"
            bind:value={deliveryDays}
            required
            placeholder="e.g. 1-2 business days, 3-5 Days"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Min Weight (grams)</label>
            <input
              type="number"
              bind:value={minWeightG}
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Max Weight (grams, 0 = no limit)</label>
            <input
              type="number"
              bind:value={maxWeightG}
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div>
          <span class="block text-slate-300 font-semibold mb-1">Max. package size (cm, empty = no size limit)</span>
          <div class="grid grid-cols-3 gap-2">
            <input type="number" step="0.1" min="0" bind:value={maxLengthCm} placeholder="Length" aria-label="Maximum length in cm" class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
            <input type="number" step="0.1" min="0" bind:value={maxWidthCm} placeholder="Width" aria-label="Maximum width in cm" class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
            <input type="number" step="0.1" min="0" bind:value={maxHeightCm} placeholder="Height" aria-label="Maximum height in cm" class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
          </div>
          <p class="text-[10px] text-slate-500 mt-1">The order of the sides does not matter. Checkout only offers packages the cart fits into (by weight and size) and picks the cheapest.</p>
        </div>

        <div class="flex items-center justify-end gap-2 pt-3 border-t border-slate-800">
          <button type="button" on:click={() => isAddRateOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
          <button type="submit" class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold shadow-lg shadow-orange-600/25">Add Rate</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- MODAL 6: Edit Rate -->
{#if isEditRateOpen && editingRate}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150">
    <div class="w-full max-w-md rounded-2xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-sm font-bold text-white flex items-center gap-2">
          <Edit3 size={16} class="text-orange-500" />
          <span>Edit Rate Tier: {editingRate.name}</span>
        </h3>
        <button on:click={() => isEditRateOpen = false} class="text-slate-400 hover:text-white">
          <X size={16} />
        </button>
      </div>

      <form on:submit|preventDefault={handleUpdateRate} class="space-y-4 text-xs">
        <div>
          <label class="block text-slate-300 font-semibold mb-1">Tier / Rate Name</label>
          <input
            type="text"
            bind:value={rateName}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Package Type</label>
            <select
              bind:value={packageType}
              class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            >
              <option value="standard">Standard Paket</option>
              <option value="express">Express</option>
              <option value="medium">Package M</option>
              <option value="large">Package L</option>
              <option value="xlarge">Package XL</option>
              <option value="heavy">Heavy / Freight</option>
            </select>
          </div>

          <div>
            <label class="block text-slate-300 font-semibold mb-1">Price (EUR €)</label>
            <input
              type="number"
              step="0.01"
              bind:value={priceEuros}
              required
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-slate-300 font-semibold mb-1">Estimated Delivery Time</label>
          <input
            type="text"
            bind:value={deliveryDays}
            required
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Min Weight (grams)</label>
            <input
              type="number"
              bind:value={minWeightG}
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
          <div>
            <label class="block text-slate-300 font-semibold mb-1">Max Weight (grams, 0 = no limit)</label>
            <input
              type="number"
              bind:value={maxWeightG}
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div>
          <span class="block text-slate-300 font-semibold mb-1">Max. package size (cm, empty = no size limit)</span>
          <div class="grid grid-cols-3 gap-2">
            <input type="number" step="0.1" min="0" bind:value={maxLengthCm} placeholder="Length" aria-label="Maximum length in cm" class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
            <input type="number" step="0.1" min="0" bind:value={maxWidthCm} placeholder="Width" aria-label="Maximum width in cm" class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
            <input type="number" step="0.1" min="0" bind:value={maxHeightCm} placeholder="Height" aria-label="Maximum height in cm" class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500" />
          </div>
          <p class="text-[10px] text-slate-500 mt-1">The order of the sides does not matter. Checkout only offers packages the cart fits into (by weight and size) and picks the cheapest.</p>
        </div>

        <div class="flex items-center justify-end gap-2 pt-3 border-t border-slate-800">
          <button type="button" on:click={() => isEditRateOpen = false} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
          <button type="submit" class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold shadow-lg shadow-orange-600/25">Save Rate</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- MODAL 7: Copy zone to a provider -->
{#if isCopyZoneOpen && copyZoneSource}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm">
    <div class="w-full max-w-md rounded-2xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto text-xs">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-sm font-bold text-white flex items-center gap-2"><Copy size={16} class="text-emerald-400" /> Copy zone "{copyZoneSource.zone_name}"</h3>
        <button on:click={() => (isCopyZoneOpen = false)} class="text-slate-400 hover:text-white" aria-label="Close"><X size={16} /></button>
      </div>
      {#if modalError}<div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400">{modalError}</div>{/if}
      <p class="text-slate-400">Creates a new zone with the same countries and all {(copyZoneSource.rates || []).length} rate(s). You can change prices afterwards.</p>
      <div>
        <label for="copy-zone-provider" class="block text-slate-300 font-semibold mb-1">Copy to provider</label>
        <select id="copy-zone-provider" bind:value={copyTargetProviderId} class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500">
          {#each providers as p}
            <option value={p.id}>{p.name}</option>
          {/each}
        </select>
      </div>
      <div>
        <label for="copy-zone-name" class="block text-slate-300 font-semibold mb-1">Name of the new zone</label>
        <input id="copy-zone-name" bind:value={copyZoneName} class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500" />
      </div>
      <div class="flex justify-end gap-2 pt-3 border-t border-slate-800">
        <button type="button" on:click={() => (isCopyZoneOpen = false)} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
        <button type="button" on:click={handleCopyZone} disabled={isSubmitting || !copyTargetProviderId} class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white font-bold">{isSubmitting ? 'Copying…' : 'Copy zone'}</button>
      </div>
    </div>
  </div>
{/if}

<!-- MODAL 8: Copy rates from another zone into this zone -->
{#if isCopyRatesOpen && copyRatesTarget}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm">
    <div class="w-full max-w-lg rounded-2xl bg-slate-900 border border-slate-800 p-6 shadow-2xl space-y-4 max-h-[90vh] overflow-y-auto text-xs">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 class="text-sm font-bold text-white flex items-center gap-2"><Copy size={16} class="text-sky-400" /> Copy rates into "{copyRatesTarget.zone_name}"</h3>
        <button on:click={() => (isCopyRatesOpen = false)} class="text-slate-400 hover:text-white" aria-label="Close"><X size={16} /></button>
      </div>
      {#if modalError}<div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400">{modalError}</div>{/if}
      <div>
        <label for="copy-rates-source" class="block text-slate-300 font-semibold mb-1">Copy from zone</label>
        <select id="copy-rates-source" bind:value={copySourceZoneId} class="w-full px-3 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500">
          {#each allZones.filter((z) => z.id !== copyRatesTarget.id) as z}
            <option value={z.id}>{z.providerName} · {z.zone_name} ({(z.rates || []).length} rates)</option>
          {/each}
        </select>
      </div>
      {#if copySourceZone}
        {#if (copySourceZone.rates || []).length === 0}
          <p class="text-slate-500">This zone has no rates.</p>
        {:else}
          <div class="max-h-64 overflow-auto rounded-xl border border-slate-800 divide-y divide-slate-800">
            {#each copySourceZone.rates as r}
              <label class="flex items-center justify-between gap-3 px-3 py-2 cursor-pointer hover:bg-slate-800/50">
                <span class="flex items-center gap-2">
                  <input type="checkbox" value={r.id} bind:group={copySelectedRateIds} class="accent-orange-500" />
                  <span class="text-white">{r.name}</span>
                  <span class="text-slate-500">{r.estimated_delivery_days}</span>
                </span>
                <span class="font-mono text-emerald-400">{((r.price_cents || 0) / 100).toFixed(2)} €</span>
              </label>
            {/each}
          </div>
        {/if}
      {/if}
      <div class="flex justify-end gap-2 pt-3 border-t border-slate-800">
        <button type="button" on:click={() => (isCopyRatesOpen = false)} class="px-4 py-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white">Cancel</button>
        <button type="button" on:click={handleCopyRates} disabled={isSubmitting || !copySourceZoneId} class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white font-bold">{isSubmitting ? 'Copying…' : `Copy ${copySelectedRateIds.length} rate(s)`}</button>
      </div>
    </div>
  </div>
{/if}
