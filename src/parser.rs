use core::fmt;
use std::{collections::HashMap, path::PathBuf};

use eyre::Result;
use ordered_float::OrderedFloat;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Note {
    /// */1
    Whole,

    /// */2
    Half,

    /// */4
    Quarter,

    /// */8
    Eigth,

    /// */16
    Sixteenth,

    /// */32
    ThirtySecond,
}

impl Note {
    pub fn name(&self) -> &'static str {
        match self {
            Note::Whole => "whole",
            Note::Half => "half",
            Note::Quarter => "quarter",
            Note::Eigth => "8th",
            Note::Sixteenth => "16th",
            Note::ThirtySecond => "32nd",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node<'a> {
    Flag(Flag<'a>),
    Expr(Expr<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Flag<'a> {
    Resolution {
        width: usize,
        height: usize,
    },

    Fps {
        fps: usize,
    },

    Bpm {
        bpm: OrderedFloat<f32>,
        divisor: Note,
    },

    Song {
        path: &'a str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expr<'a> {
    NamedBlock {
        name: &'a str,
        exprs: Vec<Expr<'a>>,
    },

    Repeat {
        count: usize,
        exprs: Vec<Expr<'a>>,
    },

    TimeMarker {
        count: usize,
        num: usize,
        den: usize,
    },

    MeasureTimeMarker {
        start: usize,
        end: usize,
        num: usize,
        den: usize,
    },

    BlockUse {
        name: &'a str,
    },

    Comment,
}

peg::parser! {
    grammar uvis_grammar() for str {
        rule _number() -> u32
            = n:$(['0'..='9']+) {? n.parse().or(Err("u32")) }

        rule _ident() -> &'input str
            = n:$(['A'..='Z'|'a'..='z'|'_']['A'..='Z'|'a'..='z'|'0'..='9'|'_']*) { n }

        rule _float() -> OrderedFloat<f32>
            = a:$(['0'..='9']+) "." b:$(['0'..='9']+) {? format!("{a}.{b}").parse().or(Err("f32")) }

        rule _res_flag() -> Flag<'input>
            = "@" _ "resolution" _ "=" _ w: _number() _ "x" _ h: _number()
              { Flag::Resolution { width: w as usize, height: h as usize } }

        rule _fps_flag() -> Flag<'input>
            = "@" _ "fps" _ "=" _ fps: _number()
              { Flag::Fps { fps: fps as usize } }

        rule _note_ty_1() -> Note = "*" _ "/" _ "1" { Note::Whole }
        rule _note_ty_2() -> Note = "*" _ "/" _ "2" { Note::Half }
        rule _note_ty_4() -> Note = "*" _ "/" _ "4" { Note::Quarter }
        rule _note_ty_8() -> Note = "*" _ "/" _ "8" { Note::Eigth }
        rule _note_ty_16() -> Note = "*" _ "/" _ "16" { Note::Sixteenth }
        rule _note_ty_32() -> Note = "*" _ "/" _ "32" { Note::ThirtySecond }

        rule _note() -> Note = _note_ty_1() / _note_ty_2() / _note_ty_4() / _note_ty_8() / _note_ty_16() / _note_ty_32()

        rule _bpm_flag() -> Flag<'input>
            = "@" _ "bpm" _ "=" _ bpm: _float() _ div: _note()
              { Flag::Bpm { bpm, divisor: div } }

        rule _qstr() -> &'input str
            = "\"" s: $([^'"']*) "\"" { s }

        rule _song_flag() -> Flag<'input>
            = "@" _ "song" _ "=" _ path: _qstr()
              { Flag::Song { path } }

        rule _flag() -> Flag<'input>
            = it: (_res_flag() / _fps_flag() / _bpm_flag() / _song_flag()) _comment()? { it }

        rule _block() -> Expr<'input>
            = ":" id: _ident() _ "{" _ exprs: _exprs() _ "}"
              { Expr::NamedBlock { name: id, exprs } }

        rule _marker() -> Expr<'input>
            = count: _number() "x" _ num: _number() _ "/" _ den: _number()
              { Expr::TimeMarker { count: count as _, num: num as _, den: den as _ } }

        rule _measure_marker() -> Expr<'input>
            = "[" _ "m" start: _number() _ "->" _ "m" end: _number() _ "]" _ num: _number() _ "/" _ den: _number()
              { Expr::MeasureTimeMarker { start: start as _, end: end as _, num: num as _, den: den as _ } }

        rule _repeat() -> Expr<'input>
            = "#repeat" _ "(" _ count: _number() _ ")" _ "{" _ exprs: _exprs() _ "}"
              { Expr::Repeat { count: count as _, exprs } }

        rule _block_use() -> Expr<'input>
            = "&" _ ":" name: _ident()
              { Expr::BlockUse { name } }

        rule _expr_inner() -> Expr<'input> = _marker() / _measure_marker() / _repeat() / _block_use() / _comment()
        rule _expr() -> Expr<'input> = it: _expr_inner() _comment()? { it }
        rule _exprs() -> Vec<Expr<'input>> = (_ it: _expr() ** _ { it })
        rule _tl_expr() -> Expr<'input> = it: (_expr_inner() / _block()) _comment()? { it }

        rule _tl_expr_node() -> Node<'input> = it: _tl_expr() { Node::Expr(it) }
        rule _flag_node() -> Node<'input> = it: _flag() { Node::Flag(it) }

        rule _comment() -> Expr<'input> = quiet!{" "* "//" [^'\n']*} { Expr::Comment }
        rule _node() -> Node<'input> = _flag_node() / _tl_expr_node()

        rule whitespace() = quiet!{[' ' | '\n' | '\t']*}
        rule _ = quiet!{whitespace()}

        pub rule nodes() -> Vec<Node<'input>> = (_node() ** _)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot find block: {0}")]
    MissingBlock(String),

    #[error("cannot declare block {0} while inside of another block!")]
    BlockInBlock(String),

    #[error("cannot find parent directory for source file!")]
    NoParent,
}

fn parse_base<'a>(input: &'a str) -> Result<Vec<Node<'a>>> {
    Ok(uvis_grammar::nodes(input.trim())?)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeSigItem {
    pub measures: usize,
    pub num: usize,
    pub den: usize,
}

impl fmt::Debug for TimeSigItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(({}, {}), {})", self.num, self.den, self.measures)
    }
}

fn process_exprs<'a>(
    out: &mut Vec<TimeSigItem>,
    input: Vec<Expr<'a>>,
    blocks: &HashMap<&'a str, Vec<Expr<'a>>>,
    is_root: bool,
) -> Result<(), Error> {
    for expr in input {
        match expr {
            Expr::Repeat { count, exprs } => {
                let mut new = Vec::new();

                process_exprs(&mut new, exprs, blocks, false)?;

                for _ in 0..count {
                    out.extend(new.clone());
                }
            }

            Expr::BlockUse { name } => {
                let block = blocks.get(name).ok_or(Error::MissingBlock(name.into()))?;
                let mut new = Vec::new();

                process_exprs(&mut new, block.clone(), blocks, false)?;

                out.extend(new);
            }

            Expr::TimeMarker { count, num, den } => {
                out.push(TimeSigItem {
                    measures: count,
                    num,
                    den,
                });
            }

            Expr::MeasureTimeMarker {
                start,
                end,
                num,
                den,
            } => {
                out.push(TimeSigItem {
                    measures: end - start,
                    num,
                    den,
                });
            }

            Expr::NamedBlock { name, exprs } => {
                if !is_root {
                    return Err(Error::BlockInBlock(name.into()));
                } else {
                    let mut new = Vec::new();

                    process_exprs(&mut new, exprs, blocks, false)?;

                    out.extend(new);
                }
            }

            Expr::Comment => {}
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramOptions {
    pub width: usize,
    pub height: usize,
    pub fps: usize,
    pub bpm: OrderedFloat<f32>,
    pub bpm_divisor: Note,
    pub song_path: Option<PathBuf>,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 60,
            bpm: 120.0_f32.into(),
            bpm_divisor: Note::Quarter,
            song_path: None,
        }
    }
}

fn simplify<'a>(
    file_path: &PathBuf,
    nodes: Vec<Node<'a>>,
) -> Result<(Vec<TimeSigItem>, ProgramOptions), Error> {
    let mut blocks = HashMap::<&'a str, Vec<Expr<'a>>>::new();

    for node in &nodes {
        match node {
            Node::Expr(e) => match e {
                Expr::NamedBlock { name, exprs } => {
                    blocks.insert(name, exprs.clone());
                }

                _ => {}
            },

            Node::Flag(_) => {}
        }
    }

    let (exprs, flags) =
        nodes
            .into_iter()
            .fold((Vec::new(), Vec::new()), |(mut exprs, mut flags), node| {
                match node {
                    Node::Expr(e) => exprs.push(e),
                    Node::Flag(f) => flags.push(f),
                };

                (exprs, flags)
            });

    let mut items = Vec::new();

    process_exprs(&mut items, exprs, &blocks, true)?;

    let mut opts = ProgramOptions::default();

    for flag in flags {
        match flag {
            Flag::Resolution { width, height } => {
                opts.width = width;
                opts.height = height;
            }

            Flag::Fps { fps } => opts.fps = fps,

            Flag::Bpm { bpm, divisor } => {
                opts.bpm = bpm;
                opts.bpm_divisor = divisor;
            }

            Flag::Song { path } => {
                if path.starts_with("/") {
                    opts.song_path = Some(path.into());
                } else {
                    let parent = file_path.parent().ok_or(Error::NoParent)?;

                    opts.song_path = Some(parent.join(path));
                }
            }
        }
    }

    Ok((items, opts))
}

pub fn parse<'a>(
    file_path: &PathBuf,
    input: &'a str,
) -> Result<(Vec<TimeSigItem>, ProgramOptions)> {
    Ok(simplify(file_path, parse_base(input)?)?)
}
