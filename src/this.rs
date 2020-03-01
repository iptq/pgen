use symbol::Symbol as Id;

use crate::Grammar;

macro_rules! make_grammar {
    {
        start_symbols: [$($start_symbol:ident),* $(,)?],
        terminals: {
            $($tname:ident: $regex:expr),* $(,)?
        },
        productions: {$(
            $ntname:ident: [$(
                [$($symbol:ident),* $(,)?]
            ),* $(,)?]
        ),* $(,)?},
    } => {
        Grammar {
            start_symbols: vec![$(Id::from(stringify!($start_symbol)))*],
            terminals: vec![$((Id::from(stringify!($tname)), $regex.to_owned()),)*].into_iter().collect(),
            productions: vec![$(
                (Id::from(stringify!($ntname)), vec![
                    $(vec![$(Id::from(stringify!($symbol)),)*].into_iter().into(),)*
                ]),
            )*].into_iter().collect(),
        }
    }
}

pub fn pgen_grammar() -> Grammar {
    make_grammar! {
        start_symbols: [E],
        terminals: {
            N: r"\d+",
            Add: r"\+",
            Mul: r"\*",
            LP: r"\(",
            RP: r"\)",
        },
        productions: {
            E: [ [T, E_] ],
            E_: [ [Add, T, E_], [ɛ] ],
            T: [ [F, T_] ],
            T_: [ [Mul, F, T_], [ɛ] ],
            F: [ [LP, E, RP], [N] ],
        },
    }
}
