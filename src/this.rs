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
        ),* $(,)?}
        $(,)?
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

    // make_grammar! {
    //     start_symbols: [E],
    //     terminals: {
    //         N: r"\d+",
    //         Add: r"\+",
    //         Mul: r"\*",
    //         LP: r"\(",
    //         RP: r"\)",
    //     },
    //     productions: {
    //         // E: [ [T, E_] ],
    //         // E_: [ [Add, T, E_], [ɛ] ],
    //         // T: [ [F, T_] ],
    //         // T_: [ [Mul, F, T_], [ɛ] ],
    //         // F: [ [LP, E, RP], [N] ],
    //         E: [ [E, Add, T], [T], ],
    //         T: [ [T, Mul, F], [F], ],
    //         F: [ [LP, E, RP], [N], ],
    //     },
    // }
}
