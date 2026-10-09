use std::collections::BTreeSet;

use graph_storage_sdk::gts::{OWNED_NODE_TYPE, PROVENANCE_ATTRIBUTE_TYPE};
use serde_json::{Value, json};

use super::*;
use crate::gts::{IDENTITY_TYPE, PREFERENCE_TYPE, ROLE_TYPE, SKILL_TYPE};

fn schema(type_id: &str) -> Value {
    let (_, text) = PERSON_TYPES
        .iter()
        .find(|(id, _)| *id == type_id)
        .expect("a person type");
    serde_json::from_str(text).expect("a JSON schema")
}

fn payload_schema(type_id: &str) -> Value {
    schema(type_id)["allOf"][1]["properties"]["payload"].clone()
}

fn type_of(property: &str) -> &'static str {
    PROPERTIES
        .iter()
        .find(|(_, listed, _)| *listed == property)
        .map(|(type_id, _, _)| *type_id)
        .expect("a listed property")
}

fn fact_properties(type_id: &str) -> BTreeSet<String> {
    payload_schema(type_id)["properties"]
        .as_object()
        .expect("payload properties")
        .keys()
        .filter(|name| *name != ORIGIN_PROPERTY && *name != RECORD_PROPERTY)
        .cloned()
        .collect()
}

fn fits(type_id: &str, payload: &Value) -> bool {
    let mut schema = payload_schema(type_id);
    schema["properties"][ORIGIN_PROPERTY] = json!(true);
    jsonschema::validator_for(&schema)
        .expect("a valid payload schema")
        .is_valid(payload)
}

fn origin() -> Value {
    json!({ "produced_by": { "subject_id": "connector" }, "produced_at": "2026-10-07T00:00:00Z" })
}

fn fact(property: &str, value: &Value) -> Value {
    json!({ property: value, ORIGIN_PROPERTY: origin() })
}

fn value_fits(property: &str, value: &Value) -> bool {
    fits(type_of(property), &fact(property, value))
}

#[test]
fn each_schema_names_its_type_and_derives_from_the_owned_node() {
    for (type_id, _) in PERSON_TYPES {
        let document = schema(type_id);
        assert_eq!(document["$id"], format!("gts://{type_id}"));
        assert_eq!(
            document["allOf"][0]["$ref"],
            format!("gts://{OWNED_NODE_TYPE}")
        );
    }
}

#[test]
fn the_origin_is_required_and_is_graph_storage_provenance() {
    for (type_id, _) in PERSON_TYPES {
        let payload = payload_schema(type_id);
        assert_eq!(payload["required"], json!([ORIGIN_PROPERTY]), "{type_id}");
        assert_eq!(
            payload["properties"][ORIGIN_PROPERTY]["$ref"],
            format!("gts://{PROVENANCE_ATTRIBUTE_TYPE}"),
            "{type_id}"
        );
    }
}

#[test]
fn every_fact_property_has_a_cardinality_and_only_those() {
    for (type_id, _) in PERSON_TYPES {
        let listed: BTreeSet<String> = PROPERTIES
            .iter()
            .filter(|(listed_type, _, _)| *listed_type == type_id)
            .map(|(_, property, _)| (*property).to_owned())
            .collect();
        assert_eq!(listed, fact_properties(type_id), "{type_id}");
    }
}

#[test]
fn each_property_holds_one_value_unless_a_person_can_have_several() {
    let many: BTreeSet<&str> = [
        "name_variants",
        "public_mentions",
        "public_profiles",
        "assistive_tools",
        "communication_channel",
        "current_focus_areas",
        "focus_windows",
        "local_environment_constraint",
        "long_term_goals",
        "notification_type",
        "preferred_format",
        "short_term_goals",
        "study_window",
        "technology_constraint",
        "time_availability_pattern",
        "time_constraint",
        "affiliation_history",
        "board_memberships",
        "group_memberships",
        "roles_and_affiliations",
        "work_history",
        "activity_by_year",
        "awards",
        "languages",
        "publications",
        "research_areas",
        "skills",
    ]
    .into_iter()
    .collect();

    for (_, property, cardinality) in PROPERTIES {
        let expected = if many.contains(property) {
            Cardinality::Many
        } else {
            Cardinality::One
        };
        assert_eq!(*cardinality, expected, "{property}");
    }
}

