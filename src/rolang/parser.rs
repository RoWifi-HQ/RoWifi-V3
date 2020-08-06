use super::{token::*, expression::Expression};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize
}

#[derive(Debug)]
pub struct ParseError(pub Token, pub String);

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0
        }
    }

    pub fn expression(&mut self) -> Result<Expression, ParseError> {
        Ok(self.equality()?)
    }

    fn equality(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.comparison1()?;
        while self.match_types(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().to_owned();
            let right = self.comparison1()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison1(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.comparison2()?;
        while self.match_types(vec![TokenType::And, TokenType::Or]) {
            let operator = self.previous().to_owned();
            let right = self.comparison2()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison2(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.unary()?;
        while self.match_types(vec![TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, ParseError> {
        if self.match_types(vec![TokenType::Not, TokenType::Bang]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            return Ok(Expression::Unary(operator, Box::new(right)))
        }

        Ok(self.primary()?)
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        if self.match_type(TokenType::False) {return Ok(Expression::Literal(Literal::Bool(false)))}
        if self.match_type(TokenType::True) {return Ok(Expression::Literal(Literal::Bool(true)))}
        
        if self.match_types(vec![TokenType::String, TokenType::Number]) {
            return Ok(Expression::Literal(self.previous().to_owned().literal.unwrap()))
        }

        if self.match_types(vec![TokenType::HasRank, TokenType::WithString, TokenType::HasRole, TokenType::IsInGroup,
            TokenType::GetRank]) {
            let func = self.previous().to_owned();
            self.consume(TokenType::LeftParen, "Expect ( after function call".into())?;
            let mut args = Vec::<Literal>::new();
            while self.match_types(vec![TokenType::String, TokenType::Number]) {
                args.push(self.previous().to_owned().literal.unwrap());
            }
            self.consume(TokenType::RightParen, "Expect ) after function args".into())?;
            let function = Expression::Function(func, args);
            function.validate()?;
            return Ok(function)
        }

        if self.match_type(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ) after expression".into())?;
            return Ok(Expression::Grouping(Box::new(expr)))
        }

        Err(ParseError(self.peek().to_owned(), "Expect expression".into()))
    }

    fn match_type(&mut self, token: TokenType) -> bool {
        if self.check(token) {
            self.advance();
            return true
        }
        false
    }

    fn match_types(&mut self, tokens: Vec<TokenType>) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true
            }
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<(), ParseError> {
        if self.check(token_type) {
            self.advance();
            return Ok(())
        }
        Err(ParseError(self.peek().to_owned(), message))
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {return false}
        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current-1]
    }
}