<script>
  import { onMount } from 'svelte';
  import { 
    Tag, Plus, Trash2, Edit2, CheckCircle2, AlertCircle, 
    Percent, Truck, DollarSign, Calendar, Layers, ShieldCheck, X
  } from 'lucide-svelte';

  export let initialCoupons = [];
  let coupons = initialCoupons || [];
  let isLoading = false;
  let successNotice = '';
  let errorNotice = '';

  // Modal State
  let isModalOpen = false;
  let modalMode = 'create'; // 'create' | 'edit'
  let editId = null;
  let isSaving = false;

  // Form Fields
  let code = '';
  let discountType = 'percentage'; // 'free_shipping' | 'fixed_amount' | 'percentage'
  let valueAmount = 10; // in EUR (for fixed_amount) or % (for percentage)
  let minOrderAmount = 0; // in EUR
  let maxUses = ''; // optional number or empty
  let isActive = true;
  let expiresAt = ''; // YYYY-MM-DD or empty

  onMount(async () => {
    if (coupons.length === 0) {
      await reloadCoupons();
    }
  });

  async function reloadCoupons() {
    isLoading = true;
    try {
      const token = localStorage.getItem('admin_token');
      const res = await fetch('/api/v1/admin/coupons', {
        headers: {
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (res.ok) {
        coupons = await res.json();
      }
    } catch (e) {
      console.error('Failed to load coupons:', e);
    } finally {
      isLoading = false;
    }
  }

  function openCreateModal() {
    modalMode = 'create';
    editId = null;
    code = '';
    discountType = 'percentage';
    valueAmount = 10;
    minOrderAmount = 0;
    maxUses = '';
    isActive = true;
    expiresAt = '';
    errorNotice = '';
    isModalOpen = true;
  }

  function openEditModal(c) {
    modalMode = 'edit';
    editId = c.id;
    code = c.code;
    discountType = c.discount_type;
    valueAmount = c.discount_type === 'percentage' ? c.value_cents : (c.value_cents / 100);
    minOrderAmount = (c.min_order_cents || 0) / 100;
    maxUses = c.max_uses !== null && c.max_uses !== undefined ? String(c.max_uses) : '';
    isActive = c.is_active;
    expiresAt = c.expires_at ? c.expires_at.split('T')[0] : '';
    errorNotice = '';
    isModalOpen = true;
  }

  async function handleSaveCoupon() {
    isSaving = true;
    errorNotice = '';

    const cleanCode = code.trim().toUpperCase();
    if (!cleanCode) {
      errorNotice = 'Coupon code cannot be empty.';
      isSaving = false;
      return;
    }

    let valueCents = 0;
    if (discountType === 'percentage') {
      const pct = parseInt(valueAmount) || 0;
      if (pct <= 0 || pct > 100) {
        errorNotice = 'Percentage discount must be between 1% and 100%.';
        isSaving = false;
        return;
      }
      valueCents = pct;
    } else if (discountType === 'fixed_amount') {
      const amt = parseFloat(valueAmount) || 0;
      if (amt <= 0) {
        errorNotice = 'Fixed discount amount must be greater than 0.';
        isSaving = false;
        return;
      }
      valueCents = Math.round(amt * 100);
    }

    const minOrderCents = Math.max(0, Math.round((parseFloat(minOrderAmount) || 0) * 100));
    const parsedMaxUses = maxUses.trim() !== '' ? parseInt(maxUses) : null;
    const parsedExpiresAt = expiresAt ? new Date(expiresAt).toISOString() : null;

    const token = localStorage.getItem('admin_token');
    const headers = {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {})
    };

    try {
      if (modalMode === 'create') {
        const res = await fetch('/api/v1/admin/coupons', {
          method: 'POST',
          headers,
          body: JSON.stringify({
            code: cleanCode,
            discount_type: discountType,
            value_cents: valueCents,
            min_order_cents: minOrderCents,
            max_uses: parsedMaxUses,
            is_active: isActive,
            expires_at: parsedExpiresAt
          })
        });

        if (res.ok) {
          successNotice = `Coupon code "${cleanCode}" created successfully!`;
          isModalOpen = false;
          await reloadCoupons();
          setTimeout(() => successNotice = '', 4000);
        } else {
          const err = await res.text();
          errorNotice = err || 'Failed to create coupon';
        }
      } else {
        const res = await fetch(`/api/v1/admin/coupons/${editId}`, {
          method: 'PUT',
          headers,
          body: JSON.stringify({
            code: cleanCode,
            discount_type: discountType,
            value_cents: valueCents,
            min_order_cents: minOrderCents,
            max_uses: parsedMaxUses,
            is_active: isActive,
            expires_at: parsedExpiresAt
          })
        });

        if (res.ok) {
          successNotice = `Coupon code "${cleanCode}" updated successfully!`;
          isModalOpen = false;
          await reloadCoupons();
          setTimeout(() => successNotice = '', 4000);
        } else {
          const err = await res.text();
          errorNotice = err || 'Failed to update coupon';
        }
      }
    } catch (e) {
      errorNotice = e.message || 'Error saving coupon';
    } finally {
      isSaving = false;
    }
  }

  async function toggleActive(c) {
    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/coupons/${c.id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({
          is_active: !c.is_active
        })
      });
      if (res.ok) {
        c.is_active = !c.is_active;
        coupons = [...coupons];
      }
    } catch (e) {
      console.error('Failed to toggle coupon status:', e);
    }
  }

  async function handleDelete(c) {
    if (!confirm(`Are you sure you want to permanently delete coupon code "${c.code}"?`)) return;

    const token = localStorage.getItem('admin_token');
    try {
      const res = await fetch(`/api/v1/admin/coupons/${c.id}`, {
        method: 'DELETE',
        headers: {
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        }
      });
      if (res.ok) {
        coupons = coupons.filter(item => item.id !== c.id);
        successNotice = `Coupon code "${c.code}" deleted successfully.`;
        setTimeout(() => successNotice = '', 3500);
      }
    } catch (e) {
      console.error('Failed to delete coupon:', e);
    }
  }

  function formatDiscount(c) {
    if (c.discount_type === 'free_shipping') {
      return 'Free Shipping';
    }
    if (c.discount_type === 'percentage') {
      return `${c.value_cents}% Off`;
    }
    if (c.discount_type === 'fixed_amount') {
      return `${(c.value_cents / 100).toFixed(2)} € Off`;
    }
    return 'Discount';
  }
