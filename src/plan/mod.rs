mod domain;
mod solver;

use crate::{db, entity, logging, money};
use domain::{AllocationRuleInput, AssetBalanceRuleInput, CreatePlanInput, LiabilityRuleInput};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use std::io::{self, Write};

pub async fn create() {
    let db = db::get_db().await;
    let assets = entity::asset_accounts::Entity::find()
        .all(&db)
        .await
        .unwrap_or_default();
    let liabilities = entity::liability_accounts::Entity::find()
        .all(&db)
        .await
        .unwrap_or_default();

    if assets.is_empty() {
        logging::error("link at least one asset account before creating a plan");
        return;
    }

    let input = match collect_input(&assets, &liabilities) {
        Ok(input) => input,
        Err(error) => {
            logging::error(&error);
            return;
        }
    };
    if let Err(errors) = domain::validate(&input, &assets, &liabilities) {
        for error in errors {
            logging::error(&error);
        }
        return;
    }

    print_summary(&input);
    if !prompt_yes_no("Save this plan?", false).unwrap_or(false) {
        logging::info("plan creation cancelled");
        return;
    }

    match save(&db, input).await {
        Ok(plan) => logging::success(&format!("saved plan {} with id {}", plan.name, plan.id)),
        Err(error) => logging::error(&format!("failed to save plan: {error}")),
    }
}

pub async fn execute(plan_id: i32) {
    let db = db::get_db().await;
    let Some(plan) = entity::plans::Entity::find_by_id(plan_id)
        .one(&db)
        .await
        .unwrap_or(None)
    else {
        logging::error(&format!("plan {plan_id} does not exist"));
        return;
    };
    let balance_rules = entity::asset_balance_rules::Entity::find()
        .filter(entity::asset_balance_rules::Column::PlanId.eq(plan_id))
        .all(&db)
        .await
        .unwrap_or_default();
    let liability_rules = entity::plan_liability_rules::Entity::find()
        .filter(entity::plan_liability_rules::Column::PlanId.eq(plan_id))
        .all(&db)
        .await
        .unwrap_or_default();
    let allocation_rules = entity::plan_excess_allocation_rules::Entity::find()
        .filter(entity::plan_excess_allocation_rules::Column::PlanId.eq(plan_id))
        .all(&db)
        .await
        .unwrap_or_default();

    let mut balances = solver::AccountBalances::default();
    for item in crate::plaid::get_linked_accounts().await {
        for account in item.plaid_item.accounts {
            match account.account_type {
                crate::plaid::types::AccountType::Credit
                | crate::plaid::types::AccountType::Loan => {
                    balances
                        .liabilities
                        .insert(account.account_id, account.balances.current_cents);
                }
                _ => {
                    balances
                        .assets
                        .insert(account.account_id, account.balances.current_cents);
                }
            }
        }
    }

    match solver::solve(
        &balance_rules,
        &liability_rules,
        &allocation_rules,
        &balances,
    ) {
        solver::SolveResult::Infeasible { failures } => {
            logging::error(&format!("plan {} cannot be executed", plan.name));
            for failure in failures {
                println!("  - {}", failure.message);
            }
        }
        solver::SolveResult::Executable {
            steps,
            projected_balances,
        } => {
            println!("\nPlan: {}\n", plan.name);
            if steps.is_empty() {
                println!("No actions are needed.");
            }
            for step in steps {
                let verb = match step.kind {
                    solver::PlanStepKind::LiabilityPayment => "Pay",
                    solver::PlanStepKind::AssetTransfer => "Transfer",
                };
                println!(
                    "{}. {verb} {} from {} to {}",
                    step.sequence,
                    money::format_cents(step.amount_cents),
                    account_name(&step.source_account_id, &db).await,
                    account_name(&step.destination_account_id, &db).await
                );
                println!("   {}", step.reason);
            }
            println!("\nExpected final balances:");
            let mut assets: Vec<_> = projected_balances.assets.into_iter().collect();
            assets.sort_by(|a, b| a.0.cmp(&b.0));
            for (id, value) in assets {
                println!(
                    "  {:24} {}",
                    account_name(&id, &db).await,
                    money::format_cents(value)
                );
            }
            let mut liabilities: Vec<_> = projected_balances.liabilities.into_iter().collect();
            liabilities.sort_by(|a, b| a.0.cmp(&b.0));
            for (id, value) in liabilities {
                println!(
                    "  {:24} {} owed",
                    account_name(&id, &db).await,
                    money::format_cents(value)
                );
            }
        }
    }
}

