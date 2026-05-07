use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip(r"//[^\r\n]*", allow_greedy = true))]
enum Token {
    // Keywords
    #[token("let")]
    Let,

    #[token("sprite")]
    Sprite,

    #[token("match")]
    Match,

    #[token("self")]
    SelfKw,

    // Events
    #[token("on_flag")]
    OnFlag,

    #[token("on_message")]
    OnMessage,

    // Built-in functions
    #[token("broadcast")]
    Broadcast,

    #[token("wait")]
    Wait,

    // Operations
    #[token("=")]
    Equals,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("+=")]
    Add,

    #[token("-=")]
    Subtract,

    #[token(".")]
    Dot,

    #[token(",")]
    Comma,

    // Identifiers and types
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),

    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string() // strip the surrounding quotes
    })]
    #[regex(r"'[^']*'", |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    StringLit(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Delimiters
    #[token(";")]
    SemiColon,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("=>")]
    FatArrow,
}

static CODE: &str = "

// All sprites
let counter = 0;

sprite Cat {
    // This sprite only
    let x;
    let y;

    on_flag() {
        self.x = 67;
        self.y = \"goodbye world.. :(\";
        wait(5);
        broadcast(\"message1\");
    }

    on_message(message) {
        match message {
            \"message1\" => { self.x -= 61; self.y = \"oh hello again!\"; counter += 1;},
        }
    }
}
";

fn main() {
    let lexer = Token::lexer(CODE);
    for result in lexer {
        match result {
            Ok(token) => println!("{:?}", token),
            Err(err) => panic!("unknown token: {:?}", err),
        }
    }
}
