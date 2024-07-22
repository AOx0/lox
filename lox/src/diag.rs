use crate::span::{Location, Span};
use owo_colors::OwoColorize;

pub struct Diagnostic<'src> {
    msg: String,
    source: &'src str,
    path: &'src std::path::Path,
    span: Span,
}

#[derive(Debug, PartialEq, Eq)]
struct Context<'src> {
    source: &'src str,
    line: usize,
    highlight: Option<std::ops::Range<usize>>,
}

impl<'src> Diagnostic<'src> {
    pub fn new(source: &'src str, path: &'src std::path::Path, span: Span, msg: String) -> Self {
        Self {
            msg,
            source,
            path,
            span,
        }
    }

    fn get_context(&self, n: std::ops::Range<i16>) -> Vec<Context> {
        assert!(n.start <= 0);
        assert!(n.end >= 0);

        let mut res = Vec::new();
        let n_lines = self.source.chars().filter(|c| c == &'\n').count() + 1;

        let Location {
            line: start_line,
            col: start_col,
        } = self.span.get_start_location(self.source);
        let Location { line: end_line, .. } = self.span.get_end_location(self.source);

        let context_start = start_line
            .checked_sub(n.start.unsigned_abs() as usize)
            .unwrap_or(1);
        let context_end = n_lines.min(end_line + n.end as usize);

        let span_lines = self.source[self.span.range()]
            .chars()
            .filter(|c| c == &'\n')
            .count();
        let mut left = self.span.len() - span_lines;
        for (line_num, src) in self
            .source
            .lines()
            .enumerate()
            .map(|(i, src)| (i + 1, src))
            .filter(|(i, _)| (context_start..=context_end).contains(i))
        {
            res.push(Context {
                source: src,
                line: line_num,
                highlight: (start_line..=end_line).contains(&line_num).then(|| {
                    let start = if line_num == start_line {
                        start_col - 1
                    } else {
                        0
                    };

                    let end = left.min(src.len() - start);
                    left -= end;

                    start..start + end
                }),
            });
        }

        res
    }

    pub fn out(self) {
        println!("{self}")
    }

    pub fn err(self) {
        eprintln!("{self}")
    }
}

impl std::fmt::Display for Diagnostic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Location { line, col } = self.span.get_start_location(self.source);
        writeln!(
            f,
            "{error_rojo} at {file}:{line}:{col}: {error_msg}",
            error_rojo = "Error".if_supports_color(owo_colors::Stream::Stdout, |s| {
                s.style(owo_colors::Style::new().bold().red())
            }),
            file = self.path.display(),
            line = line,
            col = col,
            error_msg = self.msg
        )?;

        let lines = self.get_context(-1..1);
        for Context {
            source,
            line,
            highlight,
        } in lines.iter()
        {
            write!(f, " ")?;
            write!(
                f,
                "{}",
                format!("{line: >4} | ").if_supports_color(owo_colors::Stream::Stdout, |s| {
                    s.style(owo_colors::Style::new().bright_black())
                }),
            )?;
            writeln!(f, "{source}")?;
            if let Some(range) = highlight {
                write!(
                    f,
                    "{}{}",
                    " ".repeat(range.start + 8),
                    "^".repeat(range.len())
                        .if_supports_color(owo_colors::Stream::Stdout, |s| {
                            s.style(owo_colors::Style::new().bold().yellow())
                        }),
                )?;
                if lines.last().is_some_and(|l| l.line != *line) {
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::{
        diag::{Context, Diagnostic},
        span::Span,
    };

    #[test]
    fn single_line_ctx() {
        let source = "...\n...\n.@.\n...\n...";
        let span = Span::from(9..10);

        assert_eq!(&source[span.range()], "@");

        let path = PathBuf::new();
        let diag = Diagnostic::new(source, &path, span, String::new());
        let lines = diag.get_context(-2..2);

        assert_eq!(
            lines,
            vec![
                Context {
                    source: "...",
                    line: 1,
                    highlight: None
                },
                Context {
                    source: "...",
                    line: 2,
                    highlight: None
                },
                Context {
                    source: ".@.",
                    line: 3,
                    highlight: Some(1..2)
                },
                Context {
                    source: "...",
                    line: 4,
                    highlight: None
                },
                Context {
                    source: "...",
                    line: 5,
                    highlight: None
                },
            ]
        )
    }

    #[test]
    fn multiple_line_ctx() {
        let source = "...\n...\n.@@\n@@@\n@..";
        let span = Span::from(9..17);

        assert_eq!(&source[span.range()], "@@\n@@@\n@");

        let path = PathBuf::new();
        let diag = Diagnostic::new(source, &path, span, String::new());
        let lines = diag.get_context(-2..2);

        assert_eq!(
            lines,
            vec![
                Context {
                    source: "...",
                    line: 1,
                    highlight: None
                },
                Context {
                    source: "...",
                    line: 2,
                    highlight: None
                },
                Context {
                    source: ".@@",
                    line: 3,
                    highlight: Some(1..3)
                },
                Context {
                    source: "@@@",
                    line: 4,
                    highlight: Some(0..3)
                },
                Context {
                    source: "@..",
                    line: 5,
                    highlight: Some(0..1)
                }
            ]
        )
    }
}
