use crate::models::StoreSettings;
use lettre::message::header::ContentType;
use lettre::message::SinglePart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tracing::{error, info};

pub async fn send_email_raw(
    settings: &StoreSettings,
    to_email: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), String> {
    if !settings.smtp_enabled || settings.smtp_host.trim().is_empty() {
        info!(
            "[SMTP DISABLED / DEV SIMULATION] To: {} | Subject: {}\nBody snippet: {}",
            to_email,
            subject,
            &html_body[..std::cmp::min(160, html_body.len())]
        );
        return Ok(());
    }

    let from_header = format!("{} <{}>", settings.smtp_from_name, settings.smtp_from_email);
    let to_parsed = to_email.parse().map_err(|e| format!("Invalid to_email: {}", e))?;
    let from_parsed = from_header.parse().map_err(|e| format!("Invalid from_email: {}", e))?;

    let email = Message::builder()
        .from(from_parsed)
        .to(to_parsed)
        .subject(subject)
        .singlepart(SinglePart::builder().header(ContentType::TEXT_HTML).body(html_body.to_string()))
        .map_err(|e| format!("Failed to build email message: {}", e))?;

    let port = settings.smtp_port as u16;
    let mut transport_builder = if settings.smtp_encryption == "tls" {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&settings.smtp_host)
            .map_err(|e| format!("Failed to configure SMTP relay: {}", e))?
            .port(port)
    } else if settings.smtp_encryption == "starttls" {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&settings.smtp_host)
            .map_err(|e| format!("Failed to configure SMTP STARTTLS relay: {}", e))?
            .port(port)
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&settings.smtp_host).port(port)
    };

    if !settings.smtp_username.is_empty() {
        let creds = Credentials::new(settings.smtp_username.clone(), settings.smtp_password.clone());
        transport_builder = transport_builder.credentials(creds);
    }

    let transport = transport_builder.build();

    transport.send(email).await.map_err(|e| {
        error!("SMTP send failure to {}: {}", to_email, e);
        format!("SMTP send failed: {}", e)
    })?;

    info!("Email successfully dispatched to {}", to_email);
    Ok(())
}

pub async fn send_test_email(settings: &StoreSettings, to_email: &str) -> Result<(), String> {
    let subject = format!("Test Email from {}", settings.store_name);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #0f172a; color: #f8fafc; padding: 40px 20px;">
  <div style="max-width: 560px; margin: 0 auto; background: #1e293b; border-radius: 16px; border: 1px solid #334155; padding: 32px; box-shadow: 0 10px 25px rgba(0,0,0,0.5);">
    <h2 style="color: #f97316; margin-top: 0;">SMTP Test Successful!</h2>
    <p style="color: #94a3b8; font-size: 14px; line-height: 1.6;">
      This is a test notification confirming that the email server configuration for <strong>{}</strong> is properly connected and functioning.
    </p>
    <div style="margin-top: 24px; padding: 16px; background: #0f172a; border-radius: 12px; font-size: 12px; font-family: monospace; color: #38bdf8;">
      Host: {}:{}<br/>
      Encryption: {}<br/>
      From: {} &lt;{}&gt;
    </div>
  </div>
</body>
</html>"#,
        settings.store_name,
        settings.smtp_host,
        settings.smtp_port,
        settings.smtp_encryption,
        settings.smtp_from_name,
        settings.smtp_from_email
    );

    send_email_raw(settings, to_email, &subject, &html).await
}

pub async fn send_verification_email(
    settings: &StoreSettings,
    to_email: &str,
    customer_name: &str,
    token: &str,
    origin: &str,
) {
    let verify_url = format!("{}/account/verify?token={}", origin.trim_end_matches('/'), token);
    let subject = format!("Verify your account - {}", settings.store_name);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #0f172a; color: #f8fafc; padding: 40px 20px;">
  <div style="max-width: 560px; margin: 0 auto; background: #1e293b; border-radius: 16px; border: 1px solid #334155; padding: 32px;">
    <h2 style="color: #f97316; margin-top: 0;">Welcome to {}, {}!</h2>
    <p style="color: #94a3b8; font-size: 14px; line-height: 1.6;">
      Thank you for registering. Please click the button below to verify your email address and activate your account.
    </p>
    <div style="margin: 30px 0; text-align: center;">
      <a href="{}" style="background-color: #ea580c; color: #ffffff; text-decoration: none; padding: 14px 28px; border-radius: 10px; font-weight: bold; font-size: 14px; display: inline-block;">
        Verify Email Address
      </a>
    </div>
    <p style="color: #64748b; font-size: 12px;">Or copy and paste this link: <br/><a href="{}" style="color: #38bdf8;">{}</a></p>
  </div>
</body>
</html>"#,
        settings.store_name, customer_name, verify_url, verify_url, verify_url
    );

    let _ = send_email_raw(settings, to_email, &subject, &html).await;
}

