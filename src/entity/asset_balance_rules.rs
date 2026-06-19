use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "asset_balance_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    pub minimum_balance_cents: i32,

    #[sea_orm(belongs_to, from = "plan_id", to = "id")]
    pub plan: HasOne<super::plans::Entity>,
    pub plan_id: Option<i32>,

    #[sea_orm(belongs_to, from = "asset_account_id", to = "account_id")]
    pub asset_account: HasOne<super::asset_accounts::Entity>,
    pub asset_account_id: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}
