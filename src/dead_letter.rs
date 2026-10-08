use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentEvent { pub payment_id: String, pub amount_cents: u64, pub attempts: u8, pub reason: String }

#[derive(Debug, Clone, PartialEq)]
pub enum RiskAction { FreezeAccount, ReviewManually, Retry }

#[derive(Debug, Clone, PartialEq)]
pub struct DeadLetterDecision { pub action: RiskAction, pub audit_message: String }

pub fn decide(event: &PaymentEvent) -> DeadLetterDecision {
    let action = if event.amount_cents >= 100_000 || event.reason == "fraud_signal" { RiskAction::FreezeAccount } else if event.attempts >= 3 { RiskAction::ReviewManually } else { RiskAction::Retry };
    DeadLetterDecision { action, audit_message: format!("payment {} moved to dead-letter review: {}", event.payment_id, event.reason) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn high_value_failure_freezes_and_is_audited() {
        let event = PaymentEvent { payment_id: "pay-42".into(), amount_cents: 250_000, attempts: 3, reason: "processor_timeout".into() };
        let decision = decide(&event);
        assert_eq!(decision.action, RiskAction::FreezeAccount);
        assert!(decision.audit_message.contains("pay-42"));
    }
}

