// Newsletter box ticked at checkout: after the order the normal double opt-in starts
// (confirmation email). Kept in sessionStorage because card payments may redirect.
const KEY = 'rustwebshop_newsletter_opt_in';

export function rememberNewsletterOptIn(email) {
  try {
    if (email) sessionStorage.setItem(KEY, email);
    else sessionStorage.removeItem(KEY);
  } catch (_) {}
}

export async function submitRememberedNewsletterOptIn() {
  let email = null;
  try {
    email = sessionStorage.getItem(KEY);
    sessionStorage.removeItem(KEY);
  } catch (_) {}
  if (!email) return;
  try {
    await fetch('/api/v1/newsletter/subscribe', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, consent: true, source: 'checkout' })
    });
  } catch (_) {
    // the order is complete either way
  }
}
