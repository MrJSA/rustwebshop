// Payment methods offered at checkout. Keys match `config_data.methods` in the admin and
// `stripe::OPTIONAL_METHODS` in the backend; card is always on.
export const STRIPE_OPTIONAL_METHODS = [
  { key: 'link', label: 'Link', hint: 'Pay with your saved Link details' },
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

// Display order in the checkout list
const ORDER = ['card', 'paypal', 'apple_pay', 'google_pay', 'amazon_pay', 'klarna', 'link', 'sepa_debit', 'ideal', 'bancontact', 'eps', 'p24', 'revolut_pay', 'mobilepay', 'alipay', 'wechat_pay'];

/**
 * One entry per payment method, all listed underneath each other at checkout.
 * kind: 'element' = Stripe form for exactly this method, 'wallet' = Apple Pay / Google Pay button,
 *       'paypal' = PayPal's own buttons, 'hosted' = redirect to the Stripe payment page.
 */
export function checkoutMethods({ stripeEnabled, stripeHosted, stripeMethods = {}, paypalEnabled, walletAvailability = {} }) {
  const list = [];
  if (stripeEnabled && stripeHosted) {
    list.push({ id: 'hosted', kind: 'hosted', label: 'Credit / Debit card & more', hint: 'You will be forwarded to a secure payment page' });
  } else if (stripeEnabled) {
    list.push({ id: 'card', kind: 'element', type: 'card', label: 'Credit / Debit card', hint: 'Visa, Mastercard, American Express' });
    if (stripeMethods.apple_pay && walletAvailability.apple_pay) {
      list.push({ id: 'apple_pay', kind: 'wallet', label: 'Apple Pay', hint: 'Pay with Face ID or Touch ID' });
    }
    if (stripeMethods.google_pay && walletAvailability.google_pay) {
      list.push({ id: 'google_pay', kind: 'wallet', label: 'Google Pay', hint: 'Pay with a card saved in your Google account' });
    }
    for (const m of STRIPE_OPTIONAL_METHODS) {
      // PayPal's own integration takes precedence over PayPal via Stripe
      if (stripeMethods[m.key] && !(m.key === 'paypal' && paypalEnabled)) {
        list.push({ id: m.key, kind: 'element', type: m.key, label: m.label, hint: m.hint });
      }
    }
  }
  if (paypalEnabled) {
    list.push({ id: 'paypal', kind: 'paypal', label: 'PayPal', hint: 'Pay with your PayPal account or Pay Later' });
  }
  return list.sort((a, b) => ORDER.indexOf(a.id === 'hosted' ? 'card' : a.id) - ORDER.indexOf(b.id === 'hosted' ? 'card' : b.id));
}
