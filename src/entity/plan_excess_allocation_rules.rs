use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
pub enum AllocationType {
    FixedAmount,
    Percentage,
    Remainder,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "plan_excess_allocation_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    pub allocation_type: AllocationType,

    #[sea_orm(nullable)]
    pub amount_cents: Option<i32>,
    #[sea_orm(nullable)]
    pub percentage_bps: Option<i32>,

    #[sea_orm(belongs_to, from = "plan_id", to = "id")]
    pub plan: HasOne<super::plans::Entity>,
    pub plan_id: Option<i32>,

    #[sea_orm(belongs_to, from = "asset_account_id", to = "account_id")]
    pub asset_account: HasOne<super::asset_accounts::Entity>,
    pub asset_account_id: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}
