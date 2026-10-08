use construct_sdk::gts::RECORD_TYPES;
use construct_sdk::person_types::Cardinality;
use serde_json::Value;

use super::catalog::PropertyCatalog;
use crate::domain::profile::Category;
use crate::domain::record_intake::ReceivedRecord;

const INSTRUCTIONS: &str = include_str!("instructions.md");
const PROPERTIES_PLACEHOLDER: &str = "{properties}";
const CATEGORIES: [Category; 4] = [
    Category::Identity,
    Category::Roles,
    Category::Skills,
    Category::Preferences,
];

/// @cpt-dod:cpt-cf-construct-dod-planner-prompt:p1
#[must_use]
pub fn instructions(catalog: &PropertyCatalog) -> String {
    let properties = CATEGORIES
        .iter()
        .flat_map(|category| {
            catalog.names().filter_map(move |property| {
                let spec = catalog.get(property)?;
                (spec.category == *category).then(|| {
                    let cardinality = match spec.cardinality {
                        Cardinality::One => "one",
                        Cardinality::Many => "many",
                    };
                    format!(
                        "{}: {property} ({cardinality}): {}",
                        category.name(),
                        form(spec.schema())
                    )
                })
            })
        })
        .collect::<Vec<_>>()
        .join("\n");
    INSTRUCTIONS.replace(PROPERTIES_PLACEHOLDER, &properties)
}

#[must_use]
pub fn record_input(record: &ReceivedRecord, numbered_profile: &str) -> String {
    let schema = RECORD_TYPES
        .iter()
        .find(|record_type| record_type.type_id == record.envelope.type_id)
        .and_then(|record_type| serde_json::from_str::<Value>(record_type.schema).ok());
    let kind = schema
        .as_ref()
        .and_then(|schema| schema.get("description"))
        .and_then(Value::as_str)
        .and_then(|description| description.split(". ").next())
        .unwrap_or("A record");
    let payload_schema = schema
        .as_ref()
        .and_then(|schema| schema.pointer("/allOf/1/properties/payload"))
        .map_or_else(
            || "(none)".to_owned(),
            |schema| schema_without_ids(schema).to_string(),
        );
    let payload = record.record.get("payload").map_or_else(
        || "(none)".to_owned(),
        |payload| without_ids(payload).to_string(),
    );
    let profile = if numbered_profile.is_empty() {
        "(empty)"
    } else {
        numbered_profile
    };
    format!(
        "Record: {kind}.\nPayload schema: {payload_schema}\nPayload: {payload}\n\nProfile:\n{profile}"
    )
}

fn is_id(name: &str) -> bool {
    name == "id" || name.ends_with("_id")
}

fn without_ids(value: &Value) -> Value {
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .filter(|(name, _)| !is_id(name))
                .map(|(name, field)| (name.clone(), without_ids(field)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(without_ids).collect()),
        other => other.clone(),
    }
}

fn schema_without_ids(schema: &Value) -> Value {
    let Value::Object(keywords) = schema else {
        return schema.clone();
    };
    Value::Object(
        keywords
            .iter()
            .map(|(keyword, value)| {
                let value = match (keyword.as_str(), value) {
                    ("properties", Value::Object(properties)) => Value::Object(
                        properties
                            .iter()
                            .filter(|(name, _)| !is_id(name))
                            .map(|(name, property)| (name.clone(), schema_without_ids(property)))
                            .collect(),
                    ),
                    ("required", Value::Array(names)) => Value::Array(
                        names
                            .iter()
                            .filter(|name| name.as_str().is_none_or(|name| !is_id(name)))
                            .cloned()
                            .collect(),
                    ),
                    (_, value) => schema_without_ids(value),
                };
                (keyword.clone(), value)
            })
            .collect(),
    )
}

fn form(schema: &Value) -> String {
    let base = if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        values
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" | ")
    } else {
        match schema.get("type").and_then(Value::as_str) {
            Some("object") => object_form(schema),
            Some("array") => format!(
                "a list of {}",
                schema
                    .get("items")
                    .map_or_else(|| "values".to_owned(), form)
            ),
            Some("integer") => "integer".to_owned(),
            Some("number") => "number".to_owned(),
            Some("boolean") => "true or false".to_owned(),
            _ if schema.get("format").and_then(Value::as_str) == Some("uri") => "URL".to_owned(),
            _ => match schema.get("pattern").and_then(Value::as_str) {
                Some(pattern) if schema.get("description").is_none() => {
                    format!("text matching {pattern}")
                }
                _ => "text".to_owned(),
            },
        }
    };
    match schema.get("description").and_then(Value::as_str) {
        Some(description) => format!("{base} ({description})"),
        None => base,
    }
}

fn object_form(schema: &Value) -> String {
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let fields = schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(name, field)| {
                    let marker = if required.contains(&name.as_str()) {
                        " (required)"
                    } else {
                        ""
                    };
                    format!("{name}{marker}: {}", form(field))
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    format!("{{{fields}}}")
}
