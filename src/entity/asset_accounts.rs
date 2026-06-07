use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "asset_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub created_at: DateTime,
    pub token: String,

    #[sea_orm(has_one)]
    pub liability_payments: HasOne<super::plan_liability_rules::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
