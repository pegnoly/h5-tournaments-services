use rust_xlsxwriter::Workbook;

use crate::utils::StatsGeneratorDataModel;

pub mod pair;
pub mod player;
pub mod race;
pub mod styles;

pub trait StatsBuilder {
    fn build(&mut self, data: &StatsGeneratorDataModel, workbook: &mut Workbook);
}