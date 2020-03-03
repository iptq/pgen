use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::Write;

use indexmap::IndexMap;
use symbol::Symbol as Id;

use crate::items::LR0Item;
use crate::parser::{Action, ParseTable};
use crate::Parser;

#[derive(Debug, Error)]
pub enum GrammarError {
    #[error("Name conflict: {0}")]
    NameConflict(Id),

    #[error("Invalid symbol: {0}")]
    InvalidSymbol(Id),

    #[error("Start symbols must be nonterminals: {0}")]
    StartingTerminal(Id),
}

#[derive(Debug)]
pub struct Grammar {
    pub(crate) start_symbols: Vec<Id>,
    pub(crate) terminals: IndexMap<Id, String>,
    pub(crate) productions: IndexMap<Id, Vec<Production>>,
}

impl Grammar {
    fn create_grammar_helper(&self) -> Result<GrammarHelper, GrammarError> {
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
        let mut counter = 0;
        let mut grammar_productions = HashMap::new();
        let items = {
            let mut items = Vec::new();
            for (nonterminal, productions) in self.productions.iter() {
                grammar_productions.insert(nonterminal.clone(), Vec::new());
                for production in productions {
                    grammar_productions
                        .get_mut(nonterminal)
                        .unwrap()
                        .push((counter, production.clone()));
                    items.push(LR0Item {
                        lhs: nonterminal.clone(),
                        dot: 0,
                        symbols: production.symbols(&grammar_symbols)?,
                        is_start: false,
                        production_number: Some(counter),
                    });
                    counter += 1;
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

        let grammar_helper = GrammarHelper {
            grammar: &self,
            grammar_symbols,
            canonical_collection: BTreeSet::new(),
            first_sets: BTreeMap::new(),
            follow_sets: BTreeMap::new(),
            productions: grammar_productions,
        };

        Ok(grammar_helper)
    }

    /// Builds the main Parser struct.
    pub fn build(self) -> Result<Parser, GrammarError> {
        let mut grammar_helper = self.create_grammar_helper()?;
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

        let table = grammar_helper.parse_table();

        Ok(Parser {
            start_symbols: self.start_symbols,
            terminals: self.terminals,
            nonterminals: self.productions.keys().cloned().collect::<HashSet<Id>>(),
            // productions: grammar_productions,
            table,
        })
    }
}

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum Symbol {
    T(Id),
    NT(Id),
    Epsilon,
    EOF,
}

impl Symbol {
    pub fn name(&self) -> Id {
        match self {
            Symbol::T(name) | Symbol::NT(name) => *name,
            Symbol::Epsilon => Id::from("\u{025b}"),
            Symbol::EOF => Id::from("$"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Production(pub(crate) Vec<Id>);

impl<T: Iterator<Item = Id>> From<T> for Production {
    fn from(iter: T) -> Self {
        Production(iter.collect())
    }
}

impl Production {
    pub fn symbols(
        &self,
        grammar_symbols: &HashMap<Id, Symbol>,
    ) -> Result<Vec<Symbol>, GrammarError> {
        let mut symbols = Vec::new();
        for symbol_candidate in self.0.iter() {
            if symbol_candidate.as_str() == "\u{025b}" {
                symbols.push(Symbol::Epsilon);
            } else if let Some(symbol) = grammar_symbols.get(symbol_candidate) {
                symbols.push(symbol.clone());
            } else {
                return Err(GrammarError::InvalidSymbol(symbol_candidate.clone()));
            }
        }
        Ok(symbols)
    }
}

#[derive(Debug)]
struct GrammarHelper<'a> {
    /// The reference to the actual grammar
    grammar: &'a Grammar,

    /// This is a map from the name to the actual symbol
    grammar_symbols: HashMap<Id, Symbol>,

    /// A map from the name to the index of the production
    productions: HashMap<Id, Vec<(usize, Production)>>,
    canonical_collection: BTreeSet<BTreeSet<LR0Item>>,
    first_sets: BTreeMap<Symbol, BTreeSet<Symbol>>,
    follow_sets: BTreeMap<Symbol, BTreeSet<Symbol>>,
}

impl<'a> GrammarHelper<'a> {
    pub fn init(&mut self, start_symbols: Vec<Id>) {
        self.compute_first_sets();
        self.compute_follow_sets();
        // TODO: predict sets?

        println!("First sets:");
        for (sym, set) in self.first_sets.iter() {
            println!(" {:?} - {:?}", sym, set);
        }
        println!("");

        println!("Follow sets:");
        for (sym, set) in self.follow_sets.iter() {
            println!(" {:?} - {:?}", sym, set);
        }
        println!("");

        for start_symbol in start_symbols {
            let mut new_set = BTreeSet::new();
            let new_item = LR0Item {
                lhs: Id::from(format!("{}'", start_symbol)),
                dot: 0,
                symbols: vec![Symbol::NT(start_symbol)],
                is_start: true,
                production_number: None,
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
                            self.first_sets.insert(symbol.clone(), BTreeSet::new());
                        }
                        let mut first_set = self.first_sets.get_mut(symbol).unwrap().clone();
                        'outer: for production in self.grammar.productions.get(name).unwrap() {
                            let symbol_list = production.symbols(&self.grammar_symbols).unwrap();
                            for yi in symbol_list {
                                if yi == Symbol::Epsilon {
                                    if first_set.insert(Symbol::Epsilon) {
                                        changes = true;
                                    }
                                }

                                if let Some(sym_first_set) = self.first_sets.get(&yi) {
                                    // if there's extra elements in FIRST(Yi)
                                    if !first_set.is_superset(sym_first_set) {
                                        first_set =
                                            first_set.union(sym_first_set).cloned().collect();
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

    #[allow(non_snake_case)]
    fn compute_follow_sets(&mut self) {
        for start_symbol in self.grammar.start_symbols.iter() {
            let start_symbol = Symbol::NT(start_symbol.to_owned());
            if !self.follow_sets.contains_key(&start_symbol) {
                self.follow_sets
                    .insert(start_symbol.clone(), BTreeSet::new());
            }
            // put $ in FOLLOW(S) for every start symbol
            self.follow_sets
                .get_mut(&start_symbol)
                .unwrap()
                .insert(Symbol::EOF);
        }

        for nonterminal in self.grammar.productions.keys() {
            let nonterminal = Symbol::NT(nonterminal.to_owned());
            if !self.follow_sets.contains_key(&nonterminal) {
                self.follow_sets
                    .insert(nonterminal.clone(), BTreeSet::new());
            }
        }

        loop {
            let mut changes = false;

            for (nonterminal, productions) in self.grammar.productions.iter() {
                let nonterminal = Symbol::NT(nonterminal.to_owned());
                for production in productions {
                    let symbol_list = production.symbols(&self.grammar_symbols).unwrap();
                    // if A -> aBb, then take all of {FIRST(b) - e} and add it to FOLLOW(B)
                    if symbol_list.len() >= 2 {
                        for window in symbol_list.windows(2) {
                            let B = &window[0];
                            let b = &window[1];

                            let mut b_first_set = self.first_sets.get(b).unwrap().clone();
                            b_first_set.remove(&Symbol::Epsilon);

                            if let Some(follow_set) = self.follow_sets.get_mut(&B) {
                                if !follow_set.is_superset(&b_first_set) {
                                    follow_set.append(&mut b_first_set);
                                    changes = true;
                                }
                            }
                        }
                    }

                    // if A -> aB, then everythign in FOLLOW(A) is in FOLLOW(B)
                    if symbol_list.len() >= 1 {
                        for window in symbol_list.windows(2) {
                            let B = &window[1];

                            let mut a_follow_set =
                                self.follow_sets.get_mut(&nonterminal).unwrap().clone();
                            if let Some(follow_set) = self.follow_sets.get_mut(&B) {
                                if !follow_set.is_superset(&a_follow_set) {
                                    follow_set.append(&mut a_follow_set);
                                    changes = true;
                                }
                            }
                        }
                    }

                    // if A -> aBb, and FIRST(b) contains epsilon, then everything in FOLLOW(A) is in FOLLOW(B)
                    if symbol_list.len() >= 2 {
                        let b = &symbol_list[symbol_list.len() - 1];
                        let B = &symbol_list[symbol_list.len() - 2];
                        let b_first_set = self.first_sets.get(b).unwrap().clone();
                        if b_first_set.contains(&Symbol::Epsilon) {
                            let mut a_follow_set =
                                self.follow_sets.get_mut(&nonterminal).unwrap().clone();
                            if let Some(follow_set) = self.follow_sets.get_mut(&B) {
                                if !follow_set.is_superset(&a_follow_set) {
                                    follow_set.append(&mut a_follow_set);
                                    changes = true;
                                }
                            }
                        }
                    }
                }
            }

            if !changes {
                break;
            }
        }
    }

    // Figure 4.34 of the dragon book
    #[allow(non_snake_case)]
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

    /// Converts this GrammarHelper instance into a ParseTable.
    pub fn parse_table(&self) -> ParseTable {
        let mut states = Vec::new();
        let reverse_map: BTreeMap<_, _> = self
            .canonical_collection
            .iter()
            .enumerate()
            .map(|(a, b)| (b, a))
            .collect();
        for (i, item_set) in self.canonical_collection.iter().enumerate() {
            let mut action = HashMap::new();
            let mut goto = HashMap::new();
            for item in item_set {
                if let Some(next_symbol) = item.symbol_after_dot() {
                    let g = self.goto(item_set.clone(), next_symbol.clone());
                    if let Some(num) = reverse_map.get(&g) {
                        action.insert(next_symbol, Action::Shift(*num));
                    }
                }

                if item.dot_at_end() {
                    if item.is_start {
                        action.insert(Symbol::EOF, Action::Accept);
                    } else {
                        let prev_symbol = item.symbol_before_dot().unwrap();
                        if let Some(n) = item.production_number {
                            action.insert(prev_symbol, Action::Reduce(n));
                        }
                    }
                }

                for nonterminal in self.grammar.productions.keys() {
                    let nterm = Symbol::NT(nonterminal.to_owned());
                    let g = self.goto(item_set.clone(), nterm.clone());
                    if let Some(num) = reverse_map.get(&g) {
                        goto.insert(nterm, *num);
                    }
                }
            }
            states.push((action, goto));
        }
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
                        .productions
                        .get(&next_symbol)
                        .expect("this better succeed")
                    {
                        let (counter, production) = production;
                        let new_item = LR0Item {
                            lhs: next_symbol.clone(),
                            dot: 0,
                            symbols: production.symbols(&self.grammar_symbols).unwrap(),
                            is_start: false,
                            production_number: Some(*counter),
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

#[cfg(test)]
mod tests {
    use super::Grammar;
    use super::Symbol;
    use std::collections::{BTreeMap, BTreeSet};
    use symbol::Symbol as Id;

    fn remove_terminals<V: Clone>(map: &BTreeMap<Symbol, V>) -> BTreeMap<Symbol, V> {
        let mut map = map.clone();
        let mut to_remove = BTreeSet::new();
        for key in map.keys() {
            if let Symbol::T(_) = key {
                to_remove.insert(key.clone());
            }
        }
        for key in to_remove {
            map.remove(&key);
        }
        map
    }

    fn make_arith_1() -> Grammar {
        make_grammar! {
            start_symbols: [E],
            terminals: {
                Add: r"\+",
                Mul: r"\*",
                N0: r"0",
                N1: r"1",
            },
            productions: {
                E: [ [E, Mul, B], [E, Add, B], [B] ],
                B: [ [N0], [N1] ],
            }
        }
    }

    #[test]
    fn test_arith_1() {
        use super::Symbol::*;
        let grammar = make_arith_1();
        let mut helper = grammar.create_grammar_helper().unwrap();

        // First sets
        helper.compute_first_sets();
        let actual_first_sets = remove_terminals(&helper.first_sets);
        let expected_first_sets = btreemap! {
            NT(Id::from("E")) => btreeset!{ T(Id::from("N0")), T(Id::from("N1")), },
            NT(Id::from("B")) => btreeset!{ T(Id::from("N0")), T(Id::from("N1")), },
        };
        assert!(
            actual_first_sets.iter().eq(expected_first_sets.iter()),
            "Expected: {:?}, Got: {:?}",
            expected_first_sets,
            actual_first_sets
        );

        // Follow sets
        helper.compute_follow_sets();
        let actual_follow_sets = remove_terminals(&helper.follow_sets);
        let expected_follow_sets = btreemap! {
            NT(Id::from("E")) => btreeset!{ T(Id::from("Add")), T(Id::from("Mul")), EOF, },
            NT(Id::from("B")) => btreeset!{ T(Id::from("Add")), T(Id::from("Mul")), EOF, },
        };
        assert!(
            actual_follow_sets.iter().eq(expected_follow_sets.iter()),
            "Expected: {:?}, Got: {:?}",
            expected_follow_sets,
            actual_follow_sets
        );
    }
}
