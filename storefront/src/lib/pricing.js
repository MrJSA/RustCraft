// Prices as computed by the backend (sale, regular, lowest price of the last 30 days, unit price).

export const formatEuro = (cents) => `${((cents || 0) / 100).toFixed(2)} €`;

/** Pricing of a variant; falls back to the regular price for data without `pricing`. */
export function variantPricing(product, variant) {
  if (variant?.pricing) return variant.pricing;
  const regular = variant?.price_override_cents || product?.base_price_cents || 0;
  return { regular_cents: regular, price_cents: regular, on_sale: false, reference_cents: null, sale_ends_at: null, unit_price: null };
}

/** The cheapest variant's pricing ("from" price in listings). */
export function lowestPricing(product, variants) {
  const list = (variants && variants.length ? variants : product?.variants || []).map((v) => variantPricing(product, v));
  if (list.length === 0) return variantPricing(product, null);
  return list.reduce((best, p) => (p.price_cents < best.price_cents ? p : best), list[0]);
}

/** Percentage of an advertised reduction (relative to the 30-day lowest price). */
export function reductionPercent(pricing) {
  if (!pricing?.on_sale || !pricing.reference_cents) return 0;
  return Math.round(((pricing.reference_cents - pricing.price_cents) / pricing.reference_cents) * 100);
}

/** VAT wording next to prices (§ 1 PAngV / § 19 UStG). */
export function vatNote(store) {
  switch (store?.tax_mode) {
    case 'included':
      return 'incl. VAT';
    case 'excluded':
      return 'excl. VAT';
    default:
      return 'No VAT charged (small business, § 19 UStG)';
  }
}

/**
 * Delivery information that is the same for every visitor: digital products, or a delivery time
 * set on the product itself (e.g. made to order). Country-dependent times come from the shipping
 * zones (product page for logged-in customers with an address, otherwise at checkout).
 */
export function deliveryTime(product) {
  if (product?.product_type === 'digital') return 'Available for download immediately after payment';
  const t = (product?.delivery_time || '').trim();
  return t ? `Delivery time: ${t}` : '';
}

/** Delivery estimate of the cheapest package this variant fits into, for one country (shipping zones). */
export async function deliveryForCountry(countryCode, variantId) {
  const res = await fetch('/api/v1/checkout/shipping-options', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ country_code: countryCode, items: [{ variant_id: variantId, quantity: 1 }] })
  });
  if (!res.ok) return null;
  const data = await res.json();
  if (!data.ships) return { ships: false };
  if (!data.options.length) return { ships: true, fits: false };
  const best = data.options[0];
  return { ships: true, fits: true, estimate: (best.estimated_delivery_days || '').trim(), rateName: best.name };
}

export function formatWeight(grams) {
  if (!grams || grams <= 0) return '';
  return grams >= 1000 ? `${(grams / 1000).toLocaleString('en', { maximumFractionDigits: 2 })} kg` : `${grams} g`;
}
