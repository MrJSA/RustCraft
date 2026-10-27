<script>
  import { onMount } from 'svelte';
  import { customer } from '$lib/stores/customer.js';
  import { goto } from '$app/navigation';
  import { Settings, ArrowLeft, User, Mail, Lock, CheckCircle2, AlertCircle, Coins, Save } from 'lucide-svelte';

  let profile = {};
  let firstName = '';
  let lastName = '';
  let displayName = '';
  let email = '';
  let preferredCurrency = 'EUR';
  let phone = '';

  // Password change state
  let currentPassword = '';
  let newPassword = '';
  let confirmPassword = '';

  let profileMessage = '';
  let profileError = '';
  let passwordMessage = '';
  let passwordError = '';

  let isLoading = true;
  let isSavingProfile = false;
  let isSavingPassword = false;

  async function loadProfile() {
    if (!$customer || !$customer.isLoggedIn) {
      goto('/account/login');
      return;
    }
    try {
      const res = await fetch('/api/v1/customer/profile', {
        headers: { Authorization: `Bearer ${$customer.token}` }
      });
      if (res.ok) {
        profile = await res.json();
        email = profile.email || $customer.email || '';
        firstName = profile.first_name || '';
        lastName = profile.last_name || '';
        displayName = profile.display_name || $customer.full_name || '';
        preferredCurrency = profile.preferred_currency || 'EUR';
        phone = profile.phone || '';
      }
    } catch (e) {
      console.error('Failed to load profile:', e);
    } finally {
      isLoading = false;
    }
  }

  async function handleUpdateProfile() {
    isSavingProfile = true;
    profileMessage = '';
    profileError = '';

    try {
      const res = await fetch('/api/v1/customer/profile', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${$customer.token}`
        },
        body: JSON.stringify({
          first_name: firstName,
          last_name: lastName,
          display_name: displayName,
          email,
          preferred_currency: preferredCurrency,
          phone
        })
      });

      if (res.ok) {
        profileMessage = 'Profile details updated successfully!';
        // Update customer store
        customer.login($customer.token, email, displayName);
        setTimeout(() => profileMessage = '', 4000);
      } else {
        const err = await res.json().catch(() => ({}));
        profileError = err.message || 'Failed to update account details.';
      }
    } catch (e) {
      profileError = 'Network error while updating profile.';
    } finally {
      isSavingProfile = false;
    }
  }

  async function handleUpdatePassword() {
    isSavingPassword = true;
    passwordMessage = '';
    passwordError = '';

    if (!currentPassword) {
      passwordError = 'Please enter your current password.';
      isSavingPassword = false;
      return;
    }
    if (!newPassword || newPassword.length < 6) {
      passwordError = 'New password must be at least 6 characters.';
      isSavingPassword = false;
      return;
    }
    if (newPassword !== confirmPassword) {
      passwordError = 'New password and confirmation password do not match.';
      isSavingPassword = false;
      return;
    }

    try {
      const res = await fetch('/api/v1/customer/change-password', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${$customer.token}`
        },
        body: JSON.stringify({
          current_password: currentPassword,
          new_password: newPassword,
          confirm_password: confirmPassword
        })
      });

      if (res.ok) {
        passwordMessage = 'Password successfully updated!';
        currentPassword = '';
        newPassword = '';
        confirmPassword = '';
        setTimeout(() => passwordMessage = '', 4000);
      } else {
        const err = await res.text();
        passwordError = err || 'Incorrect current password or update failed.';
      }
    } catch (e) {
      passwordError = 'Network error updating password.';
    } finally {
      isSavingPassword = false;
    }
  }

  onMount(() => {
    loadProfile();
  });
</script>

<svelte:head>
  <title>Account Details & Currency | RustCraft</title>
</svelte:head>

