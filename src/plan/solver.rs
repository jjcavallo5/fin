use crate::entity;
use crate::entity::plan_excess_allocation_rules::AllocationType;
use crate::entity::plan_liability_rules::LiabilityRuleType;
use crate::entity::types::RuleRequirement;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountBalances {
    pub assets: HashMap<String, i64>,
    pub liabilities: HashMap<String, i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStepKind {
    LiabilityPayment,
    AssetTransfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStep {
    pub sequence: usize,
    pub kind: PlanStepKind,
    pub source_account_id: String,
    pub destination_account_id: String,
    pub amount_cents: i64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleFailure {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolveResult {
    Executable {
        steps: Vec<PlanStep>,
        projected_balances: AccountBalances,
    },
    Infeasible {
        failures: Vec<RuleFailure>,
    },
}

pub fn solve(
    balance_rules: &[entity::asset_balance_rules::Model],
    liability_rules: &[entity::plan_liability_rules::Model],
    allocation_rules: &[entity::plan_excess_allocation_rules::Model],
    balances: &AccountBalances,
) -> SolveResult {
    let minimums: HashMap<&str, i64> = balance_rules
        .iter()
        .map(|rule| (rule.asset_account_id.as_str(), rule.minimum_balance_cents))
        .collect();
    let mut projected = balances.clone();
    let mut failures = missing_balance_failures(
        balance_rules,
        liability_rules,
        allocation_rules,
        balances,
    );
    if !failures.is_empty() {
        return SolveResult::Infeasible { failures };
    }
    let mut spendable: HashMap<String, i64> = balances
        .assets
        .iter()
        .map(|(id, balance)| {
            (
                id.clone(),
                balance
                    .saturating_sub(*minimums.get(id.as_str()).unwrap_or(&0))
                    .max(0),
            )
        })
        .collect();

    let mut required = liability_rules
        .iter()
        .filter(|rule| rule.requirement == RuleRequirement::Required)
        .collect::<Vec<_>>();
    required.sort_by_key(|rule| (rule.position, rule.id));
    let mut best_effort = liability_rules
        .iter()
        .filter(|rule| rule.requirement == RuleRequirement::BestEffort)
        .collect::<Vec<_>>();
    best_effort.sort_by_key(|rule| (rule.position, rule.id));

    let mut demand_by_source: HashMap<&str, i64> = HashMap::new();
    failures.clear();
    for rule in &required {
        match requested_payment(rule, &projected.liabilities) {
            Some(requested) => {
                *demand_by_source
                    .entry(&rule.payment_asset_account_id)
                    .or_default() += requested
            }
            None => failures.push(RuleFailure {
                message: format!(
                    "missing current balance for liability {} ({})",
                    rule.name, rule.liability_account_id
                ),
            }),
        }
    }
    for (source, demand) in demand_by_source {
        match spendable.get(source) {
            Some(available) if *available >= demand => {}
            Some(available) => failures.push(RuleFailure { message: format!("source {source} needs {demand} cents for required rules but only {available} cents are spendable (shortfall: {} cents)", demand - available) }),
            None => failures.push(RuleFailure { message: format!("missing current balance for required funding source {source}") }),
        }
    }
    if !failures.is_empty() {
        return SolveResult::Infeasible { failures };
    }

    let mut steps = Vec::new();
    for rule in required {
        apply_liability_rule(rule, false, &mut spendable, &mut projected, &mut steps);
    }
    for rule in best_effort {
        apply_liability_rule(rule, true, &mut spendable, &mut projected, &mut steps);
    }

    let mut fixed = allocation_rules
        .iter()
        .filter(|rule| rule.allocation_type == AllocationType::FixedAmount)
        .collect::<Vec<_>>();
    fixed.sort_by_key(|rule| (rule.position, rule.id));
    for rule in fixed {
        let amount = rule
            .allocation_value
            .unwrap_or(0)
            .min(*spendable.get(&rule.source_asset_account_id).unwrap_or(&0));
        apply_transfer(
            rule,
            amount,
            &mut spendable,
            &mut projected,
            &mut steps,
            "fixed excess allocation",
        );
    }

    let percentage_bases = spendable.clone();
    let mut percentages = allocation_rules
        .iter()
        .filter(|rule| rule.allocation_type == AllocationType::Percentage)
        .collect::<Vec<_>>();
    percentages.sort_by_key(|rule| (rule.position, rule.id));
    for rule in percentages {
        let base = *percentage_bases
            .get(&rule.source_asset_account_id)
            .unwrap_or(&0);
        let bps = rule.allocation_value.unwrap_or(0);
        let amount = ((i128::from(base) * i128::from(bps)) / 10_000) as i64;
        apply_transfer(
            rule,
            amount,
            &mut spendable,
            &mut projected,
            &mut steps,
            &format!("{bps} bps of post-fixed excess"),
        );
    }

    let mut remainders = allocation_rules
        .iter()
        .filter(|rule| rule.allocation_type == AllocationType::Remainder)
        .collect::<Vec<_>>();
    remainders.sort_by_key(|rule| (rule.position, rule.id));
    for rule in remainders {
        let amount = *spendable.get(&rule.source_asset_account_id).unwrap_or(&0);
        apply_transfer(
            rule,
            amount,
            &mut spendable,
            &mut projected,
            &mut steps,
            "remaining excess",
        );
    }

    for (index, step) in steps.iter_mut().enumerate() {
        step.sequence = index + 1;
    }
    if let Err(message) = verify_steps(balance_rules, balances, &steps, &projected) {
        return SolveResult::Infeasible {
            failures: vec![RuleFailure { message }],
        };
    }
    SolveResult::Executable {
        steps,
        projected_balances: projected,
    }
}

fn missing_balance_failures(
    balance_rules: &[entity::asset_balance_rules::Model],
    liability_rules: &[entity::plan_liability_rules::Model],
    allocation_rules: &[entity::plan_excess_allocation_rules::Model],
    balances: &AccountBalances,
) -> Vec<RuleFailure> {
    let mut missing = std::collections::BTreeSet::new();
    for rule in balance_rules {
        if !balances.assets.contains_key(&rule.asset_account_id) {
            missing.insert(format!("asset {}", rule.asset_account_id));
        }
    }
    for rule in liability_rules {
        if !balances.assets.contains_key(&rule.payment_asset_account_id) {
            missing.insert(format!("asset {}", rule.payment_asset_account_id));
        }
        if !balances.liabilities.contains_key(&rule.liability_account_id) {
            missing.insert(format!("liability {}", rule.liability_account_id));
        }
    }
    for rule in allocation_rules {
        if !balances.assets.contains_key(&rule.source_asset_account_id) {
            missing.insert(format!("asset {}", rule.source_asset_account_id));
        }
        if !balances
            .assets
            .contains_key(&rule.destination_asset_account_id)
        {
            missing.insert(format!("asset {}", rule.destination_asset_account_id));
        }
    }
    missing
        .into_iter()
        .map(|account| RuleFailure {
            message: format!("missing current balance for {account}"),
        })
        .collect()
}

fn requested_payment(
    rule: &entity::plan_liability_rules::Model,
    liabilities: &HashMap<String, i64>,
) -> Option<i64> {
    let current = *liabilities.get(&rule.liability_account_id)?;
    Some(match rule.rule_type {
        LiabilityRuleType::TargetBalance => current.saturating_sub(rule.value_cents).max(0),
        LiabilityRuleType::FixedPayment => rule.value_cents.min(current.max(0)),
    })
}

fn apply_liability_rule(
    rule: &entity::plan_liability_rules::Model,
    cap_to_available: bool,
    spendable: &mut HashMap<String, i64>,
    projected: &mut AccountBalances,
    steps: &mut Vec<PlanStep>,
) {
    let Some(requested) = requested_payment(rule, &projected.liabilities) else {
        return;
    };
    let available = *spendable.get(&rule.payment_asset_account_id).unwrap_or(&0);
    let amount = if cap_to_available {
        requested.min(available)
    } else {
        requested
    };
    if amount <= 0 {
        return;
    }
    *spendable
        .entry(rule.payment_asset_account_id.clone())
        .or_default() -= amount;
    *projected
        .assets
        .entry(rule.payment_asset_account_id.clone())
        .or_default() -= amount;
    *projected
        .liabilities
        .entry(rule.liability_account_id.clone())
        .or_default() -= amount;
    steps.push(PlanStep {
        sequence: 0,
        kind: PlanStepKind::LiabilityPayment,
        source_account_id: rule.payment_asset_account_id.clone(),
        destination_account_id: rule.liability_account_id.clone(),
        amount_cents: amount,
        reason: rule.name.clone(),
    });
}

fn apply_transfer(
    rule: &entity::plan_excess_allocation_rules::Model,
    amount: i64,
    spendable: &mut HashMap<String, i64>,
    projected: &mut AccountBalances,
    steps: &mut Vec<PlanStep>,
    reason: &str,
) {
    if amount <= 0 {
        return;
    }
    *spendable
        .entry(rule.source_asset_account_id.clone())
        .or_default() -= amount;
    *projected
        .assets
        .entry(rule.source_asset_account_id.clone())
        .or_default() -= amount;
    *projected
        .assets
        .entry(rule.destination_asset_account_id.clone())
        .or_default() += amount;
    steps.push(PlanStep {
        sequence: 0,
        kind: PlanStepKind::AssetTransfer,
        source_account_id: rule.source_asset_account_id.clone(),
        destination_account_id: rule.destination_asset_account_id.clone(),
        amount_cents: amount,
        reason: reason.to_string(),
    });
}

fn verify_steps(
    balance_rules: &[entity::asset_balance_rules::Model],
    initial: &AccountBalances,
    steps: &[PlanStep],
    expected: &AccountBalances,
) -> Result<(), String> {
    let mut replayed = initial.clone();
    for step in steps {
        if step.amount_cents <= 0 {
            return Err(format!("step {} has a non-positive amount", step.sequence));
        }
        let source = replayed
            .assets
            .get_mut(&step.source_account_id)
            .ok_or_else(|| {
                format!(
                    "step {} has unknown source {}",
                    step.sequence, step.source_account_id
                )
            })?;
        if *source < step.amount_cents {
            return Err(format!(
                "step {} overdraws source {}",
                step.sequence, step.source_account_id
            ));
        }
        *source -= step.amount_cents;
        match step.kind {
            PlanStepKind::AssetTransfer => {
                *replayed
                    .assets
                    .entry(step.destination_account_id.clone())
                    .or_default() += step.amount_cents
            }
            PlanStepKind::LiabilityPayment => {
                let liability = replayed
                    .liabilities
                    .get_mut(&step.destination_account_id)
                    .ok_or_else(|| {
                        format!(
                            "step {} has unknown liability {}",
                            step.sequence, step.destination_account_id
                        )
                    })?;
                if *liability < step.amount_cents {
                    return Err(format!(
                        "step {} overpays liability {}",
                        step.sequence, step.destination_account_id
                    ));
                }
                *liability -= step.amount_cents;
            }
        }
    }
    for rule in balance_rules {
        let initial_balance = initial
            .assets
            .get(&rule.asset_account_id)
            .copied()
            .unwrap_or_default();
        let protected_floor = initial_balance.min(rule.minimum_balance_cents);
        if replayed
            .assets
            .get(&rule.asset_account_id)
            .copied()
            .unwrap_or_default()
            < protected_floor
        {
            return Err(format!(
                "steps reduce {} below its protected minimum",
                rule.asset_account_id
            ));
        }
    }
    if &replayed != expected {
        return Err("step replay does not match projected balances".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::entity::prelude::DateTime;

    fn liability_rule(
        id: i32,
        requirement: RuleRequirement,
        position: i32,
        value: i64,
    ) -> entity::plan_liability_rules::Model {
        entity::plan_liability_rules::Model {
            id,
            name: format!("rule {id}"),
            created_at: DateTime::default(),
            rule_type: LiabilityRuleType::FixedPayment,
            value_cents: value,
            requirement,
            position,
            plan_id: 1,
            liability_account_id: format!("debt{id}"),
            payment_asset_account_id: "checking".into(),
        }
    }
    fn allocation(
        id: i32,
        kind: AllocationType,
        value: Option<i64>,
        position: i32,
        destination: &str,
    ) -> entity::plan_excess_allocation_rules::Model {
        entity::plan_excess_allocation_rules::Model {
            id,
            created_at: DateTime::default(),
            allocation_type: kind,
            allocation_value: value,
            position,
            plan_id: 1,
            source_asset_account_id: "checking".into(),
            destination_asset_account_id: destination.into(),
        }
    }
    fn balances() -> AccountBalances {
        AccountBalances {
            assets: HashMap::from([
                ("checking".into(), 100_000),
                ("savings".into(), 0),
                ("brokerage".into(), 0),
            ]),
            liabilities: HashMap::from([("debt1".into(), 50_000), ("debt2".into(), 50_000)]),
        }
    }

    #[test]
    fn required_shortfall_returns_no_steps() {
        let mut balances = balances();
        balances.liabilities.insert("debt1".into(), 200_000);
        let result = solve(
            &[],
            &[liability_rule(1, RuleRequirement::Required, 0, 100_001)],
            &[],
            &balances,
        );
        assert!(matches!(result, SolveResult::Infeasible { .. }));
    }

    #[test]
    fn protects_minimum_and_partially_funds_best_effort() {
        let minimum = entity::asset_balance_rules::Model {
            id: 1,
            created_at: DateTime::default(),
            minimum_balance_cents: 75_000,
            plan_id: 1,
            asset_account_id: "checking".into(),
        };
        let result = solve(
            &[minimum],
            &[liability_rule(1, RuleRequirement::BestEffort, 0, 50_000)],
            &[],
            &balances(),
        );
        let SolveResult::Executable {
            steps,
            projected_balances,
        } = result
        else {
            panic!("expected executable")
        };
        assert_eq!(steps[0].amount_cents, 25_000);
        assert_eq!(projected_balances.assets["checking"], 75_000);
    }

    #[test]
    fn an_already_underfunded_minimum_is_protected_but_not_replenished() {
        let minimum = entity::asset_balance_rules::Model {
            id: 1,
            created_at: DateTime::default(),
            minimum_balance_cents: 125_000,
            plan_id: 1,
            asset_account_id: "checking".into(),
        };
        let result = solve(&[minimum], &[], &[], &balances());
        let SolveResult::Executable {
            steps,
            projected_balances,
        } = result
        else {
            panic!("expected executable")
        };
        assert!(steps.is_empty());
        assert_eq!(projected_balances.assets["checking"], 100_000);
    }

    #[test]
    fn reports_missing_referenced_balances() {
        let mut balances = balances();
        balances.assets.remove("checking");
        let result = solve(
            &[],
            &[liability_rule(1, RuleRequirement::BestEffort, 0, 1_000)],
            &[],
            &balances,
        );
        let SolveResult::Infeasible { failures } = result else {
            panic!("expected infeasible")
        };
        assert!(failures[0].message.contains("asset checking"));
    }

    #[test]
    fn percentages_share_the_post_fixed_base_and_remainder_gets_rounding() {
        let rules = vec![
            allocation(1, AllocationType::FixedAmount, Some(10_000), 0, "savings"),
            allocation(2, AllocationType::Percentage, Some(3_333), 1, "savings"),
            allocation(3, AllocationType::Percentage, Some(6_667), 2, "brokerage"),
            allocation(4, AllocationType::Remainder, None, 3, "savings"),
        ];
        let result = solve(&[], &[], &rules, &balances());
        let SolveResult::Executable {
            projected_balances, ..
        } = result
        else {
            panic!("expected executable")
        };
        assert_eq!(projected_balances.assets["checking"], 0);
        assert_eq!(projected_balances.assets["savings"], 39_997);
        assert_eq!(projected_balances.assets["brokerage"], 60_003);
    }
}
