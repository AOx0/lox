#![deny(clippy::unwrap_used)]
#![feature(let_chains)]
#![feature(try_trait_v2)]

mod ast;
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

use owo_colors::OwoColorize;
use parser::Parser;
use span::Span;

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

fn run<'src>(source: &'src Path, ibuf: &'src str) -> Result<(), Vec<CompError<'src>>> {
    let mut errores = Vec::new();
    let scanner = scanner::Scanner::new(ibuf);
    let tokens: Vec<_> = scanner
        .into_iter()
        .filter_map(|token| match token {
            Err(err) => {
                errores.push(CompError::ScannerError(ScannerError {
                    path: source,
                    line: err.span.get_line(ibuf),
                    col: err.span.get_col(ibuf),
                    invalid_token: &ibuf[err.span.range().clone()],
                    error: err,
                    source: ibuf,
                }));
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

    if errores.is_empty().not() {
        Err(errores)
    } else {
        let mut parser = Parser::new(&tokens, &ibuf);

        let res = parser.parse();

        println!("{:?}", res);

        Ok(())
    }
}

fn compf<'src>(path: &'src Path, buf: &'src mut String) -> Result<(), AppError<'src>> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| AppError::FileRead(path, e))?;

    let n = file
        .read_to_string(buf)
        .map_err(|e| AppError::FileRead(path, e))?;

    run(path, &buf[..n]).map_err(AppError::CompErrors)
}

#[derive(Debug)]
struct ScannerError<'src> {
    path: &'src Path,
    line: usize,
    col: usize,
    invalid_token: &'src str,
    error: scanner::Error,
    source: &'src str,
}

#[derive(Debug)]
enum CompError<'src> {
    ScannerError(ScannerError<'src>),
}

impl std::fmt::Display for CompError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! report {
            ($ruta:expr, $line:expr, $col:expr, $($arg:tt)*) => {
                writeln!(f, "{}:{}:{} {}", $ruta, $line, $col, format_args!($($arg)*))
            };
        }

        match self {
            CompError::ScannerError(ScannerError {
                path: ruta,
                line,
                col,
                invalid_token: token,
                error,
                source,
            }) => {
                report!(
                    ruta.display(),
                    line,
                    col,
                    "Scanner error with token {token:?}: {error:?}"
                )?;
                let lines = error.span.get_context(source, -2..2);
                let len = error.span.len();
                let last_context_line = lines.last();

                for (i, sr) in lines.iter() {
                    write!(f, " ")?;
                    write!(
                        f,
                        "{}",
                        format!("{i: >4} | ").if_supports_color(owo_colors::Stream::Stdout, |s| {
                            s.style(owo_colors::Style::new().bright_black())
                        }),
                    )?;
                    writeln!(f, "{sr}")?;
                    if i == line {
                        write!(
                            f,
                            "{}{}",
                            " ".repeat(col + 7),
                            "^".repeat(len)
                                .if_supports_color(owo_colors::Stream::Stdout, |s| {
                                    s.style(owo_colors::Style::new().bold().yellow())
                                }),
                        )?;
                        if let Some((last, _)) = last_context_line
                            && last != line
                        {
                            writeln!(f)?;
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
enum AppError<'src> {
    FileRead(&'src Path, std::io::Error),
    WrongArgs,
    CompErrors(Vec<CompError<'src>>),
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
