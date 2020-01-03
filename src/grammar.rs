use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::Write;

use crate::items::LR0Item;
use crate::parser::ParseTable;
use crate::Parser;

#[derive(Debug, Error)]
pub enum GrammarError {
    #[error("Name conflict: {0}")]
    NameConflict(String),

    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),

    #[error("Start symbols must be nonterminals: {0}")]
    StartingTerminal(String),
}

pub struct Grammar {
    pub(crate) start_symbols: Vec<String>,
    pub(crate) terminals: HashMap<String, String>,
    pub(crate) productions: HashMap<String, Vec<Production>>,
}

impl Grammar {
    pub fn build(self) -> Result<Parser, GrammarError> {
        // name -> symbol map
        let grammar_symbols = {
            let mut symbols = HashMap::new();
            // should have no problem with terminals only
            for terminal in self.terminals.keys() {
                symbols.insert(terminal.to_owned(), Symbol::T(terminal.clone()));
            }
            for nonterminal in self.productions.keys() {
                if symbols.contains_key(nonterminal) {
                    return Err(GrammarError::NameConflict(nonterminal.clone()));
                }
                symbols.insert(nonterminal.to_owned(), Symbol::NT(nonterminal.clone()));
            }
            symbols
        };

        // build items
        let items = {
            let mut items = Vec::new();
            for (nonterminal, productions) in self.productions.iter() {
                for production in productions {
                    items.push(LR0Item {
                        lhs: nonterminal.clone(),
                        dot: 0,
                        symbols: production.symbols(&grammar_symbols)?,
                    });
                }
            }
            items
        };

        // start symbols must be nonterminals
        for symbol in self.start_symbols.iter() {
            if let Some(Symbol::T(symbol)) = grammar_symbols.get(symbol) {
                return Err(GrammarError::StartingTerminal(symbol.clone()));
            }
        }

        let mut grammar_helper = GrammarHelper {
            grammar: &self,
            grammar_symbols,
            canonical_collection: BTreeSet::new(),
            first_sets: BTreeMap::new(),
        };
        grammar_helper.init(self.start_symbols.clone());
        grammar_helper.build();

        println!("canonical collection:");
        for (i, item_set) in grammar_helper.canonical_collection.iter().enumerate() {
            println!("state {}:", i);
            for item in item_set {
                println!(" - {:?}", item);
            }
            println!("");
        }

        Ok(Parser {
            start_symbols: self.start_symbols,
            terminals: self.terminals,
            nonterminals: self.productions.keys().cloned().collect(),
        })
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Symbol {
    T(String),
    NT(String),
    Epsilon,
    EOF,
}

impl Symbol {
    pub fn name(&self) -> String {
        match self {
            Symbol::T(name) | Symbol::NT(name) => name.clone(),
            Symbol::Epsilon => "\u{025b}".to_owned(),
            Symbol::EOF => "$".to_owned(),
        }
    }
}

pub struct Production(pub(crate) Vec<String>);

impl<T: Iterator<Item = String>> From<T> for Production {
    fn from(iter: T) -> Self {
        Production(iter.collect())
    }
}

impl Production {
    pub fn symbols(
        &self,
        grammar_symbols: &HashMap<String, Symbol>,
    ) -> Result<Vec<Symbol>, GrammarError> {
        let mut symbols = Vec::new();
        for symbol_candidate in self.0.iter() {
            if let Some(symbol) = grammar_symbols.get(symbol_candidate) {
                symbols.push(symbol.clone());
            } else {
                return Err(GrammarError::InvalidSymbol(symbol_candidate.clone()));
            }
        }
        Ok(symbols)
    }
}

pub struct GrammarHelper<'a> {
    pub(crate) grammar: &'a Grammar,
    pub(crate) grammar_symbols: HashMap<String, Symbol>,
    pub(crate) canonical_collection: BTreeSet<BTreeSet<LR0Item>>,
    pub(crate) first_sets: BTreeMap<Symbol, BTreeSet<Symbol>>,
}

impl<'a> GrammarHelper<'a> {
    pub fn init(&mut self, start_symbols: Vec<String>) {
        self.compute_first_sets();
        println!("First sets:");
        for (sym, set) in self.first_sets.iter() {
            println!(" {:?} - {:?}", sym, set);
        }
        println!("");

        for start_symbol in start_symbols {
            let mut new_set = BTreeSet::new();
            let new_item = LR0Item {
                lhs: start_symbol.clone() + "'",
                dot: 0,
                symbols: vec![Symbol::NT(start_symbol), Symbol::EOF],
            };
            new_set.insert(new_item);
            self.canonical_collection.insert(self.closure(new_set));
        }
    }