</script>

<div class="space-y-6">
  <!-- Top Banner & Action Button -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
        <Tag size={24} class="text-orange-500" />
        Discount & Promotional Codes
      </h1>
      <p class="text-xs text-slate-400 mt-1">
        Create promo codes for free shipping, fixed currency deductions, or percentage discounts applied during checkout.
      </p>
    </div>

    <button
      type="button"
      on:click={openCreateModal}
      class="px-4 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 flex items-center gap-2 transition-all self-start sm:self-auto"
    >
      <Plus size={16} />
      <span>Create Promo Code</span>
    </button>
  </div>

  {#if successNotice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <CheckCircle2 size={16} />
      <span>{successNotice}</span>
    </div>
  {/if}

  <!-- Coupons List Table -->
  <div class="rounded-2xl bg-slate-900 border border-slate-800 shadow-xl overflow-hidden">
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs">
        <thead class="bg-slate-950/60 border-b border-slate-800 text-slate-400 uppercase tracking-wider text-[11px]">
          <tr>
            <th class="py-3.5 px-4 font-bold">Code</th>
            <th class="py-3.5 px-4 font-bold">Discount Type</th>
            <th class="py-3.5 px-4 font-bold">Value / Deduction</th>
            <th class="py-3.5 px-4 font-bold">Min. Order</th>
            <th class="py-3.5 px-4 font-bold">Uses / Limit</th>
            <th class="py-3.5 px-4 font-bold">Expires</th>
            <th class="py-3.5 px-4 font-bold text-center">Status</th>
            <th class="py-3.5 px-4 font-bold text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#if coupons.length === 0}
            <tr>
              <td colspan="8" class="py-12 text-center text-slate-500">
                <Tag size={32} class="mx-auto mb-2 text-slate-600" />
                <p class="font-bold text-slate-300">No promo codes created yet</p>
                <p class="text-[11px] text-slate-500 mt-0.5">Click "Create Promo Code" above to launch your first promotional campaign.</p>
              </td>
            </tr>
          {:else}
            {#each coupons as c}
              <tr class="hover:bg-slate-800/40 transition-colors">
                <!-- Code -->
                <td class="py-3.5 px-4">
                  <div class="font-mono font-black text-white text-sm tracking-wider flex items-center gap-1.5">
                    <span class="px-2.5 py-1 rounded-lg bg-orange-600/15 border border-orange-500/30 text-orange-400">
                      {c.code}
                    </span>
                  </div>
                </td>

                <!-- Discount Type Badge -->
                <td class="py-3.5 px-4">
                  {#if c.discount_type === 'free_shipping'}
                    <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-sky-500/10 text-sky-400 border border-sky-500/20">
                      <Truck size={11} /> Free Shipping
                    </span>
                  {:else if c.discount_type === 'fixed_amount'}
                    <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                      <span>€</span> Fixed Currency
                    </span>
                  {:else}
                    <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-purple-500/10 text-purple-400 border border-purple-500/20">
                      <Percent size={11} /> Percentage
                    </span>
                  {/if}
                </td>

                <!-- Value / Deduction -->
                <td class="py-3.5 px-4 font-mono font-bold text-slate-200">
                  {formatDiscount(c)}
                </td>

                <!-- Min Order -->
                <td class="py-3.5 px-4 text-slate-300">
                  {#if c.min_order_cents > 0}
                    <span class="font-mono">{(c.min_order_cents / 100).toFixed(2)} €</span>
                  {:else}
                    <span class="text-slate-500 italic">None</span>
                  {/if}
                </td>

                <!-- Uses / Limit -->
                <td class="py-3.5 px-4 text-slate-300 font-mono">
                  <span class="font-bold text-white">{c.used_count || 0}</span>
                  <span class="text-slate-500">/</span>
                  {#if c.max_uses}
                    <span>{c.max_uses}</span>
                  {:else}
                    <span class="text-slate-500 text-[11px]">Unlimited</span>
                  {/if}
                </td>

                <!-- Expires -->
                <td class="py-3.5 px-4 text-slate-400 text-[11px]">
                  {#if c.expires_at}
                    <span>{new Date(c.expires_at).toLocaleDateString()}</span>
                  {:else}
                    <span class="text-slate-500">Never</span>
                  {/if}
                </td>

                <!-- Status Active Toggle -->
                <td class="py-3.5 px-4 text-center">
                  <button
                    type="button"
                    on:click={() => toggleActive(c)}
                    class="px-2.5 py-1 rounded-full text-[10px] font-bold transition-all {c.is_active ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30' : 'bg-slate-800 text-slate-500 border border-slate-700'}"
                  >
                    {c.is_active ? 'Active' : 'Inactive'}
                  </button>
                </td>

                <!-- Actions -->
                <td class="py-3.5 px-4 text-right">
                  <div class="inline-flex items-center gap-1.5">
                    <button
                      type="button"
                      on:click={() => openEditModal(c)}
                      title="Edit Promo Code"
                      class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
                    >
                      <Edit2 size={14} />
                    </button>
                    <button
                      type="button"
                      on:click={() => handleDelete(c)}
                      title="Delete Promo Code"
                      class="p-1.5 rounded-lg text-rose-400 hover:text-rose-300 hover:bg-rose-500/15 transition-colors"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<!-- Modal: Create / Edit Promo Code -->
{#if isModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="w-full max-w-lg rounded-2xl bg-slate-900 border border-slate-800 shadow-2xl p-6 space-y-5">
      <div class="flex items-center justify-between border-b border-slate-800 pb-3">
        <h2 class="text-base font-bold text-white flex items-center gap-2">
          <Tag size={18} class="text-orange-400" />
          <span>{modalMode === 'create' ? 'Create New Promo Code' : 'Edit Promo Code'}</span>
        </h2>
        <button
          type="button"
          on:click={() => isModalOpen = false}
          class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800"
        >
          <X size={16} />
        </button>
      </div>

      {#if errorNotice}
        <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={15} />
          <span>{errorNotice}</span>
        </div>
      {/if}

      <div class="space-y-4 text-xs">
        <!-- Code Field -->
        <div>
          <label for="coupon-code-input" class="block font-semibold text-slate-300 mb-1">Coupon Code (Uppercase)</label>
          <input
            id="coupon-code-input"
            type="text"
            bind:value={code}
            on:input={(e) => code = e.target.value.toUpperCase().replace(/[^A-Z0-9_-]/g, '')}
            required
            placeholder="e.g. SUMMER20 or FREESHIP"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono font-bold tracking-wider text-sm focus:outline-none focus:border-orange-500 uppercase"
          />
        </div>

        <!-- Discount Type Selector (3 Options) -->
        <div>
          <label class="block font-semibold text-slate-300 mb-2">Discount Type</label>
          <div class="grid grid-cols-3 gap-2">
            <button
              type="button"
              on:click={() => discountType = 'free_shipping'}
              class="p-3 rounded-xl border text-center transition-all {discountType === 'free_shipping' ? 'bg-orange-600/15 border-orange-500 text-white ring-1 ring-orange-500 font-bold' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}"
            >
              <Truck size={18} class="mx-auto mb-1 text-sky-400" />
              <div class="text-[11px]">Free Shipping</div>
            </button>

            <button
              type="button"
              on:click={() => discountType = 'fixed_amount'}
              class="p-3 rounded-xl border text-center transition-all {discountType === 'fixed_amount' ? 'bg-orange-600/15 border-orange-500 text-white ring-1 ring-orange-500 font-bold' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}"
            >
              <div class="text-base font-black text-emerald-400 mb-0.5">€</div>
              <div class="text-[11px]">Fixed Amount (€)</div>
            </button>

            <button
              type="button"
              on:click={() => discountType = 'percentage'}
              class="p-3 rounded-xl border text-center transition-all {discountType === 'percentage' ? 'bg-orange-600/15 border-orange-500 text-white ring-1 ring-orange-500 font-bold' : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700'}"
            >
              <Percent size={18} class="mx-auto mb-1 text-purple-400" />
              <div class="text-[11px]">Percentage (%)</div>
            </button>
          </div>
        </div>

        <!-- Value Input (Hidden if Free Shipping) -->
        {#if discountType !== 'free_shipping'}
          <div>
            <label for="coupon-value-input" class="block font-semibold text-slate-300 mb-1">
              {discountType === 'percentage' ? 'Discount Percentage (%)' : 'Discount Amount (EUR €)'}
            </label>
            <div class="relative">
              <input
                id="coupon-value-input"
                type="number"
                step={discountType === 'percentage' ? '1' : '0.01'}
                min="0"
                max={discountType === 'percentage' ? '100' : '9999'}
                bind:value={valueAmount}
                required
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
              />
              <span class="absolute right-3.5 top-1/2 -translate-y-1/2 text-slate-500 font-bold">
                {discountType === 'percentage' ? '%' : '€'}
              </span>
            </div>
          </div>
        {/if}

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <!-- Minimum Order Amount -->
          <div>
            <label for="coupon-min-order" class="block font-semibold text-slate-300 mb-1">Minimum Order Subtotal (€)</label>
            <input
              id="coupon-min-order"
              type="number"
              step="0.01"
              min="0"
              bind:value={minOrderAmount}
              placeholder="0.00"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
            <p class="text-[10px] text-slate-500 mt-1">Leave 0 for no minimum subtotal requirement.</p>
          </div>

          <!-- Max Uses Limit -->
          <div>
            <label for="coupon-max-uses" class="block font-semibold text-slate-300 mb-1">Usage Limit (Max Uses)</label>
            <input
              id="coupon-max-uses"
              type="number"
              min="1"
              bind:value={maxUses}
              placeholder="Unlimited"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
            <p class="text-[10px] text-slate-500 mt-1">Leave blank for unlimited redemptions.</p>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <!-- Expiration Date -->
          <div>
            <label for="coupon-expires" class="block font-semibold text-slate-300 mb-1">Expiration Date (Optional)</label>
            <input
              id="coupon-expires"
              type="date"
              bind:value={expiresAt}
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>

          <!-- Active Toggle -->
          <div class="flex flex-col justify-center">
            <span class="block font-semibold text-slate-300 mb-1">Active Status</span>
            <label class="inline-flex items-center gap-2 cursor-pointer mt-1">
              <input
                type="checkbox"
                bind:checked={isActive}
                class="w-4 h-4 rounded text-orange-600 bg-slate-950 border-slate-800 focus:ring-0"
              />
              <span class="text-xs text-slate-300">Code is currently active</span>
            </label>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-3 border-t border-slate-800 pt-4">
        <button
          type="button"
          on:click={() => isModalOpen = false}
          class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold"
        >
          Cancel
        </button>
        <button
          type="button"
          on:click={handleSaveCoupon}
          disabled={isSaving}
          class="px-4 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 text-white text-xs font-bold shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50"
        >
          {isSaving ? 'Saving...' : (modalMode === 'create' ? 'Create Promo Code' : 'Save Changes')}
        </button>
      </div>
    </div>
  </div>
{/if}
