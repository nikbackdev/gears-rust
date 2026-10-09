use std::collections::BTreeMap;

use construct_sdk::person_types::{Cardinality, PERSON_TYPES, PROPERTIES};
use jsonschema::{Draft, Validator};
use serde_json::Value;
use toolkit_macros::domain_model;

use crate::domain::error::DomainError;
use crate::domain::profile::Category;

/// One property of the person types: its category, its cardinality and its schema.
#[domain_model]
#[derive(Debug)]
pub struct PropertySpec {
    pub category: Category,
    pub cardinality: Cardinality,
    schema: Value,
    validator: Validator,
}

impl PropertySpec {
    /// The property's JSON schema in its person type.
    #[must_use]
    pub fn schema(&self) -> &Value {
        &self.schema
    }

    /// Checks a value against the property's schema.
    ///
    /// # Errors
    ///
    /// The first rule the value breaks, followed by the schema the property takes.
    pub fn check(&self, value: &Value) -> Result<(), String> {
        match self.validator.iter_errors(value).next() {
            None => Ok(()),
            Some(error) => Err(format!("{error}; it takes {}", self.schema)),
        }
    }
}

/// Every property of the person types, by name.
#[domain_model]
#[derive(Debug)]
pub struct PropertyCatalog {
    properties: BTreeMap<&'static str, PropertySpec>,
}

impl PropertyCatalog {
    /// Reads every property from the person types the SDK ships.
    ///
    /// # Errors
    ///
    /// [`DomainError::Internal`] when a person type does not parse or a property has no schema.
    pub fn load() -> Result<Self, DomainError> {
        let mut schemas = BTreeMap::new();
        for (type_id, json) in PERSON_TYPES {
            let schema: Value = serde_json::from_str(json).map_err(|error| {
                DomainError::internal(format!("person type {type_id} does not parse: {error}"))
            })?;
            schemas.insert(type_id, schema);
        }
        let mut properties = BTreeMap::new();
        for (type_id, property, cardinality) in PROPERTIES {
            let category = Category::from_type_id(type_id)
                .ok_or_else(|| DomainError::internal(format!("{type_id} is not a person type")))?;
            let schema = schemas
                .get(type_id)
                .and_then(|schema| {
                    schema.pointer(&format!(
                        "/allOf/1/properties/payload/properties/{property}"
                    ))
                })
                .ok_or_else(|| {
                    DomainError::internal(format!("{type_id} has no schema for {property}"))
                })?;
            let validator = jsonschema::options()
                .with_draft(Draft::Draft7)
                .build(schema)
                .map_err(|error| {
                    DomainError::internal(format!(
                        "the schema of {property} does not compile: {error}"
                    ))
                })?;
            properties.insert(
                *property,
                PropertySpec {
                    category,
                    cardinality: *cardinality,
                    schema: schema.clone(),
                    validator,
                },
            );
        }
        Ok(Self { properties })
    }

    /// The property named `property`, or `None` when the person types have no such property.
    #[must_use]
    pub fn get(&self, property: &str) -> Option<&PropertySpec> {
        self.properties.get(property)
    }

    /// The names of all properties, in alphabetical order.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.properties.keys().copied()
    }
}
