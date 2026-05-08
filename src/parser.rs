use crate::Token;

#[derive(Debug)]
pub enum Expression {
    Number(f64),
    StringLit(String),
    Ident(String),
    SelfField(String),
    BinOp {
        left: Box<Expression>,
        op: Operation,
        right: Box<Expression>,
    },
    BoolOp {
        left: Box<Expression>,
        op: Comparison,
        right: Box<Expression>,
    },
}

#[derive(Debug)]
pub enum Operation {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum Comparison {
    Equals,
    GreaterThan,
    LessThan,
    And,
    Or,
}

#[derive(Debug)]
pub enum Statement {
    If {
        condition: Expression,
        body: Vec<Statement>,
    },
    IfElse {
        condition: Expression,
        body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    VarDecl {
        name: String,
    },
    AssignVar {
        name: String,
        operation: Operation,
        value: Expression,
    },
    SelfAssign {
        field: String,
        operation: Operation,
        value: Expression,
    },
    Sprite {
        name: String,
        body: Vec<Statement>,
    },
    OnFlag {
        body: Vec<Statement>,
    },
    OnMessage {
        message: String,
        body: Vec<Statement>,
    },

    Wait {
        length: f64,
    },
    Broadcast {
        message: String,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    fn expect(&mut self, t: &Token) {
        let got = self.next().expect("unexpected EOF");
        assert!(
            std::mem::discriminant(&got) == std::mem::discriminant(t),
            "expected {:?} got {:?}",
            t,
            got
        );
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        let left = self.parse_comparison()?;
        match self.peek().cloned() {
            Some(Token::And) => {
                self.next();
                Some(Expression::BoolOp {
                    left: Box::new(left),
                    op: Comparison::And,
                    right: Box::new(self.parse_expression()?),
                })
            }
            Some(Token::Or) => {
                self.next();
                Some(Expression::BoolOp {
                    left: Box::new(left),
                    op: Comparison::Or,
                    right: Box::new(self.parse_expression()?),
                })
            }
            _ => Some(left),
        }
    }

    fn parse_comparison(&mut self) -> Option<Expression> {
        let left = self.parse_term()?;
        match self.peek().cloned() {
            Some(Token::Equals) => {
                self.next();
                Some(Expression::BoolOp {
                    left: Box::new(left),
                    op: Comparison::Equals,
                    right: Box::new(self.parse_term()?),
                })
            }
            Some(Token::GreaterThan) => {
                self.next();
                Some(Expression::BoolOp {
                    left: Box::new(left),
                    op: Comparison::GreaterThan,
                    right: Box::new(self.parse_term()?),
                })
            }
            Some(Token::LessThan) => {
                self.next();
                Some(Expression::BoolOp {
                    left: Box::new(left),
                    op: Comparison::LessThan,
                    right: Box::new(self.parse_term()?),
                })
            }
            _ => Some(left),
        }
    }

    fn parse_term(&mut self) -> Option<Expression> {
        let left = self.parse_factor()?;
        match self.peek().cloned() {
            Some(Token::Add) => {
                self.next();
                Some(Expression::BinOp {
                    left: Box::new(left),
                    op: Operation::Add,
                    right: Box::new(self.parse_term()?),
                })
            }
            Some(Token::Subtract) => {
                self.next();
                Some(Expression::BinOp {
                    left: Box::new(left),
                    op: Operation::Subtract,
                    right: Box::new(self.parse_term()?),
                })
            }
            _ => Some(left),
        }
    }

    fn parse_factor(&mut self) -> Option<Expression> {
        let left = self.parse_primary()?;
        match self.peek().cloned() {
            Some(Token::Multiply) => {
                self.next();
                Some(Expression::BinOp {
                    left: Box::new(left),
                    op: Operation::Multiply,
                    right: Box::new(self.parse_factor()?),
                })
            }
            Some(Token::Divide) => {
                self.next();
                Some(Expression::BinOp {
                    left: Box::new(left),
                    op: Operation::Divide,
                    right: Box::new(self.parse_factor()?),
                })
            }
            _ => Some(left),
        }
    }

    fn parse_primary(&mut self) -> Option<Expression> {
        let token = self.peek().cloned();
        match token {
            Some(Token::LParen) => {
                self.next();
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(expr)
            }
            Some(Token::Number(_)) => {
                if let Some(Token::Number(n)) = self.next() {
                    Some(Expression::Number(n))
                } else {
                    None
                }
            }
            Some(Token::StringLit(_)) => {
                if let Some(Token::StringLit(s)) = self.next() {
                    Some(Expression::StringLit(s))
                } else {
                    None
                }
            }
            Some(Token::Ident(_)) => {
                if let Some(Token::Ident(s)) = self.next() {
                    Some(Expression::Ident(s))
                } else {
                    None
                }
            }
            Some(Token::SelfKw) => {
                self.next();
                self.expect(&Token::Dot);
                if let Some(Token::Ident(field)) = self.next() {
                    Some(Expression::SelfField(field))
                } else {
                    panic!("expected field name after self.")
                }
            }
            Some(Token::Subtract) => {
                self.next();
                if let Some(Token::Number(n)) = self.next() {
                    Some(Expression::Number(-n))
                } else {
                    panic!("expected number after -")
                }
            }
            Some(Token::AssSubtract) => {
                self.next();
                if let Some(Token::Number(n)) = self.next() {
                    Some(Expression::Number(-n))
                } else {
                    panic!("expected number after -=")
                }
            }
            _ => None,
        }
    }

    fn parse_block(&mut self) -> Vec<Statement> {
        self.expect(&Token::OpenBrace);
        let mut stmts = vec![];
        while !matches!(self.peek(), Some(Token::CloseBrace) | None) {
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                break;
            }
        }
        self.expect(&Token::CloseBrace);
        stmts
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        self.next();
        match self.next() {
            Some(Token::Ident(name)) => {
                self.expect(&Token::SemiColon);
                Some(Statement::VarDecl { name })
            }
            other => panic!("expected variable name, got {:?}", other),
        }
    }

    fn parse_assign_statement(&mut self, name: String) -> Option<Statement> {
        let operation = match self.next() {
            Some(Token::Assign) => Operation::Assign,
            Some(Token::AssAdd) => Operation::Add,
            Some(Token::AssSubtract) => Operation::Subtract,
            Some(Token::AssMultiply) => Operation::Multiply,
            Some(Token::AssDivide) => Operation::Divide,
            other => panic!("expected operation after self., got {:?}", other),
        };
        let value = self
            .parse_expression()
            .expect("expected expression after =");
        self.expect(&Token::SemiColon);
        Some(Statement::AssignVar {
            name,
            operation,
            value,
        })
    }

    fn parse_self_assign(&mut self) -> Option<Statement> {
        self.next();
        self.expect(&Token::Dot);
        let field = match self.next() {
            Some(Token::Ident(f)) => f,
            other => panic!("expected field name after self., got {:?}", other),
        };
        let operation = match self.next() {
            Some(Token::Assign) => Operation::Assign,
            Some(Token::AssAdd) => Operation::Add,
            Some(Token::AssSubtract) => Operation::Subtract,
            Some(Token::AssMultiply) => Operation::Multiply,
            Some(Token::AssDivide) => Operation::Divide,
            other => panic!("expected operation after self., got {:?}", other),
        };
        let value = self
            .parse_expression()
            .expect("expected expression after =");
        self.expect(&Token::SemiColon);
        Some(Statement::SelfAssign {
            field,
            operation,
            value,
        })
    }

    fn parse_sprite(&mut self) -> Option<Statement> {
        self.next();
        let name = match self.next() {
            Some(Token::Ident(s)) => s,
            other => panic!("expected sprite name, got {:?}", other),
        };
        let body = self.parse_block();
        Some(Statement::Sprite { name, body })
    }

    fn parse_on_flag(&mut self) -> Option<Statement> {
        self.next();
        self.expect(&Token::LParen);
        self.expect(&Token::RParen);
        let body = self.parse_block();
        Some(Statement::OnFlag { body })
    }

    fn parse_wait(&mut self) -> Option<Statement> {
        self.next();
        self.expect(&Token::LParen);
        let length = match self.next() {
            Some(Token::Number(n)) => n,
            other => panic!("expected number, got {:?}", other),
        };
        self.expect(&Token::RParen);
        self.expect(&Token::SemiColon);
        Some(Statement::Wait { length })
    }

    fn parse_broadcast(&mut self) -> Option<Statement> {
        self.next();
        self.expect(&Token::LParen);
        let message = match self.next() {
            Some(Token::StringLit(s)) => s,
            other => panic!("expected message string, got {:?}", other),
        };
        self.expect(&Token::RParen);
        self.expect(&Token::SemiColon);
        Some(Statement::Broadcast { message })
    }

    fn parse_on_message(&mut self) -> Option<Statement> {
        self.next();
        self.expect(&Token::LParen);
        let message = match self.next() {
            Some(Token::StringLit(s)) => s,
            other => panic!("expected message string, got {:?}", other),
        };
        self.expect(&Token::RParen);
        let body = self.parse_block();
        Some(Statement::OnMessage { message, body })
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        self.next();
        self.expect(&Token::LParen);
        let condition = self.parse_expression().unwrap();
        self.expect(&Token::RParen);
        let body = self.parse_block();
        match self.peek() {
            Some(Token::Else) => {
                self.next();
                let else_body = self.parse_block();
                Some(Statement::IfElse {
                    condition,
                    body,
                    else_body,
                })
            }
            _ => Some(Statement::If { condition, body }),
        }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.peek() {
            Some(Token::Let) => self.parse_let_statement(),
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::Sprite) => self.parse_sprite(),
            Some(Token::OnFlag) => self.parse_on_flag(),
            Some(Token::OnMessage) => self.parse_on_message(),
            Some(Token::Wait) => self.parse_wait(),
            Some(Token::Broadcast) => self.parse_broadcast(),
            Some(Token::SelfKw) => self.parse_self_assign(),
            Some(Token::Ident(_)) => {
                let name = match self.next() {
                    Some(Token::Ident(n)) => n,
                    _ => unreachable!(),
                };
                self.parse_assign_statement(name)
            }
            _ => None,
        }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut stmts = vec![];
        while self.peek().is_some() {
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                break;
            }
        }
        stmts
    }
}
