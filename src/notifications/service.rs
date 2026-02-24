use chrono::Datelike;
use lettre::{
    message::MultiPart,
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        Error as SmtpError,
    },
    Message, SmtpTransport, Transport,
};
use serde::{Deserialize, Serialize};
use std::{env, num::ParseIntError};
use thiserror::Error;

use crate::versions::models::{Change, ChangeType};

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("SMTP error: {0}")]
    SmtpError(#[from] SmtpError),
    #[error("General SMTP error: {0}")]
    LettreError(#[from] lettre::error::Error),
    #[error("Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),
    #[error("Address parse error: {0}")]
    AddressError(#[from] lettre::address::AddressError),
    #[error("Database error: {0}")]
    DbError(#[from] tokio_postgres::Error),
    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailNotification {
    pub to_email: String,
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
}

pub struct EmailService {
    mailer: SmtpTransport,
    from_address: String,
    frontend_url: String,
}

impl EmailService {
    pub fn new() -> Result<Self, EmailError> {
        let smtp_credentials =
            Credentials::new(env::var("SMTP_USERNAME")?, env::var("SMTP_PASSWORD")?);
        let smtp_port: u16 = env::var("SMTP_PORT")?
            .parse()
            .map_err(|e: ParseIntError| EmailError::ParseError(e.to_string()))?;

        let smtp_host = env::var("SMTP_HOST")?;

        let tls_params = lettre::transport::smtp::client::TlsParameters::builder(smtp_host.clone())
            .dangerous_accept_invalid_certs(true)
            .build()
            .map_err(|e| EmailError::Other(e.to_string()))?;

        let mailer = SmtpTransport::relay(&smtp_host)
            .map_err(EmailError::SmtpError)?
            .port(smtp_port)
            .credentials(smtp_credentials)
            .authentication(vec![Mechanism::Plain])
            .tls(lettre::transport::smtp::client::Tls::Required(tls_params))
            .build();

        let from_address = env::var("SMTP_FROM_ADDRESS")?;

        Ok(EmailService {
            mailer,
            from_address,
            frontend_url: env::var("FRONTEND_URL")?,
        })
    }

    pub fn build_email_content(
        &self,
        content: &[&str],
        action_link: Option<(&str, &str)>,
    ) -> (String, String) {
        let mut text_body = String::new();
        let mut html_body = String::new();

        // Common text body construction
        text_body.push_str(&content.join("\n"));

        // Common HTML template - using table layout for email client compatibility
        html_body.push_str(&format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="color-scheme" content="light">
    <meta name="supported-color-schemes" content="light">
    <!--[if mso]>
    <style type="text/css">
        table {{border-collapse:collapse;border-spacing:0;margin:0;}}
        div, td {{padding:0;}}
        div {{margin:0;}}
    </style>
    <![endif]-->
</head>
<body style="margin: 0; padding: 0; background-color: #f8fafc; font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif; -webkit-font-smoothing: antialiased;">
    <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%" style="background-color: #f8fafc; padding: 40px 20px;">
        <tr>
            <td align="center">
                <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%" style="max-width: 600px; background-color: #ffffff; border-radius: 20px; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05), 0 2px 4px -2px rgba(0,0,0,0.05); overflow: hidden;">
                    <!-- Header -->
                    <tr>
                        <td align="center" style="padding: 32px 32px 24px; border-bottom: 1px solid #f1f5f9;">
                            <img src="{}/assets/icons/favicon.png" alt="Lojban Dictionary Logo" style="height: 56px; width: auto; display: block; margin: 0 auto;">
                        </td>
                    </tr>
                    <!-- Content -->
                    <tr>
                        <td style="padding: 40px 48px;">
                            <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%">
                                <tr>
                                    <td style="font-size: 16px; line-height: 1.6; color: #334155; padding-bottom: 32px; text-align: center;">
                                        {}"#,
            self.frontend_url,
            content.join("<br><br>")
        ));

        // Add action button if provided
        if let Some((action_text, action_url)) = action_link {
            html_body.push_str(&format!(
                r#"                                    </td>
                                </tr>
                                <tr>
                                    <td align="center" style="padding-top: 8px;">
                                        <table role="presentation" cellspacing="0" cellpadding="0" border="0">
                                            <tr>
                                                <td align="center" style="border-radius: 9999px; background: linear-gradient(180deg, #60a5fa 0%, #3b82f6 100%); box-shadow: 0 1px 2px rgba(0,0,0,0.05);">
                                                    <a href="{}" style="display: inline-block; padding: 14px 36px; font-size: 15px; font-weight: 600; color: #ffffff; text-decoration: none; border-radius: 9999px; border: 1px solid #3b82f6;">{}</a>
                                                </td>
                                            </tr>
                                        </table>
                                    </td>
                                </tr>"#,
                action_url, action_text
            ));
        } else {
            html_body.push_str(
                r#"                                    </td>
                                </tr>"#,
            );
        }

        // Common footer
        html_body.push_str(&format!(
            r#"                            </table>
                        </td>
                    </tr>
                    <!-- Footer -->
                    <tr>
                        <td align="center" style="padding: 32px 48px; background-color: #f8fafc; border-top: 1px solid #f1f5f9;">
                            <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%">
                                <tr>
                                    <td align="center" style="font-size: 13px; line-height: 1.5; color: #64748b;">
                                        <p style="margin: 0 0 8px 0;">This message was sent by the Lojban Dictionary service.</p>
                                        <p style="margin: 0;">© {} Lojban Dictionary</p>
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>"#,
            chrono::Local::now().year()
        ));

        (text_body, html_body)
    }

    /// Build (text_body, html_body) for "definition updated" notifications.
    /// Uses the same template shell as other emails (logo, footer) with structured content:
    /// valsi word, who updated it, new definition snippet, change summary, and action button.
    pub fn build_definition_updated_email_content(
        &self,
        valsi_word: &str,
        actor_username: Option<&str>,
        new_definition: Option<&str>,
        changes: Option<&[Change]>,
        action_url: &str,
    ) -> (String, String) {
        let escape = |s: &str| {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
        };

        // Plain text
        let mut text_body = String::new();
        text_body.push_str(&format!("Definition updated for {}\n\n", valsi_word));
        if let Some(u) = actor_username {
            text_body.push_str(&format!("Updated by: {}\n\n", u));
        }
        if let Some(def) = new_definition {
            let trunc = if def.len() > 400 { &def[..400] } else { def };
            text_body.push_str("New definition:\n");
            text_body.push_str(trunc);
            if def.len() > 400 {
                text_body.push_str("...");
            }
            text_body.push_str("\n\n");
        }
        if let Some(chgs) = changes {
            if !chgs.is_empty() {
                text_body.push_str("Changes:\n");
                for c in chgs {
                    let label = match c.change_type {
                        ChangeType::Added => "Added",
                        ChangeType::Removed => "Removed",
                        ChangeType::Modified => "Modified",
                    };
                    text_body.push_str(&format!("  - {} ({}): ", c.field, label));
                    if let Some(v) = &c.new_value {
                        let t = if v.len() > 80 {
                            format!("{}...", &v[..80])
                        } else {
                            v.clone()
                        };
                        text_body.push_str(&t);
                    } else if let Some(v) = &c.old_value {
                        text_body.push_str("(removed)");
                        let t = if v.len() > 60 {
                            format!("{}...", &v[..60])
                        } else {
                            v.clone()
                        };
                        text_body.push_str(&t);
                    }
                    text_body.push('\n');
                }
                text_body.push('\n');
            }
        }
        text_body.push_str(&format!("View update: {}", action_url));

        // HTML: same shell as build_email_content, with structured content
        let valsi_esc = escape(valsi_word);
        let actor_line = actor_username
            .map(|u| format!(r#"<div style="display: inline-block; background-color: #f1f5f9; color: #475569; padding: 4px 12px; border-radius: 9999px; font-size: 13px; font-weight: 500; margin-bottom: 24px;">By <strong>{}</strong></div>"#, escape(u)))
            .unwrap_or_else(String::new);

        let def_block = new_definition.map(|def| {
            let esc = escape(def);
            let trunc = if esc.len() > 400 {
                format!("{}...", esc.chars().take(400).collect::<String>())
            } else {
                esc.clone()
            };
            format!(
                r#"<tr><td style="padding-bottom: 24px;"><div style="font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b; margin-bottom: 8px;">Definition</div><div style="font-size: 15px; line-height: 1.6; color: #1e293b; background: #f8fafc; border-left: 4px solid #3b82f6; padding: 16px; border-radius: 0 12px 12px 0; text-align: left;">{}</div></td></tr>"#,
                trunc.replace('\n', "<br>")
            )
        }).unwrap_or_else(String::new);

        let changes_block = changes.filter(|chgs| !chgs.is_empty()).map(|chgs| {
            let rows: String = chgs.iter().map(|c| {
                let (label, bg, color) = match c.change_type {
                    ChangeType::Added => ("Added", "#d1fae5", "#059669"),
                    ChangeType::Removed => ("Removed", "#fee2e2", "#dc2626"),
                    ChangeType::Modified => ("Modified", "#fef3c7", "#d97706"),
                };
                let field_esc = escape(&c.field);
                let content = c.new_value.as_ref()
                    .or(c.old_value.as_ref())
                    .map(|v| {
                        let esc = escape(v);
                        if esc.len() > 120 { format!("{}...", esc.chars().take(120).collect::<String>()) } else { esc }
                    })
                    .unwrap_or_else(|| "(empty)".to_string());
                format!(
                    r#"<tr><td style="padding: 12px 16px; border-bottom: 1px solid #f1f5f9; text-align: left;"><div style="margin-bottom: 4px;"><span style="font-size: 13px; font-weight: 600; color: #334155;">{}</span> <span style="display: inline-block; background-color: {}; color: {}; padding: 2px 8px; border-radius: 9999px; font-size: 11px; font-weight: 600; margin-left: 8px; vertical-align: middle;">{}</span></div><div style="font-size: 14px; color: #64748b; line-height: 1.5;">{}</div></td></tr>"#,
                    field_esc, bg, color, label, content.replace('\n', " ")
                )
            }).collect();
            format!(
                r#"<tr><td style="padding-bottom: 32px;"><div style="font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b; margin-bottom: 8px;">Changes</div><table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%" style="background: #ffffff; border-radius: 12px; border: 1px solid #e2e8f0; overflow: hidden;">{}</table></td></tr>"#,
                rows
            )
        }).unwrap_or_else(String::new);

        let content_inner = format!(
            r#"<tr><td style="text-align: center; padding-bottom: 8px;">{}</td></tr>
               <tr><td style="font-size: 24px; font-weight: 700; color: #0f172a; padding-bottom: 24px; letter-spacing: -0.02em; text-align: center;">{} <span style="color: #3b82f6;">{}</span></td></tr>
               {}
               {}"#,
            if actor_line.is_empty() {
                String::new()
            } else {
                format!("{}", actor_line)
            },
            if changes.as_ref().map(|c| c.is_empty()).unwrap_or(true) {
                "New valsi added:"
            } else {
                "Definition updated for"
            },
            valsi_esc,
            def_block,
            changes_block
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="color-scheme" content="light">
    <meta name="supported-color-schemes" content="light">
    <!--[if mso]>
    <style type="text/css">
        table {{border-collapse:collapse;border-spacing:0;margin:0;}}
        div, td {{padding:0;}}
        div {{margin:0;}}
    </style>
    <![endif]-->
</head>
<body style="margin: 0; padding: 0; background-color: #f8fafc; font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif; -webkit-font-smoothing: antialiased;">
    <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%" style="background-color: #f8fafc; padding: 40px 20px;">
        <tr>
            <td align="center">
                <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%" style="max-width: 600px; background-color: #ffffff; border-radius: 20px; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05), 0 2px 4px -2px rgba(0,0,0,0.05); overflow: hidden;">
                    <!-- Header -->
                    <tr>
                        <td align="center" style="padding: 32px 32px 24px; border-bottom: 1px solid #f1f5f9;">
                            <img src="{}/assets/icons/favicon.png" alt="Lojban Dictionary Logo" style="height: 56px; width: auto; display: block; margin: 0 auto;">
                        </td>
                    </tr>
                    <!-- Content -->
                    <tr>
                        <td style="padding: 40px 48px; text-align: center;">
                            <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%">
                                {}
                                <tr>
                                    <td align="center" style="padding-top: 16px;">
                                        <table role="presentation" cellspacing="0" cellpadding="0" border="0">
                                            <tr>
                                                <td align="center" style="border-radius: 9999px; background: linear-gradient(180deg, #60a5fa 0%, #3b82f6 100%); box-shadow: 0 1px 2px rgba(0,0,0,0.05);">
                                                    <a href="{}" style="display: inline-block; padding: 14px 36px; font-size: 15px; font-weight: 600; color: #ffffff; text-decoration: none; border-radius: 99999px; border: 1px solid #3b82f6;">View on Dictionary</a>
                                                </td>
                                            </tr>
                                        </table>
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                    <!-- Footer -->
                    <tr>
                        <td align="center" style="padding: 32px 48px; background-color: #f8fafc; border-top: 1px solid #f1f5f9;">
                            <table role="presentation" cellspacing="0" cellpadding="0" border="0" width="100%">
                                <tr>
                                    <td align="center" style="font-size: 13px; line-height: 1.5; color: #64748b;">
                                        <p style="margin: 0 0 8px 0;">This message was sent by the Lojban Dictionary service.</p>
                                        <p style="margin: 0;">© {} Lojban Dictionary</p>
                                    </td>
                                </tr>
                            </table>
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>"#,
            self.frontend_url,
            content_inner,
            action_url,
            chrono::Local::now().year()
        );

        (text_body, html_body)
    }

    pub fn send_notification(&self, notification: EmailNotification) -> Result<(), EmailError> {
        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(notification.to_email.parse()?)
            .subject(notification.subject)
            .multipart(MultiPart::alternative_plain_html(
                notification.text_body,
                notification.html_body,
            ))
            .map_err(|e| EmailError::Other(e.to_string()))?;

        self.mailer.send(&email).map_err(EmailError::SmtpError)?;
        Ok(())
    }
}

pub async fn send_notification_emails(pool: &deadpool_postgres::Pool) -> Result<(), EmailError> {
    let client = pool.get().await?;

    let notifications = client
        .query(
            "SELECT n.*, u.email, v.word as valsi_word,
                    actor.username as actor_username,
                    n.definition_id
             FROM user_notifications n
             JOIN users u ON n.user_id = u.userid
             LEFT JOIN valsi v ON n.valsi_id = v.valsiid
             LEFT JOIN users actor ON n.actor_id = actor.userid
             WHERE n.email_sent IS NULL
             AND u.email IS NOT NULL
             AND u.email <> ''
             LIMIT 50",
            &[],
        )
        .await?;

    if notifications.is_empty() {
        return Ok(());
    }

    let email_service = EmailService::new()?;

    for row in notifications {
        let notification_id: i32 = row.get("notification_id");
        let email: String = row.get("email");
        let message: String = row.get("message");
        let notification_type: String = row.get("notification_type");
        let valsi_word: Option<String> = row.get("valsi_word");
        let link: Option<String> = row.get("link");
        let actor_username: Option<String> = row.get("actor_username");
        let definition_id: Option<i32> = row.get("definition_id");

        let subject = if notification_type == "edit" {
            match &valsi_word {
                Some(word) => format!("Lojban Dictionary: Definition updated - {}", word),
                None => "Lojban Dictionary: Definition updated".to_string(),
            }
        } else {
            match &valsi_word {
                Some(word) => format!("Lojban Dictionary: {} - {}", notification_type, word),
                None => format!("Lojban Dictionary: {}", notification_type),
            }
        };

        let (text_body, html_body) = if notification_type == "edit" || notification_type == "add" {
            let word = valsi_word.as_deref().unwrap_or("a valsi");
            let url = link.as_deref().unwrap_or("");
            let (new_definition, changes) = if let Some(def_id) = definition_id {
                let (version_pair, single_def) =
                    match crate::versions::service::get_definition_history(pool, def_id, 1, 2).await
                    {
                        Ok(history) if history.versions.len() >= 2 => (
                            Some((
                                history.versions[1].version_id,
                                history.versions[0].version_id,
                            )),
                            None,
                        ),
                        Ok(history) if history.versions.len() == 1 => {
                            (None, Some(history.versions[0].content.definition.clone()))
                        }
                        _ => (None, None),
                    };
                if let Some(single) = single_def {
                    (Some(single), None)
                } else if let Some((from_id, to_id)) = version_pair {
                    match crate::versions::service::get_diff_with_transaction(pool, from_id, to_id)
                        .await
                    {
                        Ok(diff) => (Some(diff.new_content.definition), Some(diff.changes)),
                        Err(_) => (None, None),
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };
            email_service.build_definition_updated_email_content(
                word,
                actor_username.as_deref(),
                new_definition.as_deref(),
                changes.as_deref().filter(|c| !c.is_empty()),
                url,
            )
        } else {
            let message_lines: Vec<&str> = message.split('\n').collect();
            email_service.build_email_content(
                &message_lines,
                link.as_ref().map(|url| ("View Link", url.as_str())),
            )
        };

        if let Err(e) = email_service.send_notification(EmailNotification {
            to_email: email,
            subject,
            text_body,
            html_body,
        }) {
            log::error!("Failed to send notification {}: {}", notification_id, e);
            continue;
        }

        if let Err(e) = client
            .execute(
                "UPDATE user_notifications 
                 SET email_sent = CURRENT_TIMESTAMP 
                 WHERE notification_id = $1",
                &[&notification_id],
            )
            .await
        {
            log::error!(
                "Failed to mark notification {} as sent: {}",
                notification_id,
                e
            );
        }
    }

    // Clean up any remaining notifications with empty emails
    client
        .execute(
            "DELETE FROM user_notifications 
         WHERE notification_id IN (
             SELECT notification_id 
             FROM user_notifications n
             JOIN users u ON n.user_id = u.userid
             WHERE u.email IS NULL OR u.email = ''
         )",
            &[],
        )
        .await?;

    Ok(())
}
