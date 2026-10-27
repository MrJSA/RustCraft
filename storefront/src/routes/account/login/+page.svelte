<script>
  import { onDestroy } from 'svelte';
  import { customer } from '$lib/stores/customer.js';
  import { goto } from '$app/navigation';
  import { LogIn, UserPlus, Lock, Mail, User, AlertCircle, CheckCircle2, RotateCw } from 'lucide-svelte';

  let activeTab = 'login'; // 'login' | 'register'
  let email = '';
  let password = '';
  let fullName = '';
  let errorMsg = '';
  let successMsg = '';
  let isLoading = false;
  let needsVerification = false;
  let isResending = false;
  let resendCooldown = 0;
  let cooldownTimer = null;

  function checkStoredCooldown() {
    if (!email) return;
    try {
      const stored = sessionStorage.getItem('resend_until_' + email.trim().toLowerCase());
      if (stored) {
        const remaining = Math.ceil((parseInt(stored, 10) - Date.now()) / 1000);
        if (remaining > 0) {
          startCooldown(remaining);
        } else {
          sessionStorage.removeItem('resend_until_' + email.trim().toLowerCase());
        }
      }
    } catch (e) {}
  }

  $: if (email) {
    checkStoredCooldown();
  }

  function startCooldown(seconds = 60) {
    if (cooldownTimer) clearInterval(cooldownTimer);
    resendCooldown = seconds;
    try {
      sessionStorage.setItem('resend_until_' + email.trim().toLowerCase(), (Date.now() + seconds * 1000).toString());
    } catch (e) {}

    cooldownTimer = setInterval(() => {
      resendCooldown -= 1;
      if (resendCooldown <= 0) {
        clearInterval(cooldownTimer);
        cooldownTimer = null;
        resendCooldown = 0;
        try {
          sessionStorage.removeItem('resend_until_' + email.trim().toLowerCase());
        } catch (e) {}
      }
    }, 1000);
  }

  onDestroy(() => {
    if (cooldownTimer) clearInterval(cooldownTimer);
  });

  async function resendVerification() {
    if (resendCooldown > 0 || isResending || !email) return;
    isResending = true;
    errorMsg = '';
    successMsg = '';
    try {
      const res = await fetch('/api/v1/auth/customer/resend-verification', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email })
      });
      const data = await res.json().catch(() => ({}));
      if (res.ok) {
        startCooldown(data.cooldown_seconds || 60);
        successMsg = data.message;
      } else {
        errorMsg = data.error || 'The email could not be sent. Please try again later.';
        if (res.status === 429) {
          startCooldown(60);
        }
      }
    } catch (e) {
      errorMsg = 'Network error. Please try again.';
    } finally {
      isResending = false;
    }
  }

  async function handleLogin() {
    errorMsg = '';
    isLoading = true;
    try {
      const res = await fetch('/api/v1/auth/customer/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
      });
      const data = await res.json();
      if (!res.ok) {
        errorMsg = data.error || 'Invalid email or password';
        // 403 = correct password but email not verified yet
        needsVerification = res.status === 403;
        if (needsVerification) {
          checkStoredCooldown();
        }
      } else {
        customer.login(data.token, data.email, data.full_name);
        goto('/');
      }
    } catch (e) {
      errorMsg = 'Network error. Please try again.';
    } finally {
      isLoading = false;
    }
  }

  async function handleRegister() {
    errorMsg = '';
    isLoading = true;
    try {
      const res = await fetch('/api/v1/auth/customer/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password, full_name: fullName })
      });
      const data = await res.json();
      if (!res.ok) {
        errorMsg = data.error || 'Registration failed. Email may already be in use.';
      } else if (data.verification_pending || !data.token) {
        activeTab = 'login';
        password = '';
        needsVerification = true;
        if (data.verification_email_sent === false) {
          errorMsg = 'Your account was created, but the verification email could not be sent right now. Please try "Resend verification email" later or contact the shop.';
        } else {
          startCooldown(60);
          successMsg = `Account created! We sent a verification link to ${data.email}. Please confirm your email address, then log in.`;
        }
      } else {
        customer.login(data.token, data.email, data.full_name);
        goto('/');
      }
    } catch (e) {
      errorMsg = 'Network error. Please try again.';
    } finally {
      isLoading = false;
    }
  }
