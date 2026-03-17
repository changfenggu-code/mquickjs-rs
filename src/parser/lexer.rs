//! JavaScript lexer/tokenizer
//!
//! Converts source text into a stream of tokens.

use alloc::{format, string::String, string::ToString};

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Ident(String),
    RegExp { pattern: String, flags: String },

    // Operators and punctuation
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    StarStar,   // **
    PlusPlus,   // ++
    MinusMinus, // --

    Eq,       // =
    EqEq,     // ==
    EqEqEq,   // ===
    Bang,     // !
    BangEq,   // !=
    BangEqEq, // !==

    Lt,   // <
    LtEq, // <=
    Gt,   // >
    GtEq, // >=

    LtLt,   // <<
    GtGt,   // >>
    GtGtGt, // >>>

    Amp,      // &
    AmpAmp,   // &&
    Pipe,     // |
    PipePipe, // ||
    Caret,    // ^
    Tilde,    // ~

    Question,  // ?
    Colon,     // :
    Semicolon, // ;
    Comma,     // ,
    Dot,       // .

    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    RBrace,   // }

    // Compound assignment
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    StarStarEq,
    LtLtEq,
    GtGtEq,
    GtGtGtEq,
    AmpEq,
    PipeEq,
    CaretEq,

    // Keywords
    Break,
    Case,
    Catch,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    False,
    Finally,
    For,
    Function,
    If,
    In,
    InstanceOf,
    New,
    Null,
    Of,
    Return,
    Switch,
    This,
    Throw,
    True,
    Try,
    TypeOf,
    Var,
    Void,
    While,

    // ES6+ keywords (limited support in stricter mode)
    Const,
    Let,

    // Built-in statements
    Print,

    // Special
    Eof,
    Error(String),
}

