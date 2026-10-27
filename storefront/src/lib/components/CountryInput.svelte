<script>
  // Text field for one country: typing a name (English or German) or code suggests a few matches
  // underneath. The ISO code is bound as `value`; the field never holds an unknown country.
  import { createEventDispatcher } from 'svelte';
  import { searchCountries } from '$lib/countries.js';

  export let value = '';
  export let countries = [];
  export let id = 'country-input';
  export let name = 'country';
  export let autocomplete = 'country-name';
  export let placeholder = 'Start typing a country';
  export let maxSuggestions = 6;

  const dispatch = createEventDispatcher();

  let text = '';
  let focused = false;
  let open = false;
  let highlighted = 0;

  const nameOf = (code) => countries.find((c) => c.code === code)?.name || code || '';

  // Show the chosen country while the field is not being edited (also after a saved address is used)
  $: if (!focused) text = nameOf(value);

  $: results = open ? searchCountries(countries, text).slice(0, maxSuggestions) : [];
  $: if (highlighted >= results.length) highlighted = 0;

  function choose(code) {
    text = nameOf(code);
    open = false;
    highlighted = 0;
    if (code !== value) {
      value = code;
      dispatch('change', code);
    }
  }

  // Leaving the field: take an exact or the only match (e.g. browser autofill), otherwise show the
  // previous country again
  function commit() {
    const q = text.trim().toLowerCase();
    const matches = searchCountries(countries, text);
    const exact = matches.find((c) => c.code.toLowerCase() === q || c.name.toLowerCase() === q || c.german.toLowerCase() === q);
    const pick = exact || (matches.length === 1 ? matches[0] : null);
    if (pick) choose(pick.code);
    else text = nameOf(value);
    open = false;
  }

  function onFocus(e) {
    focused = true;
    e.currentTarget.select();
  }

  function onBlur() {
    focused = false;
    commit();
  }

  function onKeydown(e) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      open = true;
      highlighted = Math.min(highlighted + 1, Math.max(results.length - 1, 0));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlighted = Math.max(highlighted - 1, 0);
    } else if (e.key === 'Enter') {
      // Never submit the checkout form from here
      e.preventDefault();
      if (results[highlighted]) choose(results[highlighted].code);
      else commit();
    } else if (e.key === 'Escape' && open) {
      e.preventDefault();
      text = nameOf(value);
      open = false;
    }
  }
</script>

<div class="relative">
  <input
    {id}
    {name}
    {autocomplete}
    {placeholder}
    type="text"
    role="combobox"
    aria-autocomplete="list"
    aria-expanded={results.length > 0}
    aria-controls="{id}-list"
    aria-activedescendant={results.length > 0 ? `${id}-opt-${highlighted}` : undefined}
    bind:value={text}
    on:focus={onFocus}
    on:blur={onBlur}
    on:input={() => (open = true)}
    on:change={() => !focused && commit()}
    on:keydown={onKeydown}
    class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-white text-sm focus:outline-none focus:border-orange-500"
  />
  {#if results.length > 0}
    <ul id="{id}-list" class="absolute z-30 mt-1 w-full max-h-60 overflow-auto rounded-xl bg-slate-900 border border-slate-700 shadow-2xl py-1" role="listbox">
      {#each results as c, i}
        <li
          id="{id}-opt-{i}"
          role="option"
          aria-selected={i === highlighted}
          on:mousedown|preventDefault={() => choose(c.code)}
          on:mouseenter={() => (highlighted = i)}
          class="cursor-pointer px-3.5 py-2 text-sm flex items-center justify-between gap-2 {i === highlighted ? 'bg-orange-600/20 text-white' : 'text-slate-300'}"
        >
          <span>{c.name}{#if c.german && c.german !== c.name} <span class="text-slate-500 text-xs">· {c.german}</span>{/if}</span>
          <span class="font-mono text-[10px] text-slate-500">{c.code}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>
