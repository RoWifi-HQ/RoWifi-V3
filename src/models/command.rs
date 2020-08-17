use std::{collections::HashMap, fmt, sync::Arc};

use crate::rolang::{expression::Expression, scanner::Scanner, parser::Parser, token::Literal};
use crate::cache::CachedMember;
use super::user::RoUser;

pub struct RoCommand {
    pub code: String,
    pub expr: Expression
}

pub struct RoCommandUser<'rc> {
    pub user: &'rc RoUser,
    pub member: Arc<CachedMember>,
    pub ranks: &'rc HashMap<i64, i64>,
    pub username: &'rc str
}

impl RoCommand {
    pub fn new(code: &str) -> Result<Self, String> {
        let mut scanner = Scanner::new(code);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let expr = parser.expression().map_err(|e| e.1)?;
        Ok(Self {
            code: code.into(),
            expr
        })
    }

    pub fn evaluate(&self, user: &RoCommandUser) -> Result<bool, String> {
        let success = match self.expr.evaluate(user)? {
            Literal::Bool(b) => b,
            _ => true
        };
        Ok(success)
    }
}

impl fmt::Display for RoCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoCommand").field("Code", &self.code).finish()
    }
}