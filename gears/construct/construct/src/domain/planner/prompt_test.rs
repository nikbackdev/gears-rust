use construct_sdk::person_types::PROPERTIES;
use serde_json::Value;
use uuid::Uuid;

use super::prompt::{instructions, record_input};
use super::test_data::catalog;
use crate::domain::record_intake::{Envelope, ReceivedRecord};
use crate::test_support::example_records;

fn received(record: Value) -> ReceivedRecord {
    let text = |field: &str| record[field].as_str().expect(field).to_owned();
    ReceivedRecord {
        tenant_id: Uuid::new_v4(),
        connector: "connector".to_owned(),
        envelope: Envelope {
            type_id: text("type"),
            provenance: text("provenance"),
            version: text("version"),
            subject_id: record["subject_id"]
                .as_str()
                .map(|subject| Uuid::parse_str(subject).expect("a subject")),
        },
        record,
    }
}

#[test]
fn the_instructions_list_every_property_with_its_cardinality_and_form() {
    let text = instructions(&catalog());

    for line in [
        "roles: role (one): text",
        "skills: skills (many): {level: beginner | intermediate | advanced | expert, name (required): text, years: integer}",
        "preferences: tone_preference (one): concise | standard | detailed",
        "identity: timezone (one): text (IANA time zone name: Europe/Berlin, UTC.)",
        "identity: orcid_id (one): text matching ^\\d{4}-\\d{4}-\\d{4}-\\d{3}[\\dX]$",
        "identity: public_profiles (many): URL",
    ] {
        assert!(text.contains(line), "missing: {line}");
    }
    for (_, property, _) in PROPERTIES {
        assert!(
            text.contains(&format!(": {property} (")),
            "missing: {property}"
        );
    }
    assert!(!text.contains("{properties}"));
    assert!(!text.contains("gts."));
}

#[test]
fn each_record_type_reaches_the_model_with_its_payload_and_schema() {
    for example in example_records() {
        let record = received(example);

        let input = record_input(&record, "1. role (roles): teacher");

        assert!(!input.starts_with("Record: A record."), "{input}");
        assert!(input.contains("Payload schema: {"), "{input}");
        assert!(input.contains("Payload: {"), "{input}");
        for id_field in ["\"id\"", "_id\""] {
            assert!(!input.contains(id_field), "{input} shows {id_field}");
        }
        assert!(
            input.ends_with("Profile:\n1. role (roles): teacher"),
            "{input}"
        );
        assert!(!input.contains("gts."), "{input}");
    }
}

#[test]
fn an_empty_profile_is_said_to_be_empty() {
    let record = received(example_records().remove(0));

    assert!(record_input(&record, "").ends_with("Profile:\n(empty)"));
}