</script>

<svelte:head>
  <title>Customer Login & Register | RustCraft</title>
</svelte:head>

<div class="max-w-md mx-auto px-4 py-16">
  <div class="p-8 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-6">
    <!-- Header Tabs -->
    <div class="flex items-center rounded-xl bg-slate-950 p-1 border border-slate-800">
      <button
        type="button"
        on:click={() => { activeTab = 'login'; errorMsg = ''; }}
        class="flex-1 py-2 rounded-lg text-xs font-bold transition-all flex items-center justify-center gap-1.5 {activeTab === 'login' ? 'bg-orange-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
      >
        <LogIn size={14} />
        <span>Login</span>
      </button>
      <button
        type="button"
        on:click={() => { activeTab = 'register'; errorMsg = ''; }}
        class="flex-1 py-2 rounded-lg text-xs font-bold transition-all flex items-center justify-center gap-1.5 {activeTab === 'register' ? 'bg-orange-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
      >
        <UserPlus size={14} />
        <span>Register</span>
      </button>
    </div>

    <!-- Error Alert -->
    {#if errorMsg}
      <div role="alert" class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-2">
        <AlertCircle size={16} class="flex-shrink-0" />
        <span>{errorMsg}</span>
      </div>
    {/if}

    {#if successMsg}
      <div role="status" class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs flex items-center gap-2">
        <CheckCircle2 size={16} class="flex-shrink-0" />
        <span>{successMsg}</span>
      </div>
    {/if}

    {#if needsVerification}
      <div class="p-3.5 rounded-2xl bg-slate-950/80 border border-slate-800 space-y-2.5">
        <div class="text-[11px] text-slate-400 leading-relaxed">
          Need a new verification link? You can request another email once every 1 minute.
        </div>
        <button
          type="button"
          on:click={resendVerification}
          disabled={isResending || !email || resendCooldown > 0}
          class="w-full py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 border border-slate-700"
        >
          {#if isResending}
            <RotateCw size={13} class="animate-spin text-orange-400" />
            <span>Sending verification email…</span>
          {:else if resendCooldown > 0}
            <span>Resend verification email in {resendCooldown}s</span>
          {:else}
            <span>Resend verification email</span>
          {/if}
        </button>
      </div>
    {/if}

    {#if activeTab === 'login'}
      <!-- Login Form -->
      <form on:submit|preventDefault={handleLogin} class="space-y-4">
        <div>
          <label class="block text-xs font-semibold text-slate-300 mb-1">Email Address</label>
          <div class="relative">
            <input
              type="email"
              bind:value={email}
              required
              placeholder="customer@example.com"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <Mail size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="block text-xs font-semibold text-slate-300">Password</label>
            <a href="/account/reset-password" class="text-[11px] text-orange-400 hover:underline">
              Lost password?
            </a>
          </div>
          <div class="relative">
            <input
              type="password"
              bind:value={password}
              required
              placeholder="••••••••"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <Lock size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <button
          type="submit"
          disabled={isLoading}
          class="w-full py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50"
        >
          {isLoading ? 'Signing In...' : 'Sign In to Account'}
        </button>
      </form>
    {:else}
      <!-- Register Form -->
      <form on:submit|preventDefault={handleRegister} class="space-y-4">
        <div>
          <label class="block text-xs font-semibold text-slate-300 mb-1">Full Name</label>
          <div class="relative">
            <input
              type="text"
              bind:value={fullName}
              required
              placeholder="Max Mustermann"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <User size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <div>
          <label class="block text-xs font-semibold text-slate-300 mb-1">Email Address</label>
          <div class="relative">
            <input
              type="email"
              bind:value={email}
              required
              placeholder="customer@example.com"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <Mail size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <div>
          <label class="block text-xs font-semibold text-slate-300 mb-1">Create Password</label>
          <div class="relative">
            <input
              type="password"
              bind:value={password}
              required
              minlength="6"
              placeholder="••••••••"
              class="w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500"
            />
            <Lock size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
        </div>

        <button
          type="submit"
          disabled={isLoading}
          class="w-full py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50"
        >
          {isLoading ? 'Creating Account...' : 'Create Customer Account'}
        </button>
      </form>
    {/if}
  </div>
</div>
