use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "liability_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTime,
    pub nonce: String,
    pub encrypted_token: String,

    #[sea_orm(has_one)]
    pub plan_liability_rules: HasOne<super::plan_liability_rules::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
