<script>
  // Stripe payment form for exactly the given payment method type(s).
  // Owns its own Elements group: created on mount, destroyed on unmount — never shared.
  import { onMount, onDestroy } from 'svelte';

  export let stripe;
  export let types = ['card'];
  export let amount;
  export let appearance = {};
  export let error = '';

  let node;
  let elements = null;
  let element = null;
  let ready = false;

  onMount(() => {
    elements = stripe.elements({ mode: 'payment', currency: 'eur', amount, paymentMethodTypes: types, appearance });
    element = elements.create('payment', {
      layout: { type: 'tabs' },
      // Wallets and Link have their own entries in the checkout list (Link would otherwise prompt inside the card form)
      wallets: { applePay: 'never', googlePay: 'never', link: 'never' }
    });
    element.on('ready', () => (ready = true));
    element.on('loaderror', (e) => (error = e.error?.message || 'The payment form could not be loaded.'));
    element.mount(node);
  });

  $: if (elements && amount > 0) elements.update({ amount });

  onDestroy(() => {
    element?.destroy();
    element = null;
    elements = null;
  });

  /** The Elements group of this mounted form, or null while it is still loading. */
  export function getElements() {
    return element && ready ? elements : null;
  }
</script>

<div bind:this={node}></div>
