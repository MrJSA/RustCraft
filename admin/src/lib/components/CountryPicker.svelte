<script>
  // Pick countries by name (English or German) or code; the ISO codes are stored.
  import { X, Globe2 } from 'lucide-svelte';
  import { searchCountries, countryName } from '$lib/countries.js';
  import { REGIONS, REGION_GROUPS } from '$lib/regions.js';

  export let codes = [];
  export let id = 'country-picker';

  let query = '';
  let open = false;
  let highlighted = 0;

  $: results = searchCountries(query, codes).slice(0, 12);
  $: if (highlighted >= results.length) highlighted = 0;

  function add(code) {
    if (!codes.includes(code)) codes = [...codes, code];
    query = '';
    highlighted = 0;
  }
  function remove(code) {
    codes = codes.filter((c) => c !== code);
  }
  let regionId = '';
  $: region = REGIONS.find((r) => r.id === regionId);
  $: regionNew = region ? region.codes.filter((c) => !codes.includes(c)).length : 0;
  $: regionPresent = region ? region.codes.filter((c) => codes.includes(c)).length : 0;
  function addRegion() {
    if (region) codes = [...codes, ...region.codes.filter((c) => !codes.includes(c))];
  }
  // e.g. "Africa (all)" added, then "North Africa" removed
  function removeRegion() {
    if (region) codes = codes.filter((c) => !region.codes.includes(c));
  }
  function onKeydown(e) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      open = true;
      highlighted = Math.min(highlighted + 1, results.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlighted = Math.max(highlighted - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (results[highlighted]) add(results[highlighted].code);
    } else if (e.key === 'Backspace' && !query && codes.length) {
      remove(codes[codes.length - 1]);
    } else if (e.key === 'Escape') {
      open = false;
    }
  }
</script>

<div class="space-y-2">
  <div class="flex flex-wrap gap-1.5 min-h-[28px] max-h-40 overflow-y-auto pr-1">
    {#each codes as code}
      <span class="inline-flex items-center gap-1 pl-2 pr-1 py-0.5 rounded-lg bg-orange-500/10 border border-orange-500/30 text-orange-200 text-[11px]">
        <span class="font-mono font-bold">{code}</span> {countryName(code)}
        <button type="button" on:click={() => remove(code)} class="p-0.5 rounded hover:bg-orange-500/20" aria-label="Remove {countryName(code)}"><X size={11} /></button>
      </span>
    {:else}
      <span class="text-[11px] text-slate-500">No countries yet — type a country name below.</span>
    {/each}
  </div>
  <div class="relative">
    <input
      {id}
      type="text"
      bind:value={query}
      on:focus={() => (open = true)}
      on:input={() => (open = true)}
      on:blur={() => setTimeout(() => (open = false), 150)}
      on:keydown={onKeydown}
      autocomplete="off"
      placeholder="Type a country, e.g. Austria / Österreich / AT"
      class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-xs focus:outline-none focus:border-orange-500"
    />
    {#if open && results.length > 0}
      <ul class="absolute z-30 mt-1 w-full max-h-60 overflow-auto rounded-xl bg-slate-900 border border-slate-700 shadow-2xl py-1" role="listbox">
        {#each results as c, i}
          <li role="option" aria-selected={i === highlighted}>
            <button
              type="button"
              on:mousedown|preventDefault={() => add(c.code)}
              on:mouseenter={() => (highlighted = i)}
              class="w-full text-left px-3 py-1.5 text-xs flex items-center justify-between gap-2 {i === highlighted ? 'bg-orange-600/20 text-white' : 'text-slate-300'}"
            >
              <span>{c.name}{#if c.german && c.german !== c.name} <span class="text-slate-500">· {c.german}</span>{/if}</span>
              <span class="font-mono text-[10px] text-slate-500">{c.code}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
  <div class="flex flex-wrap items-center gap-2">
    <select bind:value={regionId} aria-label="Region" class="flex-1 min-w-[200px] px-2.5 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-white text-[11px] focus:outline-none focus:border-orange-500">
      <option value="">Region…</option>
      {#each REGION_GROUPS as group}
        <optgroup label={group}>
          {#each REGIONS.filter((r) => r.group === group) as r}
            <option value={r.id}>{r.label} ({r.codes.length})</option>
          {/each}
        </optgroup>
      {/each}
    </select>
    <button type="button" on:click={addRegion} disabled={!regionNew} class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 disabled:opacity-40 text-slate-200 text-[11px] font-bold flex items-center gap-1"><Globe2 size={12} /> Add{regionNew ? ` ${regionNew}` : ''}</button>
    <button type="button" on:click={removeRegion} disabled={!regionPresent} class="px-2.5 py-1 rounded-lg bg-slate-900 border border-slate-800 hover:text-rose-300 disabled:opacity-40 text-slate-400 text-[11px] font-bold">Remove{regionPresent ? ` ${regionPresent}` : ''}</button>
    {#if codes.length > 0}
      <button type="button" on:click={() => (codes = [])} class="px-2.5 py-1 rounded-lg bg-slate-900 border border-slate-800 hover:text-rose-300 text-slate-400 text-[11px] font-bold">Remove all</button>
    {/if}
  </div>
</div>
