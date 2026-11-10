use std::collections::HashMap;

use ironcalc_base::Model;
use ironcalc_base::types::CellType;

use super::{Computed, Engine, is_formula};

const SHEET: u32 = 0;

pub struct IronCalc;

impl Engine for IronCalc {
    fn compute(&self, grid: &[Vec<String>]) -> HashMap<(usize, usize), Computed> {
        let mut out = HashMap::new();
        let Ok(mut model) = Model::new_empty("table", "en", "UTC", "en") else {
            return out;
        };
        for (r, row) in grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let cell = cell.trim();
                if !cell.is_empty() {
                    let _ =
                        model.set_user_input(SHEET, r as i32 + 1, c as i32 + 1, cell.to_string());
                }
            }
        }
        model.evaluate();
        for (r, row) in grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if !is_formula(cell) {
                    continue;
                }
                let (row, column) = (r as i32 + 1, c as i32 + 1);
                let Ok(text) = model.get_formatted_cell_value(SHEET, row, column) else {
                    continue;
                };
                let error = matches!(
                    model.get_cell_type(SHEET, row, column),
                    Ok(CellType::ErrorValue)
                );
                out.insert((r, c), Computed { text, error });
            }
        }
        out
    }
}
