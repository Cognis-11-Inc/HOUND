//! Hypotheses are separate records. No conversion to observations is provided.
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisStatus {
    Provisional,
    Supported,
    Rejected,
    Unresolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hypothesis {
    pub hypothesis_id: String,
    pub suspected_origin: Option<String>,
    pub supporting_observations: Vec<String>,
    pub contradicting_observations: Vec<String>,
    pub status: HypothesisStatus,
}

impl Hypothesis {
    pub fn validate(&self, known_observations: &HashSet<String>) -> Result<()> {
        if self.hypothesis_id.trim().is_empty()
            || self
                .suspected_origin
                .as_ref()
                .is_some_and(|s| s.trim().is_empty())
            || (self.suspected_origin.is_none() && self.status != HypothesisStatus::Unresolved)
        {
            return Err("invalid hypothesis identity or origin".into());
        }
        let mut refs = HashSet::new();
        for id in self
            .supporting_observations
            .iter()
            .chain(&self.contradicting_observations)
        {
            if !known_observations.contains(id) || !refs.insert(id) {
                return Err("hypothesis reference is unknown, duplicated, or both supporting and contradicting".into());
            }
        }
        Ok(())
    }
}
