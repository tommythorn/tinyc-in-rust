//! The lexical analysis for Tiny-C (henceforth the "lexer")
//!
//! The lexer is responsible for taking the source code as a sequence
//! of characters and turn it into a sequence of token which the
//! parser consumes.

#![warn(clippy::all, clippy::pedantic)]

/// The tokens are keywards, special characters, integer constants,
/// and identifiers.  Strong types are really helpful here.  Note, in
/// contrast to typical C implementations, the integer value and the
/// identifier string is strongly tied to the corresponding token.
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

/// Source code position for syntax error reporting.  Both are 1-based
/// (ie. the starting position is (1,1).  Note, the implementation
/// below only accepts spaces and tabs as whitespace.  No tabs nor
/// comments.
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

    /// The source code position of the peekable character
    pos: Pos,

    /// The starting position of the token in `sym`
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

    /// Report a error message in the context of the current lexer
    /// position and terminate
    pub fn syntax_error(&mut self, msg: &str) {
        eprintln!("input:{}:{}:{msg}", self.sym_pos.line, self.sym_pos.col);
        // Proper error handling is out of scope for now.
        std::process::exit(1);
    }

    /// Consumes the current character and advances to the next,
    /// updating the current position in the process
    fn next_ch(&mut self) {
        self.itr.next();
        self.pos.col += 1;
        if self.ch() == '\n' {
            self.pos.line += 1;
            self.pos.col = 1;
        }
    }

    /// Convenient access to the current position.  We turn
    /// end-of-file into the null ('\0') charater which isn't the
    /// usual Rust approach (which would use Option<> types), however
    /// this make the code a little simpler and follows the original
    /// more closely.
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

                // As we have already advanced past the current we
                // return to skip the next_ch() below.
                return;
            }

            'a'..='z' => {
                let mut id_name: String = String::new();
                while 'a' <= self.ch() && self.ch() <= 'z' || self.ch() == '_' {
                    id_name.push(self.ch());
                    self.next_ch();
                }

                // Note, a more conventional approach would use a hash
                // table for the symbol table and store the keywords
                // there along with source code symbols.
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
