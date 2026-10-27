<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { Lock, User, ArrowRight, ShieldAlert, KeyRound, CheckCircle2 } from 'lucide-svelte';

  let username = '';
  let password = '';
  let error = '';
  let loading = false;

  onMount(() => {
    ['admin_token', 'admin_username', 'admin_is_default'].forEach((k) => localStorage.removeItem(k));
  });

  // Only allow redirects to local admin pages (no open redirect)
  function nextPath() {
    const next = new URLSearchParams(window.location.search).get('next') || '/';
    return next.startsWith('/') && !next.startsWith('//') && !next.startsWith('/login') ? next : '/';
  }

  async function handleLogin() {
    error = '';
    loading = true;

    try {
      // The server keeps the session token in an httpOnly cookie; it is never exposed to this page
      const res = await fetch('/api/v1/admin/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password })
      });

      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        error = data.error || data.message || 'Invalid username or password.';
        loading = false;
        return;
      }

      password = '';
      window.location.href = nextPath();
    } catch (e) {
      error = 'Failed to connect to authentication server.';
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Admin Login | RustCraft Back-Office</title>
</svelte:head>

<div class="min-h-screen bg-slate-950 flex flex-col items-center justify-center p-4 selection:bg-orange-500 selection:text-white">
  <div class="w-full max-w-md">
    <!-- Brand Header -->
    <div class="text-center mb-8">
      <div class="w-16 h-16 rounded-2xl bg-gradient-to-tr from-orange-600 to-amber-500 flex items-center justify-center text-3xl mx-auto shadow-2xl shadow-orange-600/30 mb-4">
        🦀
      </div>
      <h1 class="text-2xl font-black text-white tracking-tight">RustCraft Admin Console</h1>
      <p class="text-xs text-slate-400 mt-1">High-Performance Rust Store Management</p>
    </div>

    <!-- Login Card -->
    <div class="p-8 rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl space-y-6">
      <!-- Default Credentials Notice -->
      <div class="p-3.5 rounded-2xl bg-orange-950/30 border border-orange-500/30 text-xs text-slate-300 space-y-1">
        <div class="flex items-center gap-1.5 font-bold text-orange-400">
          <KeyRound size={14} />
          <span>First login?</span>
        </div>
        <p class="text-[11px] text-slate-300">
          Use username <span class="text-white font-bold">admin</span> and the initial password from the backend log
          (<code class="font-mono">docker compose logs backend</code>) or your <code class="font-mono">ADMIN_INITIAL_PASSWORD</code>. You will be asked to change it.
        </p>
      </div>

      {#if error}
        <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-400 text-xs font-semibold flex items-center gap-2">
          <ShieldAlert size={16} />
          <span>{error}</span>
        </div>
      {/if}

      <form on:submit|preventDefault={handleLogin} class="space-y-4">
        <div>
          <label class="block text-xs font-bold text-slate-300 uppercase tracking-wider mb-2">Username</label>
          <div class="relative">
            <input
              type="text"
              bind:value={username}
              required
              class="w-full pl-10 pr-4 py-3 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs placeholder-slate-500 focus:outline-none focus:border-orange-500"
              placeholder="Username"
            />
            <User size={16} class="absolute left-3.5 top-3.5 text-slate-500" />
          </div>
        </div>

        <div>
          <label class="block text-xs font-bold text-slate-300 uppercase tracking-wider mb-2">Password</label>
          <div class="relative">
            <input
              type="password"
              bind:value={password}
              required
              class="w-full pl-10 pr-4 py-3 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs placeholder-slate-500 focus:outline-none focus:border-orange-500"
              placeholder="Password"
            />
            <Lock size={16} class="absolute left-3.5 top-3.5 text-slate-500" />
          </div>
        </div>

        <button
          type="submit"
          disabled={loading}
          class="w-full py-3.5 px-4 rounded-xl bg-gradient-to-r from-orange-600 to-amber-600 hover:from-orange-500 hover:to-amber-500 text-white font-bold text-xs sm:text-sm shadow-xl shadow-orange-600/30 flex items-center justify-center gap-2 transition-all disabled:opacity-50"
        >
          <span>{loading ? 'Authenticating...' : 'Sign In to Console'}</span>
          <ArrowRight size={16} />
        </button>
      </form>
    </div>

    <!-- Security Footnote -->
    <p class="text-center text-[11px] text-slate-500 mt-6">
      Secured with bcrypt password hashing &bull; JWT Authentication
    </p>
  </div>
</div>
