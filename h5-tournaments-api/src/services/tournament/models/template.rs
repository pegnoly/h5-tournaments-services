use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use sea_orm::{FromJsonQueryResult, prelude::*};

#[derive(
    Debug,
    DeriveActiveEnum,
    EnumIter,
    EnumString,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Display,
    async_graphql::Enum
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum TemplateType {
    #[sea_orm(string_value = "UniS Casino")]
    #[serde(rename = "UniS Casino")]
    #[strum(serialize = "UniS Casino")]
    UniSCasino,
    #[sea_orm(string_value = "UniS NeutralPool")]
    #[serde(rename = "UniS NeutralPool")]
    #[strum(serialize = "UniS NeutralPool")]
    UniSNeutralPool,
    #[sea_orm(string_value = "UniS CentralRush")]
    #[serde(rename = "UniS CentralRush")]
    #[strum(serialize = "UniS CentralRush")]
    UniSCentralRush,
    #[sea_orm(string_value = "UniS Native")]
    #[serde(rename = "UniS Native")]
    #[strum(serialize = "UniS Native")]
    UniSNative,
    #[sea_orm(string_value = "Jebus Casino")]
    #[serde(rename = "Jebus Casino")]
    #[strum(serialize = "Jebus Casino")]
    JebusCasino,
    #[sea_orm(string_value = "Jebus Native")]
    #[serde(rename = "Jebus Native")]
    #[strum(serialize = "Jebus Native")]
    JebusNative,
    #[sea_orm(string_value = "Jebus MegaTreasure")]
    #[serde(rename = "Jebus MegaTreasure")]
    #[strum(serialize = "Jebus MegaTreasure")]
    JebusMegaTreasure,
    #[sea_orm(string_value = "Moon Casino")]
    #[serde(rename = "Moon Casino")]
    #[strum(serialize = "Moon Casino")]
    MoonCasino,
    #[sea_orm(string_value = "Moon Native")]
    #[serde(rename = "Moon Native")]
    #[strum(serialize = "Moon Native")]
    MoonNative,
    #[sea_orm(string_value = "Moon CentralRush")]
    #[serde(rename = "Moon CentralRush")]
    #[strum(serialize = "Moon CentralRush")]
    MoonCentralRush,
    #[sea_orm(string_value = "Moon MegaTreasure")]
    #[serde(rename = "Moon MegaTreasure")]
    #[strum(serialize = "Moon MegaTreasure")]
    MoonMegaTreasure,
    #[sea_orm(string_value = "Echo Casino")]
    #[serde(rename = "Echo Casino")]
    #[strum(serialize = "Echo Casino")]
    EchoCasino,
    #[sea_orm(string_value = "Echo Native")]
    #[serde(rename = "Echo Native")]
    #[strum(serialize = "Echo Native")]
    EchoNative,
    #[sea_orm(string_value = "Echo Redline")]
    #[serde(rename = "Echo Redline")]
    #[strum(serialize = "Echo Redline")]
    EchoRedline,
    #[sea_orm(string_value = "Echo Timelock")]
    #[serde(rename = "Echo Timelock")]
    #[strum(serialize = "Echo Timelock")]
    EchoTimelock
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct TemplatesList {
    pub templates: Vec<TemplateType>
}

#[async_graphql::Object]
impl TemplatesList {
    async fn templates(&self) -> &Vec<TemplateType> {
        &self.templates
    }
}