use core::fmt;
use std::{collections::HashMap, path::PathBuf};

use eyre::Result;
use ordered_float::OrderedFloat;
use thiserror::Error;

use crate::style::Style;

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

pub type ColorRgb = (u8, u8, u8);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StyleExpr<'a> {
    /// `&:[name]`
    Extend {
        name: &'a str,
    },

    Color {
        value: ColorRgb,
    },

    OnColor {
        value: ColorRgb,
    },

    Bold {
        value: bool,
    },

    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PatternOp {
    /// +
    Big,

    /// -
    Small,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MarkerFlag<'a> {
    Style { name: &'a str },

    Pattern { value: Vec<PatternOp> },
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

        flags: Option<Vec<MarkerFlag<'a>>>,
    },

    MeasureTimeMarker {
        start: usize,
        end: usize,
        num: usize,
        den: usize,

        flags: Option<Vec<MarkerFlag<'a>>>,
    },

    BlockUse {
        name: &'a str,
    },

    StyleBlock {
        name: &'a str,
        exprs: Vec<StyleExpr<'a>>,
    },

    Bpm {
        bpm: OrderedFloat<f32>,
        divisor: Note,
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

        rule _bpm_expr() -> Expr<'input>
            = "#bpm" _ "(" _ bpm: _float() _ div: _note() _ ")"
              { Expr::Bpm { bpm, divisor: div } }

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

        rule _pattern_op_big() -> PatternOp = "+" { PatternOp::Big }
        rule _pattern_op_sm() -> PatternOp = "-" { PatternOp::Small }
        rule _pattern_op() -> PatternOp = _pattern_op_big() / _pattern_op_sm()
        rule _pattern() -> Vec<PatternOp> = _pattern_op()*

        rule _marker_flag_style() -> MarkerFlag<'input>
            = "style" _ ":" _ name: _ident()
              { MarkerFlag::Style { name } }

        rule _marker_flag_pattern() -> MarkerFlag<'input>
            = "pattern" _ ":" _ pat: _pattern()
              { MarkerFlag::Pattern { value: pat } }

        rule _marker_flag() -> MarkerFlag<'input> = _marker_flag_style() / _marker_flag_pattern()

        rule _marker_flags() -> Vec<MarkerFlag<'input>>
            = "[" _ v: (_marker_flag() ** ",") _ "]"
              { v }

        rule _marker() -> Expr<'input>
            = count: _number() "x" _ num: _number() _ "/" _ den: _number() flags: (_ it: _marker_flags()? { it })
              { Expr::TimeMarker { count: count as _, num: num as _, den: den as _, flags } }

        rule _measure_marker() -> Expr<'input>
            = "[" _ "m" start: _number() _ "->" _ "m" end: _number() _ "]" _ num: _number() _ "/" _ den: _number() flags: (_ it: _marker_flags()? { it })
              { Expr::MeasureTimeMarker { start: start as _, end: end as _, num: num as _, den: den as _, flags } }

        rule _repeat() -> Expr<'input>
            = "#repeat" _ "(" _ count: _number() _ ")" _ "{" _ exprs: _exprs() _ "}"
              { Expr::Repeat { count: count as _, exprs } }

        rule _block_use() -> Expr<'input>
            = "&" _ ":" name: _ident()
              { Expr::BlockUse { name } }

        rule _hex_color() -> ColorRgb
            = "#" value: ['A'..='F' | 'a'..='f' | '0'..='9']*<6,6>
              {
                let hex = u32::from_str_radix(&String::from_iter(value), 16).expect("failed to parse hex value");
                (((hex >> 16) & 0xFF) as u8, ((hex >> 8) & 0xFF) as u8, (hex & 0xFF) as u8)
              }

        rule _bool_true() -> bool = "true" { true }
        rule _bool_false() -> bool = "false" { false }
        rule _bool() -> bool = _bool_true() / _bool_false()

        rule _style_expr_extend() -> StyleExpr<'input>
            = "&" _ ":" _ name: _ident()
              { StyleExpr::Extend { name } }

        rule _style_expr_color() -> StyleExpr<'input>
            = "color" _ "=" _ value: _hex_color()
              { StyleExpr::Color { value } }

        rule _style_expr_on_color() -> StyleExpr<'input>
            = "on_color" _ "=" _ value: _hex_color()
              { StyleExpr::OnColor { value } }

        rule _style_expr_bold() -> StyleExpr<'input>
            = "bold" _ "=" _ value: _bool()
              { StyleExpr::Bold { value } }

        rule _style_expr_inner() -> StyleExpr<'input> = _style_expr_extend() / _style_expr_color() / _style_expr_on_color() / _style_expr_bold() / _comment_s()
        rule _style_expr() -> StyleExpr<'input> = it: _style_expr_inner() _comment()? { it }
        rule _style_exprs() -> Vec<StyleExpr<'input>> = (_ it: _style_expr() ** _ { it })

        rule _style() -> Expr<'input>
            = "#style" _ "(" _ name: _ident() _ ")" _ "{" _ exprs: _style_exprs() _ "}"
              { Expr::StyleBlock { name, exprs } }

        rule _expr_inner() -> Expr<'input> = _marker() / _measure_marker() / _repeat() / _block_use() / _style() / _bpm_expr() / _comment()
        rule _expr() -> Expr<'input> = it: _expr_inner() _comment()? { it }
        rule _exprs() -> Vec<Expr<'input>> = (_ it: _expr() ** _ { it })
        rule _tl_expr() -> Expr<'input> = it: (_expr_inner() / _block()) _comment()? { it }

        rule _tl_expr_node() -> Node<'input> = it: _tl_expr() { Node::Expr(it) }
        rule _flag_node() -> Node<'input> = it: _flag() { Node::Flag(it) }

        rule _comment() -> Expr<'input> = quiet!{" "* "//" [^'\n']*} { Expr::Comment }
        rule _comment_s() -> StyleExpr<'input> = quiet!{" "* "//" [^'\n']*} { StyleExpr::Comment }
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
    pub flags: MarkerFlagsData,
    pub bpm: OrderedFloat<f32>,
    pub bpm_divisor: Note,
}

impl TimeSigItem {
    pub fn style(&self) -> Style {
        self.flags.style.unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MarkerFlagsData {
    pub style: Option<Style>,

    // pattern encoded as a u32, each bit 1 = big, 0 = small
    pub pattern: Option<u32>,
}

impl MarkerFlagsData {
    pub fn decode<'a>(
        styles: &HashMap<String, Style>,
        flags: &Option<Vec<MarkerFlag<'a>>>,
    ) -> Self {
        let mut me = Self {
            style: None,
            pattern: None,
        };

        if let Some(flags) = flags {
            for flag in flags {
                match flag {
                    MarkerFlag::Style { name } => {
                        if let Some(style) = styles.get(*name) {
                            me.style = Some(*style);
                        } else {
                            // TODO: error handling
                        }
                    }

                    MarkerFlag::Pattern { value } => {
                        let mut res = 0u32;

                        if value.len() > 32 {
                            panic!("pattern length must be less than or equal to 32!");
                        }

                        for i in 0..value.len() {
                            let v = value[i] == PatternOp::Big;
                            let v = (v as u32) << (32 - i - 1);

                            res |= v;
                        }

                        me.pattern = Some(res);
                    }
                }
            }
        }

        me
    }
}

impl fmt::Debug for TimeSigItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(({}, {}), {})", self.num, self.den, self.measures)
    }
}

