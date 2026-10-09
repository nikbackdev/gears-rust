use serde_json::json;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use super::error::DomainError;
use super::plan::RecordOrigin;
use crate::test_support::received_chat_record;

#[test]
fn the_record_origin_comes_from_the_received_record() {
    let tenant = Uuid::new_v4();
    let subject = Uuid::new_v4();

    let origin = RecordOrigin::of(&received_chat_record(tenant, subject)).expect("an origin");

    assert_eq!(
        origin,
        RecordOrigin {
            connector: "chat_engine".to_owned(),
            type_id: construct_sdk::gts::CHAT_MESSAGE_TYPE.to_owned(),
            provenance: "chat_engine/thread-42/msg-7".to_owned(),
            version: "v1".to_owned(),
            observed_at: OffsetDateTime::parse("2026-09-10T09:13:05Z", &Rfc3339).expect("a time"),
        }
    );
}

#[test]
fn a_record_without_a_readable_observed_at_has_no_origin() {
    for observed_at in [
        json!(null),
        json!("10 September 2026"),
        json!(1_757_495_585),
    ] {
        let mut record = received_chat_record(Uuid::new_v4(), Uuid::new_v4());
        record.record["observed_at"] = observed_at.clone();

        let error = RecordOrigin::of(&record).expect_err("no origin");

        assert!(
            matches!(error, DomainError::Internal(_)),
            "{observed_at}: {error:?}"
        );
    }
}