    fn compute_first_sets(&mut self) {
        loop {
            let mut changes = false;
            for (name, symbol) in self.grammar_symbols.iter() {
                match symbol {
                    Symbol::T(_) => {
                        if !self.first_sets.contains_key(symbol) {
                            self.first_sets
                                .insert(symbol.clone(), vec![symbol.clone()].into_iter().collect());
                            changes = true;
                        }
                    }
                    Symbol::NT(_) => {
                        if !self.first_sets.contains_key(symbol) {
                            self.first_sets
                                .insert(symbol.clone(), BTreeSet::new());
                        }
                        let mut first_set = self.first_sets.get_mut(symbol).unwrap().clone();
                        println!("FIRST({}): existing = {:?}", name, first_set);
                        'outer: for production in self.grammar.productions.get(name).unwrap() {
                            let symbol_list = production.symbols(&self.grammar_symbols).unwrap();
                            for yi in symbol_list {
                                if let Some(sym_first_set) = self.first_sets.get(&yi) {
                                    // if there's extra elements in FIRST(Yi)
                                    if !first_set.is_superset(sym_first_set) {
                                        first_set = first_set.union(sym_first_set).cloned().collect();
                                        changes = true;
                                    }

                                    // if it contains epsilon, continue looking
                                    // this means we see X -> ... "" "" Yi
                                    // and this current one is also "", so keep going
                                    // otherwise, break
                                    if !sym_first_set.contains(&Symbol::Epsilon) {
                                        continue 'outer;
                                    }
                                }

                                // no symbols calculated here yet, continue
                                continue 'outer;
                            }

                            // at this point we've reached the end of the list
                            // since we haven't broken out, it means the last one also contains epsilon
                            // add epsilon to the first set now
                            first_set.insert(Symbol::Epsilon);
                            changes = true;
                        }
                        self.first_sets.insert(symbol.clone(), first_set);
                    }
                    Symbol::EOF => {
                        if !self.first_sets.contains_key(symbol) {
                            self.first_sets.insert(symbol.clone(), BTreeSet::new());
                        }
                        if self.first_sets.get_mut(symbol).unwrap().insert(Symbol::EOF) {
                            changes = true;
                        }
                    }
                    Symbol::Epsilon => {}
                }
            }

            if !changes {
                break;
            }
        }
    }

    // Figure 4.34 of the dragon book
    pub fn build(&mut self) {
        let mut to_add = BTreeSet::new();
        loop {
            for item_set in self.canonical_collection.iter() {
                for X in self.grammar_symbols.values() {
                    let g = self.goto(item_set.clone(), X.clone());
                    if !g.is_empty() && !self.canonical_collection.contains(&g) {
                        to_add.insert(g);
                    }
                }
            }

            if to_add.is_empty() {
                break;
            } else {
                self.canonical_collection.extend(to_add.clone());
                to_add.clear();
            }
        }
    }

    // TODO:
    pub fn parse_table(&self) -> ParseTable {
        let mut states = Vec::new();
        for (i, state) in self.canonical_collection.iter().enumerate() {}
        ParseTable(states)
    }

    fn goto(&self, item_set: BTreeSet<LR0Item>, X: Symbol) -> BTreeSet<LR0Item> {
        let mut valid_items = BTreeSet::new();
        for item in item_set {
            if let Some(next_symbol) = item.name_after_dot() {
                if next_symbol == X.name() {
                    let mut new_item = item.clone();
                    new_item.dot += 1;
                    valid_items.insert(new_item);
                }
            }
        }
        self.closure(valid_items)
    }

    fn closure(&self, mut item_set: BTreeSet<LR0Item>) -> BTreeSet<LR0Item> {
        let mut to_add = BTreeSet::new();
        loop {
            for item in item_set.iter() {
                if let Some(Symbol::NT(next_symbol)) = item.symbol_after_dot() {
                    for production in self
                        .grammar
                        .productions
                        .get(&next_symbol)
                        .expect("this better succeed")
                    {
                        let new_item = LR0Item {
                            lhs: next_symbol.clone(),
                            dot: 0,
                            symbols: production.symbols(&self.grammar_symbols).unwrap(),
                        };
                        if !item_set.contains(&new_item) {
                            to_add.insert(new_item);
                        }
                    }
                }
            }

            if to_add.is_empty() {
                break;
            } else {
                item_set.extend(to_add.clone());
                to_add.clear();
            }
        }
        item_set
    }
}