<div class="max-w-2xl mx-auto px-4 py-12 space-y-8">
  <div>
    <a href="/" class="inline-flex items-center gap-1.5 text-xs text-slate-400 hover:text-white mb-2 transition-colors">
      <ArrowLeft size={14} />
      <span>Back to Store</span>
    </a>
    <h1 class="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
      <Settings size={24} class="text-sky-500" />
      Account Details & Security
    </h1>
    <p class="text-xs text-slate-400 mt-0.5">
      Manage your personal name, display name, preferred currency, and secure password.
    </p>
  </div>

  {#if isLoading}
    <div class="text-center py-16 text-xs text-slate-400">Loading your profile details...</div>
  {:else}
    <!-- Profile & Currency Form -->
    <div class="p-6 sm:p-8 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-6">
      <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
        <User size={18} class="text-orange-400" />
        <span>Personal & Shopping Profile</span>
      </h2>

      {#if profileMessage}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <CheckCircle2 size={16} />
          <span>{profileMessage}</span>
        </div>
      {/if}

      {#if profileError}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={16} />
          <span>{profileError}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleUpdateProfile} class="space-y-4 text-xs">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label for="first-name" class="block font-semibold text-slate-300 mb-1">First Name</label>
            <input
              id="first-name"
              type="text"
              bind:value={firstName}
              placeholder="Max"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>

          <div>
            <label for="last-name" class="block font-semibold text-slate-300 mb-1">Last Name</label>
            <input
              id="last-name"
              type="text"
              bind:value={lastName}
              placeholder="Rust"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div>
          <label for="display-name" class="block font-semibold text-slate-300 mb-1">Display Name (Visible in Account & Reviews)</label>
          <input
            id="display-name"
            type="text"
            bind:value={displayName}
            required
            placeholder="Max M."
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label for="account-email" class="block font-semibold text-slate-300 mb-1">Email Address</label>
            <input
              id="account-email"
              type="email"
              bind:value={email}
              required
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono focus:outline-none focus:border-orange-500"
            />
          </div>

          <div>
            <label for="pref-currency" class="block font-semibold text-slate-300 mb-1 flex items-center gap-1.5">
              <Coins size={14} class="text-amber-400" />
              <span>Preferred Currency</span>
            </label>
            <select
              id="pref-currency"
              bind:value={preferredCurrency}
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white font-bold focus:outline-none focus:border-orange-500"
            >
              <option value="EUR">EUR (€) - Euro</option>
              <option value="USD">USD ($) - US Dollar</option>
              <option value="GBP">GBP (£) - British Pound</option>
              <option value="CHF">CHF (Fr.) - Swiss Franc</option>
              <option value="JPY">JPY (¥) - Japanese Yen</option>
            </select>
          </div>
        </div>

        <div class="flex justify-end pt-2">
          <button
            type="submit"
            disabled={isSavingProfile}
            class="px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-md transition-all flex items-center gap-2 disabled:opacity-50"
          >
            <Save size={15} />
            <span>{isSavingProfile ? 'Saving Changes...' : 'Save Profile Changes'}</span>
          </button>
        </div>
      </form>
    </div>

    <!-- Password Change Section -->
    <div class="p-6 sm:p-8 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-6">
      <h2 class="text-base font-bold text-white border-b border-slate-800 pb-3 flex items-center gap-2">
        <Lock size={18} class="text-amber-400" />
        <span>Change Password</span>
      </h2>

      <p class="text-xs text-slate-400">
        To change your password, you must enter your current password, followed by your new password twice.
      </p>

      {#if passwordMessage}
        <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
          <CheckCircle2 size={16} />
          <span>{passwordMessage}</span>
        </div>
      {/if}

      {#if passwordError}
        <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <AlertCircle size={16} />
          <span>{passwordError}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleUpdatePassword} class="space-y-4 text-xs">
        <div>
          <label for="current-password" class="block font-semibold text-slate-300 mb-1">Current Password</label>
          <input
            id="current-password"
            type="password"
            bind:value={currentPassword}
            required
            placeholder="Enter your current password"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
          />
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label for="new-password" class="block font-semibold text-slate-300 mb-1">New Password (min. 6 chars)</label>
            <input
              id="new-password"
              type="password"
              bind:value={newPassword}
              required
              minlength="6"
              placeholder="Enter new password"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>

          <div>
            <label for="confirm-password" class="block font-semibold text-slate-300 mb-1">Confirm New Password</label>
            <input
              id="confirm-password"
              type="password"
              bind:value={confirmPassword}
              required
              minlength="6"
              placeholder="Re-enter new password"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white focus:outline-none focus:border-orange-500"
            />
          </div>
        </div>

        <div class="flex justify-end pt-2">
          <button
            type="submit"
            disabled={isSavingPassword}
            class="px-5 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs shadow-md transition-all flex items-center gap-2 disabled:opacity-50"
          >
            <Lock size={15} class="text-orange-400" />
            <span>{isSavingPassword ? 'Updating Password...' : 'Change Password'}</span>
          </button>
        </div>
      </form>
    </div>
  {/if}
</div>
