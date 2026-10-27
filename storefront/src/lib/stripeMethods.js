// Payment methods offered at checkout. Keys match `config_data.methods` in the admin and
// `stripe::OPTIONAL_METHODS` in the backend; card is always on.
export const STRIPE_OPTIONAL_METHODS = [
  { key: 'link', label: 'Link', hint: 'Log in to Link and pay with your saved details' },
  { key: 'amazon_pay', label: 'Amazon Pay', hint: 'Pay with your Amazon account' },
  { key: 'paypal', label: 'PayPal', hint: 'Pay with your PayPal account' },
  { key: 'klarna', label: 'Klarna', hint: 'Pay now, later or in instalments' },
  { key: 'sepa_debit', label: 'SEPA Direct Debit', hint: 'Direct debit from your bank account' },
  { key: 'ideal', label: 'iDEAL', hint: 'Online banking (Netherlands)' },
  { key: 'bancontact', label: 'Bancontact', hint: 'Online banking (Belgium)' },
  { key: 'eps', label: 'EPS', hint: 'Online banking (Austria)' },
  { key: 'p24', label: 'Przelewy24', hint: 'Online banking (Poland)' },
  { key: 'revolut_pay', label: 'Revolut Pay', hint: 'Pay with the Revolut app' },
  { key: 'mobilepay', label: 'MobilePay', hint: 'Pay with the MobilePay app' },
  { key: 'alipay', label: 'Alipay', hint: 'Pay with Alipay' },
  { key: 'wechat_pay', label: 'WeChat Pay', hint: 'Pay with WeChat' }
];

/**
 * Methods paid with a branded button (Stripe Express Checkout) instead of a form.
 * types = Stripe payment method types of the button's Elements group / PaymentIntent.
 */
export const EXPRESS_METHODS = {
  apple_pay: { label: 'Apple Pay', hint: 'Pay with Face ID or Touch ID', button: 'applePay', types: ['card'] },
  google_pay: { label: 'Google Pay', hint: 'Pay with a card saved in your Google account', button: 'googlePay', types: ['card'] },
  amazon_pay: { label: 'Amazon Pay', hint: 'You will be forwarded to Amazon Pay', button: 'amazonPay', types: ['amazon_pay'] },
  link: { label: 'Link', hint: 'Log in to Link and pay with your saved details', button: 'link', types: ['link', 'card'] }
};

const ALL_BUTTONS = ['applePay', 'googlePay', 'amazonPay', 'link', 'paypal', 'klarna'];

/** Express Checkout options showing only the given buttons. */
export function expressButtons(methodIds) {
  const wanted = new Set(methodIds.map((id) => EXPRESS_METHODS[id]?.button).filter(Boolean));
  return Object.fromEntries(
    ALL_BUTTONS.map((b) => [b, wanted.has(b) ? (b === 'applePay' || b === 'googlePay' ? 'always' : 'auto') : 'never'])
  );
}

/** Stripe types needed to probe the availability of the given express methods. */
export function expressProbeTypes(methodIds) {
  return [...new Set(methodIds.flatMap((id) => EXPRESS_METHODS[id]?.types || []))];
}

// Display order in the checkout list
const ORDER = ['card', 'paypal', 'apple_pay', 'google_pay', 'amazon_pay', 'klarna', 'link', 'sepa_debit', 'ideal', 'bancontact', 'eps', 'p24', 'revolut_pay', 'mobilepay', 'alipay', 'wechat_pay'];

/**
 * One entry per payment method, all listed underneath each other at checkout.
 * activeMethods: methods enabled in the admin AND activated in the Stripe account (from the backend).
 * expressAvailability: which button methods this device/browser can use (from the probe).
 * kind: 'element' = Stripe form for exactly this method, 'express' = branded button,
 *       'paypal' = PayPal's own buttons, 'hosted' = redirect to the Stripe payment page.
 */
export function checkoutMethods({ stripeEnabled, stripeHosted, activeMethods = [], paypalEnabled, expressAvailability = {} }) {
  const list = [];
  if (stripeEnabled && stripeHosted) {
    list.push({ id: 'hosted', kind: 'hosted', label: 'Credit / Debit card & more', hint: 'You will be forwarded to a secure payment page' });
  } else if (stripeEnabled) {
    for (const id of activeMethods) {
      if (id === 'card') {
        list.push({ id, kind: 'element', types: ['card'], label: 'Credit / Debit card', hint: 'Visa, Mastercard, American Express' });
      } else if (EXPRESS_METHODS[id]) {
        if (expressAvailability[id]) list.push({ id, kind: 'express', ...EXPRESS_METHODS[id] });
      } else if (id === 'paypal' && paypalEnabled) {
        // PayPal's own integration takes precedence over PayPal via Stripe
      } else {
        const m = STRIPE_OPTIONAL_METHODS.find((x) => x.key === id);
        if (m) list.push({ id, kind: 'element', types: [id], label: m.label, hint: m.hint });
      }
    }
  }
  if (paypalEnabled) {
    list.push({ id: 'paypal', kind: 'paypal', label: 'PayPal', hint: 'Pay with your PayPal account or Pay Later' });
  }
  const rank = (m) => ORDER.indexOf(m.id === 'hosted' ? 'card' : m.id);
  return list.sort((a, b) => rank(a) - rank(b));
}