/// Source position
#[derive(Debug, Clone, Copy, Default)]
pub struct SourcePos {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

/// Lexer for JavaScript source code
#[derive(Clone)]
pub struct Lexer<'a> {
    source: &'a [u8],
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source: source.as_bytes(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Get the current source position
    pub fn position(&self) -> SourcePos {
        SourcePos {
            offset: self.pos,
            line: self.line,
            column: self.column,
        }
    }

    /// Peek at the current character without consuming it
    fn peek(&self) -> Option<u8> {
        self.source.get(self.pos).copied()
    }

    /// Peek at the next character
    fn peek_next(&self) -> Option<u8> {
        self.source.get(self.pos + 1).copied()
    }

    /// Consume the current character
    fn advance(&mut self) -> Option<u8> {
        let c = self.peek()?;
        self.pos += 1;
        if c == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    /// Skip whitespace and comments
    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\r' | b'\n') => {
                    self.advance();
                }
                Some(b'/') if self.peek_next() == Some(b'/') => {
                    // Line comment
                    while let Some(c) = self.advance() {
                        if c == b'\n' {
                            break;
                        }
                    }
                }
                Some(b'/') if self.peek_next() == Some(b'*') => {
                    // Block comment
                    self.advance(); // /
                    self.advance(); // *
                    while let Some(c) = self.advance() {
                        if c == b'*' && self.peek() == Some(b'/') {
                            self.advance();
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    /// Read the next token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let Some(c) = self.peek() else {
            return Token::Eof;
        };

        // Identifiers and keywords
        if c.is_ascii_alphabetic() || c == b'_' || c == b'$' {
            return self.read_identifier();
        }

        // Numbers
        if c.is_ascii_digit() {
            return self.read_number();
        }

        // Strings
        if c == b'"' || c == b'\'' {
            return self.read_string();
        }

        // Operators and punctuation
        self.advance();
        match c {
            b'+' => match self.peek() {
                Some(b'+') => {
                    self.advance();
                    Token::PlusPlus
                }
                Some(b'=') => {
                    self.advance();
                    Token::PlusEq
                }
                _ => Token::Plus,
            },
            b'-' => match self.peek() {
                Some(b'-') => {
                    self.advance();
                    Token::MinusMinus
                }
                Some(b'=') => {
                    self.advance();
                    Token::MinusEq
                }
                _ => Token::Minus,
            },
            b'*' => match self.peek() {
                Some(b'*') => {
                    self.advance();
                    if self.peek() == Some(b'=') {
                        self.advance();
                        Token::StarStarEq
                    } else {
                        Token::StarStar
                    }
                }
                Some(b'=') => {
                    self.advance();
                    Token::StarEq
                }
                _ => Token::Star,
            },
            b'/' => match self.peek() {
                Some(b'=') => {
                    self.advance();
                    Token::SlashEq
                }
                _ => Token::Slash,
            },
            b'%' => match self.peek() {
                Some(b'=') => {
                    self.advance();
                    Token::PercentEq
                }
                _ => Token::Percent,
            },
            b'=' => match self.peek() {
                Some(b'=') => {
                    self.advance();
                    if self.peek() == Some(b'=') {
                        self.advance();
                        Token::EqEqEq
                    } else {
                        Token::EqEq
                    }
                }
                _ => Token::Eq,
            },
            b'!' => match self.peek() {
                Some(b'=') => {
                    self.advance();
                    if self.peek() == Some(b'=') {
                        self.advance();
                        Token::BangEqEq
                    } else {
                        Token::BangEq
                    }
                }
                _ => Token::Bang,
            },
            b'<' => match self.peek() {
                Some(b'<') => {
                    self.advance();
                    if self.peek() == Some(b'=') {
                        self.advance();
                        Token::LtLtEq
                    } else {
                        Token::LtLt
                    }
                }
                Some(b'=') => {
                    self.advance();
                    Token::LtEq
                }
                _ => Token::Lt,
            },
            b'>' => match self.peek() {
                Some(b'>') => {
                    self.advance();
                    match self.peek() {
                        Some(b'>') => {
                            self.advance();
                            if self.peek() == Some(b'=') {
                                self.advance();
                                Token::GtGtGtEq
                            } else {
                                Token::GtGtGt
                            }
                        }
                        Some(b'=') => {
                            self.advance();
                            Token::GtGtEq
                        }
                        _ => Token::GtGt,
                    }
                }
                Some(b'=') => {
                    self.advance();
                    Token::GtEq
                }
                _ => Token::Gt,
            },
            b'&' => match self.peek() {
                Some(b'&') => {
                    self.advance();
                    Token::AmpAmp
                }
                Some(b'=') => {
                    self.advance();
                    Token::AmpEq
                }
                _ => Token::Amp,
            },
            b'|' => match self.peek() {
                Some(b'|') => {
                    self.advance();
                    Token::PipePipe
                }
                Some(b'=') => {
                    self.advance();
                    Token::PipeEq
                }
                _ => Token::Pipe,
            },
            b'^' => match self.peek() {
                Some(b'=') => {
                    self.advance();
                    Token::CaretEq
                }
                _ => Token::Caret,
            },
            b'~' => Token::Tilde,
            b'?' => Token::Question,
            b':' => Token::Colon,
            b';' => Token::Semicolon,
            b',' => Token::Comma,
            b'.' => {
                // Leading decimal: .5 → 0.5
                if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    return self.read_leading_decimal();
                }
                Token::Dot
            }
            b'(' => Token::LParen,
            b')' => Token::RParen,
            b'[' => Token::LBracket,
            b']' => Token::RBracket,
            b'{' => Token::LBrace,
            b'}' => Token::RBrace,
            _ => Token::Error(format!("Unexpected character: {}", c as char)),
        }
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> Token {
        let start = self.pos;

        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
                self.advance();
            } else {
                break;
            }
        }

        let ident = core::str::from_utf8(&self.source[start..self.pos])
            .unwrap_or("")
            .to_string();

        // Check for keywords
        match ident.as_str() {
            "break" => Token::Break,
            "case" => Token::Case,
            "catch" => Token::Catch,
            "const" => Token::Const,
            "continue" => Token::Continue,
            "debugger" => Token::Debugger,
            "default" => Token::Default,
            "delete" => Token::Delete,
            "do" => Token::Do,
            "else" => Token::Else,
            "false" => Token::False,
            "finally" => Token::Finally,
            "for" => Token::For,
            "function" => Token::Function,
            "if" => Token::If,
            "in" => Token::In,
            "instanceof" => Token::InstanceOf,
            "let" => Token::Let,
            "new" => Token::New,
            "null" => Token::Null,
            "of" => Token::Of,
            "print" => Token::Print,
            "return" => Token::Return,
            "switch" => Token::Switch,
            "this" => Token::This,
            "throw" => Token::Throw,
            "true" => Token::True,
            "try" => Token::Try,
            "typeof" => Token::TypeOf,
            "var" => Token::Var,
            "void" => Token::Void,
            "while" => Token::While,
            _ => Token::Ident(ident),
        }
    }

