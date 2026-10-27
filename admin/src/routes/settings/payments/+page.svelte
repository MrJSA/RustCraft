<script>
  import PaymentsManager from '$lib/components/PaymentsManager.svelte';

  export let data;
</script>

<svelte:head>
  <title>Payment Providers | RustCraft Admin</title>
</svelte:head>

<div class="max-w-5xl mx-auto">
  <PaymentsManager configs={data.paymentConfigs || []} />
</div>
