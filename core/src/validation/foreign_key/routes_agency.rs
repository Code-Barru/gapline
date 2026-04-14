//! FK rule: `routes.agency_id` → `agency.agency_id`.

use std::collections::HashSet;

use crate::models::GtfsFeed;
use crate::validation::{Severity, ValidationError, ValidationRule};

const FILE: &str = "routes.txt";
use super::{RULE_ID, SECTION};

/// Every `agency_id` in routes.txt must exist in agency.txt.
///
/// Special case: when agency.txt contains exactly one agency and
/// `agency_id` is empty in routes.txt, the agency is implicit and
/// no violation is reported.
pub struct RoutesAgencyFkRule;

impl ValidationRule for RoutesAgencyFkRule {
    fn rule_id(&self) -> &'static str {
        RULE_ID
    }

    fn section(&self) -> &'static str {
        SECTION
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, feed: &GtfsFeed) -> Vec<ValidationError> {
        let single_agency = feed.agencies.len() == 1;

        let valid_ids: HashSet<&str> = feed
            .agencies
            .iter()
            .filter_map(|a| a.agency_id.as_ref().map(AsRef::as_ref))
            .collect();

        let mut errors = Vec::new();

        for (i, route) in feed.routes.iter().enumerate() {
            let line = i + 2;

            match &route.agency_id {
                None if single_agency => {}
                None => {
                    errors.push(
                        ValidationError::new(RULE_ID, SECTION, Severity::Error)
                            .message(
                                "agency_id is empty in routes.txt but agency.txt contains multiple agencies"
                                    .to_string(),
                            )
                            .file(FILE)
                            .line(line)
                            .field("agency_id"),
                    );
                }
                Some(aid) if !valid_ids.contains(aid.as_ref()) => {
                    errors.push(
                        ValidationError::new(RULE_ID, SECTION, Severity::Error)
                            .message(format!(
                                "agency_id '{aid}' in routes.txt line {line} references non-existent agency in agency.txt"
                            ))
                            .file(FILE)
                            .line(line)
                            .field("agency_id")
                            .value(aid.as_ref()),
                    );
                }
                _ => {}
            }
        }

        errors
    }
}
