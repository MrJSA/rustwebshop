use crate::models::StoreSettings;
use lettre::message::header::ContentType;
use lettre::message::{Mailbox, SinglePart};
use lettre::Address;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::collections::{BTreeSet, HashSet};
use tracing::{error, info};
use uuid::Uuid;

pub async fn send_email_raw(
    settings: &StoreSettings,
    to_email: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), String> {
    let host = settings.smtp_host.trim();
    if !settings.smtp_enabled || host.is_empty() {
        info!("[SMTP disabled] Not sending \"{}\" to {}", subject, to_email);
        return Err("Email sending is switched off or no SMTP server is configured (Settings → Email).".to_string());
    }

    // Sender: explicit from-address, otherwise the SMTP login (most providers require them to match)
    let from_address = if !settings.smtp_from_email.trim().is_empty() {
        settings.smtp_from_email.trim()
    } else {
        settings.smtp_username.trim()
    };
    let from_address: Address = from_address
        .parse()
        .map_err(|_| format!("The sender address '{}' is not a valid email address.", from_address))?;
    let from_name = settings.smtp_from_name.trim();
    let from = Mailbox::new((!from_name.is_empty()).then(|| from_name.to_string()), from_address);
    let to: Mailbox = to_email
        .trim()
        .parse()
        .map_err(|_| format!("The recipient '{}' is not a valid email address.", to_email))?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .singlepart(SinglePart::builder().header(ContentType::TEXT_HTML).body(html_body.to_string()))
        .map_err(|e| format!("Failed to build email message: {}", e))?;

    let port = u16::try_from(settings.smtp_port).unwrap_or(587);
    // Well-known ports dictate the TLS mode; a mismatch would otherwise just hang or be rejected
    let mode = match port {
        465 => "tls",
        587 => "starttls",
        _ => settings.smtp_encryption.as_str(),
    };
    let mut transport_builder = match mode {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|e| format!("Invalid SMTP host '{}': {}", host, e))?,
        "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
            .map_err(|e| format!("Invalid SMTP host '{}': {}", host, e))?,
        _ => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host),
    }
    .port(port)
    .timeout(Some(std::time::Duration::from_secs(20)));

    if !settings.smtp_username.trim().is_empty() {
        let creds = Credentials::new(settings.smtp_username.trim().to_string(), settings.smtp_password.clone());
        transport_builder = transport_builder.credentials(creds);
    }

    let transport = transport_builder.build();

    transport.send(email).await.map_err(|e| {
        error!("SMTP send failure to {} via {}:{} ({}): {}", to_email, host, port, mode, e);
        format!("The mail server {}:{} ({}) rejected or did not answer: {}", host, port, mode, e)
    })?;

    info!("Email successfully dispatched to {}", to_email);
    Ok(())
}

