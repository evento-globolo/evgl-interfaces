use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Draft,
    Published,
    Cancelled,
    Completed,
}

impl Default for EventStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub summary: String,
    pub source: String,
    pub venue: String,
    pub starts_at: DateTime<Utc>,
    pub source_url: String,
    pub status: EventStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvent {
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub source: String,
    pub venue: String,
    pub starts_at: DateTime<Utc>,
    pub source_url: String,
}

impl CreateEvent {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.title.trim().is_empty() {
            return Err(ValidationError("title must not be empty".into()));
        }
        if self.summary.len() > 4_000 {
            return Err(ValidationError("summary exceeds 4000 bytes".into()));
        }
        if self.source.trim().is_empty() {
            return Err(ValidationError("source must not be empty".into()));
        }
        if self.venue.trim().is_empty() {
            return Err(ValidationError("venue must not be empty".into()));
        }
        if self.source_url.trim().is_empty() {
            return Err(ValidationError("source_url must not be empty".into()));
        }
        Ok(())
    }

    pub fn into_record(self, id: Uuid, now: DateTime<Utc>) -> Result<Event, ValidationError> {
        self.validate()?;
        Ok(Event {
            id,
            title: self.title,
            summary: self.summary,
            source: self.source,
            venue: self.venue,
            starts_at: self.starts_at,
            source_url: self.source_url,
            status: EventStatus::default(),
            created_at: now,
            updated_at: now,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub data: Event,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(pub String);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_serializes_as_wire_value() {
        let value = serde_json::to_string(&EventStatus::default()).unwrap();
        assert_eq!(value, serde_json::to_string(&"draft").unwrap());
    }
}
