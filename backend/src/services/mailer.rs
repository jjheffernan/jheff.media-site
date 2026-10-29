use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::env;

pub async fn notify_contact(
    name: &str,
    email: &str,
    subject: &str,
    message: &str,
    kind: &str,
) {
    let host = env_trimmed("YEW_FULLSTACK_SMTP_HOST");
    let to = env_trimmed("YEW_FULLSTACK_CONTACT_EMAIL");
    if host.is_empty() || to.is_empty() {
        info!("Contact email skipped");
        return;
    }

    if let Err(err) = deliver(&host, &to, name, email, subject, message, kind).await {
        error!("Contact email send failed: {err}");
    }
}

async fn deliver(
    host: &str,
    to: &str,
    name: &str,
    email: &str,
    subject: &str,
    message: &str,
    kind: &str,
) -> Result<(), String> {
    let from = {
        let configured = env_trimmed("YEW_FULLSTACK_SMTP_FROM");
        if configured.is_empty() {
            to.to_string()
        } else {
            configured
        }
    };
    let from: Mailbox = parse_mailbox(&from)?;
    let to: Mailbox = parse_mailbox(to)?;
    let outgoing = Message::builder()
        .from(from)
        .to(to)
        .subject(subject_line(subject, kind))
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Kind: {kind}\nName: {name}\nEmail: {email}\nSubject: {subject}\n\n{message}"
        ))
        .map_err(|err| err.to_string())?;

    let username = env_trimmed("YEW_FULLSTACK_SMTP_USERNAME");
    let password = env::var("YEW_FULLSTACK_SMTP_PASSWORD").unwrap_or_default();
    let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
        .map_err(|err| err.to_string())?
        .port(smtp_port());
    let transport = if username.is_empty() {
        transport
    } else {
        transport.credentials(Credentials::new(username, password))
    };
    let mailer: AsyncSmtpTransport<Tokio1Executor> = transport.build();
    mailer
        .send(outgoing)
        .await
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn env_trimmed(key: &str) -> String {
    env::var(key).unwrap_or_default().trim().to_string()
}

fn smtp_port() -> u16 {
    env::var("YEW_FULLSTACK_SMTP_PORT")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(587)
}

fn subject_line(subject: &str, kind: &str) -> String {
    let subject = subject.trim();
    if kind == "booking" {
        let subject = if subject.is_empty() { "request" } else { subject };
        return format!("[jheff.media] Booking: {subject}");
    }
    let subject = if subject.is_empty() {
        "New message"
    } else {
        subject
    };
    format!("[jheff.media] {subject}")
}

fn parse_mailbox(address: &str) -> Result<Mailbox, String> {
    address
        .parse()
        .map_err(|err: lettre::address::AddressError| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::subject_line;

    #[test]
    fn subject_line_prefixes_or_defaults() {
        assert_eq!(subject_line("Prints", "contact"), "[jheff.media] Prints");
        assert_eq!(subject_line("  ", "contact"), "[jheff.media] New message");
        assert_eq!(
            subject_line("Track day", "booking"),
            "[jheff.media] Booking: Track day"
        );
        assert_eq!(subject_line("", "booking"), "[jheff.media] Booking: request");
    }
}
