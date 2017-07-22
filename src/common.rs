use errors::*;
use syntax::{AST, Tokenizer, Parser};
use error_chain::ChainedError;
use std::sync::Arc;
use std::fmt::{Formatter, Debug, Display, self};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{Read, self};
use std::cmp::Ordering;

pub type RcStr = Arc<String>;

/// Identity function.
pub fn id<T>(x: T) -> T { x }

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
pub fn process_source_path<P: AsRef<Path>, Q: AsRef<Path>>(path: P, search_dirs: &[Q]) -> Result<AST> {
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
    use std::mem;
    eprintln!("{}", err_chain.iter().nth(0).unwrap());
    for err in err_chain.iter().skip(1) {
        eprintln!("... {}", err);
    }
    
    let mut ranges = err_chain.iter()
        // XXX : ugly hack to mark ranged errors
        // see https://github.com/rust-lang/rust/issues/35943 for details
        .map(|e| unsafe { mem::transmute::<&::std::error::Error, &(::std::error::Error+'static)>(e) })
        .filter_map(|e| e.downcast_ref::<Error>())
        .filter_map(|e| if let &Error(ErrorKind::Ranged(ref r), _) = e { Some(r.clone()) } else { None })
        .collect::<Vec<_>>();
    if !ranges.is_empty() {
        ranges.sort_by(|a, b| a.start.cmp(&b.start));
        let range = Range::new(ranges.first().unwrap().start.clone(), ranges.last().unwrap().end.clone());
        eprintln!();
        print_range_underline(range);
    }
}

/// Prints an underlined range.
pub fn print_range_underline(range: Range) {
    const MAX_LINES: usize = 4;
    let source_text = range.source_text();
    let lines = source_text.split('\n')
        .collect::<Vec<_>>();
    assert!(range.start.line_index < lines.len() as isize);
    assert!(range.end.line_index < lines.len() as isize);
    eprintln!("    {}:", range);
    let start_index = range.start.line_index as usize;
    let end_index = range.end.line_index as usize;
    if start_index == end_index {
        print_error_line(lines[start_index], start_index as isize,
                         range.start.col_index, range.end.col_index);
    }
    else if (end_index - start_index) > MAX_LINES {
        print_error_line(lines[start_index], start_index as isize,
                         range.start.col_index, lines[start_index].len() as isize);

        for idx in start_index + 1 .. start_index + (MAX_LINES / 2) {
            print_error_line(lines[idx], idx as isize, 0, lines[idx as usize].len() as isize);
        }

        eprintln!("<{} lines omitted>", end_index - start_index - MAX_LINES);

        for idx in end_index - (MAX_LINES / 2) + 1 .. end_index + 1 {
            print_error_line(lines[idx], idx as isize, 0, lines[idx as usize].len() as isize);
        }
    }
    else {
        print_error_line(lines[start_index], start_index as isize,
                         range.start.col_index, lines[start_index].len() as isize);
        for idx in range.start.line_index + 1 .. range.end.line_index + 1 {
            print_error_line(lines[idx as usize], idx, 0, lines[idx as usize].len() as isize);
        }
    }
}

/// Takes an *unstripped* line, and prints it with an underline between the
/// given start and end indices.
fn print_error_line(line: &str, line_index: isize, start: isize, end: isize) {
    const INDENT: usize = 8;
    const MAX_LEN: usize = 72;
    const LINE_NUMBER_WIDTH: usize = 4;
    let elipses = if line.len() > MAX_LEN {
        "..."
    }
    else { "" };
    let line_offset = line.find(|c: char| c != ' ' && c != '\t')
        .unwrap_or(0);
    let line = line.trim()
        .chars()
        .take(MAX_LEN)
        .collect::<String>(); 
    // for now, we're just underlining the first line
    // strip the initial whitespace, and indent by 4 plus the line number
    eprintln!("{1: >0$}{2}{3}{4}",
              LINE_NUMBER_WIDTH,
              line_index + 1,
              " ".repeat(INDENT),
              line,
              elipses);
    // figure out where to start and end. if we're on the same line, just use the
    // start and end of the range. if we're on different lines, start at the start
    // and end at the end of the line. 
    let len = end - start;
    /*
    let start = start
            - line_offset as isize
            + DOTS_LEN as isize
            + LINE_NUMBER_WIDTH as isize
            + INDENT as isize;
    let end = end
            - line_offset as isize
            + DOTS_LEN as isize
            + LINE_NUMBER_WIDTH as isize
            + INDENT as isize;
    */
    if len > 0 {
        eprintln!("{}{}",
              " ".repeat(LINE_NUMBER_WIDTH + INDENT),
              "^".repeat(len as usize));
    }
}

/*
 * Positions
 */

#[derive(Eq, PartialEq, Clone)]
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

    pub fn eof(source_path: RcStr, source_text: RcStr) -> Self {
        Pos {
            src_index: ::std::isize::MIN,
            line_index: ::std::isize::MIN,
            col_index: ::std::isize::MIN,
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

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.src_index.partial_cmp(&other.src_index) }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> Ordering { self.src_index.cmp(&other.src_index) }
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

    pub fn new(start: Pos, end: Pos) -> Self {
        Range { start, end }
    }

    pub fn eof(path: RcStr, text: RcStr) -> Self {
        Range {
            start: Pos::eof(path.clone(), text.clone()),
            end: Pos::eof(path, text),
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

    pub fn source_text(&self) -> RcStr {
        self.start.source_text
            .clone()
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.start == self.end {
            write!(f, "`{}` at {}", self.start.source_path, self.start)
        }
        else if self.start.line_index == self.end.line_index {
            write!(f, "`{}` at {}-{}", self.start.source_path, self.start, self.end.col_index + 1)
        }
        else {
            write!(f, "`{}` at {}-{}", self.start.source_path, self.start, self.end)
        }
    }
}
