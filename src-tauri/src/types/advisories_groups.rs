use serde::{Deserialize, Serialize};

/// Conditions applied to a VRChat Group for an advisory. This way, a set of conditions can be applied to the same group, rather than to any group the user is in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "type", content = "data")] // Best for TypeScript discriminated unions
#[ts(export)]
pub enum AdvisoryGroupCondition {
    /// The group has the given ID (`grp_***`).
    /// You should use `IsGroupMember` instead if you're checking for membership in a known group (it's far simpler).
    /// This is instead for cases where you want to match some conditions but exclude certain known groups.
    Id(String),
    /// The group's name contains the given substring (case-insensitive).
    NameContains(String),
    /// The user is with the given ID (`usr_***`) is the owner of the group.
    /// This is useful for applying advisories to groups owned by specific users, such as known abusers or trusted community members.
    /// For instance, you can set up an advisory to look out for potential group-staff impersonation, and ignore any groups that are owned by actual staff members.
    OwnerIs(String),

    // == Meta-conditions ==
    /// The condition _does not_ match.
    /// Combine as `Not(AnyOf(...))` to form "NoneOf". Otherwise, it's best to use it with `AllOf(vec![Not(...)])`.
    /// Can be useful to:
    /// - Exclude group members to apply an advisory only to non-members.
    /// - Exclude benign username patterns from a broader username match.
    /// - Exclude age-verified users from new account advisories.
    /// - Apply advisories to only public instances by excluding group-restricted instances.
    Not {
        /* IMPLEMENTATION NOTE:
           This condition has to be a struct member (with a "condition" field) rather than a tuple member
           (i.e., Not(Box<AdvisoryGroupCondition>)) because otherwise, Rust's recursion limit is exceeded.
           I'd prefer the simpler pattern, but it doesn't work in this case.
        */
        data: Box<AdvisoryGroupCondition>,
    },

    /// Any of the conditions are true.
    AnyOf(Vec<AdvisoryGroupCondition>),
    /// All of the conditions are true.
    AllOf(Vec<AdvisoryGroupCondition>),
    /// No conditions. Does not pass validation, so you cannot set it. Used in the UI to represent
    /// an unset condition.
    None,
}

impl AdvisoryGroupCondition {
    /// Evaluate this advisory condition using the given evaluator function.
    /// The evaluator function should return true if the condition is met, false otherwise.
    /// This function will recursively evaluate any meta-conditions.
    pub fn evaluate<F>(&self, evaluator: &F) -> bool
    where
        F: Fn(AdvisoryGroupCondition) -> bool,
    {
        match self {
            AdvisoryGroupCondition::Not { data } => {
                if let AdvisoryGroupCondition::Not { data: e } = &**data {
                    // double negation
                    if let AdvisoryGroupCondition::Not { data: _ } = &**e {
                        // triple negation!
                        eprintln!(
                            "Recursion in advisory condition is not allowed. Returning false"
                        );
                        return false;
                    }
                    return e.evaluate(evaluator);
                }
                !data.evaluate(evaluator)
            }
            AdvisoryGroupCondition::AnyOf(conditions) => {
                for condition in conditions {
                    if condition.evaluate(evaluator) {
                        return true;
                    }
                }
                false
            }
            AdvisoryGroupCondition::AllOf(conditions) => {
                for condition in conditions {
                    if !condition.evaluate(evaluator) {
                        return false;
                    }
                }
                true
            }
            AdvisoryGroupCondition::None => panic!("AdvisoryGroupCondition::None should never be evaluated"),
            _ => evaluator(self.clone()),
        }
    }
}