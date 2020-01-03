use crate::grammar::Symbol;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct LR0Item {
    pub(crate) lhs: String,
    pub(crate) symbols: Vec<Symbol>,
    pub(crate) dot: usize,
    pub(crate) is_start: bool,
    pub(crate) production_number: Option<usize>,
}

impl LR0Item {
    pub fn name_after_dot(&self) -> Option<String> {
        self.symbol_after_dot().map(|symbol| symbol.name())
    }

    pub fn symbol_after_dot(&self) -> Option<Symbol> {
        if self.dot >= self.symbols.len() {
            return None;
        } else {
            return Some(self.symbols[self.dot].clone());
        }
    }

    pub fn symbol_before_dot(&self) -> Option<Symbol> {
        if self.dot == 0 {
            return None;
        } else {
            return Some(self.symbols[self.dot - 1].clone());
        }
    }

    pub fn dot_at_end(&self) -> bool {
        self.dot == self.symbols.len()
    }
}
