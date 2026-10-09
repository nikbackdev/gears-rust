use std::num::{NonZeroU32, NonZeroU64};

use serde::Deserialize;
use uuid::Uuid;

use crate::domain::planner::agent::PlannerCaps;

/// Default for [`ConstructConfig::personalization_default`]: on, so a host
/// without the settings service takes records for new subjects. A deployment
/// that wants new subjects to start with personalization off sets it to false.
pub const DEFAULT_PERSONALIZATION: bool = true;

/// Default for [`PlannerConfig::max_rounds`].
pub const DEFAULT_PLANNER_MAX_ROUNDS: NonZeroU32 = match NonZeroU32::new(10) {
    Some(rounds) => rounds,
    None => NonZeroU32::MIN,
};

/// Default for [`PlannerConfig::max_tokens`].
pub const DEFAULT_PLANNER_MAX_TOKENS: NonZeroU64 = match NonZeroU64::new(50_000) {
    Some(tokens) => tokens,
    None => NonZeroU64::MIN,
};

/// The planner's caps for one record. Zero is refused, so the gear does not start with a cap that drops every record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PlannerConfig {
    /// The most rounds of model calls.
    pub max_rounds: NonZeroU32,
    /// The most tokens, input and output, summed over all rounds.
    pub max_tokens: NonZeroU64,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_rounds: DEFAULT_PLANNER_MAX_ROUNDS,
            max_tokens: DEFAULT_PLANNER_MAX_TOKENS,
        }
    }
}

impl From<PlannerConfig> for PlannerCaps {
    fn from(config: PlannerConfig) -> Self {
        Self {
            max_rounds: config.max_rounds,
            max_tokens: config.max_tokens,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructConfig {
    /// Whether personalization is on for a new subject, used only while the
    /// settings service gives no tenant default: no settings client in the
    /// host, or the setting not declared there.
    #[serde(default = "default_personalization")]
    pub personalization_default: bool,
    /// Connectors that are off, by connector identity: the subject id of the
    /// connector's login, as a UUID. Every other connector is on. A value that
    /// is not a UUID stops the gear from starting, so a typo cannot leave a
    /// connector on.
    #[serde(default)]
    pub connectors_off: Vec<Uuid>,
    /// The planner's caps.
    #[serde(default)]
    pub planner: PlannerConfig,
}

impl Default for ConstructConfig {
    fn default() -> Self {
        Self {
            personalization_default: DEFAULT_PERSONALIZATION,
            connectors_off: Vec::new(),
            planner: PlannerConfig::default(),
        }
    }
}

fn default_personalization() -> bool {
    DEFAULT_PERSONALIZATION
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::value::{Error, MapDeserializer};

    fn parse(json: serde_json::Value) -> Result<ConstructConfig, serde_json::Error> {
        serde_json::from_value(json)
    }

    #[test]
    fn an_empty_config_takes_the_defaults() {
        let config = ConstructConfig::deserialize(MapDeserializer::<_, Error>::new(
            std::iter::empty::<(&str, bool)>(),
        ))
        .expect("an empty config is valid");
        assert!(config.personalization_default);
        assert!(config.connectors_off.is_empty());
    }

    #[test]
    fn both_keys_are_read() {
        let config = parse(serde_json::json!({
            "personalization_default": false,
            "connectors_off": ["8b0b4c4e-0000-4000-8000-000000000001"],
        }))
        .expect("known keys");
        assert!(!config.personalization_default);
        assert_eq!(config.connectors_off.len(), 1);
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(
            parse(serde_json::json!({ "personalisation_default": false })).is_err(),
            "typo must not be ignored"
        );
        assert!(
            parse(serde_json::json!({ "max_text_length": 10 })).is_err(),
            "the foundation's key is gone"
        );
    }

    #[test]
    fn the_planner_caps_are_read() {
        let config = parse(serde_json::json!({ "planner": { "max_rounds": 4, "max_tokens": 12_000 } }))
            .expect("caps");
        assert_eq!(
            (config.planner.max_rounds.get(), config.planner.max_tokens.get()),
            (4, 12_000)
        );
    }

    #[test]
    fn a_cap_left_out_takes_its_default() {
        let config = parse(serde_json::json!({ "planner": { "max_tokens": 12_000 } })).expect("caps");
        assert_eq!(config.planner.max_rounds.get(), 10);
        let config = parse(serde_json::json!({ "planner": { "max_rounds": 4 } })).expect("caps");
        assert_eq!(config.planner.max_tokens.get(), 50_000);
    }

    #[test]
    fn a_zero_cap_or_a_typo_stops_the_gear() {
        for planner in [
            serde_json::json!({ "max_rounds": 0 }),
            serde_json::json!({ "max_tokens": 0 }),
            serde_json::json!({ "max_round": 4 }),
        ] {
            assert!(parse(serde_json::json!({ "planner": planner })).is_err(), "{planner}");
        }
    }

    #[test]
    fn a_connector_that_is_not_a_uuid_is_rejected() {
        assert!(parse(serde_json::json!({ "connectors_off": ["  "] })).is_err());
        assert!(parse(serde_json::json!({ "connectors_off": ["connector-a"] })).is_err());
    }

    #[test]
    fn a_connector_in_another_spelling_is_the_same_connector() {
        let id = Uuid::new_v4();
        let upper = id.to_string().to_uppercase();
        let config = parse(serde_json::json!({ "connectors_off": [upper] })).expect("a UUID");
        assert_eq!(config.connectors_off, vec![id]);
    }
}
