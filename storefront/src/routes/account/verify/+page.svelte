<script>
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { CheckCircle2, XCircle, ArrowRight, Loader2 } from 'lucide-svelte';

  let token = '';
  let status = 'loading'; // 'loading', 'success', 'error'
  let message = '';

  onMount(async () => {
    token = $page.url.searchParams.get('token') || '';
    if (!token) {
      status = 'error';
      message = 'No verification token provided.';
      return;
    }

    try {
      const res = await fetch(`/api/v1/customer/verify?token=${encodeURIComponent(token)}`);
      const data = await res.json();
      if (res.ok) {
        status = 'success';
        message = data.message || 'Your email address has been verified successfully!';
      } else {
        status = 'error';
        message = data.message || data.error || 'Verification link is invalid or has expired.';
      }
    } catch (e) {
      status = 'error';
      message = 'Failed to connect to verification server.';
    }
  });
</script>

<svelte:head>
  <title>Account Verification | RustCraft</title>
</svelte:head>

<div class="max-w-md mx-auto px-4 py-20 text-center">
  <div class="p-8 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl">
    {#if status === 'loading'}
      <div class="w-16 h-16 rounded-full bg-slate-800 flex items-center justify-center mx-auto mb-4 text-orange-400">
        <Loader2 size={32} class="animate-spin" />
      </div>
      <h1 class="text-xl font-bold text-white">Verifying Account...</h1>
      <p class="text-xs text-slate-400 mt-2">Please wait while we validate your email verification token.</p>

    {:else if status === 'success'}
      <div class="w-16 h-16 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center mx-auto mb-4 border border-emerald-500/30">
        <CheckCircle2 size={36} />
      </div>
      <h1 class="text-2xl font-black text-white">Account Verified!</h1>
      <p class="text-xs text-slate-300 mt-2 leading-relaxed">{message}</p>

      <div class="mt-8">
        <a
          href="/account/login"
          class="w-full py-3.5 px-6 rounded-xl bg-gradient-to-r from-orange-600 to-amber-600 hover:from-orange-500 hover:to-amber-500 text-white font-bold text-sm shadow-xl shadow-orange-600/30 flex items-center justify-center gap-2 transition-all"
        >
          <span>Log In to Your Account</span>
          <ArrowRight size={16} />
        </a>
      </div>

    {:else}
      <div class="w-16 h-16 rounded-full bg-rose-500/20 text-rose-400 flex items-center justify-center mx-auto mb-4 border border-rose-500/30">
        <XCircle size={36} />
      </div>
      <h1 class="text-2xl font-black text-white">Verification Failed</h1>
      <p class="text-xs text-rose-300 mt-2 leading-relaxed">{message}</p>

      <div class="mt-8 flex flex-col gap-2">
        <a
          href="/account/login"
          class="w-full py-3 px-4 rounded-xl bg-slate-800 hover:bg-slate-700 text-white font-bold text-xs transition-colors"
        >
          Back to Login
        </a>
      </div>
    {/if}
  </div>
</div>
