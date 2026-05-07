struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens.get(self.pos);
            self.pos += 1;
            token
        } else {
            None
        }
    }
}