/// Renders a responsive, professional HTML email wrapper with store logo/header, card layout, and compliant footer.
pub fn render_email_layout(
    settings: &StoreSettings,
    preheader: &str,
    heading_title: &str,
    heading_color: &str,
    content_body_html: &str,
) -> String {
    let store_name = html_escape(&settings.store_name);
    let pub_url = shop_public_url();
    let year = chrono::Utc::now().format("%Y");

    // Resolve full logo URL if provided
    let logo_url = settings.logo_url.trim();
    let logo_img_html = if !logo_url.is_empty() {
        let full_logo_url = if logo_url.starts_with("http://") || logo_url.starts_with("https://") {
            logo_url.to_string()
        } else {
            format!("{}/{}", pub_url, logo_url.trim_start_matches('/'))
        };
        format!(
            r#"<img src="{}" alt="{}" style="max-height: 44px; max-width: 180px; height: auto; border: 0; outline: none; text-decoration: none; display: block;" />"#,
            html_escape(&full_logo_url),
            store_name
        )
    } else {
        String::new()
    };

    let header_brand_html = if !logo_img_html.is_empty() {
        format!(
            r#"<table role="presentation" cellpadding="0" cellspacing="0" border="0">
              <tr>
                <td style="vertical-align: middle; padding-right: 14px;">{}</td>
                <td style="vertical-align: middle;">
                  <span style="font-size: 20px; font-weight: 800; color: #ffffff; letter-spacing: -0.3px; text-decoration: none; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">{}</span>
                </td>
              </tr>
            </table>"#,
            logo_img_html, store_name
        )
    } else {
        format!(
            r#"<table role="presentation" cellpadding="0" cellspacing="0" border="0">
              <tr>
                <td style="background: linear-gradient(135deg, #ea580c, #f97316); width: 36px; height: 36px; border-radius: 10px; text-align: center; vertical-align: middle; font-size: 18px; color: #ffffff;">🦀</td>
                <td style="padding-left: 12px; vertical-align: middle;">
                  <span style="font-size: 20px; font-weight: 800; color: #ffffff; letter-spacing: -0.3px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">{}</span>
                </td>
              </tr>
            </table>"#,
            store_name
        )
    };

    // Legal / contact details for footer
    let mut footer_details = Vec::new();
    if !settings.company_address.trim().is_empty() {
        footer_details.push(html_escape(settings.company_address.trim()));
    }
    if !settings.support_email.trim().is_empty() {
        footer_details.push(format!("Support: <a href=\"mailto:{}\" style=\"color: #94a3b8; text-decoration: underline;\">{}</a>", html_escape(settings.support_email.trim()), html_escape(settings.support_email.trim())));
    } else if !settings.smtp_from_email.trim().is_empty() {
        footer_details.push(format!("Contact: <a href=\"mailto:{}\" style=\"color: #94a3b8; text-decoration: underline;\">{}</a>", html_escape(settings.smtp_from_email.trim()), html_escape(settings.smtp_from_email.trim())));
    }
    if !settings.phone.trim().is_empty() {
        footer_details.push(format!("Tel: {}", html_escape(settings.phone.trim())));
    }
    if !settings.vat_id.trim().is_empty() {
        footer_details.push(format!("VAT ID: {}", html_escape(settings.vat_id.trim())));
    }

    let footer_details_html = if !footer_details.is_empty() {
        format!(r#"<p style="margin: 8px 0 0; color: #64748b; font-size: 11px; line-height: 1.6;">{}</p>"#, footer_details.join(" &bull; "))
    } else {
        String::new()
    };

    let tax_notice_html = if !settings.tax_notice.trim().is_empty() {
        format!(r#"<p style="margin: 8px 0 0; color: #64748b; font-size: 10px; line-height: 1.5; font-style: italic;">{}</p>"#, html_escape(settings.tax_notice.trim()))
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <title>{}</title>
</head>
<body style="margin: 0; padding: 0; background-color: #0b0f19; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; color: #f8fafc;">
  <div style="display: none; font-size: 1px; color: #0b0f19; line-height: 1px; max-height: 0px; max-width: 0px; opacity: 0; overflow: hidden;">
    {}
  </div>

  <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%" style="background-color: #0b0f19; table-layout: fixed;">
    <tr>
      <td align="center" style="padding: 40px 16px;">
        <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%" style="max-width: 580px; background-color: #151e2e; border: 1px solid #1e293b; border-radius: 18px; overflow: hidden; box-shadow: 0 20px 40px rgba(0, 0, 0, 0.45);">
          <tr>
            <td style="padding: 26px 32px; background: linear-gradient(180deg, #182338 0%, #151e2e 100%); border-bottom: 1px solid #1e293b;">
              <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%">
                <tr>
                  <td align="left">
                    <a href="{}" target="_blank" style="text-decoration: none; display: inline-block;">
                      {}
                    </a>
                  </td>
                  <td align="right" style="vertical-align: middle;">
                    <a href="{}" target="_blank" style="color: #ea580c; font-size: 12px; font-weight: 600; text-decoration: none;">Visit Store &rarr;</a>
                  </td>
                </tr>
              </table>
            </td>
          </tr>

          <tr>
            <td style="padding: 36px 32px;">
              <h2 style="margin: 0 0 16px 0; color: {}; font-size: 22px; font-weight: 700; letter-spacing: -0.4px; line-height: 1.3;">
                {}
              </h2>
              <div style="color: #cbd5e1; font-size: 14px; line-height: 1.65;">
                {}
              </div>
            </td>
          </tr>

          <tr>
            <td style="padding: 28px 32px; background-color: #0e1524; border-top: 1px solid #1e293b; text-align: center;">
              <p style="margin: 0; font-size: 12px; color: #94a3b8; font-weight: 600;">
                {}
              </p>
              {}
              {}
              <div style="margin-top: 16px; padding-top: 16px; border-top: 1px solid #182338;">
                <p style="margin: 0; font-size: 11px; color: #475569;">
                  &copy; {} {}. All rights reserved. &bull; <a href="{}" target="_blank" style="color: #ea580c; text-decoration: none;">Storefront</a>
                </p>
              </div>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#,
        heading_title,
        html_escape(preheader),
        pub_url,
        header_brand_html,
        pub_url,
        heading_color,
        heading_title,
        content_body_html,
        store_name,
        footer_details_html,
        tax_notice_html,
        year,
        store_name,
        pub_url
    )
}

pub async fn send_test_email(settings: &StoreSettings, to_email: &str) -> Result<(), String> {
    let subject = format!("Test Email from {}", settings.store_name);
    let content = format!(
        r#"<p style="margin: 0 0 20px 0;">
      This is an official test notification confirming that outgoing email delivery for <strong>{}</strong> is properly connected and functioning.
    </p>
    <div style="padding: 16px 20px; background-color: #0b0f19; border: 1px solid #1e293b; border-radius: 12px; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: 12px; color: #38bdf8; line-height: 1.8;">
      <div><span style="color: #64748b;">SMTP Host:</span> {}:{}</div>
      <div><span style="color: #64748b;">Encryption:</span> {}</div>
      <div><span style="color: #64748b;">Sender From:</span> {} &lt;{}&gt;</div>
    </div>"#,
        html_escape(&settings.store_name),
        html_escape(&settings.smtp_host),
        settings.smtp_port,
        html_escape(&settings.smtp_encryption),
        html_escape(&settings.smtp_from_name),
        html_escape(&settings.smtp_from_email)
    );
    let html = render_email_layout(
        settings,
        "SMTP test notification confirming your setup is functioning.",
        "SMTP Test Successful!",
        "#f97316",
        &content,
    );

    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_verification_email(
    settings: &StoreSettings,
    to_email: &str,
    customer_name: &str,
    token: &str,
    origin: &str,
) -> Result<(), String> {
    let verify_url = format!("{}/account/verify?token={}", origin.trim_end_matches('/'), token);
    let subject = format!("Verify your account - {}", settings.store_name);
    let content = format!(
        r#"<p style="margin: 0 0 20px 0;">
      Thank you for registering with <strong>{}</strong>. Please click the button below to verify your email address and activate your customer account.
    </p>
    <div style="margin: 32px 0; text-align: center;">
      <a href="{}" target="_blank" style="background: linear-gradient(135deg, #ea580c, #f97316); color: #ffffff; text-decoration: none; padding: 14px 32px; border-radius: 10px; font-weight: 700; font-size: 14px; display: inline-block; box-shadow: 0 4px 12px rgba(234, 88, 12, 0.35);">
        Verify Email Address
      </a>
    </div>
    <div style="padding: 14px 16px; background-color: #0b0f19; border: 1px solid #1e293b; border-radius: 10px; font-size: 12px; color: #64748b; line-height: 1.5;">
      If the button above does not work, copy and paste this link into your browser:<br/>
      <a href="{}" style="color: #38bdf8; word-break: break-all; text-decoration: underline;">{}</a>
    </div>"#,
        html_escape(&settings.store_name),
        verify_url,
        verify_url,
        verify_url
    );
    let heading = format!("Welcome to {}, {}!", html_escape(&settings.store_name), html_escape(customer_name));
    let html = render_email_layout(
        settings,
        "Please verify your email address to activate your customer account.",
        &heading,
        "#f97316",
        &content,
    );

    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_order_created_email(
    settings: &StoreSettings,
    to_email: &str,
    order_number: &str,
    total_cents: i32,
    currency: &str,
) -> Result<(), String> {
    let formatted_price = format!("{:.2} {}", total_cents as f64 / 100.0, currency);
    let subject = format!("Order Confirmation: #{}", order_number);
    let content = format!(
        r#"<p style="margin: 0 0 20px 0;">
      We have received your order <strong>#{}</strong> and our logistics team is preparing it for processing.
    </p>
    <div style="padding: 20px; background-color: #0b0f19; border: 1px solid #1e293b; border-radius: 12px; margin: 24px 0;">
      <table role="presentation" cellpadding="0" cellspacing="0" border="0" width="100%" style="font-size: 13px;">
        <tr>
          <td style="color: #94a3b8; padding-bottom: 8px;">Order Number:</td>
          <td align="right" style="color: #ffffff; font-weight: 700; font-family: ui-monospace, monospace; padding-bottom: 8px;">#{}</td>
        </tr>
        <tr>
          <td style="color: #94a3b8; padding-top: 8px; border-top: 1px solid #1e293b;">Total Amount:</td>
          <td align="right" style="color: #10b981; font-weight: 800; font-size: 16px; padding-top: 8px; border-top: 1px solid #1e293b;">{}</td>
        </tr>
      </table>
    </div>
    <p style="margin: 0; color: #94a3b8; font-size: 13px;">
      You will receive another notification as soon as your payment is confirmed and when your parcel is dispatched with tracking information.
    </p>"#,
        order_number, order_number, formatted_price
    );
    let heading = format!("Thank you for your order! (#{})", order_number);
    let html = render_email_layout(
        settings,
        &format!("Your order #{} has been received.", order_number),
        &heading,
        "#f97316",
        &content,
    );

    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_payment_received_email(
    settings: &StoreSettings,
    to_email: &str,
    order_number: &str,
    total_cents: i32,
    currency: &str,
) -> Result<(), String> {
    let formatted_price = format!("{:.2} {}", total_cents as f64 / 100.0, currency);
    let subject = format!("Payment Received for Order #{}", order_number);
    let content = format!(
        r#"<p style="margin: 0 0 20px 0;">
      Your payment of <strong style="color: #10b981;">{}</strong> for Order <strong>#{}</strong> has been successfully confirmed.
    </p>
    <div style="padding: 18px 20px; background-color: #0b0f19; border: 1px solid #1e293b; border-radius: 12px; margin: 24px 0;">
      <div style="color: #cbd5e1; font-size: 13px;">
        <span style="color: #10b981; font-weight: bold; margin-right: 6px;">✓ Payment Verified:</span>
        Our fulfillment warehouse is now assembling and packaging your items.
      </div>
    </div>
    <p style="margin: 0; color: #94a3b8; font-size: 13px;">
      You will receive a shipping confirmation email with tracking details as soon as the package is handed over to the courier.
    </p>"#,
        formatted_price, order_number
    );
    let heading = format!("Payment Confirmed for Order #{}", order_number);
    let html = render_email_layout(
        settings,
        &format!("Payment confirmed for Order #{}.", order_number),
        &heading,
        "#10b981",
        &content,
    );

    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_order_shipped_email(
    settings: &StoreSettings,
    to_email: &str,
    order_number: &str,
    tracking_number: &str,
) -> Result<(), String> {
    let tracking_url = if tracking_number.starts_with("DHL") || tracking_number.len() >= 10 {
        format!("https://www.dhl.de/de/privatkunden/pakete-empfangen/verfolgen.html?piececode={}", tracking_number)
    } else {
        format!("https://parcelsapp.com/en/tracking/{}", tracking_number)
    };

    let subject = format!("Your Order #{} has shipped!", order_number);
    let content = format!(
        r#"<p style="margin: 0 0 20px 0;">
      Great news! Order <strong>#{}</strong> has been dispatched from our warehouse and is on its way to you.
    </p>
    <div style="padding: 24px 20px; background-color: #0b0f19; border: 1px solid #1e293b; border-radius: 12px; text-align: center; margin: 28px 0;">
      <span style="color: #94a3b8; font-size: 11px; text-transform: uppercase; letter-spacing: 1px; font-weight: 600; display: block; margin-bottom: 6px;">Tracking Number</span>
      <span style="font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: 19px; font-weight: 800; color: #f8fafc; letter-spacing: 0.5px;">{}</span>
      <div style="margin-top: 20px;">
        <a href="{}" target="_blank" style="background: linear-gradient(135deg, #0284c7, #38bdf8); color: #ffffff; text-decoration: none; padding: 12px 28px; border-radius: 10px; font-weight: 700; font-size: 13px; display: inline-block; box-shadow: 0 4px 12px rgba(2, 132, 199, 0.35);">
          Track Shipment &rarr;
        </a>
      </div>
    </div>"#,
        order_number, tracking_number, tracking_url
    );
    let heading = format!("Your package is on its way! (#{})", order_number);
    let html = render_email_layout(
        settings,
        &format!("Order #{} has been dispatched.", order_number),
        &heading,
        "#38bdf8",
        &content,
    );

    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_back_in_stock_email(
    settings: &StoreSettings,
    to_email: &str,
    product_title: &str,
    product_url: &str,
) -> Result<(), String> {
    let subject = format!("Back in stock: {}", product_title);
    let content = format!(
        r#"<p style="margin: 0 0 20px 0;">
      You requested to be notified when <strong>{}</strong> returned to stock at <strong>{}</strong>. It is now back in stock and available to order!
    </p>
    <div style="margin: 32px 0; text-align: center;">
      <a href="{}" target="_blank" style="background: linear-gradient(135deg, #ea580c, #f97316); color: #ffffff; text-decoration: none; padding: 14px 32px; border-radius: 10px; font-weight: 700; font-size: 14px; display: inline-block; box-shadow: 0 4px 12px rgba(234, 88, 12, 0.35);">
        View Product Now &rarr;
      </a>
    </div>
    <p style="margin: 0; color: #64748b; font-size: 12px; text-align: center;">
      Stock levels may be limited. Secure yours before it sells out again!
    </p>"#,
        html_escape(product_title),
        html_escape(&settings.store_name),
        product_url
    );
    let heading = "Good news! It's back in stock!".to_string();
    let html = render_email_layout(
        settings,
        &format!("{} is back in stock at {}.", product_title, settings.store_name),
        &heading,
        "#10b981",
        &content,
    );

    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_password_reset_email(
    settings: &StoreSettings,
    to_email: &str,
    token: &str,
) -> Result<(), String> {
    let reset_url = format!("{}/account/reset-password?token={}", shop_public_url(), token);
    let subject = format!("Reset your password - {}", settings.store_name);
    let content = format!(
        r#"<p style="margin: 0 0 20px 0;">
      A password reset request was made for your customer account at <strong>{}</strong>. The security link below is valid for <strong>1 hour</strong>.
    </p>
    <div style="margin: 32px 0; text-align: center;">
      <a href="{}" target="_blank" style="background: linear-gradient(135deg, #ea580c, #f97316); color: #ffffff; text-decoration: none; padding: 14px 32px; border-radius: 10px; font-weight: 700; font-size: 14px; display: inline-block; box-shadow: 0 4px 12px rgba(234, 88, 12, 0.35);">
        Choose a New Password
      </a>
    </div>
    <div style="padding: 14px 16px; background-color: #0b0f19; border: 1px solid #1e293b; border-radius: 10px; font-size: 12px; color: #64748b; line-height: 1.5; margin-bottom: 16px;">
      If the button above does not work, copy and paste this link into your browser:<br/>
      <a href="{}" style="color: #38bdf8; word-break: break-all; text-decoration: underline;">{}</a>
    </div>
    <p style="margin: 0; color: #64748b; font-size: 12px;">
      If you did not request a password reset, you can safely ignore this email — your password remains unchanged.
    </p>"#,
        html_escape(&settings.store_name),
        reset_url,
        reset_url,
        reset_url
    );
    let heading = format!("Password Reset for {}", html_escape(&settings.store_name));
    let html = render_email_layout(
        settings,
        "Password reset link for your account.",
        &heading,
        "#f97316",
        &content,
    );

    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_part_low_stock_email(
    settings: &StoreSettings,
    to_email: &str,
    part_name: &str,
    part_sku: &str,
    stock_quantity: i32,
    low_stock_threshold: i32,
    storage_location: Option<&str>,
) -> Result<(), String> {
    let loc_str = storage_location.filter(|s| !s.trim().is_empty()).unwrap_or("Not specified");
    let subject = format!("⚠️ Low Stock Warning: {} ({}) - {}", part_name, part_sku, settings.store_name);
    let admin_url = format!("{}/admin/products", shop_public_url());

    let content = format!(
        r#"<p style="margin: 0 0 16px 0; font-size: 15px; color: #e2e8f0;">
      The inventory level for part <strong>{}</strong> has fallen to or below its configured low-stock threshold.
    </p>
    <div style="background-color: #0b0f19; border: 1px solid #334155; border-radius: 12px; padding: 20px; margin: 20px 0;">
      <table style="width: 100%; border-collapse: collapse; font-size: 14px;">
        <tr>
          <td style="padding: 8px 0; color: #94a3b8; width: 40%;">Part Name:</td>
          <td style="padding: 8px 0; color: #f8fafc; font-weight: 600;">{}</td>
        </tr>
        <tr>
          <td style="padding: 8px 0; color: #94a3b8;">SKU:</td>
          <td style="padding: 8px 0; color: #f8fafc; font-weight: 600; font-family: monospace;">{}</td>
        </tr>
        <tr>
          <td style="padding: 8px 0; color: #94a3b8;">Current In Stock:</td>
          <td style="padding: 8px 0; color: #ef4444; font-weight: 700; font-size: 16px;">{} units</td>
        </tr>
        <tr>
          <td style="padding: 8px 0; color: #94a3b8;">Threshold:</td>
          <td style="padding: 8px 0; color: #cbd5e1;">{} units</td>
        </tr>
        <tr>
          <td style="padding: 8px 0; color: #94a3b8;">Storage Location:</td>
          <td style="padding: 8px 0; color: #cbd5e1;">{}</td>
        </tr>
      </table>
    </div>
    <div style="margin: 28px 0 16px 0; text-align: center;">
      <a href="{}" target="_blank" style="background: linear-gradient(135deg, #d97706, #f59e0b); color: #ffffff; text-decoration: none; padding: 12px 28px; border-radius: 8px; font-weight: 700; font-size: 14px; display: inline-block;">
        Open Stock & Inventory
      </a>
    </div>
    <p style="margin: 0; color: #64748b; font-size: 12px; text-align: center;">
      You received this notification based on your store's inventory alert settings.
    </p>"#,
        html_escape(part_name),
        html_escape(part_name),
        html_escape(part_sku),
        stock_quantity,
        low_stock_threshold,
        html_escape(loc_str),
        admin_url
    );

    let heading = format!("Low Stock Alert: {}", html_escape(part_name));
    let html = render_email_layout(
        settings,
        &format!("Warning: {} ({}) is running low ({} units left).", part_name, part_sku, stock_quantity),
        &heading,
        "#f59e0b",
        &content,
    );

    send_email_raw(settings, to_email, &subject, &html).await
}

/// Dispatches a test email according to the specified template type (or default test email) with sample data.
pub async fn send_test_email_by_type(
    settings: &StoreSettings,
    to_email: &str,
    email_type: Option<&str>,
) -> Result<(), String> {
    match email_type.unwrap_or("test") {
        "verification" => {
            send_verification_email(
                settings,
                to_email,
                "Alex Hunter",
                "sample-verification-token-12345",
                &shop_public_url(),
            )
            .await
        }
        "order_created" => {
            send_order_created_email(
                settings,
                to_email,
                "10042",
                4999,
                &settings.currency,
            )
            .await
        }
        "payment_received" => {
            send_payment_received_email(
                settings,
                to_email,
                "10042",
                4999,
                &settings.currency,
            )
            .await
        }
        "order_shipped" => {
            send_order_shipped_email(
                settings,
                to_email,
                "10042",
                "DHL9876543210DE",
            )
            .await
        }
        "back_in_stock" => {
            let sample_url = format!("{}/products", shop_public_url());
            send_back_in_stock_email(
                settings,
                to_email,
                "Pro Mechanical Keyboard RGB",
                &sample_url,
            )
            .await
        }
        "password_reset" => {
            send_password_reset_email(
                settings,
                to_email,
                "sample-password-reset-token-67890",
            )
            .await
        }
        "low_stock_alert" => {
            send_part_low_stock_email(
                settings,
                to_email,
                "Cherry MX Red Switches",
                "PRT-SW-CHERRY-RED",
                3,
                10,
                Some("Warehouse A - Shelf 4B"),
            )
            .await
        }
        "test" | _ => {
            send_test_email(settings, to_email).await
        }
    }
}

pub async fn send_order_created_notification(pool: &sqlx::PgPool, order_number: &str) {
    let settings = match sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await {
        Ok(s) => s,
        Err(e) => {
            error!("Cannot load store settings for order email {}: {}", order_number, e);
            return;
        }
    };

    if let Ok(order_row) = sqlx::query("SELECT customer_email, total_cents FROM orders WHERE order_number = $1")
        .bind(order_number)
        .fetch_one(pool)
        .await
    {
        use sqlx::Row;
        let email: String = order_row.get("customer_email");
        let total: i32 = order_row.get("total_cents");
        let _ = send_order_created_email(&settings, &email, order_number, total, &settings.currency).await;
    }
}

pub async fn send_payment_received_notification(pool: &sqlx::PgPool, order_number: &str) {
    let settings = match sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await {
        Ok(s) => s,
        Err(e) => {
            error!("Cannot load store settings for order email {}: {}", order_number, e);
            return;
        }
    };

    if let Ok(order_row) = sqlx::query("SELECT customer_email, total_cents FROM orders WHERE order_number = $1")
        .bind(order_number)
        .fetch_one(pool)
        .await
    {
        use sqlx::Row;
        let email: String = order_row.get("customer_email");
        let total: i32 = order_row.get("total_cents");
        let _ = send_payment_received_email(&settings, &email, order_number, total, &settings.currency).await;
    }
}

pub async fn check_and_send_low_stock_alerts(pool: &sqlx::PgPool) {
    let settings = match sqlx::query_as::<_, StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await {
        Ok(s) => s,
        Err(e) => {
            error!("Cannot load store settings for low stock alerts: {}", e);
            return;
        }
    };

    if !settings.low_stock_alerts_enabled || !settings.smtp_enabled || settings.smtp_host.trim().is_empty() {
        return;
    }

    // Determine target recipient emails
    let admin_users: Vec<(Uuid, String, Option<String>, String, serde_json::Value)> = sqlx::query_as(
        "SELECT id, username, email, role, permissions FROM admin_users"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mode = settings.low_stock_alert_recipients_mode.as_str();
    let selected_ids: HashSet<String> = settings
        .low_stock_alert_selected_user_ids
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let mut recipient_emails = BTreeSet::new();

    for (id, _username, email_opt, role, perms) in &admin_users {
        let email = match email_opt {
            Some(e) if !e.trim().is_empty() => e.trim().to_string(),
            _ => continue,
        };

        let has_stock_access = role == "superadmin" || {
            let eff = crate::middleware::effective_permissions(role, perms);
            eff.get("products").copied().unwrap_or(false)
        };

        let is_selected = selected_ids.contains(&id.to_string());

        let include = match mode {
            "stock_managers" => has_stock_access,
            "selected_users" => is_selected,
            "both" | "all" => has_stock_access || is_selected,
            _ => has_stock_access,
        };

        if include {
            recipient_emails.insert(email);
        }
    }

    // Parse custom emails
    for part in settings.low_stock_alert_custom_emails.split(&[',', ';', '\n', '\r'][..]) {
        let email = part.trim();
        if !email.is_empty() && email.contains('@') {
            recipient_emails.insert(email.to_string());
        }
    }

    // Fallback to support_email if no recipients found
    if recipient_emails.is_empty() {
        let sup = settings.support_email.trim();
        if !sup.is_empty() && sup.contains('@') {
            recipient_emails.insert(sup.to_string());
        }
    }

    if recipient_emails.is_empty() {
        info!("Low stock alert triggered but no valid recipient emails configured");
        return;
    }

    // Find parts that are at or below threshold and need an alert
    let low_parts: Vec<(Uuid, String, String, i32, i32, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, name, sku, stock_quantity, low_stock_threshold, storage_location
        FROM bom_parts
        WHERE low_stock_threshold > 0
          AND stock_quantity <= low_stock_threshold
          AND (
              last_low_stock_alert_at IS NULL
              OR last_alert_stock_quantity IS NULL
              OR stock_quantity < last_alert_stock_quantity
              OR last_low_stock_alert_at < NOW() - INTERVAL '24 hours'
          )
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (part_id, name, sku, stock_qty, threshold, loc) in low_parts {
        let mut any_sent = false;
        for email in &recipient_emails {
            match send_part_low_stock_email(&settings, email, &name, &sku, stock_qty, threshold, loc.as_deref()).await {
                Ok(_) => {
                    any_sent = true;
                }
                Err(e) => {
                    error!("Failed to dispatch low stock email for {} to {}: {}", sku, email, e);
                }
            }
        }

        if any_sent {
            let _ = sqlx::query(
                "UPDATE bom_parts SET last_low_stock_alert_at = NOW(), last_alert_stock_quantity = $1 WHERE id = $2"
            )
            .bind(stock_qty)
            .bind(part_id)
            .execute(pool)
            .await;
        }
    }
}

/// Escapes text inserted into HTML emails (customer-supplied names must never become markup).
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Public base URL of the storefront used in links inside emails (e.g. https://shop.example).
pub fn shop_public_url() -> String {
    std::env::var("SHOP_PUBLIC_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
        .trim_end_matches('/')
        .to_string()
}
