mod expression;
mod parser;
mod scanner;
mod token;

use crate::user::RoGuildUser;

use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_model::id::RoleId;

use expression::Expression;
use parser::Parser;
use scanner::Scanner;
use token::Literal;

#[derive(Clone)]
pub struct RoCommand {
    pub code: String,
    pub expr: Expression,
}

#[derive(Debug)]
pub struct RoCommandUser<'rc> {
    pub user: &'rc RoGuildUser,
    pub roles: &'rc [RoleId],
    pub ranks: &'rc HashMap<i64, i64>,
    pub username: &'rc str,
}

impl RoCommand {
    pub fn new(code: &str) -> Result<Self, String> {
        let mut scanner = Scanner::new(code);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let expr = parser.expression().map_err(|e| e.1)?;
        Ok(Self {
            code: code.into(),
            expr,
        })
    }

    pub fn evaluate(&self, user: &RoCommandUser) -> Result<bool, String> {
        let success = match self.expr.evaluate(user)? {
            Literal::Bool(b) => b,
            _ => true,
        };
        Ok(success)
    }
}

impl Display for RoCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("RoCommand")
            .field("Code", &self.code)
            .finish()
    }
}
