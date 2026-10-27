<script>
  import { cart, cartSubtotal, cartCount } from '$lib/stores/cart.js';
  import { customer } from '$lib/stores/customer.js';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { onMount, onDestroy } from 'svelte';
  import { ShieldCheck, Lock, CreditCard, AlertCircle, ArrowRight, MapPin, UserCheck, Tag, Loader2, ExternalLink, Gift } from 'lucide-svelte';
  import { loadStripe } from '@stripe/stripe-js';
  import StripePaymentForm from '$lib/components/StripePaymentForm.svelte';
  import StripeExpressButton from '$lib/components/StripeExpressButton.svelte';
  import CountryInput from '$lib/components/CountryInput.svelte';
  import { checkoutMethods, EXPRESS_METHODS, expressButtons, expressProbeTypes } from '$lib/stripeMethods.js';
  import { rememberNewsletterOptIn, submitRememberedNewsletterOptIn } from '$lib/newsletter.js';
  import { countryName, countryOptions, shippingCountryCodes } from '$lib/countries.js';

  export let data;
  $: store = data.store || {};
  let paymentProviders = data.paymentProviders || [];

  // Only real, server-verified gateways are offered: Stripe (cards + wallets) and PayPal
  // Gateways without their public key cannot render, so they are hidden from customers.
  $: enabledProviders = paymentProviders.filter(
    (p) =>
      p.is_enabled &&
      ((p.provider === 'stripe' && (p.public_client_id || '').trim().startsWith('pk_')) ||
        (p.provider === 'paypal' && (p.public_client_id || '').trim() !== ''))
  );
  $: stripeProvider = enabledProviders.find((p) => p.provider === 'stripe');
  $: paypalProvider = enabledProviders.find((p) => p.provider === 'paypal');

  $: stripeKey = stripeProvider && (stripeProvider.public_client_id || '').trim().startsWith('pk_') ? stripeProvider.public_client_id.trim() : null;
  $: stripeTestMode = !!stripeKey && stripeKey.startsWith('pk_test_');
  $: stripeHosted = stripeProvider?.config_data?.checkout_mode === 'hosted';
  // Methods enabled in the admin AND activated in the Stripe account (asked from the backend),
  // so customers never see a method that would fail. Card is shown right away.
  let activeMethods = ['card'];
  let activeMethodsRequested = false;
  $: if (browser && stripeKey && !stripeHosted && !activeMethodsRequested) loadActiveMethods();

  async function loadActiveMethods() {
    activeMethodsRequested = true;
    try {
      const res = await fetch('/api/v1/checkout/stripe/methods');
      if (res.ok) activeMethods = (await res.json()).methods || ['card'];
    } catch (_) {
      // keep card only
    }
  }

  // Button methods (Apple Pay, Google Pay, Amazon Pay, Link) are listed once a hidden probe confirms
  // this device/browser can use them
  let expressAvailability = {};
  $: expressCandidates = activeMethods.filter((id) => EXPRESS_METHODS[id]);

  function onExpressProbe(event) {
    const av = event.availablePaymentMethods || {};
    expressAvailability = { apple_pay: !!av.applePay, google_pay: !!av.googlePay, amazon_pay: !!av.amazonPay, link: !!av.link };
  }

  // Every payment method as its own entry, listed underneath each other
  $: paymentMethods = checkoutMethods({
    stripeEnabled: !!stripeKey,
    stripeHosted,
    activeMethods,
    paypalEnabled: !!paypalProvider,
    expressAvailability
  });
  // Credit / debit card is the default
  let selectedMethodId = 'card';
  $: if (paymentMethods.length > 0 && !paymentMethods.some((m) => m.id === selectedMethodId)) {
    selectedMethodId = paymentMethods[0].id;
  }
  $: selectedMethod = paymentMethods.find((m) => m.id === selectedMethodId);
  $: selectedProvider = selectedMethod?.kind === 'paypal' ? 'paypal' : 'stripe';

  // Form State
  let customerName = '';
  let customerEmail = '';
  let streetAddress = '';
  let apartmentSuite = '';
  let city = '';
  let stateProvince = '';
  let postalCode = '';
  let countryCode = 'DE';

  let savedAddresses = [];
  let selectedAddressId = null;

  let selectedShippingRateId = '';
  let availableRates = [];
  let isFetchingRates = false;

  let isSubmitting = false;
  let errorMessage = '';
  let acceptedTerms = false;
  let acceptedDigitalWaiver = false;

  // Business customers: company name on the invoice; with a VAT ID from another EU country the
  // order is VAT-free (checked with the EU VIES service)
  let isBusiness = false;
  let buyerCompany = '';
  let vatId = '';
  let vatCheck = null;
  let vatChecking = false;
  let newsletterOptIn = false;

  async function checkVatId() {
    vatCheck = null;
    if (!vatId.trim()) return;
    vatChecking = true;
    try {
      const res = await fetch('/api/v1/checkout/vat-check', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ vat_id: vatId })
      });
      vatCheck = res.ok ? await res.json() : { valid: false, error: await readError(res, 'The VAT ID could not be checked.') };
    } catch (_) {
      vatCheck = { valid: false, error: 'The VAT ID could not be checked.' };
    } finally {
      vatChecking = false;
    }
  }

  // Promo / Coupon Code State
  let couponInput = '';
  let appliedCoupon = null;
  let couponError = '';
  let isApplyingCoupon = false;

  $: hasPhysicalItems = $cart.some((i) => !i.is_digital);
  $: hasDigitalItems = $cart.some((i) => i.is_digital);

  async function readError(res, fallback) {
    const text = await res.text();
    try {
      const j = JSON.parse(text);
      return j.error || j.message || text || fallback;
    } catch (_) {
      return text || fallback;
    }
  }

  async function applyCoupon() {
    if (!couponInput.trim()) return;
    couponError = '';
    isApplyingCoupon = true;
    try {
      const res = await fetch('/api/v1/coupons/validate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          code: couponInput.trim(),
          subtotal_cents: $cartSubtotal,
          shipping_cost_cents: quote ? quote.shipping_cost_cents : 0
        })
      });
      if (res.ok) {
        const result = await res.json();
        if (result.valid) {
          appliedCoupon = result;
        } else {
          couponError = result.message || 'Invalid promo code.';
          appliedCoupon = null;
        }
      } else {
        couponError = 'Failed to validate promo code.';
      }
    } catch (e) {
      couponError = e.message || 'Error validating promo code.';
    } finally {
      isApplyingCoupon = false;
    }
  }

  function removeCoupon() {
    appliedCoupon = null;
    couponInput = '';
    couponError = '';
  }

  // Every country a shipping zone serves; the current one stays selectable (e.g. from a saved address)
  // so the select never silently switches to another country
  $: shippingCodes = shippingCountryCodes(data.shippingProviders || []);
  $: countries = countryOptions(shippingCodes.includes(countryCode) ? shippingCodes : [...shippingCodes, countryCode]);

  $: selectedRateEstimate = (availableRates.find((r) => r.id === selectedShippingRateId)?.estimated_delivery_days || '').trim();

  // Shipping options come with the server quote: only packages the cart fits into (weight and
  // size) for the chosen country, cheapest first. Changing the address simply asks for a new quote.
  function fetchShippingRates() {
    scheduleQuote();
  }

  function applyShippingOptions(options) {
    availableRates = options;
    // A rate for another country or a parcel that no longer fits is not valid any more
    if (!options.some((r) => r.id === selectedShippingRateId)) {
      selectedShippingRateId = options.length > 0 ? options[0].id : '';
    }
  }
  $: if (quote) applyShippingOptions(quote.shipping_options || []);

  function selectSavedAddress(addr) {
    if (!addr) return;
    selectedAddressId = addr.id;
    if (addr.full_name) customerName = addr.full_name;
    streetAddress = addr.street_address || '';
    apartmentSuite = addr.apartment_suite || '';
    city = addr.city || '';
    stateProvince = addr.state_province || '';
    postalCode = addr.postal_code || '';
    countryCode = addr.country_code || 'DE';
    fetchShippingRates();
  }

  async function loadCustomerProfileAndAddresses() {
    if ($customer && $customer.isLoggedIn) {
      if ($customer.full_name && !customerName) customerName = $customer.full_name;
      if ($customer.email && !customerEmail) customerEmail = $customer.email;

      try {
        const res = await fetch('/api/v1/customer/addresses', {
          headers: { Authorization: `Bearer ${$customer.token}` }
        });
        if (res.ok) {
          const list = await res.json();
          if (Array.isArray(list) && list.length > 0) {
            savedAddresses = list;
            selectSavedAddress(list.find((a) => a.is_default) || list[0]);
            return;
          }
        }
      } catch (err) {
        console.warn('Failed to fetch addresses from backend:', err);
      }

      try {
        const local = localStorage.getItem('rustwebshop_saved_addresses');
        if (local) {
          const list = JSON.parse(local);
          if (Array.isArray(list) && list.length > 0) {
            savedAddresses = list;
            selectSavedAddress(list.find((a) => a.is_default) || list[0]);
          }
        }
      } catch (_) {}
    } else {
      try {
        const lastGuest = localStorage.getItem('rustwebshop_last_shipping');
        if (lastGuest) {
          const parsed = JSON.parse(lastGuest);
          if (parsed.full_name) customerName = parsed.full_name;
          if (parsed.email) customerEmail = parsed.email;
          if (parsed.street_address) streetAddress = parsed.street_address;
          if (parsed.apartment_suite) apartmentSuite = parsed.apartment_suite;
          if (parsed.city) city = parsed.city;
          if (parsed.postal_code) postalCode = parsed.postal_code;
          if (parsed.country_code) countryCode = parsed.country_code;
        }
      } catch (_) {}
    }
  }

  onMount(async () => {
    loadCustomerProfileAndAddresses();
    fetchShippingRates();
    try {
      const res = await fetch('/api/v1/store/info');
      if (res.ok) {
        const info = await res.json();
        if (info.payment_providers) paymentProviders = info.payment_providers;
      }
    } catch (e) {
      console.warn('Could not refresh payment providers on mount:', e);
    }
  });

  // ---------------------------------------------------------------- Server-side quote
  // The server is the single source of truth for what gets charged.

  function buildCheckoutRequest() {
    return {
      customer_name: customerName.trim(),
      customer_email: customerEmail.trim(),
      shipping_address: {
        full_name: customerName.trim(),
        street_address: streetAddress,
        apartment_suite: apartmentSuite,
        city,
        state_province: stateProvince,
        postal_code: postalCode,
        country_code: countryCode
      },
      shipping_rate_id: hasPhysicalItems ? selectedShippingRateId || null : null,
      coupon_code: appliedCoupon ? appliedCoupon.code : null,
      buyer_company: isBusiness && buyerCompany.trim() ? buyerCompany.trim() : null,
      vat_id: isBusiness && vatId.trim() ? vatId.trim() : null,
      items: $cart.map((i) => ({ variant_id: i.variant_id, quantity: i.quantity }))
    };
  }

  let quote = null;
  let quoteError = '';
  let isQuoting = false;
  let quoteTimer;
  let quoteSeq = 0;

  $: quoteKey = JSON.stringify({
    items: $cart.map((i) => [i.variant_id, i.quantity]),
    country: countryCode,
    rate: hasPhysicalItems ? selectedShippingRateId : null,
    coupon: appliedCoupon ? appliedCoupon.code : null,
    company: isBusiness ? buyerCompany.trim() : '',
    vat: isBusiness ? vatId.trim() : ''
  });
  $: if (browser && $cart.length > 0) scheduleQuote(quoteKey);

  function scheduleQuote() {
    clearTimeout(quoteTimer);
    isQuoting = true;
    quoteTimer = setTimeout(refreshQuote, 200);
  }

  async function refreshQuote() {
    const seq = ++quoteSeq;
    try {
      const res = await fetch('/api/v1/checkout/quote', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(buildCheckoutRequest())
      });
      if (seq !== quoteSeq) return;
      if (res.ok) {
        quote = await res.json();
        quoteError = '';
      } else {
        quote = null;
        quoteError = await readError(res, 'Could not calculate your order total.');
      }
    } catch (e) {
      if (seq === quoteSeq) quoteError = 'Could not reach the shop server to calculate your total.';
    } finally {
      if (seq === quoteSeq) isQuoting = false;
    }
  }

  onDestroy(() => clearTimeout(quoteTimer));

  $: reverseCharge = !!quote?.reverse_charge;
  // Remembered until the order is complete (also across a payment redirect)
  $: if (browser) rememberNewsletterOptIn(newsletterOptIn && store.newsletter_enabled ? customerEmail.trim() : null);

  $: itemsSubtotalCents = quote ? quote.items_subtotal_cents : $cartSubtotal;
  $: discountCents = quote ? quote.discount_cents : 0;
  $: shippingCents = quote ? quote.shipping_cost_cents : 0;
  $: taxCents = quote ? quote.tax_cents : 0;
  $: totalCents = quote ? quote.total_cents : $cartSubtotal;
  $: isFreeOrder = !!quote && quote.total_cents === 0;
  $: taxRatePercent = store.tax_rate_percent || 19.0;
  $: taxMode = store.tax_mode || 'kleingewerbe';
  const formatEuro = (cents) => `${((cents || 0) / 100).toFixed(2)} €`;

  // ---------------------------------------------------------------- Stripe (on-site Payment Element)

  let stripe = null;
  let paymentForm = null; // the mounted StripePaymentForm of the selected method
  let stripeLoadError = '';
  let stripeLoadingKey = null;
  let paymentElementError = '';

  $: if (browser && stripeKey && !stripeHosted && stripeLoadingKey !== stripeKey) initStripe(stripeKey);

  async function initStripe(key) {
    stripeLoadingKey = key;
    stripe = null;
    stripeLoadError = '';
    try {
      stripe = await loadStripe(key);
      if (!stripe) stripeLoadError = 'Stripe.js could not be loaded.';
    } catch (e) {
      stripeLoadError = e.message || 'Stripe.js could not be loaded.';
    }
  }

  // Deferred-intent mode: Elements renders before a PaymentIntent exists; Stripe needs a positive amount.
  $: stripeAmount = Math.max(totalCents || 0, 50);

  const stripeAppearance = {
    theme: 'night',
    variables: {
      colorPrimary: '#ea580c',
      colorBackground: '#020617',
      colorText: '#f8fafc',
      colorDanger: '#f43f5e',
      fontFamily: 'ui-sans-serif, system-ui, sans-serif',
      borderRadius: '12px'
    }
  };

  // ---------------------------------------------------------------- Validation & submit

  function focusField(id) {
    const el = document.getElementById(id);
    if (el) {
      el.focus();
      el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  /** First problem preventing the order, with the id of the field to focus (no side effects). */
  function formProblem() {
    if ($cart.length === 0) return ['Your cart is empty.'];
    if (!customerName.trim()) return ['Please enter your full name in Customer Information.', 'checkout-name'];
    if (!/^\S+@\S+\.\S+$/.test(customerEmail.trim())) return ['Please enter a valid email address in Customer Information.', 'checkout-email'];
    if (hasPhysicalItems && !streetAddress.trim()) return ['Please enter your street address for shipping.', 'checkout-street'];
    if (hasPhysicalItems && (!postalCode.trim() || !city.trim())) return ['Please enter your postal code and city for delivery.', 'checkout-postal'];
    if (hasPhysicalItems && availableRates.length > 0 && !selectedShippingRateId) return ['Please select a shipping option.'];
    if (!acceptedTerms) return ['Please accept the Terms and Conditions and acknowledge the Revocation Policy to place your order.'];
    if (hasDigitalItems && !acceptedDigitalWaiver) return ['Please confirm the immediate execution and revocation waiver for digital products to continue.'];
    if (quoteError) return [quoteError];
    if (!quote || isQuoting) return ['Your order total is still being calculated. Please try again in a moment.'];
    return null;
  }

  /** Synchronous so wallet sheets (Apple Pay / Google Pay / PayPal) still open within the click gesture. */
  function validateForm() {
    const problem = formProblem();
    if (!problem) return '';
    if (problem[1]) focusField(problem[1]);
    return problem[0];
  }

  // ---------------------------------------------------------------- Button methods (Apple Pay, Google Pay, Amazon Pay, Link)
  function onWalletClick(event) {
    const problem = validateForm();
    if (problem) {
      showError(problem);
      return; // not resolving keeps the payment sheet closed
    }
    errorMessage = '';
    event.resolve({ emailRequired: false });
  }

  // Never leave the customer with an endless spinner
  const PAYMENT_TIMEOUT_MS = 120000;
  function withTimeout(promise) {
    let timer;
    const timeout = new Promise((_, reject) => {
      timer = setTimeout(() => reject(new Error('The payment did not complete in time. No order was created — please try again or choose another payment method.')), PAYMENT_TIMEOUT_MS);
    });
    return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
  }

  async function onWalletConfirm(event, elements) {
    isSubmitting = true;
    errorMessage = '';
    try {
      await withTimeout(payWithStripe(elements, buildCheckoutRequest(), selectedMethodId));
    } catch (e) {
      console.error('Express payment failed:', e);
      event.paymentFailed?.({ reason: 'fail' });
      showError(e.message || 'The payment was not completed.');
    }
  }

  /** Server creates the PaymentIntent (only for the chosen method) for the server-computed total; Stripe.js confirms it. */
  async function payWithStripe(elements, request, method) {
    // Must be the first await after the click: it validates the form and opens wallet sheets.
    const { error: submitError } = await elements.submit();
    if (submitError) throw new Error(submitError.message);

    const intent = await postJson('/api/v1/checkout/stripe/intent', { ...request, payment_method: method }, 'Could not start the payment.');
    const address = request.shipping_address;
    const { error, paymentIntent } = await stripe.confirmPayment({
      elements,
      clientSecret: intent.client_secret,
      confirmParams: {
        return_url: `${window.location.origin}/checkout/complete`,
        payment_method_data: {
          billing_details: {
            name: request.customer_name,
            email: request.customer_email,
            // Required by Klarna and SEPA; harmless for cards
            address: {
              country: address.country_code,
              ...(address.street_address ? { line1: address.street_address } : {}),
              ...(address.postal_code ? { postal_code: address.postal_code } : {}),
              ...(address.city ? { city: address.city } : {})
            }
          }
        }
      },
      redirect: 'if_required'
    });
    if (error) throw new Error(error.message || 'The payment was not authorized.');

    finishOrder(
      await postJson(
        '/api/v1/checkout/stripe/complete',
        { payment_intent_id: paymentIntent?.id || intent.payment_intent_id },
        'Your payment was received but the order could not be confirmed. Please contact us.'
      )
    );
  }

  function showError(message) {
    errorMessage = message;
    isSubmitting = false;
    const el = document.getElementById('payment-section');
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }

  function finishOrder(order) {
    submitRememberedNewsletterOptIn();
    try {
      localStorage.setItem(
        'rustwebshop_last_shipping',
        JSON.stringify({
          full_name: customerName,
          email: customerEmail,
          street_address: streetAddress,
          apartment_suite: apartmentSuite,
          city,
          state_province: stateProvince,
          postal_code: postalCode,
          country_code: countryCode
        })
      );
    } catch (_) {}
    cart.clear();
    goto(`/order-success/${encodeURIComponent(order.order_number)}?token=${encodeURIComponent(order.access_token || "")}`);
  }

  async function postJson(url, body, fallbackError) {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body)
    });
    if (!res.ok) throw new Error(await readError(res, fallbackError));
    return res.json();
  }

  async function handleSubmitOrder() {
    const validationError = validateForm();
    if (validationError) {
      showError(validationError);
      return;
    }
    errorMessage = '';
    isSubmitting = true;
    const request = buildCheckoutRequest();

    try {
      if (isFreeOrder) {
        finishOrder(await postJson('/api/v1/checkout/free', request, 'Order could not be placed.'));
        return;
      }

      if (selectedMethod?.kind === 'hosted') {
        const session = await postJson(
          '/api/v1/checkout/stripe/session',
          { ...request, return_origin: window.location.origin },
          'Could not start the Stripe payment page.'
        );
        if (!session.url) throw new Error('No payment page URL received from Stripe.');
        window.location.href = session.url;
        return;
      }

      if (selectedMethod?.kind === 'element') {
        const elements = paymentForm?.getElements();
        if (!stripe || !elements) throw new Error('The payment form is still loading. Please try again in a moment.');
        await payWithStripe(elements, request, selectedMethod.id);
        return;
      }

      throw new Error('Please choose a payment method.');
    } catch (e) {
      showError(e.message || 'An error occurred during order processing.');
    }
  }

  // ---------------------------------------------------------------- PayPal Smart Buttons

  let paypalSdkPromise = null;
  let paypalSdkError = '';

  function loadPaypalSdk(provider) {
    if (window.paypal) return Promise.resolve(window.paypal);
    if (paypalSdkPromise) return paypalSdkPromise;
    const params = new URLSearchParams({
      'client-id': provider.public_client_id,
      currency: 'EUR',
      intent: 'capture',
      components: 'buttons'
    });
    const disabled = [];
    if (stripeProvider) disabled.push('card'); // cards are handled by Stripe
    if (provider.config_data?.allow_pay_later === false) disabled.push('paylater');
    if (disabled.length) params.set('disable-funding', disabled.join(','));

    paypalSdkPromise = new Promise((resolve, reject) => {
      const script = document.createElement('script');
      script.src = `https://www.paypal.com/sdk/js?${params}`;
      script.async = true;
      script.onload = () => (window.paypal ? resolve(window.paypal) : reject(new Error('PayPal SDK unavailable')));
      script.onerror = () => {
        paypalSdkPromise = null;
        reject(new Error('PayPal could not be loaded. Please check your connection or ad blocker.'));
      };
      document.head.appendChild(script);
    });
    return paypalSdkPromise;
  }

  /** Svelte action rendering the PayPal buttons into the container while PayPal is selected. */
  function paypalButtons(node, provider) {
    let buttons = null;
    let destroyed = false;
    paypalSdkError = '';

    loadPaypalSdk(provider)
      .then((paypal) => {
        if (destroyed) return;
        buttons = paypal.Buttons({
          style: { layout: 'vertical', color: 'gold', shape: 'rect', label: 'paypal', height: 45 },
          onClick: (_data, actions) => {
            const validationError = validateForm();
            if (validationError) {
              showError(validationError);
              return actions.reject();
            }
            errorMessage = '';
            return actions.resolve();
          },
          createOrder: async () => {
            try {
              const order = await postJson('/api/v1/checkout/paypal/order', buildCheckoutRequest(), 'Could not start the PayPal payment.');
              return order.id;
            } catch (e) {
              showError(e.message);
              throw e;
            }
          },
          onApprove: async (data) => {
            isSubmitting = true;
            try {
              finishOrder(
                await postJson(
                  '/api/v1/checkout/paypal/capture',
                  { order_id: data.orderID },
                  'Your PayPal payment could not be confirmed. Please contact us.'
                )
              );
            } catch (e) {
              showError(e.message);
            }
          },
          onCancel: () => {
            errorMessage = 'The PayPal payment was cancelled. You can try again or choose another payment method.';
          },
          onError: (err) => {
            console.error('PayPal error:', err);
            if (!errorMessage) showError('PayPal reported an error. Please try again or choose another payment method.');
          }
        });
        if (buttons.isEligible()) buttons.render(node);
        else paypalSdkError = 'PayPal is not available for this browser or region.';
      })
      .catch((e) => {
        paypalSdkError = e.message;
      });

    return {
      destroy() {
        destroyed = true;
        if (buttons) buttons.close().catch(() => {});
      }
    };
  }
