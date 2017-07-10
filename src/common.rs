use errors::*;
use syntax::{AST, FilledAST, Tokenizer, Parser};
use error_chain::ChainedError;
use std::rc::Rc;
use std::fmt::{Formatter, Debug, Display, self};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{Read, self};

pub type RcStr = Rc<String>;

/// Identity function.
pub fn id<T>(x: T) -> T { x }

macro_rules! printerr {
    ($arg:expr $(, $tail:expr)*) => {
        use std::io::{self, Write};
        let mut stderr = io::stderr();
        writeln!(stderr, $arg $(, $tail)*).unwrap();
    };
}

pub fn search_path<P: AsRef<Path>, Q: AsRef<Path>>(filename: P, search_dirs: &[Q]) -> Option<PathBuf> {
    for p in search_dirs {
        let mut path_buf = PathBuf::from(p.as_ref());
        path_buf.push(&filename);
        if path_buf.as_path().is_file() {
            return Some(path_buf);
        }
    }
    None
}

/// Reads a file from the given path.
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut source_text = String::new();
    {
        let mut file = File::open(path)?;
        file.read_to_string(&mut source_text)?;
    }
    Ok(source_text)
}

/// Processes the contents of a file to a filled AST.
pub fn process_source_path<P: AsRef<Path>, Q: AsRef<Path>>(path: P, search_dirs: &[Q]) -> Result<FilledAST> {
    let contents = match read_file(&path) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("error reading `{}`: {}", path.as_ref().display(), e).into());
        }
    };
    // set up tokenizer and parser
    let tokenizer = Tokenizer::new(path.as_ref().to_str().unwrap(), &contents);
    let mut parser = Parser::new(tokenizer);
    let ast = AST { ast: parser.parse()?, path: path.as_ref().display().to_string() };
    ast.preprocess(search_dirs)
}

pub fn print_error_chain<T: ChainedError>(err_chain: T) {
    printerr!("{}", err_chain.iter().nth(0).unwrap());
    for err in err_chain.iter().skip(1) {
        printerr!("... {}", err);
    }
}



/*
 * Positions
 */

#[derive(PartialEq, Clone)]
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

impl Debug for Pos {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Pos {{ src_index: {:?} line_index: {:?} col_index: {:?} }}",
               self.src_index, self.line_index, self.col_index)
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

    pub fn source_path(&self) -> RcStr {
        self.start.source_path
            .clone()
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
