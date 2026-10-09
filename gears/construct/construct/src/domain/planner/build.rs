use uuid::Uuid;

use super::draft::{DraftStep, PlanDraft};
use crate::domain::plan::{NewFact, Plan, PlanSubject, RecordOrigin, Step};

impl PlanDraft<'_> {
    /// The plan the draft's steps make. Each new value gets a new node key in the subject's tenant, also a value
    /// that returns after a delete, because a deleted key cannot be reused.
    ///
    /// @cpt-dod:cpt-cf-construct-dod-planner-plan:p1
    #[must_use]
    pub fn finish(self, subject: PlanSubject, origin: RecordOrigin) -> Plan {
        let new_key = || format!("construct:{}:{}", subject.tenant_id, Uuid::new_v4());
        let steps = self
            .steps
            .into_iter()
            .map(|step| match step {
                DraftStep::Add {
                    category,
                    property,
                    value,
                    confidence,
                } => Step::Add {
                    node_key: new_key(),
                    fact: NewFact {
                        category,
                        property,
                        value,
                    },
                    confidence,
                },
                DraftStep::Replace {
                    old,
                    value,
                    confidence,
                } => Step::Replace {
                    old_key: old.node_key,
                    node_key: new_key(),
                    fact: NewFact {
                        category: old.category,
                        property: old.property,
                        value,
                    },
                    confidence,
                },
                DraftStep::Remove { old, confidence } => Step::Remove {
                    old_key: old.node_key,
                    confidence,
                },
            })
            .collect();
        Plan {
            subject,
            origin,
            profile_version: self.version,
            steps,
        }
    }
}
