use twilight_model::id::RoleId;

use super::{token::*, parser::ParseError};
use crate::models::command::RoCommandUser;

pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Unary(Token, Box<Expression>),
    Literal(Literal),
    Grouping(Box<Expression>),
    Function(Token, Vec<Literal>)
}

impl Expression {
    pub fn evaluate(&self, user: &RoCommandUser) -> Result<Literal, String> {
        match self {
            Expression::Literal(l) => return Ok(l.to_owned()),
            Expression::Unary(t, e) => {
                let flip =  t.token_type == TokenType::Not || t.token_type == TokenType::Bang;
                let res = flip ^ e.evaluate(user)?;
                return Ok(Literal::Bool(res))
            },
            Expression::Binary(left, oper, right) => {
                let left = left.evaluate(user)?;
                let right = right.evaluate(user)?;
                match oper.token_type {
                    TokenType::And => return Ok(Literal::Bool(left.into() && right.into())),
                    TokenType::Or => return Ok(Literal::Bool(left.into() || right.into())),
                    TokenType::Greater => return Ok(Literal::Bool(left > right)),
                    TokenType::GreaterEqual => return Ok(Literal::Bool(left >= right)),
                    TokenType::Less => return Ok(Literal::Bool(left < right)),
                    TokenType::LessEqual => return Ok(Literal::Bool(left <= right)),
                    TokenType::EqualEqual => return Ok(Literal::Bool(left == right)),
                    TokenType::BangEqual => return Ok(Literal::Bool(left != right)),
                    _ => return Err("Invalid Operator".to_string())
                }
            },
            Expression::Grouping(e) => return e.evaluate(user),
            Expression::Function(token, args) => {
                match token.token_type {
                    TokenType::HasRank => {
                        if let Literal::Number(num) = &args[0] {
                            let success = match user.ranks.get(num) {
                                Some(rank) => *rank == args[1],
                                None => false
                            };
                            return Ok(Literal::Bool(success))
                        }
                    } 
                    TokenType::IsInGroup => {
                        if let Literal::Number(num) = &args[0] {
                            let success = user.ranks.contains_key(num);
                            return Ok(Literal::Bool(success))
                        }
                    } 
                    TokenType::HasRole => {
                        if let Literal::Number(num) = &args[0] {
                            let id = RoleId(*num as u64);
                            let success = user.member.roles.contains(&id);
                            return Ok(Literal::Bool(success))
                        }
                    } 
                    TokenType::WithString => {
                        if let Literal::String(name) = &args[0] {
                            let success = user.username.contains(name);
                            return Ok(Literal::Bool(success))
                        }
                    } 
                    TokenType::GetRank => {
                        if let Literal::Number(num) = &args[0] {
                            let rank = match user.ranks.get(num) {
                                Some(rank) => rank.to_owned(),
                                None => 0
                            };
                            return Ok(Literal::Number(rank))
                        }
                    },
                    _ => return Err("Invalid Function".to_string())
                }
            }
        } 
        Err("Invalid Expression".to_string())
    }

    pub fn validate(&self) -> Result<(), ParseError> {
        if let Expression::Function(token, args) = self {
            match token.token_type {
                TokenType::HasRank => {
                    if args.len() != 2 {
                        return Err(ParseError(token.to_owned(), "Expected 2 arguments. {Group Id} {Rank Id}".to_string()))
                    } else {
                        match args[0] {
                            Literal::Number(_) => {},
                            _ => return Err(ParseError(token.to_owned(), "Expected Group Id to be an integer".to_string()))
                        };
                        match args[1] {
                            Literal::Number(_) => {},
                            _ => return Err(ParseError(token.to_owned(), "Expected Rank Id to be an integer".to_string()))
                        };
                    }
                } 
                TokenType::IsInGroup | TokenType::GetRank =>  {
                    if args.len() != 1 {
                        return Err(ParseError(token.to_owned(), "Expected 1 argument. {Group Id}".to_string()))
                    } else {
                        match args[0] {
                            Literal::Number(_) => {},
                            _ => return Err(ParseError(token.to_owned(), "Expected Group Id to be an integer".to_string()))
                        }
                    }
                } 
                TokenType::HasRole => {
                    if args.len() != 1 {
                        return Err(ParseError(token.to_owned(), "Expected 1 argument. {Role Id}".to_string()))
                    } else {
                        match args[0] {
                            Literal::Number(_) => {},
                            _ => return Err(ParseError(token.to_owned(), "Expected Role Id to be an integer".to_string()))
                        }
                    }
                } 
                TokenType::WithString => {
                    if args.len() != 1 {
                        return Err(ParseError(token.to_owned(), "Expected 1 argument. {Name}".to_string()))
                    } else {
                        match args[0] {
                            Literal::String(_) => {},
                            _ => return Err(ParseError(token.to_owned(), "Expected Name to be an word".to_string()))
                        }
                    }
                }
                _ => return Err(ParseError(token.to_owned(), "Unknown function".to_string()))
            }
        }
        Ok(())
    }
}