fn process_exprs<'a>(
    opts: &mut ProgramOptions,
    out: &mut Vec<TimeSigItem>,
    styles: &mut HashMap<String, Style>,
    input: Vec<Expr<'a>>,
    blocks: &HashMap<&'a str, Vec<Expr<'a>>>,
    is_root: bool,
) -> Result<(), Error> {
    for expr in input {
        match expr {
            Expr::Repeat { count, exprs } => {
                let mut new = Vec::new();

                process_exprs(opts, &mut new, styles, exprs, blocks, false)?;

                for _ in 0..count {
                    out.extend(new.clone());
                }
            }

            Expr::BlockUse { name } => {
                let block = blocks.get(name).ok_or(Error::MissingBlock(name.into()))?;
                let mut new = Vec::new();

                process_exprs(opts, &mut new, styles, block.clone(), blocks, false)?;

                out.extend(new);
            }

            Expr::TimeMarker {
                count,
                num,
                den,
                flags,
            } => {
                out.push(TimeSigItem {
                    measures: count,
                    num,
                    den,
                    flags: MarkerFlagsData::decode(&styles, &flags),
                    bpm: opts.bpm,
                    bpm_divisor: opts.bpm_divisor,
                });
            }

            Expr::MeasureTimeMarker {
                start,
                end,
                num,
                den,
                flags,
            } => {
                out.push(TimeSigItem {
                    measures: end - start,
                    num,
                    den,
                    flags: MarkerFlagsData::decode(&styles, &flags),
                    bpm: opts.bpm,
                    bpm_divisor: opts.bpm_divisor,
                });
            }

            Expr::NamedBlock { name, exprs } => {
                if !is_root {
                    return Err(Error::BlockInBlock(name.into()));
                } else {
                    let mut new = Vec::new();

                    process_exprs(opts, &mut new, styles, exprs, blocks, false)?;

                    out.extend(new);
                }
            }

            Expr::StyleBlock { name, exprs } => {
                let s = Style::decode(&styles, &exprs);
                styles.insert(name.to_string(), s);
            }

            Expr::Bpm { bpm, divisor } => {
                opts.bpm = bpm;
                opts.bpm_divisor = divisor;
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

    let mut items = Vec::new();

    process_exprs(
        &mut opts.clone(),
        &mut items,
        &mut HashMap::new(),
        exprs,
        &blocks,
        true,
    )?;

    Ok((items, opts))
}

pub fn parse<'a>(
    file_path: &PathBuf,
    input: &'a str,
) -> Result<(Vec<TimeSigItem>, ProgramOptions)> {
    Ok(simplify(file_path, parse_base(input)?)?)
}
