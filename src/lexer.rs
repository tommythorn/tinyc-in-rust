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
#[derive(Debug, Default)]
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
    #[default]
    Eoi,
}

/// Source code position for syntax error reporting.  Both are 1-based
/// (ie. the starting position is (1,1).  Note, the implementation
/// below only accepts spaces and tabs as whitespace.  No tabs nor
/// comments.
#[derive(Clone, Copy, Default, Debug)]
pub struct SourcePosition {
    line: usize,
    col: usize,
}

/// The `Lexer` is initialized with the source code string and
/// tokenizes it `get_token()`.
pub struct Lexer<'a> {
    /// The peekable iterator that gives us chars from the source
    itr: std::iter::Peekable<std::str::Chars<'a>>,

    /// The source code position of the peekable character
    pos: SourcePosition,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(src: &'a str) -> Lexer<'a> {
        Self {
            itr: src.chars().peekable(),
            pos: SourcePosition { line: 1, col: 1 },
        }
    }

    /// Report a error message in the context of the current lexer
    /// position and terminate
    pub fn syntax_error(&mut self, pos: SourcePosition, msg: &str) -> ! {
        eprintln!("input:{}:{}:{msg}", pos.line, pos.col);
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
    pub fn get_token(&mut self) -> (SourcePosition, Token) {
        while self.ch() == ' ' || self.ch() == '\n' {
            self.next_ch();
        }

        let pos: SourcePosition = self.pos;
        let token = match self.ch() {
            '\0' => Token::Eoi,
            '{' => Token::Lbra,
            '}' => Token::Rbra,
            '(' => Token::Lpar,
            ')' => Token::Rpar,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '<' => Token::Less,
            ';' => Token::Semi,
            '=' => Token::Equal,

            '0'..='9' => {
                let mut int_val = 0;
                while '0' <= self.ch() && self.ch() <= '9' {
                    int_val = int_val * 10 + self.ch() as isize - '0' as isize;
                    self.next_ch();
                }

                // As we have already advanced past the current we
                // return to skip the next_ch() below.
                return (pos, Token::Int(int_val));
            }

            'a'..='z' => {
                let mut id_name = String::new();
                while 'a' <= self.ch() && self.ch() <= 'z' || self.ch() == '_' {
                    id_name.push(self.ch());
                    self.next_ch();
                }

                // Note, a more conventional approach would use a hash
                // table for the symbol table and store the keywords
                // there along with source code symbols.
                return (
                    pos,
                    match id_name.as_str() {
                        "do" => Token::DoSym,
                        "else" => Token::ElseSym,
                        "if" => Token::IfSym,
                        "while" => Token::WhileSym,
                        _ => Token::Id(id_name),
                    },
                );
            }

            _ => self.syntax_error(pos, "Illegal token"),
        };

        self.next_ch();

        (pos, token)
    }
}
