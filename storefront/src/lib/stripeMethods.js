// Optional Stripe payment methods (EUR). Keys match `config_data.methods` in the admin and
// `stripe::OPTIONAL_METHODS` in the backend; card is always on.
export const STRIPE_OPTIONAL_METHODS = [
  { key: 'link', label: 'Link', express: true },
  { key: 'amazon_pay', label: 'Amazon Pay', express: true },
  { key: 'paypal', label: 'PayPal', express: true },
  { key: 'klarna', label: 'Klarna', express: true },
  { key: 'sepa_debit', label: 'SEPA Direct Debit' },
  { key: 'ideal', label: 'iDEAL' },
  { key: 'bancontact', label: 'Bancontact' },
  { key: 'eps', label: 'EPS' },
  { key: 'p24', label: 'Przelewy24' },
  { key: 'revolut_pay', label: 'Revolut Pay' },
  { key: 'mobilepay', label: 'MobilePay' },
  { key: 'alipay', label: 'Alipay' },
  { key: 'wechat_pay', label: 'WeChat Pay' }
];

/** Payment method types for Elements / PaymentIntent — card first so it is the default tab. */
export function paymentMethodTypes(methods = {}) {
  return ['card', ...STRIPE_OPTIONAL_METHODS.filter((m) => methods[m.key]).map((m) => m.key)];
}

export function methodLabels(methods = {}) {
  return [
    'Card',
    methods.apple_pay && 'Apple Pay',
    methods.google_pay && 'Google Pay',
    ...STRIPE_OPTIONAL_METHODS.filter((m) => methods[m.key]).map((m) => m.label)
  ].filter(Boolean);
}

/** Wallet / one-click buttons for the Express Checkout Element. */
export function expressPaymentMethods(methods = {}) {
  const on = (enabled, value = 'auto') => (enabled ? value : 'never');
  return {
    applePay: on(methods.apple_pay, 'always'),
    googlePay: on(methods.google_pay, 'always'),
    link: on(methods.link),
    amazonPay: on(methods.amazon_pay),
    paypal: on(methods.paypal),
    klarna: on(methods.klarna)
  };
}

export function hasExpressMethods(methods = {}) {
  return Object.values(expressPaymentMethods(methods)).some((v) => v !== 'never');
}
