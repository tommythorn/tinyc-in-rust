#![warn(clippy::all, clippy::pedantic)]

/* Copyright (C) 2023 by Tommy Thorn, All Rights Reserved. */

/* Lexer. */

/// Source code is first broken into atomic `Token`s which are the
/// only thing the `Parser` seens.
#[derive(Clone, Debug)]
pub enum Token {
    DoSym,
    ElseSym,
    IfSym,
    WhileSym,
    Lbra,
    Rbra,
    Lpar,
    Rpar,
    Plus,
    Minus,
    Less,
    Semi,
    Equal,
    Int(isize),
    Id(String),
    Eoi,
}

/// Source code position
#[derive(Clone, Copy)]
struct Pos {
    line: usize,
    col: usize,
}

/// The `Lexer` is initialized with the source code string and
/// tokenizes it, keeping the current `Token` in `sym`.  We get a new
/// token by calling `next_sym()`.
///
/// NB: A much nicer lexer would track where in the source we are so
/// syntax errors could be more user friendly.
pub struct Lexer<'a> {
    /// The peekable iterator that gives us chars from the source
    itr: std::iter::Peekable<std::str::Chars<'a>>,

    /// The _current_ position (line, col) in the source
    pos: Pos,

    /// The position of the token in `sym`
    sym_pos: Pos,

    /// The current token (the "lookahead")
    pub sym: Token,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(src: &'a str) -> Lexer<'a> {
        let mut lex = Self {
            itr: src.chars().peekable(),
            pos: Pos { line: 1, col: 1 },
            sym_pos: Pos { line: 0, col: 0 },
            sym: Token::Eoi,
        };

        lex.next_sym();

        lex
    }

    /// Report a error message in the context of the current lexer position and terminate (panic)
    /// # Panics
    /// Sure does
    pub fn syntax_error(&mut self, msg: &str) {
        eprintln!("input:{}:{}:{msg}", self.sym_pos.line, self.sym_pos.col);
        // Proper error handling is out of scope for now.
        std::process::exit(1);
    }

    fn next_ch(&mut self) {
        self.itr.next();
        self.pos.col += 1;
        if self.ch() == '\n' {
            self.pos.line += 1;
            self.pos.col = 1;
        }
    }

    fn ch(&mut self) -> char {
        *self.itr.peek().unwrap_or(&'\0')
    }

    /// Parses the next `Token` and populates `self.sym` with it.
    /// `Token::Eoi` is represents the end of the source code.
    pub fn next_sym(&mut self) {
        while self.ch() == ' ' || self.ch() == '\n' {
            self.next_ch();
        }

        self.sym_pos = self.pos;

        match self.ch() {
            '\0' => self.sym = Token::Eoi,
            '{' => self.sym = Token::Lbra,
            '}' => self.sym = Token::Rbra,
            '(' => self.sym = Token::Lpar,
            ')' => self.sym = Token::Rpar,
            '+' => self.sym = Token::Plus,
            '-' => self.sym = Token::Minus,
            '<' => self.sym = Token::Less,
            ';' => self.sym = Token::Semi,
            '=' => self.sym = Token::Equal,

            '0'..='9' => {
                let mut int_val = 0;
                while '0' <= self.ch() && self.ch() <= '9' {
                    int_val = int_val * 10 + self.ch() as isize - '0' as isize;
                    self.next_ch();
                }

                self.sym = Token::Int(int_val);

                return;
            }

            'a'..='z' => {
                let mut id_name: String = String::new();
                while 'a' <= self.ch() && self.ch() <= 'z' || self.ch() == '_' {
                    id_name.push(self.ch());
                    self.next_ch();
                }

                self.sym = match id_name.as_str() {
                    "do" => Token::DoSym,
                    "else" => Token::ElseSym,
                    "if" => Token::IfSym,
                    "while" => Token::WhileSym,
                    _ => Token::Id(id_name),
                };

                return;
            }

            _ => self.syntax_error("Illegal token"),
        }

        self.next_ch();
    }
}
