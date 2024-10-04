// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rust_xlsxwriter::Workbook;

fn main() {
    test();
    h5_tournaments_manager_lib::run()
}

fn test() {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write(0, 0, "homm5").unwrap();
    let path = std::env::current_exe().unwrap().parent().unwrap().join("test.xlsx");
    workbook.save(path).unwrap();
}