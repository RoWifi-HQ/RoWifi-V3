use std::{cmp::PartialEq, convert::From, ops::BitXor, str::FromStr};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TokenType {
    LeftParen,
    RightParen,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    String,
    Number,

    And,
    Or,
    Not,
    True,
    False,
    EOF,

    HasRank,
    WithString,
    IsInGroup,
    HasRole,
    GetRank,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Literal {
    String(String),
    Number(i64),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, literal: Option<Literal>) -> Token {
        Token {
            token_type,
            lexeme: lexeme.into(),
            literal,
        }
    }
}

impl FromStr for TokenType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "HasRank" => TokenType::HasRank,
            "WithString" => TokenType::WithString,
            "IsInGroup" => TokenType::IsInGroup,
            "HasRole" => TokenType::HasRole,
            "GetRank" => TokenType::GetRank,
            _ => return Err(String::from("Invalid Keyword")),
        })
    }
}

impl From<Literal> for bool {
    fn from(l: Literal) -> Self {
        match l {
            Literal::Bool(b) => b,
            _ => true,
        }
    }
}

impl BitXor<Literal> for bool {
    type Output = Self;

    fn bitxor(self, rhs: Literal) -> Self::Output {
        match rhs {
            Literal::Bool(b) => b ^ self,
            _ => !self,
        }
    }
}

impl PartialEq<Literal> for i64 {
    fn eq(&self, other: &Literal) -> bool {
        match other {
            Literal::Number(n) => self == n,
            _ => false,
        }
    }
}
