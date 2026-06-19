use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
pub enum PlanType {
    Recurring,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "plans")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub plan_type: PlanType,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,

    #[sea_orm(has_many)]
    pub plan_liability_rules: HasMany<super::plan_liability_rules::Entity>,
    #[sea_orm(has_many)]
    pub plan_excess_allocation_rules: HasMany<super::plan_excess_allocation_rules::Entity>,
    #[sea_orm(has_many)]
    pub asset_balance_rules: HasMany<super::asset_balance_rules::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