    /// Read a number literal (decimal, hex 0x, octal 0o, binary 0b)
    fn read_number(&mut self) -> Token {
        let start = self.pos;

        // Check for 0x, 0o, 0b prefixes
        if self.peek() == Some(b'0') {
            self.advance();
            match self.peek() {
                Some(b'x' | b'X') => {
                    self.advance();
                    if !matches!(self.peek(), Some(c) if c.is_ascii_hexdigit()) {
                        return Token::Error("Invalid hex literal".to_string());
                    }
                    while let Some(c) = self.peek() {
                        if c.is_ascii_hexdigit() {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let hex_str =
                        core::str::from_utf8(&self.source[start + 2..self.pos]).unwrap_or("0");
                    return match i64::from_str_radix(hex_str, 16) {
                        Ok(n) => Token::Number(n as f64),
                        Err(_) => Token::Error(format!("Invalid hex literal: 0x{}", hex_str)),
                    };
                }
                Some(b'o' | b'O') => {
                    self.advance();
                    if !matches!(self.peek(), Some(b'0'..=b'7')) {
                        return Token::Error("Invalid octal literal".to_string());
                    }
                    while let Some(c) = self.peek() {
                        if matches!(c, b'0'..=b'7') {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let oct_str =
                        core::str::from_utf8(&self.source[start + 2..self.pos]).unwrap_or("0");
                    return match i64::from_str_radix(oct_str, 8) {
                        Ok(n) => Token::Number(n as f64),
                        Err(_) => Token::Error(format!("Invalid octal literal: 0o{}", oct_str)),
                    };
                }
                Some(b'b' | b'B') => {
                    self.advance();
                    if !matches!(self.peek(), Some(b'0' | b'1')) {
                        return Token::Error("Invalid binary literal".to_string());
                    }
                    while let Some(c) = self.peek() {
                        if matches!(c, b'0' | b'1') {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let bin_str =
                        core::str::from_utf8(&self.source[start + 2..self.pos]).unwrap_or("0");
                    return match i64::from_str_radix(bin_str, 2) {
                        Ok(n) => Token::Number(n as f64),
                        Err(_) => Token::Error(format!("Invalid binary literal: 0b{}", bin_str)),
                    };
                }
                _ => {
                    // Plain 0 or 0.xxx — continue to decimal parsing below
                }
            }
        }

        // Integer part (for non-prefix numbers, or the leading 0 already consumed)
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        // Decimal part
        if self.peek() == Some(b'.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            self.advance(); // .
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Exponent part
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.advance();
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.advance();
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let num_str = core::str::from_utf8(&self.source[start..self.pos]).unwrap_or("0");
        match num_str.parse::<f64>() {
            Ok(n) => Token::Number(n),
            Err(_) => Token::Error(format!("Invalid number: {}", num_str)),
        }
    }

    /// Read a number with leading decimal point (e.g., .5 → 0.5)
    fn read_leading_decimal(&mut self) -> Token {
        let start = self.pos - 1; // include the '.' already consumed
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        // Exponent part
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.advance();
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.advance();
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        let num_str = core::str::from_utf8(&self.source[start..self.pos]).unwrap_or("0");
        match num_str.parse::<f64>() {
            Ok(n) => Token::Number(n),
            Err(_) => Token::Error(format!("Invalid number: {}", num_str)),
        }
    }

    /// Read a string literal
    fn read_string(&mut self) -> Token {
        let quote = self.advance().unwrap();
        let mut s = String::new();

        loop {
            match self.peek() {
                None => return Token::Error("Unterminated string".to_string()),
                Some(c) if c == quote => {
                    self.advance();
                    break;
                }
                Some(b'\\') => {
                    self.advance();
                    match self.advance() {
                        Some(b'n') => s.push('\n'),
                        Some(b'r') => s.push('\r'),
                        Some(b't') => s.push('\t'),
                        Some(b'v') => s.push('\x0B'),
                        Some(b'b') => s.push('\x08'),
                        Some(b'f') => s.push('\x0C'),
                        Some(b'0') => s.push('\0'),
                        Some(b'\\') => s.push('\\'),
                        Some(b'\'') => s.push('\''),
                        Some(b'"') => s.push('"'),
                        Some(b'x') => {
                            // \xHH hex escape
                            let mut hex = [0u8; 2];
                            for h in &mut hex {
                                match self.advance() {
                                    Some(c) if c.is_ascii_hexdigit() => *h = c,
                                    _ => {
                                        return Token::Error(
                                            "Invalid hex escape sequence".to_string(),
                                        )
                                    }
                                }
                            }
                            if let Ok(val) =
                                u8::from_str_radix(core::str::from_utf8(&hex).unwrap_or("00"), 16)
                            {
                                s.push(val as char);
                            }
                        }
                        Some(b'u') => {
                            // \uHHHH unicode escape
                            let mut hex = [0u8; 4];
                            for h in &mut hex {
                                match self.advance() {
                                    Some(c) if c.is_ascii_hexdigit() => *h = c,
                                    _ => {
                                        return Token::Error(
                                            "Invalid unicode escape sequence".to_string(),
                                        )
                                    }
                                }
                            }
                            if let Ok(val) = u32::from_str_radix(
                                core::str::from_utf8(&hex).unwrap_or("0000"),
                                16,
                            ) {
                                if let Some(ch) = char::from_u32(val) {
                                    s.push(ch);
                                }
                            }
                        }
                        Some(c) => s.push(c as char),
                        None => return Token::Error("Unterminated string".to_string()),
                    }
                }
                Some(c) => {
                    self.advance();
                    s.push(c as char);
                }
            }
        }

        Token::String(s)
    }
}

// Tests moved to tests/lexer_tests.rs.
