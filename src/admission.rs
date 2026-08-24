use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub const ADMISSION_TOKEN_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionTokenClaims {
    pub version: u8,
    pub token_id: Uuid,
    pub event_id: Uuid,
    pub ticket_id: Uuid,
    pub order_id: Uuid,
    pub issuance_epoch: i64,
    pub key_id: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionVerificationKey {
    pub key_id: String,
    pub issuance_epoch: i64,
    /// URL-safe, unpadded base64 Ed25519 public key bytes.
    pub public_key: String,
    pub active_from: DateTime<Utc>,
    pub retire_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionRevocation {
    pub ticket_id: Uuid,
    pub issuance_epoch: i64,
    pub revoked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionKeySnapshot {
    pub snapshot_id: Uuid,
    pub event_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub keys: Vec<AdmissionVerificationKey>,
    pub revocations: Vec<AdmissionRevocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanReceiptClaims {
    pub receipt_id: Uuid,
    pub scanner_id: Uuid,
    pub scanner_key_id: String,
    pub scanner_sequence: i64,
    pub token_id: Uuid,
    pub event_id: Uuid,
    pub ticket_id: Uuid,
    pub order_id: Uuid,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedScanReceipt {
    pub claims: ScanReceiptClaims,
    /// URL-safe, unpadded base64 Ed25519 signature over canonical claims JSON.
    pub signature: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionOutcome {
    Accepted,
    DuplicateReview,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdmissionReceiptOutcome {
    pub receipt_id: Uuid,
    pub ticket_id: Uuid,
    pub outcome: AdmissionOutcome,
    pub reason: Option<String>,
    pub winning_receipt_id: Option<Uuid>,
}

impl AdmissionTokenClaims {
    pub fn validate_shape(&self) -> Result<(), AdmissionValidationError> {
        if self.version != ADMISSION_TOKEN_VERSION {
            return Err(AdmissionValidationError("unsupported token version".into()));
        }
        if self.key_id.trim().is_empty() || self.key_id.len() > 120 {
            return Err(AdmissionValidationError(
                "key_id must contain 1 through 120 bytes".into(),
            ));
        }
        if self.issuance_epoch < 0 {
            return Err(AdmissionValidationError(
                "issuance_epoch must be non-negative".into(),
            ));
        }
        if self.expires_at <= self.issued_at {
            return Err(AdmissionValidationError(
                "expires_at must be after issued_at".into(),
            ));
        }
        Ok(())
    }
}

impl AdmissionKeySnapshot {
    pub fn validate_shape(&self) -> Result<(), AdmissionValidationError> {
        if self.valid_until <= self.generated_at {
            return Err(AdmissionValidationError(
                "snapshot valid_until must be after generated_at".into(),
            ));
        }
        if self.keys.is_empty() {
            return Err(AdmissionValidationError(
                "snapshot must contain at least one verification key".into(),
            ));
        }
        for key in &self.keys {
            if key.key_id.trim().is_empty()
                || key.issuance_epoch < 0
                || key.retire_at <= key.active_from
            {
                return Err(AdmissionValidationError(
                    "snapshot contains an invalid verification key".into(),
                ));
            }
        }
        Ok(())
    }
}

impl ScanReceiptClaims {
    pub fn validate_shape(&self) -> Result<(), AdmissionValidationError> {
        if self.scanner_key_id.trim().is_empty() || self.scanner_key_id.len() > 120 {
            return Err(AdmissionValidationError(
                "scanner_key_id must contain 1 through 120 bytes".into(),
            ));
        }
        if self.scanner_sequence < 0 {
            return Err(AdmissionValidationError(
                "scanner_sequence must be non-negative".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionValidationError(pub String);

impl fmt::Display for AdmissionValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AdmissionValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn token_requires_a_bounded_lifetime() {
        let now = Utc::now();
        let claims = AdmissionTokenClaims {
            version: ADMISSION_TOKEN_VERSION,
            token_id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            ticket_id: Uuid::new_v4(),
            order_id: Uuid::new_v4(),
            issuance_epoch: 1,
            key_id: "key-1".into(),
            issued_at: now,
            expires_at: now - Duration::seconds(1),
        };

        assert_eq!(
            claims.validate_shape().unwrap_err().0,
            "expires_at must be after issued_at"
        );
    }

    #[test]
    fn scanner_sequence_is_monotonic_domain_data() {
        let claims = ScanReceiptClaims {
            receipt_id: Uuid::new_v4(),
            scanner_id: Uuid::new_v4(),
            scanner_key_id: "scanner-key-1".into(),
            scanner_sequence: -1,
            token_id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            ticket_id: Uuid::new_v4(),
            order_id: Uuid::new_v4(),
            scanned_at: Utc::now(),
        };

        assert!(claims.validate_shape().is_err());
    }
}
