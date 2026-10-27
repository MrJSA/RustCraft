/** Plain-text meta description from markdown/HTML, trimmed to ~160 characters on a word boundary. */
export function toDescription(text, maxLength = 160) {
  if (!text) return '';
  const plain = String(text)
    .replace(/<[^>]*>/g, ' ')
    .replace(/!\[[^\]]*\]\([^)]*\)/g, ' ')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/^\s*(?:[-*+]|\d+\.)\s+/gm, ' ')
    .replace(/[#*`>|~]+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
  if (plain.length <= maxLength) return plain;
  const cut = plain.slice(0, maxLength - 1);
  const lastSpace = cut.lastIndexOf(' ');
  return `${(lastSpace > maxLength * 0.6 ? cut.slice(0, lastSpace) : cut).replace(/[,.;:\s]+$/, '')}…`;
}

/** Absolute URL for crawlers and social cards (relative upload paths → https://shop/uploads/...). */
export function absoluteUrl(origin, path) {
  if (!path) return '';
  if (/^https?:\/\//i.test(path)) return path;
  return `${origin}${path.startsWith('/') ? '' : '/'}${path}`;
}

/** Serialises JSON-LD so that content can never terminate the surrounding <script> element. */
export function jsonLdScript(data) {
  return `<script type="application/ld+json">${JSON.stringify(data).replace(/</g, '\\u003c')}</` + 'script>';
}

/** Paths that must never appear in search results (customer data, checkout, order status). */
export const PRIVATE_PATH = /^\/(account|checkout|order-success|track|withdraw|newsletter)(\/|$)/;
