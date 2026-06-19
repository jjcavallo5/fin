use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "liability_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: String,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    pub name: String,

    #[sea_orm(has_one)]
    pub plan_liability_rules: HasOne<super::plan_liability_rules::Entity>,

    #[sea_orm(belongs_to, from = "plaid_item_id", to = "id")]
    pub plaid_item: HasOne<super::plaid_item::Entity>,
    pub plaid_item_id: Option<i32>,
}

impl ActiveModelBehavior for ActiveModel {}
