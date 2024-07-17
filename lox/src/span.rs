use std::ops::{Deref, DerefMut, Range};

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Span(Range<usize>);

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Span(value)
    }
}

impl Deref for Span {
    type Target = Range<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Span {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Span {
    pub fn range(&self) -> &Range<usize> {
        self.deref()
    }

    pub fn get_line_col(&self, source: &str) -> (usize, usize) {
        (self.get_line(source), self.get_col(source))
    }

    #[allow(clippy::unwrap_used)]
    pub fn get_context<'src>(
        &self,
        source: &'src str,
        range: Range<isize>,
    ) -> Vec<(usize, &'src str)> {
        let reference = isize::try_from(self.get_line(source)).unwrap();
        let lines = reference + range.start..=reference + range.end;

        source
            .lines()
            .enumerate()
            .filter_map(|(line, src)| {
                let line = isize::try_from(line).unwrap() + 1;
                lines
                    .contains(&line)
                    .then_some((line.try_into().unwrap(), src))
            })
            .collect()
    }

    pub fn get_col(&self, source: &str) -> usize {
        source[..self.start]
            .chars()
            .rev()
            .position(|c| c == '\n')
            .unwrap_or(self.start)
            + 1
    }

    pub fn get_line(&self, source: &str) -> usize {
        source[..self.start].chars().filter(|a| a == &'\n').count() + 1
    }
}

#[cfg(test)]
mod test {
    use super::Span;

    #[test]
    fn single_line() {
        let source = "     @   ";
        let span = Span::from(5..6);

        let line = span.get_line(source);
        let col = span.get_col(source);

        assert_eq!(line, 1);
        assert_eq!(col, 6);
        assert_eq!(source.chars().nth((line - 1) + (col - 1)), Some('@'));
    }

    #[test]
    fn multiple_line() {
        let source = "\n\n\n\n\n     @   ";
        let span = Span::from(10..11);

        let (line, col) = span.get_line_col(source);

        assert_eq!(line, 6);
        assert_eq!(col, 6);
        assert_eq!(source.chars().nth((line - 1) + (col - 1)), Some('@'));
    }

    #[test]
    fn multiple_line_non_closed_token() {
        let source = "\n\n\n\n\n     \"###### \n\n   ";
        let span = Span::from(10..22);

        let (line, col) = span.get_line_col(source);

        assert_eq!(line, 6);
        assert_eq!(col, 6);
        assert_eq!(source.chars().nth((line - 1) + (col - 1)), Some('\"'));
    }

    #[test]
    fn get_2_line_context() {
        let source = "line0\nline1\nline2\n@\nline3";
        let span = Span::from(18..19);

        let lines = span.get_context(source, -2..2);

        println!("{lines:?}");

        assert_eq!(
            lines,
            vec![(2, "line1"), (3, "line2"), (4, "@"), (5, "line3")]
        );
        assert_eq!(source.chars().nth(18), Some('@'));
    }

    #[test]
    fn get_2_line_contexts_head_missing() {
        let source = "@\nline3";
        let span = Span::from(0..1);

        let lines = span.get_context(source, -2..2);

        println!("{lines:?}");

        assert_eq!(lines, vec![(1, "@"), (2, "line3")]);
        assert_eq!(source.chars().next(), Some('@'));
    }
}
