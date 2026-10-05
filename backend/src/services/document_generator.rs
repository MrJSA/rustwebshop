use crate::models::{Order, OrderItem, StoreSettings};

pub struct DocumentGenerator;

impl DocumentGenerator {
    pub fn generate_invoice_html(
        order: &Order,
        items: &[OrderItem],
        settings: &StoreSettings,
    ) -> String {
        let billing_addr = &order.billing_address;

        let customer_name = billing_addr
            .get("full_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(&order.customer_name);

        let street = billing_addr
            .get("street_address")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let postal_code = billing_addr
            .get("postal_code")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let city = billing_addr
            .get("city")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let country = billing_addr
            .get("country_code")
            .and_then(|v| v.as_str())
            .unwrap_or("Germany");

        let customer_email = &order.customer_email;

        let customer_phone = billing_addr
            .get("phone")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Format company address lines
        let addr_lines = settings
            .company_address
            .split(',')
            .map(|s| format!("<div>{}</div>", s.trim()))
            .collect::<Vec<_>>()
            .join("\n");

        let items_rows = items
            .iter()
            .map(|item| {
                let variant_display = if !item.variant_title.is_empty() && item.variant_title != "Standard" && item.variant_title != "Default" {
                    format!(r#"<div style="font-size: 11px; color: #475569; margin-top: 2px;">Variant: {}</div>"#, item.variant_title)
                } else {
                    String::new()
                };

                let license_tag = if item.is_digital {
                    r#"<div style="font-size: 11px; color: #0284c7; margin-top: 2px;">[Digital Download License]</div>"#
                } else {
                    ""
                };

                let formatted_price = format!("{:.2} {}", (item.total_price_cents as f64) / 100.0, settings.currency_symbol).replace('.', ",");

                format!(
                    r#"<tr style="border-bottom: 1px solid #e2e8f0;">
                        <td style="padding: 14px 10px 14px 0; vertical-align: top;">
                            <div style="font-size: 13px; font-weight: 600; color: #000;">{}</div>
                            <div style="font-size: 11px; color: #475569; margin-top: 4px;"><strong>SKU:</strong> {}</div>
                            {}
                            {}
                        </td>
                        <td style="padding: 14px 10px; vertical-align: top; text-align: center; font-size: 13px; color: #000;">
                            {}
                        </td>
                        <td style="padding: 14px 0 14px 10px; vertical-align: top; text-align: right; font-size: 13px; font-weight: 500; color: #000; white-space: nowrap;">
                            {}
                        </td>
                    </tr>"#,
                    item.product_title,
                    item.sku,
                    variant_display,
                    license_tag,
                    item.quantity,
                    formatted_price
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let subtotal_str = format!("{:.2} {}", (order.subtotal_cents as f64) / 100.0, settings.currency_symbol).replace('.', ",");
        let shipping_str = format!("{:.2} {}", (order.shipping_cost_cents as f64) / 100.0, settings.currency_symbol).replace('.', ",");
        let total_str = format!("{:.2} {} {}", (order.total_cents as f64) / 100.0, settings.currency_symbol, settings.currency).replace('.', ",");

        let logo_html = if !settings.logo_url.is_empty() {
            format!(
                r#"<img src="{}" alt="{}" style="max-height: 85px; max-width: 220px; object-fit: contain;" />"#,
                settings.logo_url, settings.store_name
            )
        } else {
            format!(
                r#"<div style="font-size: 24px; font-weight: 900; color: #000; letter-spacing: -0.5px;">{}</div>"#,
                settings.store_name
            )
        };

        let payment_lower = order.payment_provider.to_lowercase();
        let payment_provider_display = match payment_lower.as_str() {
            "stripe" => "Card / Wallet (Stripe)",
            "paypal" => "PayPal",
            "free" => "No payment required",
            "apple_pay" => "Apple Pay",
            "google_pay" => "Google Pay",
            "amazon_pay" => "Amazon Pay",
            other => other,
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>Invoice {}</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            color: #111827;
            background: #fff;
            margin: 0;
            padding: 40px 50px;
            font-size: 13px;
            line-height: 1.5;
        }}
        .no-print {{
            text-align: right;
            margin-bottom: 25px;
        }}
        .print-btn {{
            background: #000;
            color: #fff;
            border: none;
            padding: 10px 22px;
            font-size: 13px;
            font-weight: 700;
            border-radius: 6px;
            cursor: pointer;
            box-shadow: 0 2px 4px rgba(0,0,0,0.15);
        }}
        .top-row {{
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: 45px;
        }}
        .company-info {{
            text-align: right;
            font-size: 12px;
            color: #1f2937;
            line-height: 1.45;
        }}
        .company-name {{
            font-weight: 700;
            font-size: 14px;
            color: #000;
            margin-bottom: 2px;
        }}
        .tax-notice {{
            font-size: 11px;
            color: #374151;
            margin-top: 6px;
            max-width: 320px;
            line-height: 1.35;
        }}
        .meta-row {{
            display: flex;
            justify-content: space-between;
            margin-bottom: 40px;
        }}
        .doc-title {{
            font-size: 22px;
            font-weight: 800;
            letter-spacing: -0.3px;
            color: #000;
            margin-bottom: 15px;
        }}
        .customer-block {{
            font-size: 13px;
            color: #1f2937;
            line-height: 1.45;
        }}
        .order-meta {{
            text-align: right;
            font-size: 13px;
            color: #1f2937;
            line-height: 1.6;
        }}
        .order-meta-table {{
            margin-left: auto;
            border-collapse: collapse;
        }}
        .order-meta-table td {{
            padding: 2px 0;
        }}
        .order-meta-table td.label {{
            color: #4b5563;
            padding-right: 18px;
            text-align: left;
        }}
        .order-meta-table td.val {{
            font-weight: 600;
            color: #000;
            text-align: left;
        }}
        table.items-table {{
            width: 100%;
            border-collapse: collapse;
            margin-bottom: 25px;
        }}
        table.items-table th {{
            background: #000;
            color: #fff;
            padding: 10px 10px;
            font-size: 13px;
            font-weight: 700;
            letter-spacing: 0.2px;
        }}
        table.items-table th:first-child {{
            text-align: left;
            padding-left: 12px;
        }}
        table.items-table th:nth-child(2) {{
            text-align: center;
            width: 15%;
        }}
        table.items-table th:last-child {{
            text-align: right;
            padding-right: 12px;
            width: 20%;
        }}
        .totals-section {{
            display: flex;
            justify-content: flex-end;
            margin-top: 10px;
        }}
        .totals-table {{
            width: 320px;
            border-collapse: collapse;
            font-size: 13px;
        }}
        .totals-table td {{
            padding: 6px 0;
        }}
        .totals-table td.label {{
            color: #000;
            font-weight: 700;
        }}
        .totals-table td.val {{
            text-align: right;
            color: #000;
            font-weight: 500;
        }}
        .totals-table tr.total-row {{
            border-top: 2px solid #000;
            border-bottom: 2px solid #000;
        }}
        .totals-table tr.total-row td {{
            padding: 10px 0;
            font-weight: 800;
        }}
        @page {{
            size: A4 portrait;
            margin: 15mm 15mm 20mm 15mm;
        }}
        @media print {{
            html, body {{
                width: 100% !important;
                max-width: 210mm !important;
                margin: 0 auto !important;
                padding: 0 !important;
                background: #fff !important;
                -webkit-print-color-adjust: exact;
                print-color-adjust: exact;
            }}
            .no-print {{
                display: none !important;
            }}
        }}
    </style>
</head>
<body>
    <div class="no-print">
        <button class="print-btn" onclick="window.print()">Print / Save as PDF</button>
    </div>

    <!-- Top Row: Logo (Left) & Store Info (Right) -->
    <div class="top-row">
        <div>
            {}
        </div>
        <div class="company-info">
            <div class="company-name">{}</div>
            {}
            {}
            {}
            {}
            {}
            <div class="tax-notice">{}</div>
        </div>
    </div>

    <!-- Heading & Meta Columns -->
    <div class="meta-row">
        <div>
            <div class="doc-title">INVOICE</div>
            <div class="customer-block">
                <div style="font-weight: 600; color: #000;">{}</div>
                {}
                {}
                {}
                <div style="margin-top: 4px;">{}</div>
                {}
            </div>
        </div>
        <div class="order-meta">
            <table class="order-meta-table">
                <tr>
                    <td class="label">Order Number:</td>
                    <td class="val">{}</td>
                </tr>
                <tr>
                    <td class="label">Order Date:</td>
                    <td class="val">{}</td>
                </tr>
                <tr>
                    <td class="label">Payment Method:</td>
                    <td class="val">{}</td>
                </tr>
            </table>
        </div>
    </div>

    <!-- Items Table with Solid Black Header -->
    <table class="items-table">
        <thead>
            <tr>
                <th>Product</th>
                <th>Quantity</th>
                <th>Price</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>

    <!-- Totals Summary -->
    <div class="totals-section">
        <table class="totals-table">
            <tr>
                <td class="label">{}</td>
                <td class="val">{}</td>
            </tr>
            {}
            <tr style="border-bottom: 1px solid #e2e8f0;">
                <td class="label">Shipping</td>
                <td class="val">{}</td>
            </tr>
            <tr class="total-row">
                <td class="label">Total</td>
                <td class="val" style="font-weight: 800;">{}</td>
            </tr>
        </table>
    </div>

    <div style="margin-top: 35px; font-size: 11px; color: #64748b; font-style: italic;">
        {}
    </div>
</body>
</html>"#,
            order.order_number,
            logo_html,
            if !settings.legal_name.is_empty() && settings.legal_name != settings.store_name {
                format!("{} (Trade: {})", settings.legal_name, settings.store_name)
            } else {
                settings.store_name.clone()
            },
            if !settings.store_owner.is_empty() { format!("<div>Represented by: {}</div>", settings.store_owner) } else { String::new() },
            addr_lines,
            if !settings.support_email.is_empty() { format!("<div>E-Mail: {}</div>", settings.support_email) } else { String::new() },
            if !settings.phone.is_empty() { format!("<div>Phone: {}</div>", settings.phone) } else { String::new() },
            if !settings.vat_id.is_empty() { format!("<div>VAT Number: {}</div>", settings.vat_id) } else { String::new() },
            match settings.tax_mode.as_str() {
                "kleingewerbe" => if !settings.tax_notice.is_empty() { &settings.tax_notice } else { "Gemäß § 19 UStG wird keine Umsatzsteuer berechnet." },
                "included" => "Preise inkl. gesetzl. MwSt.",
                "excluded" => "Preise zzgl. gesetzl. MwSt.",
                _ => &settings.tax_notice,
            },
            customer_name,
            if !street.is_empty() { format!("<div>{}</div>", street) } else { String::new() },
            if !postal_code.is_empty() || !city.is_empty() { format!("<div>{} {}</div>", postal_code, city) } else { String::new() },
            if !country.is_empty() { format!("<div>{}</div>", country) } else { String::new() },
            customer_email,
            if !customer_phone.is_empty() { format!("<div>{}</div>", customer_phone) } else { String::new() },
            order.order_number,
            order.created_at.format("%B %d, %Y"),
            payment_provider_display,
            items_rows,
            if settings.tax_mode == "excluded" { "Subtotal (Net)" } else { "Subtotal" },
            subtotal_str,
            match settings.tax_mode.as_str() {
                "kleingewerbe" => String::new(),
                "included" => format!(
                    "<tr><td class=\"label\">Enthaltene MwSt. ({:.1}%)</td><td class=\"val\">{:.2} {}</td></tr>",
                    settings.tax_rate_percent,
                    (order.tax_cents as f64) / 100.0,
                    settings.currency_symbol
                ),
                "excluded" => format!(
                    "<tr><td class=\"label\">zzgl. MwSt. ({:.1}%)</td><td class=\"val\">{:.2} {}</td></tr>",
                    settings.tax_rate_percent,
                    (order.tax_cents as f64) / 100.0,
                    settings.currency_symbol
                ),
                _ => String::new(),
            },
            shipping_str,
            total_str,
            match settings.tax_mode.as_str() {
                "kleingewerbe" => if !settings.tax_notice.is_empty() { &settings.tax_notice } else { "Gemäß § 19 UStG wird keine Umsatzsteuer berechnet." },
                "included" => "Der Rechnungsbetrag enthält die gesetzliche Mehrwertsteuer.",
                "excluded" => "Preise verstehen sich zzgl. der gesetzlichen Mehrwertsteuer.",
                _ => &settings.tax_notice,
            }
        )
    }

    pub fn generate_packing_slip_html(
        order: &Order,
        items: &[OrderItem],
        settings: &StoreSettings,
    ) -> String {
        let shipping_addr = &order.shipping_address;

        let customer_name = shipping_addr
            .get("full_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(&order.customer_name);

        let street = shipping_addr
            .get("street_address")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let postal_code = shipping_addr
            .get("postal_code")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let city = shipping_addr
            .get("city")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let country = shipping_addr
            .get("country_code")
            .and_then(|v| v.as_str())
            .unwrap_or("Germany");

        let addr_lines = settings
            .company_address
            .split(',')
            .map(|s| format!("<div>{}</div>", s.trim()))
            .collect::<Vec<_>>()
            .join("\n");

        let items_rows = items
            .iter()
            .map(|item| {
                let variant_display = if !item.variant_title.is_empty() && item.variant_title != "Standard" && item.variant_title != "Default" {
                    format!(r#"<div style="font-size: 11px; color: #475569; margin-top: 2px;">Variant: {}</div>"#, item.variant_title)
                } else {
                    String::new()
                };

                let license_tag = if item.is_digital {
                    r#"<div style="font-size: 11px; color: #0284c7; margin-top: 2px;">[Digital License - Do not pack physical]</div>"#
                } else {
                    ""
                };

                format!(
                    r#"<tr style="border-bottom: 1px solid #e2e8f0;">
                        <td style="padding: 14px 10px 14px 0; vertical-align: top;">
                            <div style="font-size: 13px; font-weight: 600; color: #000;">{}</div>
                            <div style="font-size: 11px; color: #475569; margin-top: 4px;"><strong>SKU:</strong> {}</div>
                            <div style="font-size: 11px; color: #475569;">Weight: 0.1kg</div>
                            {}
                            {}
                        </td>
                        <td style="padding: 14px 10px; vertical-align: top; text-align: center; font-size: 13px; font-weight: 600; color: #000;">
                            {}
                        </td>
                    </tr>"#,
                    item.product_title,
                    item.sku,
                    variant_display,
                    license_tag,
                    item.quantity
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let logo_html = if !settings.logo_url.is_empty() {
            format!(
                r#"<img src="{}" alt="{}" style="max-height: 85px; max-width: 220px; object-fit: contain;" />"#,
                settings.logo_url, settings.store_name
            )
        } else {
            format!(
                r#"<div style="font-size: 24px; font-weight: 900; color: #000; letter-spacing: -0.5px;">{}</div>"#,
                settings.store_name
            )
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>Packing Slip {}</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            color: #111827;
            background: #fff;
            margin: 0;
            padding: 40px 50px;
            font-size: 13px;
            line-height: 1.5;
        }}
        .no-print {{
            text-align: right;
            margin-bottom: 25px;
        }}
        .print-btn {{
            background: #000;
            color: #fff;
            border: none;
            padding: 10px 22px;
            font-size: 13px;
            font-weight: 700;
            border-radius: 6px;
            cursor: pointer;
            box-shadow: 0 2px 4px rgba(0,0,0,0.15);
        }}
        .top-row {{
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: 45px;
        }}
        .company-info {{
            text-align: right;
            font-size: 12px;
            color: #1f2937;
            line-height: 1.45;
        }}
        .company-name {{
            font-weight: 700;
            font-size: 14px;
            color: #000;
            margin-bottom: 2px;
        }}
        .tax-notice {{
            font-size: 11px;
            color: #374151;
            margin-top: 6px;
            max-width: 320px;
            line-height: 1.35;
        }}
        .meta-row {{
            display: flex;
            justify-content: space-between;
            margin-bottom: 40px;
        }}
        .doc-title {{
            font-size: 22px;
            font-weight: 800;
            letter-spacing: -0.3px;
            color: #000;
            margin-bottom: 15px;
        }}
        .customer-block {{
            font-size: 13px;
            color: #1f2937;
            line-height: 1.45;
        }}
        .order-meta {{
            text-align: right;
            font-size: 13px;
            color: #1f2937;
            line-height: 1.6;
        }}
        .order-meta-table {{
            margin-left: auto;
            border-collapse: collapse;
        }}
        .order-meta-table td {{
            padding: 2px 0;
        }}
        .order-meta-table td.label {{
            color: #4b5563;
            padding-right: 18px;
            text-align: left;
        }}
        .order-meta-table td.val {{
            font-weight: 600;
            color: #000;
            text-align: left;
        }}
        table.items-table {{
            width: 100%;
            border-collapse: collapse;
            margin-bottom: 25px;
        }}
        table.items-table th {{
            background: #000;
            color: #fff;
            padding: 10px 10px;
            font-size: 13px;
            font-weight: 700;
            letter-spacing: 0.2px;
        }}
        table.items-table th:first-child {{
            text-align: left;
            padding-left: 12px;
        }}
        table.items-table th:last-child {{
            text-align: center;
            width: 20%;
        }}
        @page {{
            size: A4 portrait;
            margin: 15mm 15mm 20mm 15mm;
        }}
        @media print {{
            html, body {{
                width: 100% !important;
                max-width: 210mm !important;
                margin: 0 auto !important;
                padding: 0 !important;
                background: #fff !important;
                -webkit-print-color-adjust: exact;
                print-color-adjust: exact;
            }}
            .no-print {{
                display: none !important;
            }}
        }}
    </style>
</head>
<body>
    <div class="no-print">
        <button class="print-btn" onclick="window.print()">Download / Save as PDF</button>
    </div>

    <!-- Top Row: Logo (Left) & Store Info (Right) -->
    <div class="top-row">
        <div>
            {}
        </div>
        <div class="company-info">
            <div class="company-name">{}</div>
            {}
            {}
            {}
            {}
            {}
            <div class="tax-notice">{}</div>
        </div>
    </div>

    <!-- Heading & Meta Columns -->
    <div class="meta-row">
        <div>
            <div class="doc-title">PACKING SLIP</div>
            <div class="customer-block">
                <div style="font-weight: 600; color: #000;">{}</div>
                {}
                {}
                {}
            </div>
        </div>
        <div class="order-meta">
            <table class="order-meta-table">
                <tr>
                    <td class="label">Order Number:</td>
                    <td class="val">{}</td>
                </tr>
                <tr>
                    <td class="label">Order Date:</td>
                    <td class="val">{}</td>
                </tr>
                <tr>
                    <td class="label">Shipping Method:</td>
                    <td class="val">DHL</td>
                </tr>
            </table>
        </div>
    </div>

    <!-- Items Table with Solid Black Header -->
    <table class="items-table">
        <thead>
            <tr>
                <th>Product</th>
                <th>Quantity</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>

    <div style="margin-top: 35px; font-size: 11px; color: #64748b; font-style: italic;">
        {}
    </div>
</body>
</html>"#,
            order.order_number,
            logo_html,
            if !settings.legal_name.is_empty() && settings.legal_name != settings.store_name {
                format!("{} (Trade: {})", settings.legal_name, settings.store_name)
            } else {
                settings.store_name.clone()
            },
            if !settings.store_owner.is_empty() { format!("<div>Represented by: {}</div>", settings.store_owner) } else { String::new() },
            addr_lines,
            if !settings.support_email.is_empty() { format!("<div>E-Mail: {}</div>", settings.support_email) } else { String::new() },
            if !settings.phone.is_empty() { format!("<div>Phone: {}</div>", settings.phone) } else { String::new() },
            if !settings.vat_id.is_empty() { format!("<div>VAT Number: {}</div>", settings.vat_id) } else { String::new() },
            if !settings.tax_notice.is_empty() { &settings.tax_notice } else { "Value added tax is not collected, as small businesses according to §19 (1) UStG." },
            customer_name,
            if !street.is_empty() { format!("<div>{}</div>", street) } else { String::new() },
            if !postal_code.is_empty() || !city.is_empty() { format!("<div>{} {}</div>", postal_code, city) } else { String::new() },
            if !country.is_empty() { format!("<div>{}</div>", country) } else { String::new() },
            order.order_number,
            order.created_at.format("%B %d, %Y"),
            items_rows,
            if !settings.tax_notice.is_empty() { &settings.tax_notice } else { "" }
        )
    }
}