pub async fn send_order_created_email(
    settings: &StoreSettings,
    to_email: &str,
    order_number: &str,
    total_cents: i32,
    currency: &str,
) {
    let formatted_price = format!("{:.2} {}", total_cents as f64 / 100.0, currency);
    let subject = format!("Order Confirmation: #{}", order_number);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #0f172a; color: #f8fafc; padding: 40px 20px;">
  <div style="max-width: 560px; margin: 0 auto; background: #1e293b; border-radius: 16px; border: 1px solid #334155; padding: 32px;">
    <h2 style="color: #f97316; margin-top: 0;">Thank you for your order!</h2>
    <p style="color: #94a3b8; font-size: 14px; line-height: 1.6;">
      We have received your order <strong>#{}</strong> for <strong>{}</strong> and are currently preparing it for processing.
    </p>
    <div style="margin-top: 24px; padding: 16px; background: #0f172a; border-radius: 12px; font-size: 13px; color: #cbd5e1;">
      Order Number: <strong style="color: #ffffff;">#{}</strong><br/>
      Total: <strong style="color: #ffffff;">{}</strong>
    </div>
    <p style="color: #94a3b8; font-size: 13px; margin-top: 20px;">
      You will receive another update once payment is confirmed and when your package is dispatched with tracking information.
    </p>
  </div>
</body>
</html>"#,
        order_number, formatted_price, order_number, formatted_price
    );

    let _ = send_email_raw(settings, to_email, &subject, &html).await;
}

pub async fn send_payment_received_email(
    settings: &StoreSettings,
    to_email: &str,
    order_number: &str,
    total_cents: i32,
    currency: &str,
) {
    let formatted_price = format!("{:.2} {}", total_cents as f64 / 100.0, currency);
    let subject = format!("Payment Received for Order #{}", order_number);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #0f172a; color: #f8fafc; padding: 40px 20px;">
  <div style="max-width: 560px; margin: 0 auto; background: #1e293b; border-radius: 16px; border: 1px solid #334155; padding: 32px;">
    <h2 style="color: #10b981; margin-top: 0;">Payment Confirmed!</h2>
    <p style="color: #94a3b8; font-size: 14px; line-height: 1.6;">
      Your payment of <strong>{}</strong> for Order <strong>#{}</strong> has been successfully processed.
    </p>
    <p style="color: #94a3b8; font-size: 13px; margin-top: 20px;">
      Our fulfillment warehouse will begin packaging your items immediately.
    </p>
  </div>
</body>
</html>"#,
        formatted_price, order_number
    );

    let _ = send_email_raw(settings, to_email, &subject, &html).await;
}

pub async fn send_order_shipped_email(
    settings: &StoreSettings,
    to_email: &str,
    order_number: &str,
    tracking_number: &str,
) {
    let tracking_url = if tracking_number.starts_with("DHL") || tracking_number.len() >= 10 {
        format!("https://www.dhl.de/de/privatkunden/pakete-empfangen/verfolgen.html?piececode={}", tracking_number)
    } else {
        format!("https://parcelsapp.com/en/tracking/{}", tracking_number)
    };

    let subject = format!("Your Order #{} has shipped!", order_number);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #0f172a; color: #f8fafc; padding: 40px 20px;">
  <div style="max-width: 560px; margin: 0 auto; background: #1e293b; border-radius: 16px; border: 1px solid #334155; padding: 32px;">
    <h2 style="color: #f97316; margin-top: 0;">Your package is on its way!</h2>
    <p style="color: #94a3b8; font-size: 14px; line-height: 1.6;">
      Great news! Order <strong>#{}</strong> has been dispatched from our warehouse.
    </p>
    <div style="margin: 24px 0; padding: 20px; background: #0f172a; border-radius: 12px; text-align: center;">
      <span style="color: #94a3b8; font-size: 12px; display: block; margin-bottom: 6px;">Tracking Number</span>
      <span style="font-family: monospace; font-size: 18px; font-weight: bold; color: #f8fafc;">{}</span>
      <div style="margin-top: 16px;">
        <a href="{}" style="background-color: #ea580c; color: #ffffff; text-decoration: none; padding: 10px 20px; border-radius: 8px; font-weight: bold; font-size: 13px; display: inline-block;">
          Track Shipment
        </a>
      </div>
    </div>
  </div>
</body>
</html>"#,
        order_number, tracking_number, tracking_url
    );

    let _ = send_email_raw(settings, to_email, &subject, &html).await;
}

