use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "plaid_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    pub institution_name: String,
    pub nonce: String,
    pub encrypted_token: String,

    #[sea_orm(has_many)]
    pub plan_asset_accounts: HasMany<super::asset_accounts::Entity>,
    #[sea_orm(has_many)]
    pub plan_liability_accounts: HasMany<super::liability_accounts::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
