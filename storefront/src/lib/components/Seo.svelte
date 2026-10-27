<script>
  // Per-page SEO tags (https://svelte.dev/docs/kit/seo): title, description, canonical URL,
  // OpenGraph / Twitter cards and JSON-LD structured data, all rendered server-side.
  import { page } from '$app/stores';
  import { toDescription, absoluteUrl, jsonLdScript } from '$lib/seo.js';

  export let title;
  export let description = '';
  /** Canonical path incl. relevant query (defaults to the current pathname without query) */
  export let path = null;
  export let image = '';
  export let type = 'website';
  export let noindex = false;
  /** Array of schema.org objects */
  export let jsonLd = [];

  $: origin = $page.url.origin;
  $: canonical = origin + (path ?? $page.url.pathname);
  $: metaDescription = toDescription(description);
  $: imageUrl = absoluteUrl(origin, image);
  $: siteName = $page.data?.store?.store_name || '';
</script>

<svelte:head>
  <title>{title}</title>
  {#if metaDescription}
    <meta name="description" content={metaDescription} />
  {/if}
  <link rel="canonical" href={canonical} />
  {#if noindex}
    <meta name="robots" content="noindex, follow" />
  {/if}

  <meta property="og:type" content={type} />
  <meta property="og:title" content={title} />
  <meta property="og:url" content={canonical} />
  {#if siteName}
    <meta property="og:site_name" content={siteName} />
  {/if}
  {#if metaDescription}
    <meta property="og:description" content={metaDescription} />
  {/if}
  {#if imageUrl}
    <meta property="og:image" content={imageUrl} />
  {/if}

  <meta name="twitter:card" content={imageUrl ? 'summary_large_image' : 'summary'} />
  <meta name="twitter:title" content={title} />
  {#if metaDescription}
    <meta name="twitter:description" content={metaDescription} />
  {/if}
  {#if imageUrl}
    <meta name="twitter:image" content={imageUrl} />
  {/if}

  {#each jsonLd.filter(Boolean) as data}
    {@html jsonLdScript(data)}
  {/each}
</svelte:head>
