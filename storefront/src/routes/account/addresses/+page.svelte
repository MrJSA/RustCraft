<script>
  import { onMount } from 'svelte';
  import { customer } from '$lib/stores/customer.js';
  import { goto } from '$app/navigation';
  import { MapPin, Plus, ArrowLeft, CheckCircle2, Trash2, Home, Building } from 'lucide-svelte';
  import { countryOptions } from '$lib/countries.js';

  const allCountries = countryOptions();

  let addresses = [];
  let isLoading = true;
  let isAddOpen = false;
  let errorMsg = '';
  let successMsg = '';

  // New address form fields
  let addressType = 'shipping';
  let fullName = '';
  let streetAddress = '';
  let apartmentSuite = '';
  let city = '';
  let stateProvince = '';
  let postalCode = '';
  let countryCode = 'DE';

  $: shippingAddresses = addresses.filter((a) => a.address_type === 'shipping');
  $: billingAddresses = addresses.filter((a) => a.address_type === 'billing');

  async function loadAddresses() {
    if (!$customer || !$customer.isLoggedIn) {
      goto('/account/login');
      return;
    }
    isLoading = true;
    errorMsg = '';
    try {
      const res = await fetch('/api/v1/customer/addresses', {
        headers: { Authorization: `Bearer ${$customer.token}` }
      });
      if (res.ok) {
        addresses = await res.json();
      } else {
        // Fallback local addresses if guest/token expired
        const local = localStorage.getItem('rustwebshop_saved_addresses');
        if (local) addresses = JSON.parse(local);
      }
    } catch (e) {
      console.error('Failed to load addresses:', e);
      const local = localStorage.getItem('rustwebshop_saved_addresses');
      if (local) addresses = JSON.parse(local);
    } finally {
      isLoading = false;
    }
  }

  async function handleSaveAddress() {
    errorMsg = '';
    successMsg = '';
    const newAddr = {
      address_type: addressType,
      full_name: fullName,
      street_address: streetAddress,
      apartment_suite: apartmentSuite || null,
      city,
      state_province: stateProvince,
      postal_code: postalCode,
      country_code: countryCode,
      is_default: true
    };

    try {
      const res = await fetch('/api/v1/customer/addresses', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${$customer.token}`
        },
        body: JSON.stringify(newAddr)
      });

      if (res.ok) {
        successMsg = `${addressType === 'shipping' ? 'Shipping' : 'Billing'} address saved successfully!`;
        // Also save to localStorage backup
        const local = JSON.parse(localStorage.getItem('rustwebshop_saved_addresses') || '[]');
        local.push({ ...newAddr, id: Date.now().toString(), created_at: new Date().toISOString() });
        localStorage.setItem('rustwebshop_saved_addresses', JSON.stringify(local));

        isAddOpen = false;
        // Reset form
        fullName = '';
        streetAddress = '';
        apartmentSuite = '';
        city = '';
        stateProvince = '';
        postalCode = '';
        await loadAddresses();
        setTimeout(() => successMsg = '', 4000);
      } else {
        errorMsg = 'Failed to save address to server. Saving locally...';
        const local = JSON.parse(localStorage.getItem('rustwebshop_saved_addresses') || '[]');
        local.push({ ...newAddr, id: Date.now().toString(), created_at: new Date().toISOString() });
        localStorage.setItem('rustwebshop_saved_addresses', JSON.stringify(local));
        addresses = local;
        isAddOpen = false;
      }
    } catch (e) {
      console.error('Failed to save address:', e);
      errorMsg = 'Network error while saving address.';
    }
  }

  async function handleDeleteAddress(id) {
    if (!confirm('Are you sure you want to delete this address?')) return;
    try {
      await fetch(`/api/v1/customer/addresses/${id}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${$customer.token}` }
      });
      addresses = addresses.filter((a) => a.id !== id);
      const local = JSON.parse(localStorage.getItem('rustwebshop_saved_addresses') || '[]');
      localStorage.setItem('rustwebshop_saved_addresses', JSON.stringify(local.filter((a) => a.id !== id)));
    } catch (e) {
      console.error('Failed to delete address:', e);
    }
  }

  onMount(() => {
    loadAddresses();
  });
</script>

<svelte:head>
  <title>Saved Addresses & Google Autofill | RustCraft</title>
</svelte:head>

<div class="max-w-4xl mx-auto px-4 py-12 space-y-6">
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <a href="/" class="inline-flex items-center gap-1.5 text-xs text-slate-400 hover:text-white mb-2 transition-colors">
        <ArrowLeft size={14} />
        <span>Back to Store</span>
      </a>
      <h1 class="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
        <MapPin size={24} class="text-emerald-500" />
        Saved Shipping & Billing Addresses
      </h1>
      <p class="text-xs text-slate-400 mt-0.5">
        Manage your delivery and invoice addresses. Browser and Google Autofill are fully enabled.
      </p>
    </div>

    <button
      on:click={() => isAddOpen = true}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold transition-all shadow-md flex items-center gap-1.5 self-start sm:self-auto"
    >
      <Plus size={15} />
      <span>Add New Address</span>
    </button>
  </div>

  {#if successMsg}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <CheckCircle2 size={16} />
      <span>{successMsg}</span>
    </div>
  {/if}

  {#if isAddOpen}
    <div class="p-6 sm:p-8 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-6">
      <div class="flex items-center justify-between border-b border-slate-800 pb-4">
        <h3 class="text-base font-bold text-white flex items-center gap-2">
          <Building size={18} class="text-orange-400" />
          <span>New Address Details</span>
        </h3>
        <button on:click={() => isAddOpen = false} class="text-xs text-slate-400 hover:text-white">
          Cancel
        </button>
      </div>

      <!-- Standard HTML Form with Complete Autocomplete Attributes for Google Chrome -->
      <form on:submit|preventDefault={handleSaveAddress} autocomplete="on" class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
        <div>
          <label for="addr-type" class="block font-semibold text-slate-300 mb-1">Address Classification</label>
          <select id="addr-type" name="address-type" bind:value={addressType} class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500 font-semibold">
            <option value="shipping">📦 Default Shipping Address</option>
            <option value="billing">💳 Default Billing Address</option>
          </select>
        </div>

        <div>
          <label for="full-name" class="block font-semibold text-slate-300 mb-1">Recipient / Company Name</label>
          <input
            id="full-name"
            name="name"
            type="text"
            autocomplete="{addressType === 'shipping' ? 'shipping name' : 'billing name'}"
            bind:value={fullName}
            required
            placeholder="Max Mustermann"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="sm:col-span-2">
          <label for="street-address" class="block font-semibold text-slate-300 mb-1">Street Address</label>
          <input
            id="street-address"
            name="street-address"
            type="text"
            autocomplete="{addressType === 'shipping' ? 'shipping address-line1' : 'billing address-line1'}"
            bind:value={streetAddress}
            required
            placeholder="Rustacean Strasse 42"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="apt-suite" class="block font-semibold text-slate-300 mb-1">Apartment, Suite, Unit (Optional)</label>
          <input
            id="apt-suite"
            name="address-line2"
            type="text"
            autocomplete="{addressType === 'shipping' ? 'shipping address-line2' : 'billing address-line2'}"
            bind:value={apartmentSuite}
            placeholder="Apt 4B / Floor 2"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="city" class="block font-semibold text-slate-300 mb-1">City / Town</label>
          <input
            id="city"
            name="city"
            type="text"
            autocomplete="{addressType === 'shipping' ? 'shipping address-level2' : 'billing address-level2'}"
            bind:value={city}
            required
            placeholder="Berlin"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="postal-code" class="block font-semibold text-slate-300 mb-1">Postal / ZIP Code</label>
          <input
            id="postal-code"
            name="postal-code"
            type="text"
            autocomplete="{addressType === 'shipping' ? 'shipping postal-code' : 'billing postal-code'}"
            bind:value={postalCode}
            required
            placeholder="10115"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
          />
        </div>

        <div>
          <label for="state" class="block font-semibold text-slate-300 mb-1">State / Province</label>
          <input
            id="state"
            name="state"
            type="text"
            autocomplete="{addressType === 'shipping' ? 'shipping address-level1' : 'billing address-level1'}"
            bind:value={stateProvince}
            required
            placeholder="Berlin"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="sm:col-span-2">
          <label for="country" class="block font-semibold text-slate-300 mb-1">Country / Region</label>
          <select
            id="country"
            name="country"
            autocomplete="{addressType === 'shipping' ? 'shipping country' : 'billing country'}"
            bind:value={countryCode}
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          >
            {#each allCountries as c}
              <option value={c.code}>{c.name} ({c.code})</option>
            {/each}
          </select>
        </div>

        <div class="sm:col-span-2 flex items-center justify-end gap-3 pt-3 border-t border-slate-800">
          <button type="button" on:click={() => isAddOpen = false} class="px-4 py-2.5 rounded-xl bg-slate-800 text-slate-300 hover:text-white transition-colors">
            Cancel
          </button>
          <button type="submit" class="px-6 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold transition-all shadow-md">
            Save Address
          </button>
        </div>
      </form>
    </div>
  {/if}

  {#if isLoading}
    <div class="text-center py-16 text-xs text-slate-400">Loading your address books...</div>
  {:else}
    <!-- Section 1: Default Shipping Addresses -->
    <div class="space-y-4">
      <h2 class="text-base font-bold text-white flex items-center gap-2">
        <Home size={18} class="text-orange-400" />
        <span>Shipping Addresses</span>
      </h2>

      {#if shippingAddresses.length === 0}
        <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800 text-center text-xs text-slate-400">
          No shipping address saved yet. Click "Add New Address" above to save one.
        </div>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {#each shippingAddresses as a}
            <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-3 relative group">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-white">{a.full_name}</span>
                <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-orange-500/10 text-orange-400 border border-orange-500/20">
                  Shipping
                </span>
              </div>
              <div class="text-xs text-slate-300 leading-relaxed font-mono">
                <div>{a.street_address} {a.apartment_suite ? `#${a.apartment_suite}` : ''}</div>
                <div>{a.postal_code} {a.city}, {a.state_province}</div>
                <div class="text-slate-400 font-sans mt-1">Country: <strong class="text-white">{a.country_code}</strong></div>
              </div>
              <div class="pt-3 border-t border-slate-800 flex justify-end">
                <button
                  on:click={() => handleDeleteAddress(a.id)}
                  class="text-[11px] text-rose-400 hover:text-rose-300 flex items-center gap-1 transition-colors"
                >
                  <Trash2 size={13} />
                  <span>Delete</span>
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Section 2: Default Billing Addresses -->
    <div class="space-y-4 pt-6 border-t border-slate-800/80">
      <h2 class="text-base font-bold text-white flex items-center gap-2">
        <Building size={18} class="text-sky-400" />
        <span>Billing Addresses</span>
      </h2>

      {#if billingAddresses.length === 0}
        <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800 text-center text-xs text-slate-400">
          No billing address saved yet. You can add one specifically for invoice receipts.
        </div>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {#each billingAddresses as a}
            <div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-3 relative group">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-white">{a.full_name}</span>
                <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-sky-500/10 text-sky-400 border border-sky-500/20">
                  Billing
                </span>
              </div>
              <div class="text-xs text-slate-300 leading-relaxed font-mono">
                <div>{a.street_address} {a.apartment_suite ? `#${a.apartment_suite}` : ''}</div>
                <div>{a.postal_code} {a.city}, {a.state_province}</div>
                <div class="text-slate-400 font-sans mt-1">Country: <strong class="text-white">{a.country_code}</strong></div>
              </div>
              <div class="pt-3 border-t border-slate-800 flex justify-end">
                <button
                  on:click={() => handleDeleteAddress(a.id)}
                  class="text-[11px] text-rose-400 hover:text-rose-300 flex items-center gap-1 transition-colors"
                >
                  <Trash2 size={13} />
                  <span>Delete</span>
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
