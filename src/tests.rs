#![warn(clippy::all, clippy::pedantic)]
use crate::codegen::compile;
use crate::lexer::{Lexer, Token};
use crate::parser::parse;
use insta::assert_snapshot;

// *** Lexer Testing ***

#[test]
fn test_lexer() {
    let mut lex = Lexer::new("2 3 alpha beta ={}");
    assert!(matches!(lex.sym, Token::Int(2)));
    lex.next_sym();
    assert!(matches!(lex.sym, Token::Int(3)));
    lex.next_sym();
    assert!(match &lex.sym {
        Token::Id(v) => v == "alpha",
        _ => false,
    });
    lex.next_sym();
    assert!(match &lex.sym {
        Token::Id(v) => v == "beta",
        _ => false,
    });
    lex.next_sym();
    assert!(matches!(lex.sym, Token::Equal));
    lex.next_sym();
    assert!(matches!(lex.sym, Token::Lbra));
    lex.next_sym();
    assert!(matches!(lex.sym, Token::Rbra));
    lex.next_sym();
    assert!(matches!(lex.sym, Token::Eoi));
    lex.next_sym();
    assert!(matches!(lex.sym, Token::Eoi));
}

// *** Compiler Testing ***

fn show_code(src: &str) -> String {
    format!("{:?}", compile(parse(src)))
}

const EXAMPLES: [&str; 7] = [
    "a=b=c=2<3;",
    "{ i=1; while (i<100) i=i+i; }",
    "{ i=125; j=100; while (i-j) if (i<j) j=j-i; else i=i-j; }",
    "{ i=1; do i=i+10; while (i<50); }",
    "{ i=1; while ((i=i+10)<50) ; }",
    "{ i=7; if (i<5) x=1; if (i<10) y=2; }",
    "{ m=n=1;k=10; while (0 < k) { t = m; m = n; n = t + n; k = k - 1; }}",
];

#[test]
fn test_cg_assignment() {
    assert_snapshot!(show_code("a = 42;"));
}

#[test]
fn test_cg_examples() {
    for ex in EXAMPLES {
        assert_snapshot!(show_code(ex));
    }
}

// *** Execution Testing ***

#[test]
fn test_run_examples() {
    for ex in EXAMPLES {
        println!("Try {ex}:");
        crate::vm::VM::new().run(compile(parse(ex)));
    }
}
