<script>
  import { onMount } from 'svelte';
  import { Shield, Settings, Check, X, Info } from 'lucide-svelte';

  export let store = {};

  let isBannerVisible = false;
  let isPreferencesModalOpen = false;

  let consent = {
    necessary: true,
    analytics: false,
    marketing: false,
    timestamp: null
  };

  $: title = store.cookie_banner_title || 'We respect your privacy';
  $: description = store.cookie_banner_description || 'We use cookies and similar technologies to enhance your browsing experience, analyze site traffic, and personalize content in accordance with EU GDPR.';
  $: policyUrl = store.cookie_banner_policy_url || '/policies/cookie-policy';
  $: acceptLabel = store.cookie_accept_label || 'Accept All';
  $: denyLabel = store.cookie_deny_label || 'Decline Optional';
  $: preferencesLabel = store.cookie_preferences_label || 'Preferences';

  onMount(() => {
    // Check if user already consented
    const stored = localStorage.getItem('rustcraft_cookie_consent');
    if (stored) {
      try {
        consent = JSON.parse(stored);
      } catch (e) {
        consent.timestamp = null;
      }
    }

    if (!consent.timestamp && store.cookie_banner_enabled !== false) {
      // Delay slightly for smooth entrance animation
      setTimeout(() => {
        isBannerVisible = true;
      }, 500);
    }

    // Global listener for "Cookie Preferences" link in footer
    const handleOpenPrefs = () => {
      isPreferencesModalOpen = true;
    };
    window.addEventListener('open-cookie-preferences', handleOpenPrefs);

    return () => {
      window.removeEventListener('open-cookie-preferences', handleOpenPrefs);
    };
  });

  function saveConsent(newConsent) {
    consent = {
      ...newConsent,
      necessary: true,
      timestamp: new Date().toISOString()
    };
    localStorage.setItem('rustcraft_cookie_consent', JSON.stringify(consent));
    isBannerVisible = false;
    isPreferencesModalOpen = false;
  }

  function handleAcceptAll() {
    saveConsent({ necessary: true, analytics: true, marketing: true });
  }

  function handleDenyOptional() {
    saveConsent({ necessary: true, analytics: false, marketing: false });
  }

  function handleSavePreferences() {
    saveConsent(consent);
  }
</script>

