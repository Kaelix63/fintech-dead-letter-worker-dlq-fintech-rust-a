mod dead_letter;
mod infrai_client;

use dead_letter::{decide, PaymentEvent};
use infrai_client::InfraiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = InfraiClient::from_env()?;
    let event = PaymentEvent { payment_id: "pay-demo".into(), amount_cents: 125_000, attempts: 3, reason: "processor_timeout".into() };
    let decision = decide(&event);
    let payload = serde_json::to_string(&event)?;
    client.queue_publish("payment-failures-dead-letter", payload).await?;
    println!("{}; action={:?}", decision.audit_message, decision.action);
    Ok(())
}