pub async fn send_back_in_stock_email(
    settings: &StoreSettings,
    to_email: &str,
    product_title: &str,
    product_url: &str,
) {
    let subject = format!("Back in stock: {}", product_title);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #0f172a; color: #f8fafc; padding: 40px 20px;">
  <div style="max-width: 560px; margin: 0 auto; background: #1e293b; border-radius: 16px; border: 1px solid #334155; padding: 32px;">
    <h2 style="color: #10b981; margin-top: 0;">Good news! It's back in stock!</h2>
    <p style="color: #94a3b8; font-size: 14px; line-height: 1.6;">
      You requested to be notified when <strong>{}</strong> returned to stock at {}. It is now available to order!
    </p>
    <div style="margin: 28px 0; text-align: center;">
      <a href="{}" style="background-color: #ea580c; color: #ffffff; text-decoration: none; padding: 12px 24px; border-radius: 10px; font-weight: bold; font-size: 14px; display: inline-block;">
        View Product Now
      </a>
    </div>
    <p style="color: #64748b; font-size: 12px;">Stock levels may be limited. Grab yours before it runs out again!</p>
  </div>
</body>
</html>"#,
        product_title, settings.store_name, product_url
    );

    let _ = send_email_raw(settings, to_email, &subject, &html).await;
}

pub async fn send_order_created_notification(pool: &sqlx::PgPool, order_number: &str) {
    let settings = match sqlx::query_as::<_, StoreSettings>("SELECT id, store_name, currency, currency_symbol, tax_rate_percent, deployment_mode, debug_mode, support_email, company_address, vat_id, logo_url, phone, hero_config, smtp_host, smtp_port, smtp_username, smtp_password, smtp_encryption, smtp_from_email, smtp_from_name, smtp_enabled, require_registered_checkout, require_email_verification, store_subtitle, show_store_title, show_store_subtitle, carousels_config, updated_at FROM store_settings WHERE id = 1").fetch_one(pool).await {
        Ok(s) => s,
        Err(_) => return,
    };

    if let Ok(order_row) = sqlx::query("SELECT customer_email, total_cents FROM orders WHERE order_number = $1")
        .bind(order_number)
        .fetch_one(pool)
        .await
    {
        use sqlx::Row;
        let email: String = order_row.get("customer_email");
        let total: i32 = order_row.get("total_cents");
        send_order_created_email(&settings, &email, order_number, total, &settings.currency).await;
    }
}

pub async fn send_payment_received_notification(pool: &sqlx::PgPool, order_number: &str) {
    let settings = match sqlx::query_as::<_, StoreSettings>("SELECT id, store_name, currency, currency_symbol, tax_rate_percent, deployment_mode, debug_mode, support_email, company_address, vat_id, logo_url, phone, hero_config, smtp_host, smtp_port, smtp_username, smtp_password, smtp_encryption, smtp_from_email, smtp_from_name, smtp_enabled, require_registered_checkout, require_email_verification, store_subtitle, show_store_title, show_store_subtitle, carousels_config, updated_at FROM store_settings WHERE id = 1").fetch_one(pool).await {
        Ok(s) => s,
        Err(_) => return,
    };

    if let Ok(order_row) = sqlx::query("SELECT customer_email, total_cents FROM orders WHERE order_number = $1")
        .bind(order_number)
        .fetch_one(pool)
        .await
    {
        use sqlx::Row;
        let email: String = order_row.get("customer_email");
        let total: i32 = order_row.get("total_cents");
        send_payment_received_email(&settings, &email, order_number, total, &settings.currency).await;
    }
}