async fn account_name(id: &str, db: &sea_orm::DatabaseConnection) -> String {
    if let Ok(Some(account)) = entity::asset_accounts::Entity::find_by_id(id).one(db).await {
        return account.name;
    }
    if let Ok(Some(account)) = entity::liability_accounts::Entity::find_by_id(id)
        .one(db)
        .await
    {
        return account.name;
    }
    id.to_string()
}

async fn save(
    db: &sea_orm::DatabaseConnection,
    input: CreatePlanInput,
) -> Result<entity::plans::Model, sea_orm::DbErr> {
    let transaction = db.begin().await?;
    let plan = entity::plans::ActiveModel {
        name: Set(input.name),
        plan_type: Set(entity::plans::PlanType::Recurring),
        ..Default::default()
    }
    .insert(&transaction)
    .await?;

    for rule in input.asset_balance_rules {
        entity::asset_balance_rules::ActiveModel {
            minimum_balance_cents: Set(rule.minimum_balance_cents),
            plan_id: Set(plan.id),
            asset_account_id: Set(rule.asset_account_id),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
    }
    for rule in input.liability_rules {
        let (name, rule_type, value_cents, requirement, position, liability_id, source_id) =
            rule.persisted_values();
        entity::plan_liability_rules::ActiveModel {
            name: Set(name.to_string()),
            rule_type: Set(rule_type),
            value_cents: Set(value_cents),
            requirement: Set(requirement),
            position: Set(position),
            plan_id: Set(plan.id),
            liability_account_id: Set(liability_id.to_string()),
            payment_asset_account_id: Set(source_id.to_string()),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
    }
    for rule in input.allocation_rules {
        let (allocation_type, allocation_value, position, source_id, destination_id) =
            rule.persisted_values();
        entity::plan_excess_allocation_rules::ActiveModel {
            allocation_type: Set(allocation_type),
            allocation_value: Set(allocation_value),
            position: Set(position),
            plan_id: Set(plan.id),
            source_asset_account_id: Set(source_id.to_string()),
            destination_asset_account_id: Set(destination_id.to_string()),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
    }
    transaction.commit().await?;
    Ok(plan)
}

fn collect_input(
    assets: &[entity::asset_accounts::Model],
    liabilities: &[entity::liability_accounts::Model],
) -> Result<CreatePlanInput, String> {
    let name = prompt("Plan name")?;
    let depository: Vec<_> = assets
        .iter()
        .filter(|account| account.account_type == entity::types::AssetAccountType::Depository)
        .collect();
    if depository.is_empty() {
        return Err("a plan needs at least one linked depository funding account".into());
    }

    println!("\nAsset minimums (leave blank for none):");
    let mut asset_balance_rules = Vec::new();
    for account in assets {
        let value = prompt(&format!("  {} minimum", account.name))?;
        if !value.is_empty() {
            asset_balance_rules.push(AssetBalanceRuleInput {
                asset_account_id: account.account_id.clone(),
                minimum_balance_cents: money::parse_dollars_to_cents(&value)?,
            });
        }
    }

    let mut liability_rules = Vec::new();
    let mut liability_position = 0;
    for liability in liabilities {
        if !prompt_yes_no(&format!("Add a rule for {}?", liability.name), false)? {
            continue;
        }
        let kind = prompt_choice("Rule type", &["target balance", "fixed payment"])?;
        let value = money::parse_dollars_to_cents(&prompt("Amount")?)?;
        let source = select_account("Payment source", &depository)?;
        let requirement = if prompt_choice("Requirement", &["required", "best effort"])? == 0 {
            entity::types::RuleRequirement::Required
        } else {
            entity::types::RuleRequirement::BestEffort
        };
        let common = (
            liability.name.clone(),
            requirement,
            liability_position,
            liability.account_id.clone(),
            source.account_id.clone(),
        );
        liability_rules.push(if kind == 0 {
            LiabilityRuleInput::TargetBalance {
                name: common.0,
                target_balance_cents: value,
                requirement: common.1,
                position: common.2,
                liability_account_id: common.3,
                payment_asset_account_id: common.4,
            }
        } else {
            LiabilityRuleInput::FixedPayment {
                name: common.0,
                payment_cents: value,
                requirement: common.1,
                position: common.2,
                liability_account_id: common.3,
                payment_asset_account_id: common.4,
            }
        });
        liability_position += 1;
    }

    let mut allocation_rules = Vec::new();
    let mut position = 0;
    while prompt_yes_no("Add an excess allocation?", false)? {
        let source = select_account("Allocation source", &depository)?;
        let destinations: Vec<_> = assets
            .iter()
            .filter(|account| account.account_id != source.account_id)
            .collect();
        if destinations.is_empty() {
            return Err("an allocation needs a destination different from its source".into());
        }
        let destination = select_account("Allocation destination", &destinations)?;
        let kind = prompt_choice(
            "Allocation type",
            &["fixed amount", "percentage", "remainder"],
        )?;
        allocation_rules.push(match kind {
            0 => AllocationRuleInput::FixedAmount {
                amount_cents: money::parse_dollars_to_cents(&prompt("Amount")?)?,
                position,
                source_asset_account_id: source.account_id.clone(),
                destination_asset_account_id: destination.account_id.clone(),
            },
            1 => AllocationRuleInput::Percentage {
                percentage_bps: parse_percentage_bps(&prompt("Percentage")?)?,
                position,
                source_asset_account_id: source.account_id.clone(),
                destination_asset_account_id: destination.account_id.clone(),
            },
            _ => AllocationRuleInput::Remainder {
                position,
                source_asset_account_id: source.account_id.clone(),
                destination_asset_account_id: destination.account_id.clone(),
            },
        });
        position += 1;
    }
    Ok(CreatePlanInput {
        name,
        asset_balance_rules,
        liability_rules,
        allocation_rules,
    })
}

fn print_summary(input: &CreatePlanInput) {
    println!("\nPlan: {}", input.name);
    println!("  {} asset minimum(s)", input.asset_balance_rules.len());
    println!("  {} liability rule(s)", input.liability_rules.len());
    println!(
        "  {} excess allocation rule(s)",
        input.allocation_rules.len()
    );
}

fn prompt(label: &str) -> Result<String, String> {
    print!("{label}: ");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| error.to_string())?;
    Ok(input.trim().to_string())
}

fn prompt_yes_no(label: &str, default: bool) -> Result<bool, String> {
    let suffix = if default { "[Y/n]" } else { "[y/N]" };
    let answer = prompt(&format!("{label} {suffix}"))?;
    if answer.is_empty() {
        return Ok(default);
    }
    match answer.to_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Err("expected yes or no".into()),
    }
}

fn prompt_choice(label: &str, choices: &[&str]) -> Result<usize, String> {
    println!("{label}:");
    for (index, choice) in choices.iter().enumerate() {
        println!("  {}. {}", index + 1, choice);
    }
    let selected = prompt("Selection")?
        .parse::<usize>()
        .map_err(|_| "selection must be a number".to_string())?;
    if !(1..=choices.len()).contains(&selected) {
        return Err("selection is out of range".into());
    }
    Ok(selected - 1)
}

fn select_account<'a>(
    label: &str,
    accounts: &[&'a entity::asset_accounts::Model],
) -> Result<&'a entity::asset_accounts::Model, String> {
    let choices: Vec<_> = accounts
        .iter()
        .map(|account| account.name.as_str())
        .collect();
    let selected = prompt_choice(label, &choices)?;
    Ok(accounts[selected])
}

fn parse_percentage_bps(value: &str) -> Result<i64, String> {
    money::parse_dollars_to_cents(value)
}
