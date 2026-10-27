<script>
  import ShippingManager from '$lib/components/ShippingManager.svelte';
  export let data;
</script>

<svelte:head>
  <title>Shipping Carriers, Zones & Rates | RustCraft Admin</title>
</svelte:head>

<div class="max-w-6xl mx-auto space-y-6">
  <ShippingManager initialProviders={data.providers || []} />
</div>
