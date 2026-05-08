use logos::Logos;

use crate::parser::Parser;

mod compiler;
mod parser;

use std::fs;
use std::io::Write;
use zip::{ZipWriter, write::FileOptions};

pub fn write_sb3(project_json: &str, output_path: &str) {
    let file = fs::File::create(output_path).expect("failed to create .sb3 file");
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("project.json", options)
        .expect("failed to write project.json");
    zip.write_all(project_json.as_bytes())
        .expect("failed to write project.json contents");

    zip.finish().expect("failed to finish zip");
}

pub fn write_json(project_json: &str, output_path: &str) {
    let mut file = fs::File::create(output_path).expect("failed to create .json file");
    file.write_all(project_json.as_bytes())
        .expect("failed to write project.json contents");
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\r\f]+")]
#[logos(skip(r"//[^\r\n]*", allow_greedy = true))]
pub enum Token {
    // Keywords
    #[token("let")]
    Let,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("sprite")]
    Sprite,

    #[token("self")]
    SelfKw,

    #[token("event")]
    Event,

    #[token("block")]
    Block,

    // Built-in functions
    #[token("broadcast")]
    Broadcast,

    #[token("wait")]
    Wait,

    // Comparisons
    #[token("==")]
    Equals,

    #[token(">")]
    GreaterThan,

    #[token("<")]
    LessThan,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    // Operations
    #[token("+=")]
    AssAdd,

    #[token("-=")]
    AssSubtract,

    #[token("*=")]
    AssMultiply,

    #[token("/=")]
    AssDivide,

    #[token("=")]
    Assign,

    #[token("+")]
    Add,

    #[token("-")]
    Subtract,

    #[token("*")]
    Multiply,

    #[token("/")]
    Divide,

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

    #[token("any")]
    Any,

    #[token("bool")]
    Bool,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Delimiters
    #[token(";")]
    SemiColon,

    #[token(":")]
    Colon,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,
}

static CODE: &str = "

// All sprites
let counter;

sprite Cat {
    // This sprite only
    let x;
    let y;

    block set_vars(x: any, y: any, be_the_one: bool) {
        if (be_the_one) {
            counter += 1;
        }
        self.x = x;
        self.y = y;
    }

    event on_flag() {
        self.x = 67;
        self.y = \"goodbye world.. :(\";
        wait(5);
        broadcast(\"message1\");
    }

    event on_message(\"message1\") {
        self.set_vars(61, \"oh hello again!\", (x == 67));
        counter += 1;
        if (self.x < 67 * (-2 + 5) && self.y == \"oh hello again!\") {
            counter += 5;
        } else {
            counter -= 3;
        }
    }
}

";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let code = if args.len() > 1 {
        let input = &args[1];
        std::fs::read_to_string(input).unwrap()
    } else {
        CODE.to_string()
    };

    let lexer = Token::lexer(&code);
    let mut tokens: Vec<Token> = vec![];

    println!(" \n\nTokens:");
    for result in lexer {
        match result {
            Ok(token) => {
                println!("{:?}", token);
                tokens.push(token);
            }
            Err(err) => panic!("unknown token: {:?}", err),
        }
    }

    println!(" \n\nStatements:");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();
    for stmt in &stmts {
        println!("{:?}", stmt);
    }

    let project = compiler::compile(stmts.as_slice());
    let json = serde_json::to_string_pretty(&project).unwrap();
    write_json(&json, "project.json");
    write_sb3(&json, "project.sb3");
    println!("wrote project.json and project.sb3");
}