#[test]
fn a_payload_holds_exactly_one_fact_and_its_origin() {
    let two_facts = json!({
        "tone_preference": "concise",
        "units": "si_metric",
        ORIGIN_PROPERTY: origin(),
    });
    let misfits = [
        (json!({ "tone_preference": "concise" }), "no origin"),
        (
            json!({ ORIGIN_PROPERTY: origin() }),
            "origin without a fact",
        ),
        (two_facts, "two facts"),
        (
            fact("favourite_snack", &json!("crisps")),
            "unknown property",
        ),
    ];

    for (payload, case) in misfits {
        assert!(!fits(PREFERENCE_TYPE, &payload), "{case}: {payload}");
    }
    assert!(fits(
        PREFERENCE_TYPE,
        &fact("tone_preference", &json!("concise"))
    ));
}

fn record() -> Value {
    json!({ "connector": "chat_engine", "provenance": "chat_engine/thread-42/msg-7", "version": "v1" })
}

#[test]
fn a_fact_from_a_record_names_the_record_beside_its_origin() {
    let from_record = json!({
        "tone_preference": "concise",
        ORIGIN_PROPERTY: origin(),
        RECORD_PROPERTY: record(),
    });
    let two_facts_and_no_record = json!({
        "tone_preference": "concise",
        "units": "si_metric",
        ORIGIN_PROPERTY: origin(),
    });
    let record_without_a_fact = json!({ ORIGIN_PROPERTY: origin(), RECORD_PROPERTY: record() });
    let record_without_its_version = json!({
        "tone_preference": "concise",
        ORIGIN_PROPERTY: origin(),
        RECORD_PROPERTY: { "connector": "chat_engine", "provenance": "chat_engine/thread-42/msg-7" },
    });

    assert!(fits(PREFERENCE_TYPE, &from_record));
    for (payload, case) in [
        (two_facts_and_no_record, "two facts"),
        (record_without_a_fact, "a record without a fact"),
        (record_without_its_version, "a record without its version"),
    ] {
        assert!(!fits(PREFERENCE_TYPE, &payload), "{case}: {payload}");
    }
}

#[test]
fn each_constrained_value_fits_its_form_and_nothing_else() {
    let cases = [
        (
            "google_scholar_id",
            json!("JicYPdAAAAAJ"),
            json!("Jane Doe"),
        ),
        (
            "openalex_author_id",
            json!("A5012345678"),
            json!("5012345678"),
        ),
        (
            "orcid_id",
            json!("0000-0002-1825-0097"),
            json!("0000-0002-1825"),
        ),
        (
            "public_profiles",
            json!("https://example.org/jane"),
            json!("example.org/jane"),
        ),
        (
            "scopus_author_id",
            json!("57190000000"),
            json!("A57190000000"),
        ),
        ("timezone", json!("Europe/Berlin"), json!("Berlin time")),
        ("answer_language", json!("en-GB"), json!("English")),
        ("ui_language", json!("de"), json!("German")),
        ("dialect", json!("bg-BG"), json!("bulgarian")),
        ("languages", json!("es"), json!("Spanish")),
        ("school_grade", json!("grade_9"), json!("Grade 9")),
        ("average_session_length", json!(45), json!(0)),
        ("captions", json!(true), json!("yes")),
        ("challenge_starting_level", json!("warm_up"), json!("hard")),
        ("chunk_size", json!("short_5_10m"), json!("short")),
        ("citation_style", json!("apa"), json!("AMA")),
        ("collaboration_style", json!("reviewer"), json!("leader")),
        ("communication_channel", json!("chat"), json!("email")),
        ("explainability", json!("on_request"), json!("sometimes")),
        ("feedback_style", json!("socratic"), json!("harsh")),
        ("group_vs_individual", json!("group"), json!("both")),
        ("hint_depth", json!("light"), json!("none")),
        ("interface_layout", json!("compact"), json!("dense")),
        ("jargon_tolerance", json!("avoid_jargon"), json!("high")),
        (
            "local_environment_constraint",
            json!("ipad_only"),
            json!("android_only"),
        ),
        ("math_notation", json!("dy_dx"), json!("leibniz")),
        (
            "notification_type",
            json!("weekly_recap"),
            json!("daily_recap"),
        ),
        ("preferred_format", json!("bullets"), json!("podcasts")),
        ("presentation_order", json!("theory_first"), json!("random")),
        ("programming_language_target", json!("python"), json!("r")),
        ("reading_level", json!("advanced"), json!("expert")),
        (
            "technology_constraint",
            json!("mobile_only"),
            json!("tablet_only"),
        ),
        ("tone_preference", json!("concise"), json!("professional")),
        ("units", json!("imperial"), json!("metric")),
        ("education_level", json!("higher_ed"), json!("university")),
        ("first_publication_year", json!(1800), json!(1799)),
        ("latest_publication_year", json!(2024), json!(1799)),
        ("h_index", json!(0), json!(-1)),
        ("i10_index", json!(12), json!(-1)),
        ("total_citations", json!(0), json!(-1)),
        ("total_publications", json!(3), json!(-1)),
        ("mean_citedness_2y", json!(1.5), json!(-0.1)),
        (
            "focus_windows",
            json!({ "day": "mon", "start": "09:00", "end": "11:30" }),
            json!({ "days": ["mon", "tue"], "start": "09:00", "end": "11:30" }),
        ),
        (
            "study_window",
            json!({ "day": "sat", "start": "18:00", "end": "20:00" }),
            json!({ "day": "sat", "start": "6pm", "end": "20:00" }),
        ),
        (
            "time_availability_pattern",
            json!({ "day": "wed", "start": "07:00", "end": "08:00" }),
            json!({ "day": "someday", "start": "07:00", "end": "08:00" }),
        ),
        (
            "work_history",
            json!({ "organization": "Acme", "start_year": 1800 }),
            json!({ "organization": "Acme", "start_year": 1799 }),
        ),
        (
            "affiliation_history",
            json!({ "organization": "ETH Zurich", "country": "CH" }),
            json!({ "organization": "ETH Zurich", "country": "Switzerland" }),
        ),
        (
            "skills",
            json!({ "name": "Python", "level": "expert", "years": 6 }),
            json!({ "name": "Python", "level": "guru" }),
        ),
        (
            "publications",
            json!({ "title": "Attention Is All You Need", "doi": "10.48550/arXiv.1706.03762" }),
            json!({ "title": "Attention Is All You Need", "doi": "arXiv.1706.03762" }),
        ),
        (
            "research_areas",
            json!({ "name": "Computational biology", "weight": 0.7 }),
            json!({ "name": "Computational biology", "weight": 1.5 }),
        ),
        (
            "awards",
            json!({ "name": "Best paper", "year": 2021 }),
            json!({ "year": 2021 }),
        ),
    ];

    for (property, valid, invalid) in cases {
        assert!(value_fits(property, &valid), "{property}: {valid}");
        assert!(!value_fits(property, &invalid), "{property}: {invalid}");
    }
}

