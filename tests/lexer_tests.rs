//! Unit tests for the lexer/tokenizer.
//!
//! Migrated from src/parser/lexer.rs.

use mquickjs::parser::{Lexer, Token};

#[test]
fn test_numbers() {
    let mut lexer = Lexer::new("42 3.14 1e10");
    let expected = 314.0_f64 / 100.0;

    assert!(matches!(lexer.next_token(), Token::Number(n) if n == 42.0));
    assert!(matches!(lexer.next_token(), Token::Number(n) if (n - expected).abs() < 0.001));
    assert!(matches!(lexer.next_token(), Token::Number(n) if n == 1e10));
}

#[test]
fn test_strings() {
    let mut lexer = Lexer::new(r#""hello" 'world'"#);

    assert_eq!(lexer.next_token(), Token::String("hello".to_string()));
    assert_eq!(lexer.next_token(), Token::String("world".to_string()));
}

#[test]
fn test_raw_utf8_string_literal() {
    let mut lexer = Lexer::new("\"caf\u{00E9}\"");
    assert_eq!(lexer.next_token(), Token::String("café".to_string()));
}

#[test]
fn test_identifiers_and_keywords() {
    let mut lexer = Lexer::new("foo var if else");

    assert_eq!(lexer.next_token(), Token::Ident("foo".to_string()));
    assert_eq!(lexer.next_token(), Token::Var);
    assert_eq!(lexer.next_token(), Token::If);
    assert_eq!(lexer.next_token(), Token::Else);
}

#[test]
fn test_operators() {
    let mut lexer = Lexer::new("+ ++ += === !==");

    assert_eq!(lexer.next_token(), Token::Plus);
    assert_eq!(lexer.next_token(), Token::PlusPlus);
    assert_eq!(lexer.next_token(), Token::PlusEq);
    assert_eq!(lexer.next_token(), Token::EqEqEq);
    assert_eq!(lexer.next_token(), Token::BangEqEq);
}

#[test]
fn test_comments() {
    let mut lexer = Lexer::new("1 // comment\n2 /* block */ 3");

    assert!(matches!(lexer.next_token(), Token::Number(n) if n == 1.0));
    assert!(matches!(lexer.next_token(), Token::Number(n) if n == 2.0));
    assert!(matches!(lexer.next_token(), Token::Number(n) if n == 3.0));
}
