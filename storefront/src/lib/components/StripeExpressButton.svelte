<script>
  // Branded one-click button (Apple Pay, Google Pay, Amazon Pay, Link) via Stripe's Express Checkout.
  // Owns its own Elements group: created on mount, destroyed on unmount — never shared.
  import { onMount, onDestroy } from 'svelte';

  export let stripe;
  export let types = ['card'];
  export let amount;
  export let appearance = {};
  /** e.g. { applePay: 'always', googlePay: 'never', amazonPay: 'never', link: 'never', paypal: 'never', klarna: 'never' } */
  export let paymentMethods = {};
  export let buttonType = {};
  export let onready = () => {};
  export let onclick = (event) => event.resolve();
  export let onconfirm = () => {};
  export let oncancel = () => {};
  export let onloaderror = () => {};

  let node;
  let elements = null;
  let element = null;

  onMount(() => {
    elements = stripe.elements({ mode: 'payment', currency: 'eur', amount, paymentMethodTypes: types, appearance });
    element = elements.create('expressCheckout', { paymentMethods, buttonType, buttonHeight: 48 });
    element.on('ready', (e) => onready(e));
    element.on('click', (e) => onclick(e));
    element.on('confirm', (e) => onconfirm(e, elements));
    element.on('cancel', () => oncancel());
    element.on('loaderror', (e) => onloaderror(e));
    element.mount(node);
  });

  $: if (elements && amount > 0) elements.update({ amount });

  onDestroy(() => {
    element?.destroy();
    element = null;
    elements = null;
  });
</script>

<div bind:this={node}></div>