<!-- Bottom Floating Cookie Banner -->
{#if isBannerVisible && !isPreferencesModalOpen}
  <div
    class="fixed bottom-4 left-4 right-4 md:left-8 md:right-8 lg:max-w-4xl lg:mx-auto z-50 bg-slate-900/95 backdrop-blur-xl border border-slate-800 rounded-3xl p-5 md:p-6 shadow-2xl shadow-black/60 animate-in fade-in slide-in-from-bottom-6 duration-300"
    role="region"
    aria-label="Cookie consent banner"
  >
    <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
      <div class="flex items-start gap-3.5 max-w-2xl">
        <div class="w-10 h-10 rounded-2xl bg-orange-500/10 border border-orange-500/20 text-orange-400 flex items-center justify-center flex-shrink-0 mt-0.5">
          <Shield size={20} />
        </div>
        <div>
          <h3 class="text-sm font-bold text-white tracking-tight">{title}</h3>
          <p class="text-xs text-slate-300 mt-1 leading-relaxed">
            {description}
            {#if policyUrl}
              <a href={policyUrl} class="text-orange-400 hover:text-orange-300 underline underline-offset-2 ml-1 transition-colors">
                Learn more in our Cookie Policy
              </a>
            {/if}
          </p>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex flex-wrap items-center gap-2.5 w-full md:w-auto justify-end flex-shrink-0 pt-2 md:pt-0">
        <button
          type="button"
          on:click={() => isPreferencesModalOpen = true}
          class="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white text-xs font-semibold flex items-center gap-1.5 transition-all border border-slate-700/80"
        >
          <Settings size={14} />
          <span>{preferencesLabel}</span>
        </button>

        <button
          type="button"
          on:click={handleDenyOptional}
          class="px-4 py-2 rounded-xl bg-slate-800/90 hover:bg-slate-700 text-slate-200 hover:text-white text-xs font-semibold transition-all border border-slate-700/80"
        >
          {denyLabel}
        </button>

        <button
          type="button"
          on:click={handleAcceptAll}
          class="px-5 py-2 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white text-xs font-bold shadow-lg shadow-orange-600/30 transition-all transform hover:scale-[1.02] active:scale-[0.98]"
        >
          {acceptLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Preferences Modal -->
{#if isPreferencesModalOpen}
  <div class="fixed inset-0 z-50 overflow-y-auto flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-md animate-in fade-in duration-200">
    <div
      class="w-full max-w-lg rounded-3xl bg-slate-900 border border-slate-800 shadow-2xl p-6 relative animate-in zoom-in-95 duration-200"
      role="dialog"
      aria-modal="true"
      aria-labelledby="cookie-preferences-title"
    >
      <!-- Header -->
      <div class="flex items-center justify-between pb-4 border-b border-slate-800">
        <div class="flex items-center gap-2.5">
          <div class="w-9 h-9 rounded-xl bg-orange-500/10 text-orange-400 flex items-center justify-center border border-orange-500/20">
            <Settings size={18} />
          </div>
          <div>
            <h2 id="cookie-preferences-title" class="text-base font-bold text-white tracking-tight">Cookie Preferences</h2>
            <p class="text-xs text-slate-400">Manage your cookie consents in detail</p>
          </div>
        </div>
        <button
          type="button"
          on:click={() => isPreferencesModalOpen = false}
          class="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 transition-colors"
          aria-label="Close modal"
        >
          <X size={18} />
        </button>
      </div>

      <!-- Categories -->
      <div class="py-4 space-y-4 max-h-[60vh] overflow-y-auto pr-1">
        <!-- Necessary Cookies -->
        <div class="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80 space-y-2">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-white">Strictly Necessary Cookies</span>
              <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                Always Active
              </span>
            </div>
            <input
              type="checkbox"
              checked={true}
              disabled
              class="w-4 h-4 rounded text-orange-500 bg-slate-800 border-slate-700 opacity-60 cursor-not-allowed"
            />
          </div>
          <p class="text-[11px] text-slate-400 leading-relaxed">
            Essential for website navigation, user sessions, shopping cart state, and security. Cannot be switched off.
          </p>
        </div>

        <!-- Analytics Cookies -->
        <div class="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80 space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold text-white">Performance & Analytics Cookies</span>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={consent.analytics}
                class="sr-only peer"
              />
              <div class="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-orange-600"></div>
            </label>
          </div>
          <p class="text-[11px] text-slate-400 leading-relaxed">
            Help us understand how visitors interact with our shop by anonymously collecting error logs and page traffic.
          </p>
        </div>

        <!-- Marketing Cookies -->
        <div class="p-4 rounded-2xl bg-slate-950/60 border border-slate-800/80 space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold text-white">Marketing & Personalization Cookies</span>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                bind:checked={consent.marketing}
                class="sr-only peer"
              />
              <div class="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-orange-600"></div>
            </label>
          </div>
          <p class="text-[11px] text-slate-400 leading-relaxed">
            Used to show tailored product recommendations and relevant content across your shopping sessions.
          </p>
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="pt-4 border-t border-slate-800 flex items-center justify-between gap-3">
        <button
          type="button"
          on:click={handleDenyOptional}
          class="px-4 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold transition-colors"
        >
          {denyLabel}
        </button>

        <div class="flex items-center gap-2">
          <button
            type="button"
            on:click={handleSavePreferences}
            class="px-4 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 border border-slate-700 text-white text-xs font-bold transition-colors"
          >
            Save Preferences
          </button>
          <button
            type="button"
            on:click={handleAcceptAll}
            class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 text-white text-xs font-bold transition-all shadow-md"
          >
            {acceptLabel}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