</script>

<svelte:head>
  <title>Checkout | {store.store_name || 'Shop'}</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
  <div class="max-w-2xl mx-auto mb-10 text-center">
    <div class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-slate-900 border border-slate-800 text-xs font-semibold text-slate-400 mb-3">
      <Lock size={12} class="text-orange-500" /> End-to-End Encrypted Checkout
    </div>
    <h1 class="text-3xl font-extrabold text-white tracking-tight">Complete Your Order</h1>
  </div>


  {#if $cart.length === 0}
    <div class="text-center py-20 bg-slate-900/40 rounded-3xl border border-slate-800">
      <p class="text-base text-slate-300 font-semibold">Your cart is currently empty.</p>
      <a href="/" class="mt-4 inline-block px-5 py-2.5 rounded-xl bg-orange-600 text-white text-xs font-bold">Return to Store</a>
    </div>
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-10 max-w-6xl mx-auto">
      <!-- Checkout Form (8 Cols) -->
      <div class="lg:col-span-7 space-y-8">
        <!-- 1. Customer Details -->
        <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-base font-bold text-white flex items-center gap-2">
              <span class="w-6 h-6 rounded-full bg-orange-600 text-white text-xs font-bold flex items-center justify-center">1</span>
              Customer Information
            </h2>
            {#if $customer && $customer.isLoggedIn}
              <span class="inline-flex items-center gap-1.5 text-[11px] text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 px-2.5 py-0.5 rounded-full font-semibold">
                <UserCheck size={12} /> Logged in: {$customer.email}
              </span>
            {/if}
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label for="checkout-name" class="block text-xs font-semibold text-slate-400 mb-1">Full Name</label>
              <input
                id="checkout-name"
                name="name"
                type="text"
                autocomplete="name"
                bind:value={customerName}
                required
                placeholder="e.g. Max Mustermann"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
              />
            </div>
            <div>
              <label for="checkout-email" class="block text-xs font-semibold text-slate-400 mb-1">Email Address</label>
              <input
                id="checkout-email"
                name="email"
                type="email"
                autocomplete="email"
                bind:value={customerEmail}
                required
                placeholder="e.g. customer@example.com"
                class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
              />
            </div>
          </div>

          <!-- Business customers -->
          <div class="mt-4 space-y-3">
            <label class="flex items-center gap-2 text-xs text-slate-300 cursor-pointer">
              <input type="checkbox" bind:checked={isBusiness} class="w-4 h-4 accent-orange-600" />
              I am ordering for a company
            </label>
            {#if isBusiness}
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label for="checkout-company" class="block text-xs font-semibold text-slate-400 mb-1">Company name</label>
                  <input id="checkout-company" bind:value={buyerCompany} autocomplete="organization" class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500" />
                </div>
                {#if store.eu_business_vat_free}
                  <div>
                    <label for="checkout-vat" class="block text-xs font-semibold text-slate-400 mb-1">VAT ID (EU businesses outside {store.address_country || 'our country'})</label>
                    <div class="flex gap-2">
                      <input id="checkout-vat" bind:value={vatId} on:blur={checkVatId} placeholder="e.g. FR12345678901" class="flex-1 min-w-0 px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm font-mono focus:outline-none focus:border-orange-500" />
                      <button type="button" on:click={checkVatId} disabled={vatChecking || !vatId.trim()} class="px-3 rounded-xl bg-slate-800 hover:bg-slate-700 disabled:opacity-50 text-white text-xs font-bold">{vatChecking ? '…' : 'Check'}</button>
                    </div>
                    {#if vatCheck}
                      <p class="mt-1 text-[11px] {vatCheck.valid ? 'text-emerald-400' : 'text-rose-400'}">
                        {#if vatCheck.valid}✓ Valid{#if vatCheck.name && vatCheck.name !== '---'} · {vatCheck.name}{/if}{:else}{vatCheck.error || 'This VAT ID is not valid according to the EU VIES service.'}{/if}
                      </p>
                    {/if}
                  </div>
                {/if}
              </div>
              {#if store.eu_business_vat_free}
                <p class="text-[11px] text-slate-500">With a valid VAT ID from another EU country the order is VAT-free (reverse charge / intra-Community supply). The ID is checked with the EU VIES service.</p>
              {/if}
            {/if}
          </div>
        </div>

        <!-- 2. Shipping Address & Country Zone -->
        <div class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-base font-bold text-white flex items-center gap-2">
              <span class="w-6 h-6 rounded-full bg-orange-600 text-white text-xs font-bold flex items-center justify-center">2</span>
              Shipping Destination & Country Zone
            </h2>
            {#if selectedAddressId}
              <span class="text-[11px] text-orange-400 bg-orange-500/10 border border-orange-500/20 px-2 py-0.5 rounded-full font-semibold">
                Saved Address Selected
              </span>
            {/if}
          </div>

          <!-- Saved Address Selector for Logged In User -->
          {#if savedAddresses.length > 0}
            <div class="p-3.5 rounded-xl bg-slate-950/80 border border-slate-800/80 space-y-2 mb-5">
              <div class="flex items-center justify-between text-xs font-semibold text-slate-300">
                <span class="flex items-center gap-1.5"><MapPin size={13} class="text-orange-400" /> Choose from your saved addresses:</span>
              </div>
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {#each savedAddresses as addr}
                  <button
                    type="button"
                    on:click={() => selectSavedAddress(addr)}
                    class="p-2.5 rounded-xl border text-left text-xs transition-all {selectedAddressId === addr.id ? 'bg-orange-600/15 border-orange-500 text-white ring-1 ring-orange-500 font-semibold' : 'bg-slate-900 text-slate-300 border-slate-800 hover:border-slate-700'}"
                  >
                    <div class="flex items-center justify-between">
                      <span class="font-bold text-white truncate">{addr.full_name || customerName}</span>
                      {#if addr.is_default}
                        <span class="text-[9px] uppercase px-1.5 py-0.2 rounded bg-orange-500/20 text-orange-400 font-mono">Default</span>
                      {/if}
                    </div>
                    <div class="text-[11px] text-slate-400 truncate mt-0.5">{addr.street_address}</div>
                    <div class="text-[10px] text-slate-500 font-mono">{addr.postal_code} {addr.city} ({addr.country_code})</div>
                  </button>
                {/each}
              </div>
            </div>
          {/if}

          <div class="space-y-4">
            <div>
              <label for="checkout-country" class="block text-xs font-semibold text-slate-400 mb-1">Country / Region</label>
              <CountryInput id="checkout-country" {countries} bind:value={countryCode} on:change={fetchShippingRates} />
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="sm:col-span-2">
                <label for="checkout-street" class="block text-xs font-semibold text-slate-400 mb-1">Street Address & House Number</label>
                <input
                  id="checkout-street"
                  name="address-line1"
                  type="text"
                  autocomplete="street-address"
                  bind:value={streetAddress}
                  placeholder="e.g. Musterstraße 123"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
                />
              </div>

              <div class="sm:col-span-2">
                <label for="checkout-apartment" class="block text-xs font-semibold text-slate-400 mb-1">Apartment, Suite, Unit, Company (optional)</label>
                <input
                  id="checkout-apartment"
                  name="address-line2"
                  type="text"
                  autocomplete="address-line2"
                  bind:value={apartmentSuite}
                  placeholder="e.g. Apt 4B or c/o Mustermann"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
                />
              </div>

              <div>
                <label for="checkout-postal" class="block text-xs font-semibold text-slate-400 mb-1">Postal Code</label>
                <input
                  id="checkout-postal"
                  name="postal-code"
                  type="text"
                  autocomplete="postal-code"
                  bind:value={postalCode}
                  placeholder="e.g. 10115"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
                />
              </div>

              <div>
                <label for="checkout-city" class="block text-xs font-semibold text-slate-400 mb-1">City</label>
                <input
                  id="checkout-city"
                  name="city"
                  type="text"
                  autocomplete="address-level2"
                  bind:value={city}
                  placeholder="e.g. Berlin"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
                />
              </div>

              <div class="sm:col-span-2">
                <label for="checkout-state" class="block text-xs font-semibold text-slate-400 mb-1">State / Province / Region (optional)</label>
                <input
                  id="checkout-state"
                  name="state"
                  type="text"
                  autocomplete="address-level1"
                  bind:value={stateProvince}
                  placeholder="e.g. Berlin or Bayern"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
                />
              </div>
            </div>
          </div>

          <!-- Shipping Provider / Tier selection -->
          {#if hasPhysicalItems}
            <div class="mt-6 pt-5 border-t border-slate-800">
              <p class="block text-xs font-bold text-slate-300 uppercase tracking-wider mb-2">Available Shipping Options:</p>
              {#if isFetchingRates}
                <div class="text-xs text-slate-400 py-2">Recalculating zone rates...</div>
              {:else if availableRates.length === 0}
                <div class="text-xs text-slate-400 py-2">Shipping: {shippingCents > 0 ? formatEuro(shippingCents) : 'free of charge'}</div>
              {:else}
                <div class="space-y-2">
                  {#each availableRates as rate}
                    <label class="flex items-center justify-between p-3.5 rounded-xl border cursor-pointer transition-all {selectedShippingRateId === rate.id ? 'bg-orange-600/10 border-orange-500 text-white' : 'bg-slate-950/60 border-slate-800 text-slate-300 hover:border-slate-700'}">
                      <div class="flex items-center gap-3">
                        <input
                          type="radio"
                          name="shipping_rate"
                          value={rate.id}
                          bind:group={selectedShippingRateId}
                          class="accent-orange-600"
                        />
                        <div>
                          <div class="text-xs font-bold">{rate.name}</div>
                          <div class="text-[11px] text-slate-400">{rate.estimated_delivery_days}</div>
                        </div>
                      </div>
                      <div class="font-mono font-bold text-xs">{(rate.price_cents / 100).toFixed(2)} €</div>
                    </label>
                  {/each}
                </div>
              {/if}
              <!-- The delivery time for the chosen country is part of the pre-contractual information -->
              {#if selectedRateEstimate || store.default_delivery_time}
                <p class="mt-2 text-[11px] text-slate-300">
                  Delivery time to {countryName(countryCode)}: {selectedRateEstimate || store.default_delivery_time}
                </p>
              {/if}
            </div>
          {:else}
            <!-- Digital Only Delivery Banner -->
            <div class="mt-6 pt-5 border-t border-slate-800">
              <div class="p-4 rounded-xl bg-sky-500/10 border border-sky-500/30 text-sky-200 text-xs flex items-start gap-3">
                <div class="text-sky-400 text-lg mt-0.5">⚡</div>
                <div class="space-y-1">
                  <div class="font-bold text-white text-sm">Instant Digital Delivery (No Shipping Required)</div>
                  <p class="text-slate-300 text-[11px] leading-relaxed">
                    All items in your cart are digital products. Shipping fee is <strong>0.00 €</strong>. Download links and assets will be instantly available in your customer account and email upon payment confirmation.
                  </p>
                </div>
              </div>
            </div>
          {/if}
        </div>

        <!-- 3. Payment -->
        <div id="payment-section" class="p-6 rounded-2xl bg-slate-900/60 border border-slate-800">
          <h2 class="text-base font-bold text-white mb-4 flex items-center gap-2">
            <span class="w-6 h-6 rounded-full bg-orange-600 text-white text-xs font-bold flex items-center justify-center">3</span>
            Payment method
          </h2>

          {#if isFreeOrder}
            <div class="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-200 text-xs flex items-center gap-3">
              <Gift size={18} class="text-emerald-400 flex-shrink-0" />
              <span>Your order total is <strong>0.00 €</strong> — no payment is required.</span>
            </div>
          {:else if paymentMethods.length === 0}
            <div class="p-4 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs">
              No payment methods are currently available. Please contact store support.
            </div>
          {:else}
            <!-- All payment methods underneath each other; the selected one opens its form -->
            <div class="rounded-xl border border-slate-800 divide-y divide-slate-800 overflow-hidden" role="radiogroup" aria-label="Payment method">
              {#each paymentMethods as m (m.id)}
                <div class="{selectedMethodId === m.id ? 'bg-slate-950' : 'bg-slate-950/40'}">
                  <label class="flex items-center gap-3 px-4 py-3.5 cursor-pointer hover:bg-slate-900/60 transition-colors">
                    <input
                      type="radio"
                      name="payment-method"
                      value={m.id}
                      bind:group={selectedMethodId}
                      on:change={() => { errorMessage = ''; paymentElementError = ''; isSubmitting = false; }}
                      class="w-4 h-4 accent-orange-600 flex-shrink-0"
                    />
                    <span class="flex-1 min-w-0">
                      <span class="block text-sm font-bold {selectedMethodId === m.id ? 'text-white' : 'text-slate-200'}">{m.label}</span>
                      <span class="block text-[11px] text-slate-500">{m.hint}</span>
                    </span>
                    {#if m.id === 'card' || m.id === 'hosted'}
                      <CreditCard size={20} class="text-slate-400 flex-shrink-0" />
                    {/if}
                  </label>

                  {#if selectedMethodId === m.id}
                    <div class="px-4 pb-4">
                      {#if m.kind === 'element'}
                        {#if stripeLoadError}
                          <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-300 text-xs">{stripeLoadError}</div>
                        {:else if !stripe}
                          <div class="py-6 flex items-center justify-center gap-2 text-xs text-slate-400">
                            <Loader2 size={18} class="animate-spin text-orange-500" /> Loading secure payment form…
                          </div>
                        {:else}
                          {#key m.id}
                            <StripePaymentForm
                              bind:this={paymentForm}
                              bind:error={paymentElementError}
                              {stripe}
                              types={m.types}
                              amount={stripeAmount}
                              appearance={stripeAppearance}
                            />
                          {/key}
                          {#if paymentElementError}
                            <p class="mt-3 text-[11px] text-rose-300">
                              {paymentElementError}
                              {#if stripeTestMode} (Shop owner: activate this payment method in the Stripe Dashboard → Settings → Payment methods.){/if}
                            </p>
                          {/if}
                          {#if m.id === 'card'}
                            <p class="mt-3 text-[11px] text-slate-500 flex items-center gap-1.5">
                              <Lock size={12} class="text-emerald-400" /> Your card details are encrypted and never stored by us.
                            </p>
                          {/if}
                        {/if}
                      {:else if m.kind === 'express'}
                        <p class="text-xs text-slate-400">Use the {m.label} button next to the order summary to pay.</p>
                      {:else if m.kind === 'paypal'}
                        <p class="text-xs text-slate-400">Use the PayPal button next to the order summary to pay with your PayPal account.</p>
                      {:else if m.kind === 'hosted'}
                        <p class="text-xs text-slate-400 flex items-start gap-2">
                          <ExternalLink size={14} class="text-orange-400 flex-shrink-0 mt-0.5" />
                          After clicking "Order with Obligation to Pay" you will be forwarded to a secure payment page to complete your payment.
                        </p>
                      {/if}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>

            {#if (selectedProvider === 'stripe' && stripeTestMode) || (selectedProvider === 'paypal' && paypalProvider?.is_sandbox)}
              <div class="mt-4 p-3 rounded-xl bg-amber-500/10 border border-amber-500/20 text-[11px] text-amber-300 flex items-center gap-2">
                <ShieldCheck size={14} class="text-amber-400" />
                <span>Test mode — no real money is charged.{selectedMethodId === 'card' ? ' Use card 4242 4242 4242 4242, any future date and any CVC.' : ''}</span>
              </div>
            {/if}
          {/if}

          <!-- Invisible probe: tells us which button methods (Apple Pay, Google Pay, Amazon Pay, Link) this device can use -->
          {#if stripe && !stripeHosted && expressCandidates.length > 0 && !isFreeOrder}
            <div class="absolute -left-[10000px] top-0 w-[300px] h-px overflow-hidden" aria-hidden="true">
              {#key expressCandidates.join(',')}
                <StripeExpressButton
                  {stripe}
                  types={expressProbeTypes(expressCandidates)}
                  amount={stripeAmount}
                  appearance={stripeAppearance}
                  paymentMethods={expressButtons(expressCandidates)}
                  onready={onExpressProbe}
                />
              {/key}
            </div>
          {/if}
        </div>
      </div>

      <!-- Order Summary -->
      <div class="lg:col-span-5">
        <div class="sticky top-28 p-6 rounded-2xl bg-slate-900 border border-slate-800 shadow-xl space-y-6">
          <h2 class="text-base font-bold text-white tracking-tight border-b border-slate-800 pb-3">
            Order Summary ({$cartCount} {$cartCount === 1 ? 'item' : 'items'})
          </h2>

          <div class="space-y-3 max-h-60 overflow-y-auto pr-1 divide-y divide-slate-800/60">
            {#each $cart as item}
              <div class="pt-3 first:pt-0 flex items-center justify-between text-xs">
                <div class="min-w-0 pr-3">
                  <div class="font-bold text-white truncate">{item.product_title}</div>
                  <div class="text-slate-400 truncate">{item.variant_title} &times; {item.quantity}</div>
                </div>
                <div class="font-mono font-bold text-white flex-shrink-0">{formatEuro(item.price_cents * item.quantity)}</div>
              </div>
            {/each}
          </div>

          <!-- Promo Code -->
          <div class="border-t border-slate-800 pt-4">
            <label for="checkout-promo-input" class="block text-xs font-semibold text-slate-300 mb-1.5 flex items-center gap-1.5">
              <Tag size={13} class="text-orange-400" />
              <span>Promo / Discount Code</span>
            </label>
            {#if appliedCoupon}
              <div class="p-2.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-between">
                <div class="text-xs">
                  <span class="font-mono font-bold text-emerald-400 uppercase">{appliedCoupon.code}</span>
                  <span class="text-[11px] text-slate-400 ml-1.5">({appliedCoupon.message})</span>
                </div>
                <button
                  type="button"
                  on:click={removeCoupon}
                  class="text-[11px] text-slate-400 hover:text-rose-400 font-semibold px-2 py-0.5 rounded hover:bg-rose-500/10 transition-colors"
                >
                  Remove
                </button>
              </div>
            {:else}
              <div class="flex items-center gap-2">
                <input
                  id="checkout-promo-input"
                  type="text"
                  bind:value={couponInput}
                  on:keydown={(e) => e.key === 'Enter' && applyCoupon()}
                  placeholder="Enter code (e.g. SUMMER10)"
                  class="flex-1 px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-white font-mono uppercase text-xs focus:outline-none focus:border-orange-500"
                />
                <button
                  type="button"
                  on:click={applyCoupon}
                  disabled={isApplyingCoupon || !couponInput.trim()}
                  class="px-3.5 py-2 rounded-xl bg-slate-800 hover:bg-orange-600 disabled:opacity-40 text-white font-bold text-xs transition-colors"
                >
                  {isApplyingCoupon ? '...' : 'Apply'}
                </button>
              </div>
              {#if couponError}
                <p class="text-[11px] text-rose-400 mt-1">{couponError}</p>
              {/if}
            {/if}
          </div>

          <!-- Totals (server-calculated) -->
          <div class="border-t border-slate-800 pt-4 space-y-2 text-xs" aria-live="polite">
            <div class="flex justify-between text-slate-400">
              <span>Subtotal</span>
              <span class="font-mono text-slate-200">{formatEuro(itemsSubtotalCents)}</span>
            </div>
            {#if appliedCoupon && discountCents > 0}
              <div class="flex justify-between text-emerald-400 font-semibold">
                <span>Discount ({appliedCoupon.code})</span>
                <span class="font-mono">{appliedCoupon.discount_type === 'free_shipping' ? 'Free Shipping' : `-${formatEuro(discountCents)}`}</span>
              </div>
            {/if}
            <div class="flex justify-between text-slate-400">
              <span>{hasPhysicalItems ? `Shipping (${countryCode})` : 'Digital Delivery'}</span>
              <span class="font-mono {shippingCents > 0 ? 'text-slate-200' : 'text-emerald-400 font-bold'}">
                {shippingCents === 0 ? '0.00 € (Free)' : formatEuro(shippingCents)}
              </span>
            </div>
            {#if taxMode === 'kleingewerbe'}
              <div class="flex justify-between text-slate-400">
                <span>VAT (§ 19 UStG)</span>
                <span class="font-mono text-slate-400">0.00 €</span>
              </div>
              <p class="text-[10px] text-slate-500 italic">{store.tax_notice || 'According to § 19 UStG, no value-added tax is charged (small business regulation).'}</p>
            {:else if reverseCharge}
              {#if taxMode === 'included' && itemsSubtotalCents - discountCents + shippingCents > totalCents}
                <div class="flex justify-between text-emerald-400">
                  <span>VAT not charged</span>
                  <span class="font-mono">−{formatEuro(itemsSubtotalCents - discountCents + shippingCents - totalCents)}</span>
                </div>
              {/if}
              <p class="text-[10px] text-emerald-300/80">VAT-free EU business order (reverse charge / intra-Community supply) — VAT ID {vatId.trim().toUpperCase()}</p>
            {:else}
              <div class="flex justify-between text-slate-400">
                <span>{taxMode === 'included' ? 'Included VAT' : 'VAT'}</span>
                <span class="font-mono text-slate-200">{formatEuro(taxCents)}</span>
              </div>
            {/if}
            <div class="border-t border-slate-800 pt-3 flex justify-between items-center text-base font-bold text-white">
              <span>Total</span>
              <span class="font-mono text-orange-400 flex items-center gap-2">
                {#if isQuoting}<Loader2 size={14} class="animate-spin text-slate-500" />{/if}
                {formatEuro(totalCents)}
              </span>
            </div>
            {#if quoteError}
              <p class="text-[11px] text-rose-400">{quoteError}</p>
            {/if}
          </div>

          <!-- Statutory German & EU Legal Checkboxes -->
          <div class="pt-2 border-t border-slate-800/80 space-y-3">
            <label class="flex items-start gap-2.5 cursor-pointer text-xs">
              <input
                type="checkbox"
                bind:checked={acceptedTerms}
                required
                class="mt-0.5 w-4 h-4 rounded bg-slate-950 border border-slate-700 text-orange-600 focus:ring-orange-500 accent-orange-600 flex-shrink-0 cursor-pointer"
              />
              <span class="text-[11px] text-slate-400 leading-relaxed">
                I have read and agree to the <a href="/policies/terms-conditions" target="_blank" class="text-orange-400 underline hover:text-orange-300">Terms & Conditions (AGB)</a> and acknowledge the <a href="/policies/revocation-policy" target="_blank" class="text-orange-400 underline hover:text-orange-300">Revocation Policy (Widerrufsbelehrung)</a>.
              </span>
            </label>

            {#if hasDigitalItems}
              <label class="flex items-start gap-2.5 cursor-pointer text-xs">
                <input
                  type="checkbox"
                  bind:checked={acceptedDigitalWaiver}
                  required
                  class="mt-0.5 w-4 h-4 rounded bg-slate-950 border border-slate-700 text-orange-600 focus:ring-orange-500 accent-orange-600 flex-shrink-0 cursor-pointer"
                />
                <span class="text-[11px] text-sky-300/90 leading-relaxed">
                  I expressly agree and demand that the execution of the contract for digital content begins before the expiration of the 14-day statutory revocation period. I confirm my knowledge that I lose my right of revocation with the start of execution (§ 356 Abs. 5 BGB).
                </span>
              </label>
            {/if}

            {#if store.newsletter_enabled}
              <!-- Optional and unticked by default; confirmed by email (double opt-in) -->
              <label class="flex items-start gap-2.5 cursor-pointer text-xs">
                <input type="checkbox" bind:checked={newsletterOptIn} class="mt-0.5 w-4 h-4 accent-orange-600 flex-shrink-0 cursor-pointer" />
                <span class="text-[11px] text-slate-400 leading-relaxed">
                  Optional: I would like to receive the newsletter by email. I will get an email to confirm the subscription and can unsubscribe at any time.
                </span>
              </label>
            {/if}
          </div>

          {#if errorMessage}
            <div role="alert" class="p-3.5 rounded-xl bg-rose-500/15 border border-rose-500/30 text-rose-300 text-xs font-semibold flex items-start gap-2.5">
              <AlertCircle size={16} class="text-rose-400 flex-shrink-0 mt-0.5" />
              <div class="space-y-0.5">
                <div class="font-bold text-rose-200">Unable to Complete Payment</div>
                <div class="text-[11px] leading-relaxed text-rose-300">{errorMessage}</div>
              </div>
            </div>
          {/if}

          <!-- Button-Lösung gem. § 312j Abs. 3 BGB -->
          {#if selectedMethod?.kind === 'express' && !isFreeOrder && stripe}
            <div class="space-y-2">
              <p class="text-xs font-bold text-white text-center">Order with Obligation to Pay ({formatEuro(totalCents)}) via {selectedMethod.label}:</p>
              {#if isSubmitting}
                <div class="py-4 flex items-center justify-center gap-2 text-xs text-slate-300">
                  <Loader2 size={16} class="animate-spin text-orange-500" /> Confirming your payment…
                </div>
              {/if}
              <!-- Never hide the button while paying: its Stripe frame is needed until the payment completes -->
              <div>
                {#key selectedMethodId}
                  <StripeExpressButton
                    {stripe}
                    types={selectedMethod.types}
                    amount={stripeAmount}
                    appearance={stripeAppearance}
                    paymentMethods={expressButtons([selectedMethodId])}
                    buttonType={{ applePay: 'order', googlePay: 'order' }}
                    onclick={onWalletClick}
                    onconfirm={onWalletConfirm}
                    oncancel={() => (isSubmitting = false)}
                  />
                {/key}
              </div>
            </div>
          {:else if selectedProvider === 'paypal' && !isFreeOrder && paypalProvider}
            <div class="space-y-2">
              <p class="text-xs font-bold text-white text-center">Order with Obligation to Pay ({formatEuro(totalCents)}) via PayPal:</p>
              {#if isSubmitting}
                <div class="py-4 flex items-center justify-center gap-2 text-xs text-slate-300">
                  <Loader2 size={16} class="animate-spin text-orange-500" /> Confirming your PayPal payment…
                </div>
              {/if}
              <div class:hidden={isSubmitting} class="rounded-xl overflow-hidden bg-white/95 p-2" use:paypalButtons={paypalProvider}></div>
              {#if paypalSdkError}
                <p class="text-[11px] text-rose-400 text-center">{paypalSdkError}</p>
              {/if}
            </div>
          {:else}
            <button
              id="submit-order-btn"
              type="button"
              on:click={handleSubmitOrder}
              disabled={isSubmitting || !acceptedTerms || (hasDigitalItems && !acceptedDigitalWaiver) || (!isFreeOrder && paymentMethods.length === 0)}
              class="w-full py-4 px-6 rounded-xl bg-gradient-to-r from-orange-600 to-amber-500 hover:from-orange-500 hover:to-amber-400 text-white font-bold text-sm shadow-xl shadow-orange-600/30 transition-all flex items-center justify-center gap-2 hover:scale-[1.01] disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {#if isSubmitting}
                <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                <span>Processing payment…</span>
              {:else}
                <span>Order with Obligation to Pay ({formatEuro(totalCents)})</span>
                <ArrowRight size={16} />
              {/if}
            </button>
          {/if}

          <p class="text-[11px] text-slate-400 text-center leading-relaxed">
            By placing your order you conclude a legally binding purchase contract.
          </p>
        </div>
      </div>
    </div>
  {/if}
</div>
