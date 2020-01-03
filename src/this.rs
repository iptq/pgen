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
            start_symbols: vec![$(stringify!($start_symbol).to_owned())*],
            terminals: vec![$((stringify!($tname).to_owned(), $regex.to_owned()),)*].into_iter().collect(),
            productions: vec![$(
                (stringify!($ntname).to_owned(), vec![
                    $(vec![$(stringify!($symbol).to_owned(),)*].into_iter().into(),)*
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
            E_: [ [Add, T, E_], [e] ],
            T: [ [F, T_] ],
            T_: [ [Mul, F, T_], [e] ],
            F: [ [LP, E, RP], [N] ],
        },
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
    //         E: [
    //             [E, Add, T],
    //             [T],
    //         ],
    //         T: [
    //             [Term, Mul, F],
    //             [F],
    //         ],
    //         F: [
    //             [LP, E, RP],
    //             [N],
    //         ],
    //     },
    // }
}