fn strings<'a>(name: &'a str, schema: &'a Value, found: &mut Vec<(&'a str, &'a Value)>) {
    match schema["type"].as_str() {
        Some("string") => found.push((name, schema)),
        Some("object") => {
            for (field, field_schema) in schema["properties"].as_object().into_iter().flatten() {
                strings(field, field_schema, found);
            }
        }
        _ => {}
    }
}

#[test]
fn every_text_value_has_a_length_limit() {
    for (type_id, property, _) in PROPERTIES {
        let payload = payload_schema(type_id);
        let mut found = Vec::new();
        strings(property, &payload["properties"][property], &mut found);
        for (name, string) in found {
            assert!(
                string.get("enum").is_some() || string["maxLength"].as_u64().is_some(),
                "{property}.{name} has no maxLength"
            );
        }
    }
}

#[test]
fn free_text_refuses_blank_text_and_text_over_its_limit() {
    for (type_id, property, _) in PROPERTIES {
        let schema = payload_schema(type_id)["properties"][property].clone();
        if schema["type"] != "string"
            || schema.get("enum").is_some()
            || schema.get("format").is_some()
        {
            continue;
        }
        let Some(limit) = schema["maxLength"]
            .as_u64()
            .and_then(|limit| usize::try_from(limit).ok())
        else {
            continue;
        };
        if schema["pattern"] == "\\S" {
            assert!(
                !value_fits(property, &json!("   ")),
                "{property} takes blank text"
            );
            assert!(
                value_fits(property, &json!("x".repeat(limit))),
                "{property} refuses text at its limit"
            );
        }
        assert!(
            !value_fits(property, &json!("1".repeat(limit + 1))),
            "{property} takes text over its limit"
        );
    }
}

#[test]
fn the_four_types_are_identity_role_skill_and_preference() {
    let ids: Vec<&str> = PERSON_TYPES.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        ids,
        vec![IDENTITY_TYPE, ROLE_TYPE, SKILL_TYPE, PREFERENCE_TYPE]
    );
}
