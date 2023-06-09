//! The parser for Tiny-C in classic recursive-descent style
//!
//! Normally in C we would use global variables for the current token,
//! source code position, etc, but in Rust we wrap everything in a
//! struct and implement the parser functions as (recursive) member
//! functions.

#![warn(clippy::all, clippy::pedantic)]

use crate::lexer::{Lexer, SourcePosition, Token};

/// To create recursive types in Rust, we heap allocate the recursive
/// subparts, via the `Box` type.  To keep the `Node` type more
/// readable we use an alias for the boxed node.
pub type BNode = Box<Node>;

/// The `Node` is the abstract syntax tree.  This would normally be
/// segregated into the syntatic categories like expression,
/// statement, etc., but for this little example we just bundle
/// everything, forgoing a bit of type safety for brevity.
#[derive(Debug)]
pub enum Node {
    /// Contains the named variable.  Note, cloning the string is a
    /// very expensive operation.  Better would be an index into the
    /// lexers symbol table, or a tricky option, a string slice from
    /// the original source (left as an exercise).
    Var(String),

    /// Contains integer constants
    Cst(isize),

    /// An addition expression
    Add(BNode, BNode),

    /// A subtraction expression
    Sub(BNode, BNode),

    /// A less-than boolean expression
    Lt(BNode, BNode),

    /// The assignment statement.  Note, the first argument must be `Var(_)`.
    Set(BNode, BNode),

    /// An `if` statement with no `else` part.
    If1(BNode, BNode),

    /// An `if` statement with an `else` part.
    If2(BNode, BNode, BNode),

    /// A `while` statement with test and body
    While(BNode, BNode),

    /// A `do-while` statement with body and test
    Do(BNode, BNode),

    /// The null statement, for compiler convenience
    Empty,

    /// The sequence node, which ties together two (or more) statements
    Seq(BNode, BNode),

    /// The expression statement
    Expr(BNode),

    /// The top-level program (there should be exactly one of these)
    Prog(BNode),
}

/// The main entry point to the parser
///
/// ```
/// use tinyc_in_rust::parser::{Node,parse};
/// let ast: Node = parse("q = 42;");
/// ```
#[must_use]
pub fn parse(src: &str) -> Node {
    Parser::new(src).program()
}

/// The `Parser` parses a source string into a `Node` tree
/// representation
struct Parser<'a> {
    lex: Lexer<'a>,
    pos: SourcePosition,
    lookahead: Token,
}

impl<'a> Parser<'a> {
    /// Prepare for parsing, given the provided source code
    fn new(src: &'a str) -> Self {
        let mut parser = Self {
            lex: Lexer::new(src),
            pos: SourcePosition::default(),
            lookahead: Token::default(),
        };
        parser.next_token();
        parser
    }

    /// Takes the next token from the lexer
    fn next_token(&mut self) {
        (self.pos, self.lookahead) = self.lex.get_token();
    }

    /// Parser for the `<term>` syntax
    /// `<term> ::= <id> | <int> | <paren_expr>`
    fn term(&mut self) -> Node {
        match &mut self.lookahead {
            // NB: "std::mem::take(name)" [thanks skeletizzle] is more
            // efficient than the more obvious `name.to_string()`
            Token::Id(name) => {
                let name = std::mem::take(name); // Altn: name.to_string();
                self.next_token();
                Node::Var(name)
            }
            Token::Int(val) => {
                let val = *val;
                self.next_token();
                Node::Cst(val)
            }
            _ => self.paren_expr(),
        }
    }

    /* <sum> ::= <term> | <sum> "+" <term> | <sum> "-" <term> */
    fn sum(&mut self) -> Node {
        let mut t = self.term();
        loop {
            match self.lookahead {
                Token::Plus => {
                    self.next_token();
                    t = Node::Add(Box::new(t), Box::new(self.term()));
                }
                Token::Minus => {
                    self.next_token();
                    t = Node::Sub(Box::new(t), Box::new(self.term()));
                }
                _ => return t,
            }
        }
    }

    /* <test> ::= <sum> | <sum> "<" <sum> */
    fn cond(&mut self) -> Node {
        let l = self.sum();
        if matches!(self.lookahead, Token::Less) {
            self.next_token();
            Node::Lt(Box::new(l), Box::new(self.sum()))
        } else {
            l
        }
    }

    /* <expr> ::= <test> | <id> "=" <expr> */
    fn expr(&mut self) -> Node {
        if !matches!(self.lookahead, Token::Id(_)) {
            return self.cond();
        }
        let t = self.cond(); // == Node::Var(..)
        if matches!(self.lookahead, Token::Equal) {
            self.next_token();
            Node::Set(Box::new(t), Box::new(self.expr()))
        } else {
            t
        }
    }

