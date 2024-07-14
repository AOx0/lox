#![deny(clippy::unwrap_used)]

mod scanner;

use std::env::args;
use std::fs::OpenOptions;
use std::io::{stdin, Read, Write};
use std::ops::Not;
use std::path::Path;
use std::process::ExitCode;
use std::str::{self};

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
        if let Err(err) = run(Path::new("Editline"), buf) {
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
    for token in scanner {
        match token {
            Err(err) => errores.push(CompError::ScannerError(
                source,
                0,
                0,
                &ibuf[err.span.clone()],
                err,
            )),
            Ok(token) => println!("{token:?}"),
        }
    }

    if errores.is_empty().not() {
        Err(errores)
    } else {
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
enum CompError<'src> {
    ScannerError(&'src Path, usize, usize, &'src str, scanner::Error),
}

impl std::fmt::Display for CompError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! report {
            ($ruta:expr, $line:expr, $col:expr, $($arg:tt)*) => {
                write!(f, "{}:{}:{} {}", $ruta, $line, $col, format_args!($($arg)*))
            };
        }

        match self {
            CompError::ScannerError(ruta, line, col, token, error) => {
                report!(
                    ruta.display(),
                    line,
                    col,
                    "Scanner error with token {token:?}: {error:?}"
                )
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
