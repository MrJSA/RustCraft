<script>
  import MenuManager from '$lib/components/MenuManager.svelte';
  export let data;
</script>

<svelte:head>
  <title>Navigation Menus Customizer | RustCraft Admin</title>
</svelte:head>

<div class="max-w-6xl mx-auto">
  <MenuManager
    menuItems={data.menuItems || []}
    categories={data.categories || []}
    pages={data.pages || []}
  />
</div>
