<script>
  import { onMount } from 'svelte';
  import { CreditCard, Check, AlertCircle, Save, ExternalLink, Key, Lock, Eye, EyeOff, Webhook, Info } from 'lucide-svelte';

  export let configs = [];

  // Keys must match the backend's stripe::OPTIONAL_METHODS (+ apple_pay / google_pay wallets)
  const STRIPE_METHOD_GROUPS = [
    {
      title: 'Wallets & quick payment',
      note: 'Each one is listed as its own option at checkout.',
      methods: [
        { key: 'apple_pay', label: 'Apple Pay', hint: 'Safari on iPhone / iPad / Mac. Needs HTTPS and your domain registered below.' },
        { key: 'google_pay', label: 'Google Pay', hint: 'Chrome / Android with a saved card. Needs HTTPS and your domain registered below.' },
        { key: 'link', label: 'Link', hint: "Stripe's one-click checkout with saved payment details." },
        { key: 'amazon_pay', label: 'Amazon Pay', hint: 'Pay with the Amazon account.' },
        { key: 'paypal', label: 'PayPal (via Stripe)', hint: 'PayPal processed by Stripe. Use this or the separate PayPal gateway below, not both.' },
        { key: 'klarna', label: 'Klarna', hint: 'Pay now, pay later or in instalments.' }
      ]
    },
    {
      title: 'Bank transfers & local methods',
      note: 'Each one is listed as its own option at checkout.',
      methods: [
        { key: 'sepa_debit', label: 'SEPA Direct Debit', hint: 'Delayed: the order is created as "payment pending" and marked paid by the webhook after 2–14 days.' },
        { key: 'ideal', label: 'iDEAL', hint: 'Netherlands' },
        { key: 'bancontact', label: 'Bancontact', hint: 'Belgium' },
        { key: 'eps', label: 'EPS', hint: 'Austria' },
        { key: 'p24', label: 'Przelewy24', hint: 'Poland' },
        { key: 'revolut_pay', label: 'Revolut Pay', hint: 'Revolut app' },
        { key: 'mobilepay', label: 'MobilePay', hint: 'Denmark / Finland' },
        { key: 'alipay', label: 'Alipay', hint: 'China' },
        { key: 'wechat_pay', label: 'WeChat Pay', hint: 'China' }
      ]
    }
  ];
  const ALL_METHOD_KEYS = STRIPE_METHOD_GROUPS.flatMap((g) => g.methods.map((m) => m.key));

  // --- Apple Pay / Google Pay domain registration ---
  // Activation status of each method in the Stripe account (methods that are not active are hidden at checkout)
  let stripeStatus = {};
  async function loadStripeStatus() {
    try {
      const res = await fetch('/api/v1/admin/settings/payments/stripe/capabilities');
      if (res.ok) stripeStatus = (await res.json()).status || {};
    } catch (_) {}
  }
  onMount(() => {
    if (stripe?.has_secret_key) loadStripeStatus();
  });

  let domainInput = '';
  let domains = [];
  let domainBusy = false;
  let domainMessage = '';

  async function readJson(res) {
    return res.json().catch(() => ({}));
  }

  async function loadDomains() {
    domainBusy = true;
    domainMessage = '';
    try {
      const res = await fetch('/api/v1/admin/settings/payments/stripe/domains');
      const body = await readJson(res);
      if (!res.ok) throw new Error(body.error || 'Could not load domains');
      domains = Array.isArray(body) ? body : [];
      if (domains.length === 0) domainMessage = 'No domains registered yet.';
    } catch (e) {
      domainMessage = e.message;
    } finally {
      domainBusy = false;
    }
  }

  async function registerDomain() {
    domainBusy = true;
    domainMessage = '';
    try {
      const res = await fetch('/api/v1/admin/settings/payments/stripe/domains', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ domain: domainInput })
      });
      const body = await readJson(res);
      if (!res.ok) throw new Error(body.error || 'Registration failed');
      domainInput = '';
      await loadDomains();
      domainMessage = `Registered ${body.domain}.`;
    } catch (e) {
      domainMessage = e.message;
    } finally {
      domainBusy = false;
    }
  }

  function normalize(c) {
    const config_data = { ...(c.config_data || {}) };
    if (c.provider === 'stripe') {
      config_data.checkout_mode = config_data.checkout_mode === 'hosted' ? 'hosted' : 'elements';
      config_data.methods = { ...Object.fromEntries(ALL_METHOD_KEYS.map((k) => [k, false])), ...(config_data.methods || {}) };
    }
    if (c.provider === 'paypal') {
      config_data.allow_pay_later = config_data.allow_pay_later !== false;
    }
    return { ...c, config_data, new_secret_key: '', new_webhook_secret: '' };
  }

  let stripe = null;
  let paypal = null;
  $: if (!stripe) stripe = configs.find((c) => c.provider === 'stripe') ? normalize(configs.find((c) => c.provider === 'stripe')) : null;
  $: if (!paypal) paypal = configs.find((c) => c.provider === 'paypal') ? normalize(configs.find((c) => c.provider === 'paypal')) : null;

  let saving = null;
  let notice = '';
  let errorMessage = '';
  let showSecret = {};

  $: stripePkMode = stripe?.public_client_id?.startsWith('pk_live_') ? 'live' : stripe?.public_client_id?.startsWith('pk_test_') ? 'test' : '';
  $: stripeNewSkMode = stripe?.new_secret_key?.includes('_live_') ? 'live' : stripe?.new_secret_key?.includes('_test_') ? 'test' : '';
  $: stripeSkMode = stripeNewSkMode || stripe?.secret_mode || '';
  $: stripeModeMismatch = stripePkMode && stripeSkMode && stripePkMode !== stripeSkMode;
  $: stripeReady = stripePkMode && (stripe?.has_secret_key || stripe?.new_secret_key) && !stripeModeMismatch;
  $: paypalShownTwice = stripe?.is_enabled && stripe?.config_data?.methods?.paypal && paypal?.is_enabled;

  function flash(message, isError = false) {
    if (isError) {
      errorMessage = message;
      notice = '';
      setTimeout(() => (errorMessage = ''), 8000);
    } else {
      notice = message;
      errorMessage = '';
      setTimeout(() => (notice = ''), 4000);
    }
  }

  async function save(p) {
    saving = p.provider;
    try {
      const body = {
        display_name: p.display_name,
        is_enabled: p.is_enabled,
        is_sandbox: p.provider === 'stripe' ? stripePkMode !== 'live' : p.is_sandbox,
        public_client_id: (p.public_client_id || '').trim(),
        config_data: p.config_data
      };
      if (p.new_secret_key.trim()) body.secret_key = p.new_secret_key.trim();
      if (p.new_webhook_secret.trim()) body.webhook_secret = p.new_webhook_secret.trim();

      const res = await fetch(`/api/v1/admin/settings/payments/${p.provider}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body)
      });
      if (!res.ok) {
        const text = await res.text();
        let msg = text;
        try {
          const j = JSON.parse(text);
          msg = j.error || j.message || text;
        } catch (_) {}
        throw new Error(msg);
      }

      // Reload masked secret state from the server
      const listRes = await fetch('/api/v1/admin/settings/payments');
      if (listRes.ok) {
        const fresh = (await listRes.json()).find((c) => c.provider === p.provider);
        if (fresh) {
          if (p.provider === 'stripe') stripe = normalize(fresh);
          else paypal = normalize(fresh);
        }
      }
      flash(`${p.display_name} saved.`);
    } catch (e) {
      flash(`Could not save ${p.display_name}: ${e.message}`, true);
    } finally {
      saving = null;
    }
  }

  const inputClass = 'w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500';
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-xl font-black text-white tracking-tight flex items-center gap-2.5">
      <CreditCard size={22} class="text-orange-500" />
      Payment Providers
    </h2>
    <p class="text-xs text-slate-400 mt-1">
      Customers pay with Stripe (cards and wallets) directly on your checkout page, or with PayPal. Every payment is verified with the provider before an order is created.
    </p>
  </div>

  {#if notice}
    <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-semibold flex items-center gap-2">
      <Check size={16} /> <span>{notice}</span>
    </div>
  {/if}
  {#if errorMessage}
    <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold flex items-center gap-2">
      <AlertCircle size={16} /> <span>{errorMessage}</span>
    </div>
  {/if}

  <!-- ============================== STRIPE ============================== -->
  {#if stripe}
    <section class="p-6 rounded-3xl border border-orange-500/40 bg-gradient-to-br from-slate-900 via-slate-900 to-orange-950/20 shadow-xl space-y-5">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-4">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center font-black text-indigo-400">S</div>
          <div>
            <div class="flex items-center gap-2 flex-wrap">
              <h3 class="text-base font-bold text-white">Stripe</h3>
              {#if stripePkMode === 'live'}
                <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">LIVE keys</span>
              {:else if stripePkMode === 'test'}
                <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/15 text-amber-400 border border-amber-500/30">TEST keys</span>
              {/if}
              {#if stripe.has_secret_key}
                <span class="px-2 py-0.5 rounded-full text-[10px] font-mono bg-slate-800 text-slate-300 flex items-center gap-1"><Key size={10} /> secret set</span>
              {/if}
            </div>
            <p class="text-[11px] text-slate-400 mt-0.5">Cards, Apple Pay, Google Pay, Link, Amazon Pay</p>
          </div>
        </div>
        <label class="flex items-center gap-2 cursor-pointer text-xs">
          <input type="checkbox" bind:checked={stripe.is_enabled} class="sr-only peer" />
          <div class="relative w-11 h-6 bg-slate-800 rounded-full peer-checked:bg-orange-600 after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
          <span class="font-bold {stripe.is_enabled ? 'text-white' : 'text-slate-500'}">{stripe.is_enabled ? 'Active' : 'Disabled'}</span>
        </label>
      </div>

      <!-- Checkout experience -->
      <fieldset class="space-y-2">
        <legend class="text-xs font-semibold text-slate-300 mb-2">Checkout experience</legend>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <label class="p-3.5 rounded-xl border cursor-pointer text-xs transition-all {stripe.config_data.checkout_mode === 'elements' ? 'border-orange-500 bg-orange-600/10 ring-1 ring-orange-500' : 'border-slate-800 bg-slate-950 hover:border-slate-700'}">
            <input type="radio" class="sr-only" bind:group={stripe.config_data.checkout_mode} value="elements" />
            <div class="font-bold text-white">On your checkout page <span class="text-[10px] text-emerald-400 font-semibold">(recommended)</span></div>
            <p class="text-[11px] text-slate-400 mt-1">Customers enter card details and use wallets directly in your shop (Stripe Payment Element). Card data never touches your server.</p>
          </label>
          <label class="p-3.5 rounded-xl border cursor-pointer text-xs transition-all {stripe.config_data.checkout_mode === 'hosted' ? 'border-orange-500 bg-orange-600/10 ring-1 ring-orange-500' : 'border-slate-800 bg-slate-950 hover:border-slate-700'}">
            <input type="radio" class="sr-only" bind:group={stripe.config_data.checkout_mode} value="hosted" />
            <div class="font-bold text-white flex items-center gap-1.5">Redirect to Stripe Checkout <ExternalLink size={12} /></div>
            <p class="text-[11px] text-slate-400 mt-1">Customers pay on checkout.stripe.com and return to your shop afterwards.</p>
          </label>
        </div>
      </fieldset>

      <!-- Keys -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
        <div>
          <label for="stripe-display" class="block font-semibold text-slate-400 mb-1">Label shown at checkout</label>
          <input id="stripe-display" type="text" bind:value={stripe.display_name} class={inputClass} />
        </div>
        <div>
          <label for="stripe-pk" class="block font-semibold text-slate-400 mb-1">Publishable key</label>
          <input id="stripe-pk" type="text" bind:value={stripe.public_client_id} placeholder="pk_live_... or pk_test_..." class="{inputClass} font-mono" />
        </div>
        <div class="sm:col-span-2">
          <label for="stripe-sk" class="block font-semibold text-slate-400 mb-1 flex items-center gap-1.5">
            <Lock size={12} class="text-orange-400" /> Secret key or restricted key
            {#if stripe.has_secret_key}<span class="ml-auto font-mono text-emerald-400 flex items-center gap-1"><Check size={12} /> {stripe.masked_secret_key}</span>{/if}
          </label>
          <div class="relative">
            <input
              id="stripe-sk"
              type={showSecret.stripe ? 'text' : 'password'}
              bind:value={stripe.new_secret_key}
              autocomplete="off"
              placeholder={stripe.has_secret_key ? 'Leave empty to keep the current key' : 'sk_live_... or rk_live_...'}
              class="{inputClass} font-mono pr-10"
            />
            <button type="button" on:click={() => (showSecret.stripe = !showSecret.stripe)} class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300" title="Show / hide">
              {#if showSecret.stripe}<EyeOff size={14} />{:else}<Eye size={14} />{/if}
            </button>
          </div>
          <p class="text-[10px] text-slate-500 mt-1">
            A restricted key needs write access to <em>PaymentIntents</em>, <em>Checkout Sessions</em> and <em>Refunds</em>. Stored server-side only — it is never sent back to the browser.
          </p>
        </div>
        <div class="sm:col-span-2">
          <label for="stripe-whsec" class="block font-semibold text-slate-400 mb-1 flex items-center gap-1.5">
            <Webhook size={12} class="text-orange-400" /> Webhook signing secret <span class="text-slate-500 font-normal">(recommended)</span>
            {#if stripe.has_webhook_secret}<span class="ml-auto font-mono text-emerald-400 flex items-center gap-1"><Check size={12} /> {stripe.masked_webhook_secret}</span>{/if}
          </label>
          <input
            id="stripe-whsec"
            type="password"
            bind:value={stripe.new_webhook_secret}
            autocomplete="off"
            placeholder={stripe.has_webhook_secret ? 'Leave empty to keep the current secret' : 'whsec_...'}
            class="{inputClass} font-mono"
          />
          <p class="text-[10px] text-slate-500 mt-1 leading-relaxed">
            Stripe Dashboard → Developers → Webhooks → add endpoint <code class="text-orange-300">https://YOUR-SHOP-DOMAIN/api/v1/payments/stripe/webhook</code>
            with events <code class="text-slate-300">payment_intent.succeeded</code>, <code class="text-slate-300">payment_intent.payment_failed</code>, <code class="text-slate-300">checkout.session.completed</code>,
            <code class="text-slate-300">checkout.session.async_payment_succeeded</code> and <code class="text-slate-300">checkout.session.async_payment_failed</code>. Required for SEPA Direct Debit; also creates the order if a customer closes the browser right after paying.
          </p>
        </div>
      </div>

      {#if stripeModeMismatch}
        <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-[11px] flex items-start gap-2">
          <AlertCircle size={14} class="flex-shrink-0 mt-0.5" /> The publishable key is a {stripePkMode} key but the secret key is a {stripeSkMode} key. Both must be test keys or both live keys.
        </div>
      {/if}

      <!-- Payment methods offered through Stripe -->
      <div class="space-y-3 pt-4 border-t border-slate-800/80">
        <div>
          <h4 class="text-sm font-bold text-white">Payment methods via Stripe</h4>
          <p class="text-[11px] text-slate-400 mt-0.5">
            Turn methods on or off for your checkout. Each one must also be activated in the
            <a href="https://dashboard.stripe.com/settings/payment_methods" target="_blank" rel="noopener noreferrer" class="text-orange-400 underline">Stripe Dashboard → Payment methods</a>.
            Wallet buttons only appear on devices and browsers that support them.
          </p>
        </div>
        <div class="p-3 rounded-xl bg-slate-950 border border-slate-800 flex items-start gap-3 opacity-80">
          <input type="checkbox" checked disabled class="mt-0.5 w-4 h-4 accent-orange-600" />
          <div class="text-xs">
            <div class="font-bold text-white">Credit / Debit card <span class="text-[10px] text-emerald-400 font-semibold">(default, always first)</span></div>
            <p class="text-[10px] text-slate-500">Visa, Mastercard, American Express … always enabled and pre-selected.</p>
          </div>
        </div>
        {#each STRIPE_METHOD_GROUPS as group}
          <div class="space-y-2">
            <div class="flex items-baseline justify-between gap-2">
              <h5 class="text-xs font-bold text-slate-200">{group.title}</h5>
              <span class="text-[10px] text-slate-500">{group.note}</span>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
              {#each group.methods as m}
                <label class="p-3 rounded-xl bg-slate-950 border cursor-pointer flex items-start gap-3 transition-all {stripe.config_data.methods[m.key] ? 'border-orange-500/60' : 'border-slate-800 hover:border-slate-700'}">
                  <input type="checkbox" bind:checked={stripe.config_data.methods[m.key]} class="mt-0.5 w-4 h-4 accent-orange-600 cursor-pointer" />
                  <div class="text-xs">
                    <div class="font-bold text-white flex items-center gap-1.5 flex-wrap">
                      {m.label}
                      {#if stripeStatus[m.key] === 'active'}
                        <span class="text-[9px] font-semibold px-1.5 py-0.5 rounded bg-emerald-500/15 text-emerald-400">active at Stripe</span>
                      {:else if stripeStatus[m.key] && stripeStatus[m.key] !== 'unknown'}
                        <span class="text-[9px] font-semibold px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-300" title="Activate it in the Stripe Dashboard → Settings → Payment methods; until then it is not offered at checkout">not activated at Stripe</span>
                      {/if}
                    </div>
                    <p class="text-[10px] text-slate-500 leading-relaxed">{m.hint}</p>
                  </div>
                </label>
              {/each}
            </div>
          </div>
        {/each}
        {#if stripe.config_data.checkout_mode === 'hosted'}
          <p class="text-[10px] text-slate-500 flex items-start gap-1.5"><Info size={12} class="flex-shrink-0" /> On the hosted Stripe page, Apple Pay and Google Pay follow your Stripe Dashboard settings.</p>
        {/if}

        <!-- Domain registration for wallets -->
        <div class="p-4 rounded-2xl bg-slate-950 border border-slate-800 space-y-3">
          <div>
            <h5 class="text-xs font-bold text-white">Apple Pay &amp; Google Pay domain</h5>
            <p class="text-[10px] text-slate-500 leading-relaxed mt-0.5">
              Wallet buttons only appear on a registered domain served over HTTPS — never on <code>http://localhost</code>.
              Save your secret key first, then register your public shop domain.
            </p>
          </div>
          <div class="flex flex-col sm:flex-row gap-2">
            <input type="text" bind:value={domainInput} placeholder="shop.example.com" aria-label="Shop domain" class="{inputClass} font-mono flex-1" />
            <button type="button" on:click={registerDomain} disabled={domainBusy || !domainInput.trim() || !stripe.has_secret_key} class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-orange-600 disabled:opacity-50 text-white text-xs font-bold">Register domain</button>
            <button type="button" on:click={loadDomains} disabled={domainBusy || !stripe.has_secret_key} class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 disabled:opacity-50 text-white text-xs font-bold">Check status</button>
          </div>
          {#if domainMessage}<p class="text-[11px] text-slate-400">{domainMessage}</p>{/if}
          {#if domains.length > 0}
            <div class="overflow-x-auto">
              <table class="w-full text-[11px] text-left">
                <thead class="text-slate-500"><tr><th class="py-1 pr-3">Domain</th><th class="pr-3">Apple Pay</th><th class="pr-3">Google Pay</th><th class="pr-3">Link</th><th>PayPal</th></tr></thead>
                <tbody>
                  {#each domains as d}
                    <tr class="border-t border-slate-800 text-slate-300">
                      <td class="py-1 pr-3 font-mono">{d.domain}</td>
                      <td class="pr-3 {d.apple_pay === 'active' ? 'text-emerald-400' : 'text-amber-400'}">{d.apple_pay}</td>
                      <td class="pr-3 {d.google_pay === 'active' ? 'text-emerald-400' : 'text-amber-400'}">{d.google_pay}</td>
                      <td class="pr-3">{d.link}</td>
                      <td>{d.paypal}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>
      </div>

      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3 border-t border-slate-800/80">
        <span class="text-[11px] {stripeReady ? 'text-emerald-400' : 'text-amber-400'} font-semibold">
          {stripeReady ? '✓ Ready to take payments' : '⚠ Enter a publishable key and a secret key to take payments'}
        </span>
        <button
          type="button"
          on:click={() => save(stripe)}
          disabled={saving === 'stripe' || stripeModeMismatch}
          class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-1.5 self-start sm:self-auto"
        >
          <Save size={14} /> {saving === 'stripe' ? 'Saving…' : 'Save Stripe settings'}
        </button>
      </div>
    </section>
  {/if}

  <!-- ============================== PAYPAL ============================== -->
  {#if paypal}
    <section class="p-6 rounded-3xl border border-slate-800 bg-slate-900 shadow-xl space-y-5">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-800 pb-4">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-slate-950 border border-slate-800 flex items-center justify-center font-black text-sky-400">P</div>
          <div>
            <h3 class="text-base font-bold text-white">PayPal</h3>
            <p class="text-[11px] text-slate-400 mt-0.5">Official PayPal buttons (PayPal account, Pay Later) via the Orders v2 API</p>
          </div>
        </div>
        <div class="flex items-center gap-5">
          <label class="flex items-center gap-2 cursor-pointer text-xs">
            <input type="checkbox" bind:checked={paypal.is_sandbox} class="sr-only peer" />
            <div class="relative w-9 h-5 bg-slate-800 rounded-full peer-checked:bg-amber-500 after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
            <span class="font-semibold {paypal.is_sandbox ? 'text-amber-400' : 'text-emerald-400'}">{paypal.is_sandbox ? 'Sandbox' : 'Live'}</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer text-xs">
            <input type="checkbox" bind:checked={paypal.is_enabled} class="sr-only peer" />
            <div class="relative w-11 h-6 bg-slate-800 rounded-full peer-checked:bg-orange-600 after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
            <span class="font-bold {paypal.is_enabled ? 'text-white' : 'text-slate-500'}">{paypal.is_enabled ? 'Active' : 'Disabled'}</span>
          </label>
        </div>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
        <div>
          <label for="paypal-display" class="block font-semibold text-slate-400 mb-1">Label shown at checkout</label>
          <input id="paypal-display" type="text" bind:value={paypal.display_name} class={inputClass} />
        </div>
        <div>
          <label for="paypal-client" class="block font-semibold text-slate-400 mb-1">Client ID</label>
          <input id="paypal-client" type="text" bind:value={paypal.public_client_id} placeholder="A..." class="{inputClass} font-mono" />
        </div>
        <div class="sm:col-span-2">
          <label for="paypal-secret" class="block font-semibold text-slate-400 mb-1 flex items-center gap-1.5">
            <Lock size={12} class="text-orange-400" /> Secret
            {#if paypal.has_secret_key}<span class="ml-auto font-mono text-emerald-400 flex items-center gap-1"><Check size={12} /> {paypal.masked_secret_key}</span>{/if}
          </label>
          <div class="relative">
            <input
              id="paypal-secret"
              type={showSecret.paypal ? 'text' : 'password'}
              bind:value={paypal.new_secret_key}
              autocomplete="off"
              placeholder={paypal.has_secret_key ? 'Leave empty to keep the current secret' : 'PayPal REST app secret'}
              class="{inputClass} font-mono pr-10"
            />
            <button type="button" on:click={() => (showSecret.paypal = !showSecret.paypal)} class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300" title="Show / hide">
              {#if showSecret.paypal}<EyeOff size={14} />{:else}<Eye size={14} />{/if}
            </button>
          </div>
          <p class="text-[10px] text-slate-500 mt-1">
            Create a REST app at
            <a href="https://developer.paypal.com/dashboard/applications" target="_blank" rel="noopener noreferrer" class="text-orange-400 underline">developer.paypal.com → Apps &amp; Credentials</a>.
            Sandbox credentials only work with the Sandbox toggle on, live credentials only with it off.
          </p>
        </div>
        <label class="sm:col-span-2 flex items-center gap-2.5 cursor-pointer">
          <input type="checkbox" bind:checked={paypal.config_data.allow_pay_later} class="w-4 h-4 accent-orange-600" />
          <span class="text-slate-300">Offer PayPal Pay Later (Später bezahlen / Ratenzahlung) where eligible</span>
        </label>
      </div>

      {#if paypalShownTwice}
        <div class="p-3 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-300 text-[11px] flex items-start gap-2">
          <AlertCircle size={14} class="flex-shrink-0 mt-0.5" /> "PayPal (via Stripe)" is also enabled above — customers would see PayPal twice. Keep only one of them.
        </div>
      {/if}

      <div class="flex justify-end pt-3 border-t border-slate-800/80">
        <button
          type="button"
          on:click={() => save(paypal)}
          disabled={saving === 'paypal'}
          class="px-5 py-2 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-1.5"
        >
          <Save size={14} /> {saving === 'paypal' ? 'Saving…' : 'Save PayPal settings'}
        </button>
      </div>
    </section>
  {/if}
</div>
