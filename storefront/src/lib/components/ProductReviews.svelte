<script>
  import { onMount } from 'svelte';
  import { customer } from '$lib/stores/customer.js';
  import { Star, ShieldCheck, MessageSquare } from 'lucide-svelte';

  export let productId;

  let loaded = false;
  let enabled = false;
  let reviews = [];
  let average = null;
  let count = 0;
  let eligibility = null;

  let rating = 0;
  let hover = 0;
  let title = '';
  let body = '';
  let authorName = '';
  let sending = false;
  let notice = '';
  let error = '';

  async function load() {
    try {
      const res = await fetch(`/api/v1/products/${productId}/reviews`);
      if (res.ok) {
        const data = await res.json();
        enabled = data.enabled;
        reviews = data.reviews || [];
        average = data.average;
        count = data.count || 0;
      }
      if (enabled && $customer?.token) {
        const er = await fetch(`/api/v1/products/${productId}/reviews/eligibility`, {
          headers: { Authorization: `Bearer ${$customer.token}` }
        });
        if (er.ok) eligibility = await er.json();
      }
    } catch (_) {
      // reviews are optional; the product page works without them
    } finally {
      loaded = true;
    }
  }

  async function submit() {
    error = '';
    if (rating < 1) {
      error = 'Please choose a star rating.';
      return;
    }
    sending = true;
    try {
      const res = await fetch(`/api/v1/products/${productId}/reviews`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${$customer.token}` },
        body: JSON.stringify({ rating, title, body, author_name: authorName })
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(data.error || 'The review could not be saved.');
      notice = data.message;
      eligibility = { can_review: false, reason: 'already_reviewed', status: 'pending' };
    } catch (e) {
      error = e.message;
    } finally {
      sending = false;
    }
  }

  const fmtDate = (d) => new Date(d).toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' });

  onMount(load);
</script>

{#if loaded && enabled}
  <section class="mt-16 pt-10 border-t border-slate-800" aria-labelledby="reviews-heading">
    <div class="max-w-4xl mx-auto space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-end justify-between gap-3">
        <div>
          <h2 id="reviews-heading" class="text-xl sm:text-2xl font-black text-white tracking-tight flex items-center gap-2">
            <MessageSquare size={20} class="text-orange-400" /> Customer reviews
          </h2>
          {#if count > 0}
            <div class="mt-2 flex items-center gap-2 text-sm text-slate-300">
              <span class="flex" aria-label="{average.toFixed(1)} of 5 stars">
                {#each [1, 2, 3, 4, 5] as s}
                  <Star size={16} class={s <= Math.round(average) ? 'fill-amber-400 text-amber-400' : 'text-slate-600'} />
                {/each}
              </span>
              <span class="font-bold">{average.toFixed(1)}</span>
              <span class="text-slate-500">({count} {count === 1 ? 'review' : 'reviews'})</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- § 5b Abs. 3 UWG: how authenticity is ensured -->
      <p class="text-[11px] text-slate-400 flex items-start gap-1.5">
        <ShieldCheck size={14} class="text-emerald-400 flex-shrink-0" />
        <span>Only customers who bought this product in our shop can write a review. Every review is checked before it is published; we publish positive and negative reviews alike.</span>
      </p>

      {#if count === 0}
        <p class="text-sm text-slate-400">No reviews yet.</p>
      {:else}
        <ul class="space-y-4">
          {#each reviews as r}
            <li class="p-5 rounded-2xl bg-slate-900/60 border border-slate-800">
              <div class="flex items-center justify-between gap-2">
                <span class="flex">
                  {#each [1, 2, 3, 4, 5] as s}
                    <Star size={14} class={s <= r.rating ? 'fill-amber-400 text-amber-400' : 'text-slate-600'} />
                  {/each}
                </span>
                <span class="text-[11px] text-slate-500">{fmtDate(r.created_at)}</span>
              </div>
              {#if r.title}<h3 class="mt-2 text-sm font-bold text-white">{r.title}</h3>{/if}
              {#if r.body}<p class="mt-1 text-sm text-slate-300 whitespace-pre-line">{r.body}</p>{/if}
              <p class="mt-2 text-[11px] text-slate-500">{r.author_name} · <span class="text-emerald-400">Verified purchase</span></p>
            </li>
          {/each}
        </ul>
      {/if}

      {#if notice}
        <div class="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-300 text-sm">{notice}</div>
      {:else if eligibility?.can_review}
        <form on:submit|preventDefault={submit} class="p-5 rounded-2xl bg-slate-900/60 border border-slate-800 space-y-3">
          <h3 class="text-sm font-bold text-white">Write a review</h3>
          <div class="flex gap-1" role="radiogroup" aria-label="Rating">
            {#each [1, 2, 3, 4, 5] as s}
              <button
                type="button"
                role="radio"
                aria-checked={rating === s}
                aria-label="{s} stars"
                on:click={() => (rating = s)}
                on:mouseenter={() => (hover = s)}
                on:mouseleave={() => (hover = 0)}
              >
                <Star size={24} class={s <= (hover || rating) ? 'fill-amber-400 text-amber-400' : 'text-slate-600'} />
              </button>
            {/each}
          </div>
          <input bind:value={title} maxlength="150" placeholder="Title (optional)" class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-700 text-white text-sm focus:outline-none focus:border-orange-500" />
          <textarea bind:value={body} maxlength="5000" rows="4" placeholder="Your experience with the product" class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-700 text-white text-sm focus:outline-none focus:border-orange-500"></textarea>
          <input bind:value={authorName} maxlength="80" placeholder="Name shown (default: first name and initial)" class="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-700 text-white text-sm focus:outline-none focus:border-orange-500" />
          <p class="text-[11px] text-slate-500">Your review is published with the name shown above after a short check. You can ask us to delete it at any time. See our <a href="/policies/privacy-policy" class="underline">privacy policy</a>.</p>
          {#if error}<p class="text-xs text-rose-400">{error}</p>{/if}
          <button type="submit" disabled={sending} class="px-5 py-2.5 rounded-xl bg-orange-600 hover:bg-orange-500 disabled:opacity-50 text-white text-sm font-bold">
            {sending ? 'Sending…' : 'Submit review'}
          </button>
        </form>
      {:else if eligibility?.reason === 'already_reviewed'}
        <p class="text-xs text-slate-400">{eligibility.status === 'approved' ? 'Thank you for your review!' : 'Thank you — your review is waiting for approval.'}</p>
      {:else if !$customer?.token}
        <p class="text-xs text-slate-400">Bought this product? <a href="/account/login" class="underline hover:text-orange-400">Log in</a> to write a review.</p>
      {/if}
    </div>
  </section>
{/if}
