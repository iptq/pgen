use crate::Grammar;

pub fn pgen_grammar() -> Grammar {
    let start_symbols = vec!["Expr".to_owned()];
    let terminals = vec![
        ("Number".to_owned(), r"\d+".to_owned()),
        ("Plus".to_owned(), r"\+".to_owned()),
        ("Times".to_owned(), r"\*".to_owned()),
        ("LeftParen".to_owned(), r"\(".to_owned()),
        ("RightParen".to_owned(), r"\)".to_owned()),
    ]
    .into_iter()
    .collect();
    let productions = vec![
        (
            "Expr".to_owned(),
            vec![
                vec!["Expr".to_owned(), "Plus".to_owned(), "Term".to_owned()]
                    .into_iter()
                    .into(),
                vec!["Term".to_owned()].into_iter().into(),
            ],
        ),
        (
            "Term".to_owned(),
            vec![
                vec!["Term".to_owned(), "Times".to_owned(), "Factor".to_owned()]
                    .into_iter()
                    .into(),
                vec!["Factor".to_owned()].into_iter().into(),
            ],
        ),
        (
            "Factor".to_owned(),
            vec![
                vec![
                    "LeftParen".to_owned(),
                    "Expr".to_owned(),
                    "RightParen".to_owned(),
                ]
                .into_iter()
                .into(),
                vec!["Number".to_owned()].into_iter().into(),
            ],
        ),
    ]
    .into_iter()
    .collect();
    Grammar {
        start_symbols,
        terminals,
        productions,
    }
}
