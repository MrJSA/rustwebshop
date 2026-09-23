use crate::models::{Order, OrderItem, StoreSettings};

pub struct DocumentGenerator;

impl DocumentGenerator {
    pub fn generate_invoice_html(
        order: &Order,
        items: &[OrderItem],
        settings: &StoreSettings,
    ) -> String {
        let shipping_addr = &order.shipping_address;
        let billing_addr = &order.billing_address;

        let items_rows = items
            .iter()
            .map(|item| {
                format!(
                    r#"<tr>
                        <td style="padding: 12px; border-bottom: 1px solid #e2e8f0;">
                            <strong>{}</strong><br/>
                            <span style="color: #64748b; font-size: 13px;">{} | SKU: <code>{}</code></span>
                            {}
                        </td>
                        <td style="padding: 12px; border-bottom: 1px solid #e2e8f0; text-align: center;">{}</td>
                        <td style="padding: 12px; border-bottom: 1px solid #e2e8f0; text-align: right;">{:.2} {}</td>
                        <td style="padding: 12px; border-bottom: 1px solid #e2e8f0; text-align: right; font-weight: bold;">{:.2} {}</td>
                    </tr>"#,
                    item.product_title,
                    item.variant_title,
                    item.sku,
                    if item.is_digital { "<br/><span style='color: #0284c7; font-size: 11px;'>[Digital License]</span>" } else { "" },
                    item.quantity,
                    (item.unit_price_cents as f64) / 100.0,
                    settings.currency_symbol,
                    (item.total_price_cents as f64) / 100.0,
                    settings.currency_symbol,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Tax Invoice - {}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color: #1e293b; margin: 0; padding: 40px; background: #fff; }}
        .header {{ display: flex; justify-content: space-between; border-bottom: 2px solid #0f172a; padding-bottom: 24px; margin-bottom: 30px; }}
        .company-name {{ font-size: 26px; font-weight: 800; color: #0f172a; letter-spacing: -0.5px; }}
        .badge {{ display: inline-block; padding: 4px 10px; border-radius: 4px; font-size: 12px; font-weight: 700; text-transform: uppercase; background: #dcfce7; color: #15803d; }}
        .meta-grid {{ display: grid; grid-template-columns: 1fr 1fr; gap: 30px; margin-bottom: 30px; }}
        .addr-card {{ background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 16px; font-size: 14px; line-height: 1.6; }}
        table {{ width: 100%; border-collapse: collapse; margin-bottom: 30px; font-size: 14px; }}
        th {{ background: #f1f5f9; padding: 12px; text-align: left; font-weight: 700; color: #475569; border-bottom: 2px solid #cbd5e1; }}
        .totals-table {{ width: 340px; margin-left: auto; margin-bottom: 40px; font-size: 14px; }}
        .totals-table td {{ padding: 8px 12px; }}
        .grand-total {{ font-size: 18px; font-weight: 800; border-top: 2px solid #0f172a; color: #0f172a; }}
        .footer {{ font-size: 12px; color: #94a3b8; border-top: 1px solid #e2e8f0; padding-top: 20px; text-align: center; }}
        @media print {{
            body {{ padding: 0; }}
            .no-print {{ display: none; }}
        }}
    </style>
</head>
<body>
    <div class="no-print" style="margin-bottom: 20px; text-align: right;">
        <button onclick="window.print()" style="background: #0f172a; color: #fff; border: none; padding: 10px 18px; border-radius: 6px; cursor: pointer; font-weight: 600;">Print / Save as PDF</button>
    </div>

    <div class="header">
        <div>
            <div class="company-name">{}</div>
            <div style="color: #64748b; font-size: 13px; margin-top: 4px;">{}</div>
            <div style="color: #64748b; font-size: 13px;">VAT ID: {} | Email: {}</div>
        </div>
        <div style="text-align: right;">
            <h1 style="margin: 0; font-size: 24px; color: #0f172a;">TAX INVOICE</h1>
            <div style="font-size: 14px; font-weight: 600; margin-top: 4px;">Order: {}</div>
            <div style="font-size: 13px; color: #64748b;">Date: {}</div>
            <div style="margin-top: 8px;"><span class="badge">PAID</span></div>
        </div>
    </div>

    <div class="meta-grid">
        <div class="addr-card">
            <strong style="color: #475569; font-size: 12px; text-transform: uppercase;">Billed To:</strong><br/>
            <strong>{}</strong><br/>
            {}<br/>
            {}, {} {}<br/>
            Country: {}<br/>
            Email: {}
        </div>
        <div class="addr-card">
            <strong style="color: #475569; font-size: 12px; text-transform: uppercase;">Ship To:</strong><br/>
            <strong>{}</strong><br/>
            {}<br/>
            {}, {} {}<br/>
            Country: {}<br/>
            Payment Method: <span style="text-transform: capitalize;">{}</span>
        </div>
    </div>

    <table>
        <thead>
            <tr>
                <th style="width: 55%;">Item & SKU</th>
                <th style="width: 15%; text-align: center;">Qty</th>
                <th style="width: 15%; text-align: right;">Unit Price</th>
                <th style="width: 15%; text-align: right;">Total</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>

    <table class="totals-table">
        <tr>
            <td style="color: #64748b;">Subtotal:</td>
            <td style="text-align: right; font-weight: 600;">{:.2} {}</td>
        </tr>
        <tr>
            <td style="color: #64748b;">Estimated Shipping:</td>
            <td style="text-align: right; font-weight: 600;">{:.2} {}</td>
        </tr>
        <tr>
            <td style="color: #64748b;">VAT / Sales Tax ({:.1}%):</td>
            <td style="text-align: right; font-weight: 600;">{:.2} {}</td>
        </tr>
        <tr class="grand-total">
            <td>Total Paid:</td>
            <td style="text-align: right;">{:.2} {}</td>
        </tr>
    </table>

    <div class="footer">
        Thank you for purchasing from {}! For warranty or inquiries, reach out to {}.
    </div>
</body>
</html>"#,
            order.order_number,
            settings.store_name,
            settings.company_address,
            settings.vat_id,
            settings.support_email,
            order.order_number,
            order.created_at.format("%B %d, %Y - %H:%M UTC"),
            billing_addr.get("full_name").and_then(|v| v.as_str()).unwrap_or(&order.customer_name),
            billing_addr.get("street_address").and_then(|v| v.as_str()).unwrap_or(""),
            billing_addr.get("city").and_then(|v| v.as_str()).unwrap_or(""),
            billing_addr.get("state_province").and_then(|v| v.as_str()).unwrap_or(""),
            billing_addr.get("postal_code").and_then(|v| v.as_str()).unwrap_or(""),
            billing_addr.get("country_code").and_then(|v| v.as_str()).unwrap_or(""),
            order.customer_email,
            shipping_addr.get("full_name").and_then(|v| v.as_str()).unwrap_or(&order.customer_name),
            shipping_addr.get("street_address").and_then(|v| v.as_str()).unwrap_or(""),
            shipping_addr.get("city").and_then(|v| v.as_str()).unwrap_or(""),
            shipping_addr.get("state_province").and_then(|v| v.as_str()).unwrap_or(""),
            shipping_addr.get("postal_code").and_then(|v| v.as_str()).unwrap_or(""),
            shipping_addr.get("country_code").and_then(|v| v.as_str()).unwrap_or(""),
            order.payment_provider,
            items_rows,
            (order.subtotal_cents as f64) / 100.0,
            settings.currency_symbol,
            (order.shipping_cost_cents as f64) / 100.0,
            settings.currency_symbol,
            settings.tax_rate_percent,
            (order.tax_cents as f64) / 100.0,
            settings.currency_symbol,
            (order.total_cents as f64) / 100.0,
            settings.currency_symbol,
            settings.store_name,
            settings.support_email
        )
    }

    pub fn generate_packing_slip_html(
        order: &Order,
        items: &[OrderItem],
        settings: &StoreSettings,
    ) -> String {
        let shipping_addr = &order.shipping_address;

        let items_rows = items
            .iter()
            .map(|item| {
                format!(
                    r#"<tr>
                        <td style="padding: 14px; border-bottom: 1px solid #cbd5e1; text-align: center; width: 40px;">
                            <div style="width: 20px; height: 20px; border: 2px solid #64748b; border-radius: 4px; margin: auto;"></div>
                        </td>
                        <td style="padding: 14px; border-bottom: 1px solid #cbd5e1;">
                            <strong style="font-size: 15px;">{}</strong><br/>
                            <span style="color: #475569; font-size: 13px;">Variant: {}</span><br/>
                            <span style="font-family: monospace; font-size: 12px; background: #e2e8f0; padding: 2px 6px; border-radius: 4px;">SKU: {}</span>
                            {}
                        </td>
                        <td style="padding: 14px; border-bottom: 1px solid #cbd5e1; text-align: center; font-size: 18px; font-weight: 800; color: #0f172a;">
                            {}
                        </td>
                    </tr>"#,
                    item.product_title,
                    item.variant_title,
                    item.sku,
                    if item.is_digital { "<br/><span style='color: #0284c7; font-size: 12px;'>[Digital License - Do not pack physical]</span>" } else { "" },
                    item.quantity
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Packing Slip - {}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color: #0f172a; margin: 0; padding: 40px; background: #fff; }}
        .header {{ display: flex; justify-content: space-between; border-bottom: 3px solid #0f172a; padding-bottom: 20px; margin-bottom: 25px; }}
        .title {{ font-size: 28px; font-weight: 900; letter-spacing: -0.5px; text-transform: uppercase; }}
        .meta-box {{ background: #f8fafc; border: 2px dashed #94a3b8; border-radius: 8px; padding: 18px; margin-bottom: 30px; font-size: 14px; line-height: 1.6; }}
        table {{ width: 100%; border-collapse: collapse; margin-bottom: 30px; }}
        th {{ background: #0f172a; color: #fff; padding: 12px; text-align: left; font-size: 13px; text-transform: uppercase; }}
        .signoff {{ display: grid; grid-template-columns: 1fr 1fr; gap: 40px; margin-top: 50px; font-size: 13px; }}
        .sign-line {{ border-bottom: 1px solid #64748b; height: 35px; margin-top: 10px; }}
        @media print {{
            body {{ padding: 0; }}
            .no-print {{ display: none; }}
        }}
    </style>
</head>
<body>
    <div class="no-print" style="margin-bottom: 20px; text-align: right;">
        <button onclick="window.print()" style="background: #0f172a; color: #fff; border: none; padding: 10px 18px; border-radius: 6px; cursor: pointer; font-weight: 600;">Print Packing Slip</button>
    </div>

    <div class="header">
        <div>
            <div class="title">PACKING SLIP</div>
            <div style="font-size: 14px; font-weight: 600; color: #475569; margin-top: 4px;">{}</div>
        </div>
        <div style="text-align: right;">
            <div style="font-size: 18px; font-weight: 800;">Order: {}</div>
            <div style="font-size: 13px; color: #64748b;">Order Date: {}</div>
            <div style="font-size: 13px; color: #64748b;">Status: <span style="text-transform: uppercase; font-weight: 700;">{}</span></div>
        </div>
    </div>

    <div class="meta-box">
        <strong style="font-size: 15px; color: #0f172a;">SHIP TO DESTINATION:</strong><br/>
        <strong style="font-size: 16px;">{}</strong><br/>
        {}<br/>
        {}, {} {}<br/>
        <strong>Country: {}</strong><br/>
        Customer Contact: {}
    </div>

    <table>
        <thead>
            <tr>
                <th style="text-align: center; width: 50px;">Check</th>
                <th>Item Description, Variant & SKU</th>
                <th style="text-align: center; width: 100px;">Qty to Pack</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>

    <div class="signoff">
        <div>
            Packed By (Name):
            <div class="sign-line"></div>
        </div>
        <div>
            Fulfillment Verification Signature:
            <div class="sign-line"></div>
        </div>
    </div>
</body>
</html>"#,
            order.order_number,
            settings.store_name,
            order.order_number,
            order.created_at.format("%Y-%m-%d %H:%M"),
            order.order_status,
            shipping_addr.get("full_name").and_then(|v| v.as_str()).unwrap_or(&order.customer_name),
            shipping_addr.get("street_address").and_then(|v| v.as_str()).unwrap_or(""),
            shipping_addr.get("city").and_then(|v| v.as_str()).unwrap_or(""),
            shipping_addr.get("state_province").and_then(|v| v.as_str()).unwrap_or(""),
            shipping_addr.get("postal_code").and_then(|v| v.as_str()).unwrap_or(""),
            shipping_addr.get("country_code").and_then(|v| v.as_str()).unwrap_or(""),
            order.customer_email,
            items_rows
        )
    }
}
