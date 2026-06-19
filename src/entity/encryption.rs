use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "encryption")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub salt: String,
}

impl ActiveModelBehavior for ActiveModel {}
