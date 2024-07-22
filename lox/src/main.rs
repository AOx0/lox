#![deny(clippy::unwrap_used)]
#![feature(let_chains)]

mod ast;
mod diag;
mod parser;
mod scanner;
mod span;

use std::env::args;
use std::fs::OpenOptions;
use std::io::{stdin, Read, Write};
use std::ops::Not;
use std::path::Path;
use std::process::ExitCode;
use std::str::{self};

use diag::Diagnostic;
use parser::Parser;

fn editline(buf: &mut String) {
    while let Ok(n) = {
        print!("> ");
        std::io::stdout()
            .flush()
            .expect("We are not expecting flush to fail");
        stdin().read_line(buf)
    } {
        if n == 0 {
            break;
        }
        if let Err(err) = run(Path::new("REPL"), buf) {
            for error in err {
                println!("{error}");
            }
        };
        buf.clear();
    }
}

fn run<'src>(path: &'src Path, source: &'src str) -> Result<(), Vec<CompError<'src>>> {
    let scanner = scanner::Scanner::new(source);

    let tokens: Vec<_> = scanner
        .into_iter()
        .filter_map(|token| match token {
            Err(err) => {
                Diagnostic::new(
                    source,
                    path,
                    err.span,
                    format!(
                        "Scanner error with token {:?}: {err:?}",
                        &source[err.span.range()]
                    ),
                )
                .err();
                None
            }
            Ok(token) => matches!(
                token.tipo,
                scanner::TokenKind::Eof
                    | scanner::TokenKind::Whitespace
                    | scanner::TokenKind::CommentLine
            )
            .not()
            .then_some(token),
        })
        .collect();

    let mut parser = Parser::new(path, &tokens, source);

    let res = parser.parse();

    match res {
        Ok(res) => println!("{res:#?}"),
        Err(err) => Diagnostic::new(
            source,
            path,
            err.span,
            format!("Error while parsing: {err:?}"),
        )
        .err(),
    }

    Ok(())
}

fn compf<'src>(path: &'src Path, buf: &'src mut String) -> Result<(), AppError<'src>> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| AppError::FileRead(path, e))?;

    let n = file
        .read_to_string(buf)
        .map_err(|e| AppError::FileRead(path, e))?;

    run(path, &buf[..n]).map_err(|_| AppError::CompErrors)
}

#[derive(Debug)]
struct ParserError<'src> {
    path: &'src Path,
    error: parser::Error,
    source: &'src str,
}

#[derive(Debug)]
struct ScannerError<'src> {
    path: &'src Path,
    invalid_token: &'src str,
    error: scanner::Error,
    source: &'src str,
}

#[derive(Debug)]
enum CompError<'src> {
    ScannerError(ScannerError<'src>),
    ParserError(ParserError<'src>),
}

impl std::fmt::Display for CompError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompError::ParserError(ParserError {
                path,
                source,
                error,
            }) => {
                Diagnostic::new(source, path, error.span, format!("Parser error: {error:?}")).fmt(f)
            }
            CompError::ScannerError(ScannerError {
                path: ruta,
                invalid_token: token,
                error,
                source,
            }) => Diagnostic::new(
                source,
                ruta,
                error.span,
                format!("Scanner error with token {token:?}: {error:?}"),
            )
            .fmt(f),
        }
    }
}

#[derive(Debug)]
enum AppError<'src> {
    FileRead(&'src Path, std::io::Error),
    WrongArgs,
    CompErrors,
}

fn main() -> ExitCode {
    let args: Vec<_> = args().skip(1).collect();
    let mut buf = String::new();

    let res = match args.as_slice() {
        [] => {
            editline(&mut buf);
            Ok(())
        }
        [file] => compf(Path::new(file), &mut buf),
        _ => Err(AppError::WrongArgs),
    };

    match res {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            match err {
                AppError::WrongArgs => eprintln!("Only expected FILE_NAME"),
                AppError::FileRead(file, error) => {
                    eprintln!("Failed to read {:?}: {}", file.display(), error)
                }
                _ => {}
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
