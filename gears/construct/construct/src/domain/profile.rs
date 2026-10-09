use construct_sdk::gts::{IDENTITY_TYPE, PREFERENCE_TYPE, ROLE_TYPE, SKILL_TYPE};
use serde_json::Value;
use toolkit_macros::domain_model;

/// A category of a person's profile: one person type in graph storage.
#[domain_model]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Identity,
    Role,
    Skill,
    Preference,
}

impl Category {
    /// The category whose person type is `type_id`, or `None` for any other type.
    #[must_use]
    pub fn from_type_id(type_id: &str) -> Option<Self> {
        [Self::Identity, Self::Role, Self::Skill, Self::Preference]
            .into_iter()
            .find(|category| category.type_id() == type_id)
    }

    /// The GTS id of the category's person type.
    #[must_use]
    pub fn type_id(self) -> &'static str {
        match self {
            Self::Identity => IDENTITY_TYPE,
            Self::Role => ROLE_TYPE,
            Self::Skill => SKILL_TYPE,
            Self::Preference => PREFERENCE_TYPE,
        }
    }

    /// The plain name the model sees.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Role => "role",
            Self::Skill => "skill",
            Self::Preference => "preference",
        }
    }
}

/// The version of a profile: the version of its root node, carried as an opaque value.
#[domain_model]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileVersion(pub i64);

/// One stored fact: its node key, its category, its property and its value.
#[domain_model]
#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub node_key: String,
    pub category: Category,
    pub property: String,
    pub value: Value,
}

/// A subject's profile as the planner reads it: its version and its facts.
#[domain_model]
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub version: ProfileVersion,
    pub facts: Vec<Fact>,
}
