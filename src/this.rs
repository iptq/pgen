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
        start_symbols: [Expr],
        terminals: {
            Number: r"\d+",
            Plus: r"\+",
            Times: r"\*",
            LeftParen: r"\(",
            RightParen: r"\)",
        },
        productions: {
            Expr: [
                [Expr, Plus, Term],
                [Term],
            ],
            Term: [
                [Term, Times, Factor],
                [Factor],
            ],
            Factor: [
                [LeftParen, Expr, RightParen],
                [Number],
            ],
        },
    }
}
