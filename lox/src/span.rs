use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Span {
            start: value.start,
            end: value.end,
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Span::from(0..1)
    }
}

impl Span {
    pub fn join(&self, rhs: Span) -> Span {
        Span::from(self.start..rhs.end)
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn len(&self) -> usize {
        self.range().len()
    }

    pub fn get_start_location(&self, source: &str) -> Location {
        Self::get_location(source, self.start)
    }

    pub fn get_end_location(&self, source: &str) -> Location {
        Self::get_location(source, self.end - 1)
    }

    pub fn get_location(source: &str, index: usize) -> Location {
        let line = source[..index].chars().filter(|a| a == &'\n').count();
        let col = source[..index]
            .chars()
            .rev()
            .position(|c| c == '\n')
            .unwrap_or(index);

        Location {
            line: line + 1,
            col: col + 1,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::span::Location;

    use super::Span;

    #[test]
    fn single_line() {
        let source = "     @   ";
        let span = Span::from(5..6);

        assert_eq!(&source[span.range()], "@");

        let Location { line, col } = span.get_start_location(source);
        let Location {
            line: eline,
            col: ecol,
        } = span.get_end_location(source);

        assert_eq!(line, eline);
        assert_eq!(col, ecol);

        assert_eq!(line, 1);
        assert_eq!(col, 6);
    }

    #[test]
    fn multiple_line() {
        let source = "\n\n\n\n\n     @@@\n@@\n@@@   ";
        let span = Span::from(10..20);

        assert_eq!(&source[span.range()], "@@@\n@@\n@@@");

        let Location { line, col } = span.get_start_location(source);

        assert_eq!(line, 6);
        assert_eq!(col, 6);

        let Location { line, col } = span.get_end_location(source);

        assert_eq!(line, 8);
        assert_eq!(col, 3);
    }
}
