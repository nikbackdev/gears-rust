use construct_sdk::person_types::PROPERTIES;
use serde_json::json;

use super::test_data::catalog;

#[test]
fn every_property_of_the_person_types_has_its_category_and_cardinality() {
    let catalog = catalog();

    for (type_id, property, cardinality) in PROPERTIES {
        let spec = catalog.get(property).expect("a listed property");
        assert_eq!(spec.category.type_id(), *type_id, "{property}");
        assert_eq!(spec.cardinality, *cardinality, "{property}");
    }
    assert_eq!(catalog.names().count(), PROPERTIES.len());
}

#[test]
fn each_property_checks_a_value_against_its_own_schema() {
    let catalog = catalog();

    for property in catalog.names() {
        let spec = catalog.get(property).expect("a listed property");
        let refusal = spec
            .check(&json!({ "no_such_field": true }))
            .expect_err("an object with an unknown field fits no property");
        assert!(
            refusal.ends_with(&format!("it takes {}", spec.schema())),
            "{property}: {refusal}"
        );
    }
}

#[test]
fn an_orcid_must_have_the_orcid_form() {
    let catalog = catalog();
    let orcid = catalog.get("orcid_id").expect("orcid_id");

    assert_eq!(orcid.check(&json!("0000-0002-1825-0097")), Ok(()));
    assert!(orcid.check(&json!(42)).is_err());
    assert!(orcid.check(&json!("orcid of Jane")).is_err());
}
