use std::rc::Rc;
use std::fmt::{Formatter, Display, self};

pub type RcStr = Rc<String>;
pub type PosIndex = isize;

/*
 * Positions
 */

#[derive(PartialEq, Clone, Debug)]
pub struct Pos {
    pub src_index: isize,
    pub line_index: isize,
    pub col_index: isize,
    pub source_path: RcStr,
    pub source_text: RcStr,
}

impl Pos {
    pub fn new_curr(source_path: RcStr, source_text: RcStr) -> Self {
        Pos {
            src_index: -2,
            line_index: 0,
            col_index: -2,
            source_path,
            source_text,
        }
    }

    pub fn new_next(source_path: RcStr, source_text: RcStr) -> Self {
        Pos {
            src_index: -1,
            line_index: 0,
            col_index: -1,
            source_path,
            source_text,
        }
    }

    /// Advances the source and column index by one.
    pub fn adv(&mut self) {
        self.src_index += 1;
        self.col_index += 1;
    }

    /// Advances the line index by one, resetting the column index.
    pub fn line(&mut self) {
        self.line_index += 1;
        self.col_index = 0;
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line_index + 1, self.col_index + 1)
    }
}

#[derive(Clone, Debug)]
pub struct Range {
    pub start: Pos,
    pub end: Pos,
}

impl Range {
    pub fn new_curr(path: RcStr, text: RcStr) -> Self {
        Range {
            start: Pos::new_curr(path.clone(), text.clone()),
            end: Pos::new_curr(path, text),
        }
    }

    pub fn new_next(path: RcStr, text: RcStr) -> Self {
        Range {
            start: Pos::new_next(path.clone(), text.clone()),
            end: Pos::new_next(path, text),
        }
    }

    /// Advances the `end` position by one.
    pub fn adv(&mut self) {
        self.end.adv();
    }

    /// Advances the `end` line index by one.
    pub fn line(&mut self) {
        self.end.line();
    }

    /// Bumps the `start` position up to the `end` position.
    pub fn catchup(&mut self) {
        self.start = self.end.clone();
    }

    pub fn as_str(&self) -> &str {
        let start = self.start.src_index;
        let end = self.end.src_index;
        assert!(start >= 0);
        assert!(start <= end);
        &self.start.source_text.as_str()[start as usize..end as usize]
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.start == self.end {
            write!(f, "{} at {}", self.start.source_path, self.start)
        }
        else if self.start.line_index == self.end.line_index {
            write!(f, "{} at {}-{}", self.start.source_path, self.start, self.end.col_index + 1)
        }
        else {
            write!(f, "{} at {}-{}", self.start.source_path, self.start, self.end)
        }
    }
}