    fn paren_expr(&mut self) -> Node {
        if !matches!(self.lookahead, Token::Lpar) {
            self.lex.syntax_error(self.pos, "`(' expected");
        }
        self.next_token();
        let x = self.expr();
        if !matches!(self.lookahead, Token::Rpar) {
            self.lex.syntax_error(self.pos, "`)' expected");
        }
        self.next_token();

        x
    }

    fn statement(&mut self) -> Node {
        match self.lookahead {
            Token::IfSym => {
                /* "if" <paren_expr> <statement> */
                self.next_token();
                let cond = self.cond();
                let then = self.statement();
                if matches!(self.lookahead, Token::ElseSym) {
                    /* ... "else" <statement> */
                    self.next_token();
                    Node::If2(Box::new(cond), Box::new(then), Box::new(self.statement()))
                } else {
                    Node::If1(Box::new(cond), Box::new(then))
                }
            }
            Token::WhileSym => {
                /* "while" <paren_expr> <statement> */
                self.next_token();
                let cond = self.paren_expr();
                Node::While(Box::new(cond), Box::new(self.statement()))
            }
            Token::DoSym => {
                /* "do" <statement> "while" <paren_expr> ";" */
                self.next_token();
                let body = self.statement();
                if !matches!(self.lookahead, Token::WhileSym) {
                    self.lex.syntax_error(self.pos, "expected `while'");
                }
                self.next_token();
                let cond = self.paren_expr();
                if !matches!(self.lookahead, Token::Semi) {
                    self.lex.syntax_error(self.pos, "expected `;'");
                }
                self.next_token();
                Node::Do(Box::new(body), Box::new(cond))
            }
            Token::Semi => {
                /* ";" */
                self.next_token();
                Node::Empty
            }
            Token::Lbra => {
                /* "{" { <statement> } "}" */
                self.next_token();
                let mut x = self.statement();
                while !matches!(self.lookahead, Token::Rbra) {
                    x = Node::Seq(Box::new(x), Box::new(self.statement()));
                }
                self.next_token();
                x
            }
            _ => {
                /* <expr> ";" */
                let x = self.expr();
                if !matches!(self.lookahead, Token::Semi) {
                    self.lex.syntax_error(self.pos, "expected `;'");
                }
                self.next_token();
                Node::Expr(Box::new(x))
            }
        }
    }

    fn program(&mut self) -> Node {
        /* <program> ::= <statement> */
        let stmt = self.statement();
        if !matches!(self.lookahead, Token::Eoi) {
            self.lex.syntax_error(self.pos, "program ended here");
        }
        Node::Prog(Box::new(stmt))
    }
}

// *** Parser Testing ***

#[cfg(test)]
use insta::assert_snapshot;

#[test]
fn test_term() {
    let mut parse = Parser::new("2 alpha");
    let n = parse.term();
    assert!(matches!(n, Node::Cst(2)));
    let n = parse.term();
    assert!(match n {
        Node::Var(v) => v == "alpha",
        _ => false,
    });
}

#[test]
fn test_sum() {
    assert_snapshot!(format!("{:?}", Parser::new("2+3-4").sum()));
    assert_snapshot!(format!("{:?}", Parser::new("a-b-c").sum()));
}

#[test]
fn test_cond() {
    assert_snapshot!(format!("{:?}", Parser::new("2 < 4").cond()));
    assert_snapshot!(format!("{:?}", Parser::new("a").cond()));
}

#[test]
fn test_expr() {
    assert_snapshot!(format!("{:?}", Parser::new("2 < 4").expr()));
    assert_snapshot!(format!("{:?}", Parser::new("a = 42 - 666").expr()));
}

#[test]
fn test_paren_expr() {
    assert_snapshot!(format!("{:?}", Parser::new("(2-(3-4))").paren_expr()));
    assert_snapshot!(format!("{:?}", Parser::new(" (x < 7) y;").paren_expr()));
}

#[test]
fn test_statement() {
    assert_snapshot!(format!("{:?}", Parser::new(";").statement()));
    assert_snapshot!(format!("{:?}", Parser::new("a;").statement()));
    assert_snapshot!(format!(
        "{:?}",
        Parser::new("if (2 < 3) b = 42;").statement()
    ));
}

#[test]
fn test_statement2() {
    assert_snapshot!(format!(
        "{:?}",
        Parser::new("if (2) b = 42; else b = 666;").statement()
    ));
}

#[test]
fn test_statement3() {
    assert_snapshot!(format!(
        "{:?}",
        Parser::new("{ b = 666; c = 3; d = b; }").statement()
    ));
}

#[test]
fn test_statement4() {
    assert_snapshot!(format!("{:?}", Parser::new("while (x < 7) y;").statement()));
    assert_snapshot!(format!(
        "{:?}",
        Parser::new("while (x < 7) { b = b - 1; c = c + b; }").statement()
    ));
}

#[test]
fn test_program() {
    assert_snapshot!(format!("{:?}", Parser::new("a = 42;").program()));
}
