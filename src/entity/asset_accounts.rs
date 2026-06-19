use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "asset_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    pub name: String,

    #[sea_orm(has_one)]
    pub liability_rule: HasOne<super::plan_liability_rules::Entity>,
    #[sea_orm(has_one)]
    pub plan_asset_allocation_rule: HasOne<super::plan_excess_allocation_rules::Entity>,
    #[sea_orm(has_one)]
    pub asset_balance_rules: HasOne<super::asset_balance_rules::Entity>,

    #[sea_orm(belongs_to, from = "plaid_item_id", to = "id")]
    pub plaid_item: HasOne<super::plaid_item::Entity>,
    pub plaid_item_id: Option<i32>,
}

impl ActiveModelBehavior for ActiveModel {}
