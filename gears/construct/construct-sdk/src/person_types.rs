//! Construct's person types: one GTS graph node type per category of a person's profile.
//!
//! Each type derives from graph storage's owned node. One node is one fact: its `payload` holds one property, which
//! names the fact, and the fact's origin. The schemas are embedded from the crate's `schemas/`, so the registration and
//! the published files cannot drift apart.

use crate::gts::{IDENTITY_TYPE, PREFERENCE_TYPE, ROLE_TYPE, SKILL_TYPE};

/// The person types with their JSON schemas, in registration order.
///
/// @cpt-dod:cpt-cf-construct-dod-profile-writer-person-types:p1
pub const PERSON_TYPES: [(&str, &str); 4] = [
    (
        IDENTITY_TYPE,
        include_str!(
            "../schemas/gts.cf.core.graph.node.v1~cf.core.graph.owned_node.v1~cf.construct.person.identity.v1~.schema.json"
        ),
    ),
    (
        ROLE_TYPE,
        include_str!(
            "../schemas/gts.cf.core.graph.node.v1~cf.core.graph.owned_node.v1~cf.construct.person.role.v1~.schema.json"
        ),
    ),
    (
        SKILL_TYPE,
        include_str!(
            "../schemas/gts.cf.core.graph.node.v1~cf.core.graph.owned_node.v1~cf.construct.person.skill.v1~.schema.json"
        ),
    ),
    (
        PREFERENCE_TYPE,
        include_str!(
            "../schemas/gts.cf.core.graph.node.v1~cf.core.graph.owned_node.v1~cf.construct.person.preference.v1~.schema.json"
        ),
    ),
];

/// How many values of one property a subject may hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cardinality {
    /// At most one value. A new value replaces the stored one.
    One,
    /// Any number of values.
    Many,
}

/// The payload member that holds the origin of a fact: graph storage's provenance attribute.
///
/// @cpt-dod:cpt-cf-construct-dod-profile-writer-fact-origin:p1
pub const ORIGIN_PROPERTY: &str = "origin";

/// The payload member that names the record a fact came from, when it came from one: the connector and the record's
/// identity and version. The provenance attribute in `origin` is closed and has no member for them.
pub const RECORD_PROPERTY: &str = "record";

