use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
pub enum LiabilityRuleType {
    TargetBalance,
    FixedPayment,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "plan_liability_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    pub rule_type: LiabilityRuleType,
    pub value_cents: i64,
    pub requirement: super::types::RuleRequirement,
    pub position: i32,

    #[sea_orm(belongs_to, from = "plan_id", to = "id")]
    pub plan: HasOne<super::plans::Entity>,
    pub plan_id: i32,

    #[sea_orm(belongs_to, from = "liability_account_id", to = "account_id")]
    pub liability_account: HasOne<super::liability_accounts::Entity>,
    pub liability_account_id: String,

    #[sea_orm(belongs_to, from = "payment_asset_account_id", to = "account_id")]
    pub payment_asset_account: HasOne<super::asset_accounts::Entity>,
    pub payment_asset_account_id: String,
}

impl ActiveModelBehavior for ActiveModel {}
