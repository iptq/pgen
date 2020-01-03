use crate::grammar::Symbol;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct LR0Item {
    pub(crate) lhs: String,
    pub(crate) symbols: Vec<Symbol>,
    pub(crate) dot: usize,
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
}
