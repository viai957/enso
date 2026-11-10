use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use gpui_kit::base::text::editor_host::{InlineNode, Paragraph, TextMark};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::document::Kind;
use super::view::VisualView;
use crate::sheet::{self, Computed};
use crate::tokens;

pub type Values = Rc<HashMap<(usize, usize), Computed>>;

impl VisualView {
    fn table_grid(&self, bix: usize, cx: &App) -> Vec<Vec<String>> {
        let block = &self.doc.blocks[bix];
        let Kind::Table { rows, .. } = &block.kind else {
            return Vec::new();
        };
        let text = self.editor.read(cx).text();
        rows.iter()
            .map(|row| {
                row.leaves
                    .clone()
                    .map(|lix| {
                        let leaf = &block.leaves[lix];
                        let src = leaf.src_range();
                        let raw = text
                            .slice(block.start + src.start..block.start + src.end)
                            .to_string();
                        if sheet::is_formula(&raw) {
                            raw.replace("\\|", "|")
                        } else {
                            leaf.text.clone()
                        }
                    })
                    .collect()
            })
            .collect()
    }

    pub fn table_values(&self, bix: usize, cx: &App) -> Option<Values> {
        let grid = self.table_grid(bix, cx);
        if !sheet::has_formula(&grid) {
            return None;
        }
        let mut hasher = DefaultHasher::new();
        grid.hash(&mut hasher);
        let key = hasher.finish();
        let id = self.doc.blocks[bix].id;
        let mut cache = self.table_calc.borrow_mut();
        if let Some((k, values)) = cache.get(&id)
            && *k == key
        {
            return Some(values.clone());
        }
        let values: Values = Rc::new(sheet::engine().compute(&grid));
        cache.insert(id, (key, values.clone()));
        Some(values)
    }

    pub fn computed_cells(
        &self,
        bix: usize,
        rows: std::ops::Range<usize>,
        cx: &App,
    ) -> Vec<Vec<Paragraph>> {
        let block = &self.doc.blocks[bix];
        let Kind::Table { rows: all, .. } = &block.kind else {
            return Vec::new();
        };
        let values = self.table_values(bix, cx);
        let caret = self
            .table_cell()
            .filter(|(b, _, _)| *b == bix)
            .map(|(_, r, c)| (r, c));
        let danger = cx.theme().danger.opacity(tokens::SUBTLE);
        let mut shown = self.formula_keys.borrow_mut();
        rows.map(|r| {
            let row = &all[r];
            row.cells
                .iter()
                .enumerate()
                .map(|(c, cell)| {
                    let lix = row.leaves.start + c;
                    let key = block.leaves.get(lix).map(|l| l.key);
                    match (values.as_ref().and_then(|v| v.get(&(r, c))), key) {
                        (Some(value), Some(key)) if caret != Some((r, c)) => {
                            shown.insert(key);
                            let mark = if value.error {
                                TextMark::default().highlight(danger)
                            } else {
                                TextMark::default()
                            };
                            let mut p = Paragraph::default();
                            p.push(
                                InlineNode::new(value.text.clone())
                                    .marks(vec![(0..value.text.len(), mark)]),
                            );
                            p.with_leaf_key(key)
                        }
                        (_, key) => {
                            if let Some(key) = key {
                                shown.remove(&key);
                            }
                            cell.clone()
                        }
                    }
                })
                .collect()
        })
        .collect()
    }
}
