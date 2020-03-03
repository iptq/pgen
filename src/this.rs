use crate::Grammar;

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
