use crate::entity;
use crate::entity::plan_excess_allocation_rules::AllocationType;
use crate::entity::plan_liability_rules::LiabilityRuleType;
use crate::entity::types::{AssetAccountType, RuleRequirement};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct CreatePlanInput {
    pub name: String,
    pub asset_balance_rules: Vec<AssetBalanceRuleInput>,
    pub liability_rules: Vec<LiabilityRuleInput>,
    pub allocation_rules: Vec<AllocationRuleInput>,
}

#[derive(Debug, Clone)]
pub struct AssetBalanceRuleInput {
    pub asset_account_id: String,
    pub minimum_balance_cents: i64,
}

#[derive(Debug, Clone)]
pub enum LiabilityRuleInput {
    TargetBalance {
        name: String,
        target_balance_cents: i64,
        requirement: RuleRequirement,
        position: i32,
        liability_account_id: String,
        payment_asset_account_id: String,
    },
    FixedPayment {
        name: String,
        payment_cents: i64,
        requirement: RuleRequirement,
        position: i32,
        liability_account_id: String,
        payment_asset_account_id: String,
    },
}

impl LiabilityRuleInput {
    pub fn persisted_values(
        &self,
    ) -> (
        &str,
        LiabilityRuleType,
        i64,
        RuleRequirement,
        i32,
        &str,
        &str,
    ) {
        match self {
            Self::TargetBalance {
                name,
                target_balance_cents,
                requirement,
                position,
                liability_account_id,
                payment_asset_account_id,
            } => (
                name,
                LiabilityRuleType::TargetBalance,
                *target_balance_cents,
                *requirement,
                *position,
                liability_account_id,
                payment_asset_account_id,
            ),
            Self::FixedPayment {
                name,
                payment_cents,
                requirement,
                position,
                liability_account_id,
                payment_asset_account_id,
            } => (
                name,
                LiabilityRuleType::FixedPayment,
                *payment_cents,
                *requirement,
                *position,
                liability_account_id,
                payment_asset_account_id,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AllocationRuleInput {
    FixedAmount {
        amount_cents: i64,
        position: i32,
        source_asset_account_id: String,
        destination_asset_account_id: String,
    },
    Percentage {
        percentage_bps: i64,
        position: i32,
        source_asset_account_id: String,
        destination_asset_account_id: String,
    },
    Remainder {
        position: i32,
        source_asset_account_id: String,
        destination_asset_account_id: String,
    },
}

impl AllocationRuleInput {
    pub fn persisted_values(&self) -> (AllocationType, Option<i64>, i32, &str, &str) {
        match self {
            Self::FixedAmount {
                amount_cents,
                position,
                source_asset_account_id,
                destination_asset_account_id,
            } => (
                AllocationType::FixedAmount,
                Some(*amount_cents),
                *position,
                source_asset_account_id,
                destination_asset_account_id,
            ),
            Self::Percentage {
                percentage_bps,
                position,
                source_asset_account_id,
                destination_asset_account_id,
            } => (
                AllocationType::Percentage,
                Some(*percentage_bps),
                *position,
                source_asset_account_id,
                destination_asset_account_id,
            ),
            Self::Remainder {
                position,
                source_asset_account_id,
                destination_asset_account_id,
            } => (
                AllocationType::Remainder,
                None,
                *position,
                source_asset_account_id,
                destination_asset_account_id,
            ),
        }
    }
}

pub fn validate(
    input: &CreatePlanInput,
    assets: &[entity::asset_accounts::Model],
    liabilities: &[entity::liability_accounts::Model],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if input.name.trim().is_empty() {
        errors.push("plan name cannot be blank".to_string());
    }

    let assets_by_id: HashMap<_, _> = assets
        .iter()
        .map(|account| (account.account_id.as_str(), account))
        .collect();
    let liability_ids: HashSet<_> = liabilities
        .iter()
        .map(|account| account.account_id.as_str())
        .collect();
    let mut balance_accounts = HashSet::new();
    for rule in &input.asset_balance_rules {
        if rule.minimum_balance_cents < 0 {
            errors.push(format!(
                "minimum for {} cannot be negative",
                rule.asset_account_id
            ));
        }
        if !assets_by_id.contains_key(rule.asset_account_id.as_str()) {
            errors.push(format!("unknown asset account {}", rule.asset_account_id));
        }
        if !balance_accounts.insert(rule.asset_account_id.as_str()) {
            errors.push(format!(
                "duplicate balance rule for {}",
                rule.asset_account_id
            ));
        }
    }

    let mut liability_positions = HashSet::new();
    let mut ruled_liabilities = HashSet::new();
    for rule in &input.liability_rules {
        let (_, _, value, _, position, liability_id, source_id) = rule.persisted_values();
        if value < 0 {
            errors.push(format!(
                "liability rule at position {position} cannot be negative"
            ));
        }
        if position < 0 || !liability_positions.insert(position) {
            errors.push(format!(
                "invalid or duplicate liability position {position}"
            ));
        }
        if !liability_ids.contains(liability_id) {
            errors.push(format!("unknown liability account {liability_id}"));
        }
        if !ruled_liabilities.insert(liability_id) {
            errors.push(format!("duplicate liability rule for {liability_id}"));
        }
        validate_source(source_id, &assets_by_id, &mut errors);
    }

    let mut allocation_positions = HashSet::new();
    let mut remainders = HashSet::new();
    let mut percentage_totals: HashMap<&str, i64> = HashMap::new();
    let mut destinations = HashSet::new();
    for rule in &input.allocation_rules {
        let (kind, value, position, source_id, destination_id) = rule.persisted_values();
        if position < 0 || !allocation_positions.insert(position) {
            errors.push(format!(
                "invalid or duplicate allocation position {position}"
            ));
        }
        validate_source(source_id, &assets_by_id, &mut errors);
        if !assets_by_id.contains_key(destination_id) {
            errors.push(format!("unknown allocation destination {destination_id}"));
        }
        if source_id == destination_id {
            errors.push(format!(
                "allocation source and destination are both {source_id}"
            ));
        }
        if !destinations.insert((kind.clone(), source_id, destination_id)) {
            errors.push(format!(
                "duplicate {kind:?} allocation from {source_id} to {destination_id}"
            ));
        }
        match kind {
            AllocationType::FixedAmount if value.unwrap_or(-1) < 0 => errors.push(format!(
                "fixed allocation at position {position} cannot be negative"
            )),
            AllocationType::Percentage => {
                let bps = value.unwrap_or(-1);
                if !(0..=10_000).contains(&bps) {
                    errors.push(format!(
                        "percentage at position {position} must be between 0 and 10000 bps"
                    ));
                }
                *percentage_totals.entry(source_id).or_default() += bps.max(0);
            }
            AllocationType::Remainder if !remainders.insert(source_id) => errors.push(format!(
                "multiple remainder allocations for source {source_id}"
            )),
            _ => {}
        }
    }
    for (source, total) in percentage_totals {
        if total > 10_000 {
            errors.push(format!(
                "percentage allocations for {source} total {total} bps"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_source(
    source_id: &str,
    assets: &HashMap<&str, &entity::asset_accounts::Model>,
    errors: &mut Vec<String>,
) {
    match assets.get(source_id) {
        Some(account) if account.account_type != AssetAccountType::Depository => {
            errors.push(format!("funding source {source_id} is not depository"))
        }
        None => errors.push(format!("unknown funding source {source_id}")),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::entity::prelude::DateTime;

    fn asset(id: &str, account_type: AssetAccountType) -> entity::asset_accounts::Model {
        entity::asset_accounts::Model {
            account_id: id.into(),
            created_at: DateTime::default(),
            name: id.into(),
            account_type,
            account_subtype: "checking".into(),
            plaid_item_id: 1,
        }
    }

    #[test]
    fn rejects_non_depository_sources_and_overallocated_percentages() {
        let input = CreatePlanInput {
            name: "Plan".into(),
            asset_balance_rules: vec![],
            liability_rules: vec![],
            allocation_rules: vec![
                AllocationRuleInput::Percentage {
                    percentage_bps: 6_000,
                    position: 0,
                    source_asset_account_id: "brokerage".into(),
                    destination_asset_account_id: "savings".into(),
                },
                AllocationRuleInput::Percentage {
                    percentage_bps: 5_000,
                    position: 1,
                    source_asset_account_id: "brokerage".into(),
                    destination_asset_account_id: "checking".into(),
                },
            ],
        };
        let errors = validate(
            &input,
            &[
                asset("brokerage", AssetAccountType::Brokerage),
                asset("savings", AssetAccountType::Depository),
                asset("checking", AssetAccountType::Depository),
            ],
            &[],
        )
        .unwrap_err();
        assert!(errors.iter().any(|error| error.contains("not depository")));
        assert!(errors.iter().any(|error| error.contains("total 11000")));
    }
}
