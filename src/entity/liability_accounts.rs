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
    pub account_type: super::types::LiabilityAccountType,
    pub account_subtype: String,

    #[sea_orm(has_many)]
    pub plan_liability_rules: HasMany<super::plan_liability_rules::Entity>,

    #[sea_orm(belongs_to, from = "plaid_item_id", to = "id")]
    pub plaid_item: HasOne<super::plaid_item::Entity>,
    pub plaid_item_id: i32,
}

impl ActiveModelBehavior for ActiveModel {}
