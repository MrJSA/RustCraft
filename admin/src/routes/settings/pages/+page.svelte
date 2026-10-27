<script>
  import PolicyManager from '$lib/components/PolicyManager.svelte';
  export let data;
</script>

<svelte:head>
  <title>Policy Pages CMS (Markdown) | RustCraft Admin</title>
</svelte:head>

<div class="max-w-7xl mx-auto">
  <PolicyManager pages={data.pages || []} />
</div>
