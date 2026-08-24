use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldStatus {
    Held,
    Converted,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Paid,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitlistStatus {
    Waiting,
    Offered,
    Fulfilled,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketClass {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub capacity: i32,
    pub sale_starts_at: DateTime<Utc>,
    pub sale_ends_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketHold {
    pub id: Uuid,
    pub event_id: Uuid,
    pub ticket_class_id: Uuid,
    pub quantity: i32,
    pub status: HoldStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketOrder {
    pub id: Uuid,
    pub event_id: Uuid,
    pub ticket_class_id: Uuid,
    pub hold_id: Uuid,
    pub quantity: i32,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitlistEntry {
    pub id: Uuid,
    pub event_id: Uuid,
    pub ticket_class_id: Uuid,
    /// Stable one-way reference supplied by the caller; never attendee PII.
    pub attendee_ref_hash: String,
    pub quantity: i32,
    pub position: i64,
    pub status: WaitlistStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitlistOffer {
    pub id: Uuid,
    pub waitlist_entry_id: Uuid,
    pub hold_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryReceipt {
    pub event_id: Uuid,
    pub event_capacity: i32,
    pub held: i64,
    pub sold: i64,
    pub remaining: i64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureEventInventory {
    pub event_id: Uuid,
    pub capacity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTicketClass {
    pub event_id: Uuid,
    pub name: String,
    pub capacity: i32,
    pub sale_starts_at: DateTime<Utc>,
    pub sale_ends_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveTickets {
    pub event_id: Uuid,
    pub ticket_class_id: Uuid,
    pub quantity: i32,
    pub idempotency_key: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTicketOrder {
    pub hold_id: Uuid,
    pub checkout_idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmTicketPayment {
    pub order_id: Uuid,
    pub payment_idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTicketOrder {
    pub order_id: Uuid,
    pub cancellation_idempotency_key: String,
    pub refund: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinWaitlist {
    pub event_id: Uuid,
    pub ticket_class_id: Uuid,
    pub attendee_ref_hash: String,
    pub quantity: i32,
}

impl ConfigureEventInventory {
    pub fn validate(&self) -> Result<(), TicketingValidationError> {
        positive("capacity", self.capacity)
    }
}

impl CreateTicketClass {
    pub fn validate(&self) -> Result<(), TicketingValidationError> {
        if self.name.trim().is_empty() || self.name.len() > 120 {
            return Err(TicketingValidationError(
                "name must contain 1 through 120 bytes".into(),
            ));
        }
        positive("capacity", self.capacity)?;
        if self.sale_ends_at <= self.sale_starts_at {
            return Err(TicketingValidationError(
                "sale_ends_at must be after sale_starts_at".into(),
            ));
        }
        Ok(())
    }
}

impl ReserveTickets {
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), TicketingValidationError> {
        positive("quantity", self.quantity)?;
        idempotency_key(&self.idempotency_key)?;
        if self.expires_at <= now {
            return Err(TicketingValidationError(
                "expires_at must be in the future".into(),
            ));
        }
        Ok(())
    }
}

impl CreateTicketOrder {
    pub fn validate(&self) -> Result<(), TicketingValidationError> {
        idempotency_key(&self.checkout_idempotency_key)
    }
}

impl ConfirmTicketPayment {
    pub fn validate(&self) -> Result<(), TicketingValidationError> {
        idempotency_key(&self.payment_idempotency_key)
    }
}

impl CancelTicketOrder {
    pub fn validate(&self) -> Result<(), TicketingValidationError> {
        idempotency_key(&self.cancellation_idempotency_key)
    }
}

impl JoinWaitlist {
    pub fn validate(&self) -> Result<(), TicketingValidationError> {
        positive("quantity", self.quantity)?;
        if self.attendee_ref_hash.len() < 32 || self.attendee_ref_hash.len() > 128 {
            return Err(TicketingValidationError(
                "attendee_ref_hash must contain 32 through 128 bytes".into(),
            ));
        }
        Ok(())
    }
}

fn positive(field: &str, value: i32) -> Result<(), TicketingValidationError> {
    if value <= 0 {
        return Err(TicketingValidationError(format!(
            "{field} must be greater than zero"
        )));
    }
    Ok(())
}

fn idempotency_key(value: &str) -> Result<(), TicketingValidationError> {
    if value.trim().is_empty() || value.len() > 200 {
        return Err(TicketingValidationError(
            "idempotency key must contain 1 through 200 bytes".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketingValidationError(pub String);

impl fmt::Display for TicketingValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TicketingValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn reservation_rejects_expired_hold() {
        let now = Utc::now();
        let input = ReserveTickets {
            event_id: Uuid::new_v4(),
            ticket_class_id: Uuid::new_v4(),
            quantity: 1,
            idempotency_key: "reserve-1".into(),
            expires_at: now - Duration::seconds(1),
        };

        assert_eq!(
            input.validate(now).unwrap_err().0,
            "expires_at must be in the future"
        );
    }

    #[test]
    fn waitlist_requires_opaque_non_pii_reference() {
        let input = JoinWaitlist {
            event_id: Uuid::new_v4(),
            ticket_class_id: Uuid::new_v4(),
            attendee_ref_hash: "short".into(),
            quantity: 1,
        };

        assert!(input.validate().is_err());
    }
}
