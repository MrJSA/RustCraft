<script>
  import { page } from '$app/stores';
  import { KeyRound, Mail, CheckCircle2, ArrowLeft, Lock, AlertCircle } from 'lucide-svelte';

  // Step 1 (no token): request a reset email. Step 2 (?token=… from the email): choose a new password.
  $: token = $page.url.searchParams.get('token') || '';

  let email = '';
  let newPassword = '';
  let confirmPassword = '';
  let isSubmitted = false;
  let isLoading = false;
  let errorMessage = '';
  let successMessage = '';

  async function readError(res) {
    const text = await res.text();
    try {
      return JSON.parse(text).error || text;
    } catch (_) {
      return text || 'Something went wrong. Please try again.';
    }
  }

  async function requestReset() {
    isLoading = true;
    errorMessage = '';
    try {
      await fetch('/api/v1/auth/customer/reset-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email })
      });
    } catch (_) {
      // The response is identical whether or not the account exists
    }
    isSubmitted = true;
    isLoading = false;
  }

  async function setNewPassword() {
    errorMessage = '';
    if (newPassword.length < 8) {
      errorMessage = 'The password must be at least 8 characters long.';
      return;
    }
    if (newPassword !== confirmPassword) {
      errorMessage = 'The passwords do not match.';
      return;
    }
    isLoading = true;
    try {
      const res = await fetch('/api/v1/auth/customer/reset-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token, new_password: newPassword })
      });
      if (!res.ok) throw new Error(await readError(res));
      successMessage = 'Your password has been changed. You can now log in.';
      newPassword = '';
      confirmPassword = '';
    } catch (e) {
      errorMessage = e.message;
    } finally {
      isLoading = false;
    }
  }

  const inputClass =
    'w-full pl-10 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white placeholder-slate-500 text-xs focus:outline-none focus:border-orange-500';
</script>

<svelte:head>
  <title>Reset Password</title>
</svelte:head>

<div class="max-w-md mx-auto px-4 py-16">
  <div class="p-8 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-6">
    <a href="/account/login" class="inline-flex items-center gap-1.5 text-xs text-slate-400 hover:text-white transition-colors">
      <ArrowLeft size={14} />
      <span>Back to Login</span>
    </a>

    <div class="text-center">
      <div class="w-12 h-12 rounded-2xl bg-orange-500/10 border border-orange-500/20 text-orange-400 flex items-center justify-center mx-auto mb-3">
        <KeyRound size={24} />
      </div>
      <h1 class="text-xl font-bold text-white">{token ? 'Choose a New Password' : 'Reset Password'}</h1>
      <p class="text-xs text-slate-400 mt-1">
        {token
          ? 'Enter your new password below.'
          : "Enter your account's email address and we'll send you a link to reset your password."}
      </p>
    </div>

    {#if errorMessage}
      <div role="alert" class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs flex items-center gap-2">
        <AlertCircle size={16} /> <span>{errorMessage}</span>
      </div>
    {/if}

    {#if token}
      {#if successMessage}
        <div class="p-4 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs space-y-3 text-center">
          <CheckCircle2 size={24} class="mx-auto" />
          <p class="font-bold">{successMessage}</p>
          <a href="/account/login" class="inline-block px-4 py-2 rounded-xl bg-orange-600 text-white font-bold">Log in</a>
        </div>
      {:else}
        <form on:submit|preventDefault={setNewPassword} class="space-y-4">
          <div class="relative">
            <label for="new-password" class="sr-only">New password</label>
            <input id="new-password" type="password" autocomplete="new-password" minlength="8" required bind:value={newPassword} placeholder="New password (min. 8 characters)" class={inputClass} />
            <Lock size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
          <div class="relative">
            <label for="confirm-password" class="sr-only">Confirm new password</label>
            <input id="confirm-password" type="password" autocomplete="new-password" minlength="8" required bind:value={confirmPassword} placeholder="Repeat new password" class={inputClass} />
            <Lock size={16} class="absolute left-3.5 top-3 text-slate-500" />
          </div>
          <button type="submit" disabled={isLoading} class="w-full py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50">
            {isLoading ? 'Saving…' : 'Save New Password'}
          </button>
        </form>
      {/if}
    {:else if isSubmitted}
      <div class="p-4 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs space-y-2 text-center">
        <CheckCircle2 size={24} class="mx-auto" />
        <p class="font-bold">Check your inbox</p>
        <p class="text-slate-300">If an account exists for <span class="font-mono text-white">{email}</span>, you will receive a reset link valid for 1 hour.</p>
      </div>
    {:else}
      <form on:submit|preventDefault={requestReset} class="space-y-4">
        <div class="relative">
          <label for="reset-email" class="sr-only">Email address</label>
          <input id="reset-email" type="email" autocomplete="email" bind:value={email} required placeholder="customer@example.com" class={inputClass} />
          <Mail size={16} class="absolute left-3.5 top-3 text-slate-500" />
        </div>
        <button type="submit" disabled={isLoading} class="w-full py-3 rounded-xl bg-orange-600 hover:bg-orange-500 text-white font-bold text-xs shadow-lg shadow-orange-600/30 transition-all disabled:opacity-50">
          {isLoading ? 'Sending link…' : 'Send Password Reset Link'}
        </button>
      </form>
    {/if}
  </div>
</div>
