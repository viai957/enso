mod ironcalc;

use std::collections::HashMap;

pub use ironcalc::IronCalc;

#[derive(Clone, Debug, PartialEq)]
pub struct Computed {
    pub text: String,
    pub error: bool,
}

pub trait Engine {
    fn compute(&self, grid: &[Vec<String>]) -> HashMap<(usize, usize), Computed>;
}

pub fn is_formula(cell: &str) -> bool {
    let cell = cell.trim_start();
    cell.len() > 1 && cell.starts_with('=')
}

pub fn has_formula(grid: &[Vec<String>]) -> bool {
    grid.iter().flatten().any(|cell| is_formula(cell))
}

pub fn engine() -> &'static dyn Engine {
    static ENGINE: IronCalc = IronCalc;
    &ENGINE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(rows: &[&[&str]]) -> Vec<Vec<String>> {
        rows.iter()
            .map(|r| r.iter().map(|c| c.to_string()).collect())
            .collect()
    }

    #[test]
    fn formulas_use_the_rows_and_columns_people_see() {
        let table = grid(&[
            &["Item", "Price", "Qty", "Total"],
            &["Tea", "4.5", "2", "=B2*C2"],
            &["Cake", "$3.25", "4", "=B3*C3"],
            &["Sum", "", "=SUM(C2:C3)", "=SUM(D2:D3)"],
        ]);
        let out = engine().compute(&table);
        assert_eq!(out[&(1, 3)].text, "9");
        assert_eq!(out[&(2, 3)].text, "$13.00");
        assert_eq!(out[&(3, 2)].text, "6");
        assert_eq!(out[&(3, 3)].text.trim_start_matches('$'), "22");
        assert!(!out.contains_key(&(1, 1)));
    }

    #[test]
    fn errors_are_flagged_and_text_is_left_alone() {
        let table = grid(&[&["a", "b"], &["1", "=A2/0"], &["=UPPER(\"x\")", "=NOPE(1)"]]);
        let out = engine().compute(&table);
        assert!(out[&(1, 1)].error, "{:?}", out[&(1, 1)]);
        assert_eq!(out[&(1, 1)].text, "#DIV/0!");
        assert_eq!(out[&(2, 0)].text, "X");
        assert!(out[&(2, 1)].error);
        assert!(!is_formula("="));
        assert!(!has_formula(&grid(&[&["a"], &["b"]])));
    }
}
