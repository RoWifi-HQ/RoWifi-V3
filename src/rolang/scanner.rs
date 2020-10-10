use super::token::*;

pub struct Scanner {
    source: Vec<char>,
    start: usize,
    current: usize,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            tokens: Vec::<Token>::new(),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token::new(TokenType::EOF, "", None));
        Ok(self.tokens.to_vec())
    }

    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),

            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual, None)
                } else {
                    self.add_token(TokenType::Bang, None)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual, None)
                } else {
                    self.add_token(TokenType::Equal, None)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual, None)
                } else {
                    self.add_token(TokenType::Less, None)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual, None)
                } else {
                    self.add_token(TokenType::Greater, None)
                }
            }

            ' ' | '\r' | '\t' | ',' => {}

            '"' => self.string()?,
            digit if self.is_digit(digit) => self.number()?,
            alpha if self.is_alpha(alpha) => self.identifier()?,

            _ => return Err(String::from("Unexpected character")),
        }
        Ok(())
    }

    fn identifier(&mut self) -> Result<(), String> {
        while self.is_alpha(self.peek()) {
            self.advance();
        }

        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        self.add_token(text.parse::<TokenType>()?, None);
        Ok(())
    }

    fn number(&mut self) -> Result<(), String> {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        let val: i64 = self.source[self.start..self.current]
            .iter()
            .collect::<String>()
            .parse()
            .map_err(|_| String::from("Unexpected number"))?;
        self.add_token(TokenType::Number, Some(Literal::Number(val)));
        Ok(())
    }

    fn string(&mut self) -> Result<(), String> {
        while self.peek() != '"' && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            return Err(String::from("Unterminated String"));
        }

        self.advance();
        let val = self.source[(self.start + 1)..(self.current - 1)]
            .iter()
            .collect::<String>();
        self.add_token(TokenType::String, Some(Literal::String(val)));
        Ok(())
    }

    fn match_char(&mut self, c: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != c {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        self.tokens.push(Token::new(token_type, &text, literal))
    }
}
