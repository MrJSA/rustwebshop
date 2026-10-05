use crate::models::StoreSettings;
use lettre::message::header::ContentType;
use lettre::message::{Mailbox, SinglePart};
use lettre::Address;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tracing::{error, info};

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
) -> Result<(), String> {
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
        html_escape(&settings.store_name), html_escape(customer_name), verify_url, verify_url, verify_url
    );

    send_email_raw(settings, to_email, &subject, &html).await
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
        send_order_created_email(&settings, &email, order_number, total, &settings.currency).await;
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
        send_payment_received_email(&settings, &email, order_number, total, &settings.currency).await;
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

pub async fn send_password_reset_email(settings: &StoreSettings, to_email: &str, token: &str) {
    let reset_url = format!("{}/account/reset-password?token={}", shop_public_url(), token);
    let subject = format!("Reset your password - {}", settings.store_name);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #0f172a; color: #f8fafc; padding: 40px 20px;">
  <div style="max-width: 560px; margin: 0 auto; background: #1e293b; border-radius: 16px; border: 1px solid #334155; padding: 32px;">
    <h2 style="color: #f97316; margin-top: 0;">Password reset for {}</h2>
    <p style="color: #94a3b8; font-size: 14px; line-height: 1.6;">
      Someone requested a password reset for your account. The link is valid for 1 hour.
      If this was not you, simply ignore this email — your password stays unchanged.
    </p>
    <div style="margin: 30px 0; text-align: center;">
      <a href="{}" style="background-color: #ea580c; color: #ffffff; text-decoration: none; padding: 14px 28px; border-radius: 10px; font-weight: bold; font-size: 14px; display: inline-block;">
        Choose a new password
      </a>
    </div>
    <p style="color: #64748b; font-size: 12px;">Or copy and paste this link: <br/><a href="{}" style="color: #38bdf8;">{}</a></p>
  </div>
</body>
</html>"#,
        html_escape(&settings.store_name), reset_url, reset_url, reset_url
    );
    let _ = send_email_raw(settings, to_email, &subject, &html).await;
}