/// Every fact property of every person type, with its cardinality.
///
/// @cpt-dod:cpt-cf-construct-dod-profile-writer-cardinality:p1
pub const PROPERTIES: &[(&str, &str, Cardinality)] = &[
    (IDENTITY_TYPE, "background", Cardinality::One),
    (IDENTITY_TYPE, "full_name", Cardinality::One),
    (IDENTITY_TYPE, "google_scholar_id", Cardinality::One),
    (IDENTITY_TYPE, "headline", Cardinality::One),
    (IDENTITY_TYPE, "location", Cardinality::One),
    (IDENTITY_TYPE, "name_variants", Cardinality::Many),
    (IDENTITY_TYPE, "openalex_author_id", Cardinality::One),
    (IDENTITY_TYPE, "orcid_id", Cardinality::One),
    (IDENTITY_TYPE, "public_mentions", Cardinality::Many),
    (IDENTITY_TYPE, "public_profiles", Cardinality::Many),
    (IDENTITY_TYPE, "scopus_author_id", Cardinality::One),
    (IDENTITY_TYPE, "timezone", Cardinality::One),
    (PREFERENCE_TYPE, "analogy_domain", Cardinality::One),
    (PREFERENCE_TYPE, "answer_language", Cardinality::One),
    (PREFERENCE_TYPE, "assistive_tools", Cardinality::Many),
    (PREFERENCE_TYPE, "average_session_length", Cardinality::One),
    (PREFERENCE_TYPE, "captions", Cardinality::One),
    (
        PREFERENCE_TYPE,
        "challenge_starting_level",
        Cardinality::One,
    ),
    (PREFERENCE_TYPE, "chunk_size", Cardinality::One),
    (PREFERENCE_TYPE, "citation_style", Cardinality::One),
    (PREFERENCE_TYPE, "cognitive_load_hint", Cardinality::One),
    (PREFERENCE_TYPE, "collaboration_style", Cardinality::One),
    (PREFERENCE_TYPE, "communication_channel", Cardinality::Many),
    (PREFERENCE_TYPE, "content_maturity_level", Cardinality::One),
    (PREFERENCE_TYPE, "current_focus_areas", Cardinality::Many),
    (PREFERENCE_TYPE, "dialect", Cardinality::One),
    (PREFERENCE_TYPE, "dyslexia_friendly", Cardinality::One),
    (PREFERENCE_TYPE, "explainability", Cardinality::One),
    (PREFERENCE_TYPE, "feedback_style", Cardinality::One),
    (PREFERENCE_TYPE, "focus_windows", Cardinality::Many),
    (PREFERENCE_TYPE, "group_vs_individual", Cardinality::One),
    (PREFERENCE_TYPE, "high_contrast", Cardinality::One),
    (PREFERENCE_TYPE, "hint_depth", Cardinality::One),
    (PREFERENCE_TYPE, "interface_layout", Cardinality::One),
    (PREFERENCE_TYPE, "jargon_tolerance", Cardinality::One),
    (
        PREFERENCE_TYPE,
        "local_environment_constraint",
        Cardinality::Many,
    ),
    (PREFERENCE_TYPE, "long_term_goals", Cardinality::Many),
    (PREFERENCE_TYPE, "math_notation", Cardinality::One),
    (PREFERENCE_TYPE, "modality_sensitivity", Cardinality::One),
    (PREFERENCE_TYPE, "motivation", Cardinality::One),
    (PREFERENCE_TYPE, "notification_type", Cardinality::Many),
    (PREFERENCE_TYPE, "nudge_preference", Cardinality::One),
    (PREFERENCE_TYPE, "preferred_format", Cardinality::Many),
    (PREFERENCE_TYPE, "presentation_order", Cardinality::One),
    (
        PREFERENCE_TYPE,
        "programming_language_target",
        Cardinality::One,
    ),
    (PREFERENCE_TYPE, "readiness_hint", Cardinality::One),
    (PREFERENCE_TYPE, "reading_level", Cardinality::One),
    (PREFERENCE_TYPE, "reduced_motion", Cardinality::One),
    (PREFERENCE_TYPE, "runtime_constraint", Cardinality::One),
    (PREFERENCE_TYPE, "self_efficacy", Cardinality::One),
    (PREFERENCE_TYPE, "short_term_goals", Cardinality::Many),
    (PREFERENCE_TYPE, "study_window", Cardinality::Many),
    (PREFERENCE_TYPE, "technology_constraint", Cardinality::Many),
    (
        PREFERENCE_TYPE,
        "time_availability_pattern",
        Cardinality::Many,
    ),
    (PREFERENCE_TYPE, "time_constraint", Cardinality::Many),
    (PREFERENCE_TYPE, "tone_preference", Cardinality::One),
    (PREFERENCE_TYPE, "tool_preference", Cardinality::One),
    (PREFERENCE_TYPE, "ui_language", Cardinality::One),
    (PREFERENCE_TYPE, "units", Cardinality::One),
    (ROLE_TYPE, "active_course", Cardinality::One),
    (ROLE_TYPE, "affiliation_history", Cardinality::Many),
    (ROLE_TYPE, "board_memberships", Cardinality::Many),
    (ROLE_TYPE, "current_affiliation", Cardinality::One),
    (ROLE_TYPE, "education_level", Cardinality::One),
    (ROLE_TYPE, "group_memberships", Cardinality::Many),
    (ROLE_TYPE, "instructor", Cardinality::One),
    (ROLE_TYPE, "major", Cardinality::One),
    (ROLE_TYPE, "program", Cardinality::One),
    (ROLE_TYPE, "role", Cardinality::One),
    (ROLE_TYPE, "roles_and_affiliations", Cardinality::Many),
    (ROLE_TYPE, "school_grade", Cardinality::One),
    (ROLE_TYPE, "section", Cardinality::One),
    (ROLE_TYPE, "work_history", Cardinality::Many),
    (SKILL_TYPE, "activity_by_year", Cardinality::Many),
    (SKILL_TYPE, "awards", Cardinality::Many),
    (SKILL_TYPE, "career_stage", Cardinality::One),
    (SKILL_TYPE, "first_publication_year", Cardinality::One),
    (SKILL_TYPE, "h_index", Cardinality::One),
    (SKILL_TYPE, "i10_index", Cardinality::One),
    (SKILL_TYPE, "languages", Cardinality::Many),
    (SKILL_TYPE, "latest_publication_year", Cardinality::One),
    (SKILL_TYPE, "mean_citedness_2y", Cardinality::One),
    (SKILL_TYPE, "primary_research_area", Cardinality::One),
    (SKILL_TYPE, "publications", Cardinality::Many),
    (SKILL_TYPE, "research_areas", Cardinality::Many),
    (SKILL_TYPE, "skills", Cardinality::Many),
    (SKILL_TYPE, "total_citations", Cardinality::One),
    (SKILL_TYPE, "total_publications", Cardinality::One),
];

#[cfg(test)]
#[path = "person_types_tests.rs"]
mod person_types_tests;
