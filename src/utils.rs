#[macro_export]
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
            start_symbols: vec![$(symbol::Symbol::from(stringify!($start_symbol)))*],
            terminals: vec![$((symbol::Symbol::from(stringify!($tname)), $regex.to_owned()),)*].into_iter().collect(),
            productions: vec![$(
                (symbol::Symbol::from(stringify!($ntname)), vec![
                    $(vec![$(symbol::Symbol::from(stringify!($symbol)),)*].into_iter().into(),)*
                ]),
            )*].into_iter().collect(),
        }
    }
}
