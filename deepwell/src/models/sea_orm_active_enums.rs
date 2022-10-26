//! SeaORM Entity. Generated by sea-orm-codegen 0.9.2

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "page_revision_type")]
pub enum PageRevisionType {
    #[sea_orm(string_value = "create")]
    Create,
    #[sea_orm(string_value = "delete")]
    Delete,
    #[sea_orm(string_value = "move")]
    Move,
    #[sea_orm(string_value = "regular")]
    Regular,
    #[sea_orm(string_value = "undelete")]
    Undelete,
}
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_type")]
#[serde(rename_all = "camelCase")]
pub enum UserType {
    #[sea_orm(string_value = "bot")]
    Bot,
    #[sea_orm(string_value = "regular")]
    Regular,
    #[sea_orm(string_value = "system")]
    System,
}
impl Default for UserType {
    #[inline]
    fn default() -> Self {
        UserType::Regular
    }
}
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "file_revision_type")]
pub enum FileRevisionType {
    #[sea_orm(string_value = "create")]
    Create,
    #[sea_orm(string_value = "delete")]
    Delete,
    #[sea_orm(string_value = "undelete")]
    Undelete,
    #[sea_orm(string_value = "update")]
    Update,
}
