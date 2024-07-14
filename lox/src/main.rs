#![deny(clippy::unwrap_used)]

use std::env::args;
use std::fs::read_to_string;
use std::io::{stdin, Write};
use std::iter::Enumerate;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::{self, FromStr};

fn editline() -> Result<(), AppError> {
    let mut buf = String::new();
    while let Ok(n) = {
        print!("> ");
        std::io::stdout()
            .flush()
            .expect("We are not expecting flush to fail");
        stdin().read_line(&mut buf)
    } {
        if n == 0 {
            break;
        }
        if let Err(err) = run(&buf) {
            for error in err {
                println!("{error}");
            }
        };
        buf.clear();
    }

    Ok(())
}

enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,
    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,
    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,
    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,
    EOF,
}

struct Token {
    tipo: TokenType,
    lexeme: String,
    // literal:Object ,
    line: u8,
}

impl Token {
    fn new(vtipo: TokenType, lexema: &str, linea: u8) -> Self {
        Token {
            tipo: vtipo,
            lexeme: String::from(lexema),
            // literal: literal,
            line: linea,
        }
    }
}

fn run(ibuf: &str) -> Result<(), Vec<CompError>> {
    let tokens: Vec<_> = ibuf.split_terminator(&[' ', '{', '}']).collect();
    for st in tokens.iter() {
        println!("{}", st);
    }

    Err(vec![
        CompError::Syntax("main.lox".into(), 1, 1),
        CompError::Syntax("main.lox".into(), 2, 2),
        CompError::Syntax("main.lox".into(), 3, 3),
    ])
}

fn compf(file: &str) -> Result<(), AppError> {
    println!("ARGS; {:?}", file);
    let buf: String = read_to_string(file).map_err(|err| AppError::FileRead(file.into(), err))?;
    run(&buf).map_err(AppError::CompErrors)
}

#[derive(Debug)]
enum CompError {
    Syntax(PathBuf, usize, usize),
}

impl std::fmt::Display for CompError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! report {
            ($ruta:expr, $line:expr, $col:expr, $($arg:tt)*) => {
                write!(f, "{}:{}:{} {}", $ruta, $line, $col, format_args!($($arg)*))
            };
        }

        match self {
            CompError::Syntax(ruta, line, col) => {
                report!(ruta.display(), line, col, "Invalid syntax")
            }
        }
    }
}

#[derive(Debug)]
enum AppError {
    FileRead(PathBuf, std::io::Error),
    WrongArgs,
    CompErrors(Vec<CompError>),
}

fn main() -> ExitCode {
    let args: Vec<_> = args().skip(1).collect();

    let res = match args.as_slice() {
        [] => editline(),
        [file] => compf(file),
        _ => Err(AppError::WrongArgs),
    };

    match res {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprint!("Error: ");
            match err {
                AppError::WrongArgs => eprintln!("Only expected FILE_NAME"),
                AppError::CompErrors(errors) => {
                    eprintln!("Failed to compile with errors: ");
                    for error in errors {
                        eprintln!("{error}");
                    }
                }
                AppError::FileRead(file, error) => {
                    eprintln!("Failed to read {:?}: {}", file.display(), error)
                }
            }
            ExitCode::FAILURE
        }
    }
}

// fn esqueleto_gramatica_lox() {
//     enum Reservadas{
//         CONTATS{"Nil",} // precedidio de "=" o "==""
//         Print{frmt,Vec<_>}, //"print"
//         Comentario{String}, // buscar entre \" y \"
//         OPERADORES{},
//     }
//     fn funciones_user() {
//         enum TFun{
//             FUN
//         }
//     }
//     fn variables&tipos(T) {
//         enum Type{
//             VAR {identificadori_name,Tipo,valor},
//             BOOLEANO{"tue","false"},
//             NUMBERS{"int","float"},
//             STRING{},
//             LOGICOS{"and","or","="},
//         }
//     }
//     fn principal() {
//         enum Flow{
//             MAIN, // le sigue "{""}"
//             BLOQUES {"{","}"};
//             IF{op1,oplog,op2,result},
//             ELSE,

//         }

//     }
//     funciones_user();
//     variables();
//     principal();
// }
