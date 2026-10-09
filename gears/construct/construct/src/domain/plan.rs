use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use toolkit_macros::domain_model;
use uuid::Uuid;

use super::error::DomainError;
use super::profile::{Category, ProfileVersion};
use super::record_intake::ReceivedRecord;

/// The tenant and the subject whose profile a plan changes.
#[domain_model]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanSubject {
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
}

/// The steps that change one subject's profile, the profile version they were built on, and the record they came
/// from. A plan is never stored as such; only its admitted steps reach the profile.
#[domain_model]
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub subject: PlanSubject,
    pub origin: RecordOrigin,
    pub profile_version: ProfileVersion,
    pub steps: Vec<Step>,
}

/// The record a plan came from: the connector, the record type, the record's identity and version, and when it was
/// observed.
#[domain_model]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordOrigin {
    pub connector: String,
    pub type_id: String,
    pub provenance: String,
    pub version: String,
    pub observed_at: OffsetDateTime,
}

impl RecordOrigin {
    /// The origin of a received record.
    ///
    /// # Errors
    ///
    /// [`DomainError::Internal`] when the record has no `observed_at` in RFC 3339.
    pub fn of(record: &ReceivedRecord) -> Result<Self, DomainError> {
        let observed_at = record
            .record
            .get("observed_at")
            .and_then(Value::as_str)
            .and_then(|text| OffsetDateTime::parse(text, &Rfc3339).ok())
            .ok_or_else(|| DomainError::internal("the record has no readable observed_at"))?;
        Ok(Self {
            connector: record.connector.clone(),
            type_id: record.envelope.type_id.clone(),
            provenance: record.envelope.provenance.clone(),
            version: record.envelope.version.clone(),
            observed_at,
        })
    }
}

/// A value a step stores: its category, its property and the whole value.
#[domain_model]
#[derive(Debug, Clone, PartialEq)]
pub struct NewFact {
    pub category: Category,
    pub property: String,
    pub value: Value,
}

/// One change to the profile, with the confidence the model gave it. `node_key` is always a key the step creates;
/// `old_key` is always a key it deletes.
#[domain_model]
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Add {
        node_key: String,
        fact: NewFact,
        confidence: f64,
    },
    Replace {
        old_key: String,
        node_key: String,
        fact: NewFact,
        confidence: f64,
    },
    Remove {
        old_key: String,
        confidence: f64,
    },
}
