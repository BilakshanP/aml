use std::collections::HashSet;

use chumsky::error::Rich;
use chumsky::prelude::*;

const BRIGHT_OFFSET: u8 = 60;
const BACKGROUND_OFFSET: u8 = 10;

type Err<'src> = extra::Err<Rich<'src, char>>;

// Colour

/// An ANSI color from the standard palette.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clr {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
}

#[repr(u8)]
enum Variant {
    Fg = 0,
    Bg = BACKGROUND_OFFSET,
}

/// A color specification supporting ANSI 8/16, 256-color, and RGB modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    /// Standard 8/16-color ANSI palette.
    Ansi { clr: Clr, bright: bool },
    /// 24-bit true color RGB.
    Rgb { r: u8, g: u8, b: u8 },
    /// 256-color fixed-palette index.
    Fixed(u8),
}

impl Colour {
    fn base(&self) -> u8 {
        match self {
            Self::Ansi { clr, bright } => *clr as u8 + if *bright { BRIGHT_OFFSET } else { 0 },
            Self::Fixed(_) | Self::Rgb { .. } => 38,
        }
    }

    fn codes(&self, variant: Variant) -> Vec<u8> {
        let base = self.base() + variant as u8;

        match *self {
            Self::Fixed(n) => vec![base, 5, n],
            Self::Rgb { r, g, b } => vec![base, 2, r, g, b],
            _ => vec![base],
        }
    }

    /// Get SGR codes for foreground color.
    pub fn fg_codes(&self) -> Vec<u8> {
        self.codes(Variant::Fg)
    }

    /// Get SGR codes for background color.
    pub fn bg_codes(&self) -> Vec<u8> {
        self.codes(Variant::Bg)
    }
}

// Modifiers

/// SGR text modifier/attribute.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mdf {
    Bold = 1,
    Dim = 2,
    Italic = 3,
    Underline = 4,
    Blink = 5,
    RapidBlink = 6,
    Invert = 7,
    Hide = 8,
    Strike = 9,
    DoubleUnderline = 21,
    Overline = 53,
}

/// A set of text modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Modifiers(pub HashSet<Mdf>);

// Tag

/// A parsed style tag.
///
/// Tags can be:
/// - `<>...</>` - hard SGR reset
/// - `<f color>...</f>` - foreground color
/// - `<b color>...</b>` - background color
/// - `<m modifiers>...</m>` - text modifiers
/// - `<s ...>...</s>` - shorthand combining multiple styles
/// - `<! codes>...</!>` - raw SGR codes (transparent to stack)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    /// Hard SGR reset: `<>...</>`
    Reset,
    /// Foreground color.
    Fg(Colour),
    /// Background color.
    Bg(Colour),
    /// Text modifiers.
    Mdf(Modifiers),
    /// Combined shorthand: `<s fg br mbi>...</s>`
    Shorthand {
        fg: Option<Colour>,
        bg: Option<Colour>,
        mdf: Option<Modifiers>,
    },
    /// Raw SGR codes: `<! 0 123 255>...</!>`
    /// Emitted verbatim and transparent to the style stack.
    Raw(String),
}

// AST Node

/// A parsed document node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// A tag with its children.
    Tag { tag: Tag, children: Vec<Node> },
    /// Plain text content.
    Text(String),
}

impl Node {
    /// Convert this node into a document.
    pub fn doc(self) -> Document {
        Document { root: vec![self] }
    }
}

// Parser helpers

/// One or more ASCII spaces.
fn wsp<'src>() -> impl Parser<'src, &'src str, (), Err<'src>> + Clone {
    just(' ')
        .repeated()
        .at_least(1)
        .ignored()
        .labelled("whitespace")
}

/// A decimal integer in 0..=255.
fn byte<'src>() -> impl Parser<'src, &'src str, u8, Err<'src>> + Clone {
    text::int(10)
        .from_str()
        .unwrapped()
        .labelled("byte (0-255)")
}

/// An escape sequence: \<, \\, \n, \t, \r, \0, \e, \c, \x
fn escape<'src>() -> impl Parser<'src, &'src str, String, Err<'src>> + Clone {
    just('\\')
        .ignore_then(choice((
            just('<').to("<"),
            just('\\').to("\\"),
            just('n').to("\n"),
            just('t').to("\t"),
            just('r').to("\r"),
            just('0').to("\0"),
            just('e').to("\x1b"),
            just('c').to("\x1b["),
            just('x').to("\x1b[0m"),
        )))
        .map(str::to_string)
        .labelled("escape sequence")
}

/// Plain text content; use \< to escape a literal <
fn text_node<'src>() -> impl Parser<'src, &'src str, Node, Err<'src>> + Clone {
    choice((
        escape(),
        any().filter(|c: &char| *c != '<').map(|c| c.to_string()),
    ))
    .repeated()
    .at_least(1)
    .collect::<Vec<String>>()
    .map(|parts| Node::Text(parts.concat()))
    .labelled("text")
}

// Colour parsers

fn fixed_colour<'src>() -> impl Parser<'src, &'src str, Colour, Err<'src>> + Clone {
    byte().map(Colour::Fixed).labelled("fixed color")
}

fn ansi_clr<'src>(upper: bool) -> impl Parser<'src, &'src str, Clr, Err<'src>> + Clone {
    let (r, g, b, c, m, y, w, k) = if upper {
        ('R', 'G', 'B', 'C', 'M', 'Y', 'W', 'K')
    } else {
        ('r', 'g', 'b', 'c', 'm', 'y', 'w', 'k')
    };

    choice((
        just(r).to(Clr::Red),
        just(g).to(Clr::Green),
        just(b).to(Clr::Blue),
        just(c).to(Clr::Cyan),
        just(m).to(Clr::Magenta),
        just(y).to(Clr::Yellow),
        just(w).to(Clr::White),
        just(k).to(Clr::Black),
    ))
}

fn ansi_colour<'src>() -> impl Parser<'src, &'src str, Colour, Err<'src>> + Clone {
    choice((
        ansi_clr(false).map(|clr| Colour::Ansi { clr, bright: false }),
        ansi_clr(true).map(|clr| Colour::Ansi { clr, bright: true }),
    ))
    .labelled("ANSI color")
}

/// Decimal R,G,B triple, e.g. 0,128,255
fn rgb_colour<'src>() -> impl Parser<'src, &'src str, Colour, Err<'src>> + Clone {
    byte()
        .then_ignore(just(','))
        .then(byte())
        .then_ignore(just(','))
        .then(byte())
        .map(|((r, g), b)| Colour::Rgb { r, g, b })
        .labelled("RGB color")
}

/// Hex color in #f, #ff, #abc, or #aabbcc format (auto-expanded)
fn hex_colour<'src>() -> impl Parser<'src, &'src str, Colour, Err<'src>> + Clone {
    let hex_digit = any()
        .filter(|c: &char| c.is_ascii_hexdigit())
        .labelled("hex digit (0-9, a-f, A-F)");

    let digits = choice((
        hex_digit.repeated().exactly(6).collect(),
        hex_digit.repeated().exactly(3).collect(),
        hex_digit.repeated().exactly(2).collect(),
        hex_digit.repeated().exactly(1).collect(),
    ));

    just('#')
        .ignore_then(digits)
        .map(expand_hex)
        .labelled("HEX color")
}

fn expand_hex(s: String) -> Colour {
    let full: String = match s.len() {
        1 => s.repeat(6),
        2 => s.repeat(3),
        3 => s.chars().flat_map(|c| [c, c]).collect(),
        6 => s,
        _ => unreachable!("hex parser only yields 1/2/3/6 hex digits"),
    };

    let n = u32::from_str_radix(&full, 16).expect("should be a valid hex");

    Colour::Rgb {
        r: (n >> 16) as u8,
        g: (n >> 8) as u8,
        b: n as u8,
    }
}

/// Any supported color: hex, rgb, ansi, or fixed-palette
fn colour<'src>() -> impl Parser<'src, &'src str, Colour, Err<'src>> + Clone {
    choice((hex_colour(), rgb_colour(), ansi_colour(), fixed_colour()))
}

// Modifier parsers

fn modifier<'src>() -> impl Parser<'src, &'src str, Mdf, Err<'src>> + Clone {
    choice((
        just('b').to(Mdf::Bold),
        just('d').to(Mdf::Dim),
        just('i').to(Mdf::Italic),
        just('u').to(Mdf::Underline),
        just('k').to(Mdf::Blink),
        just('r').to(Mdf::RapidBlink),
        just('v').to(Mdf::Invert),
        just('h').to(Mdf::Hide),
        just('s').to(Mdf::Strike),
        just('l').to(Mdf::DoubleUnderline),
        just('o').to(Mdf::Overline),
    ))
}

fn modifiers<'src>() -> impl Parser<'src, &'src str, Modifiers, Err<'src>> + Clone {
    modifier().repeated().at_least(1).collect().map(Modifiers)
}

// Shorthand arg parsers

/// Parse a shorthand tag argument (e.g. fg, bg, mdf)
fn tag_arg<'src, O, P, F>(
    prefix: char,
    inner: P,
    map_fn: F,
    label: &'static str,
) -> impl Parser<'src, &'src str, Tag, Err<'src>> + Clone
where
    P: Parser<'src, &'src str, O, Err<'src>> + Clone,
    F: Fn(O) -> Tag + Clone,
{
    just(prefix).ignore_then(inner).map(map_fn).labelled(label)
}

pub(crate) fn shorthand<'src>() -> impl Parser<'src, &'src str, Tag, Err<'src>> + Clone {
    use Tag::*;

    let fg_arg = tag_arg('f', colour(), Fg, "fg arg");
    let bg_arg = tag_arg('b', colour(), Bg, "bg arg");
    let mdf_arg = tag_arg('m', modifiers(), Mdf, "mdf arg");

    choice((fg_arg, bg_arg, mdf_arg))
        .separated_by(wsp())
        .at_least(1)
        .at_most(3)
        .collect::<Vec<_>>()
        .map(|tags| {
            let (mut fg, mut bg, mut mdf) = (None, None, None);
            for tag in tags {
                match tag {
                    Fg(c) => fg = Some(c),
                    Bg(c) => bg = Some(c),
                    Mdf(m) => mdf = Some(m),
                    _ => unreachable!(),
                }
            }
            Shorthand { fg, bg, mdf }
        })
        .labelled("shorthand")
}

// Node parser

/// Builds a `<name ...>...</name>` tag node.
fn tag_node<'src, P, A: 'src>(
    name: &'static str,
    attr_parser: P,
    into_tag: fn(A) -> Tag,
    content: impl Parser<'src, &'src str, Vec<Node>, Err<'src>> + Clone + 'src,
) -> impl Parser<'src, &'src str, Node, Err<'src>> + Clone + 'src
where
    P: Parser<'src, &'src str, A, Err<'src>> + Clone + 'src,
{
    just(format!("<{name}"))
        .ignore_then(attr_parser)
        .then_ignore(just('>').labelled("closing `>`"))
        .then(content)
        .then_ignore(just(format!("</{name}>")).labelled("closing tag"))
        .map(move |(attr, children)| Node::Tag {
            tag: into_tag(attr),
            children,
        })
        .labelled(name)
}

/// Parse a node (text, tag, reset, or raw)
pub fn node<'src>() -> impl Parser<'src, &'src str, Node, Err<'src>> + Clone {
    recursive(|node| {
        let content = node.repeated().collect();

        let reset = just("<>")
            .ignore_then(content.clone())
            .then_ignore(just("</>"))
            .map(|children| Node::Tag {
                tag: Tag::Reset,
                children,
            })
            .labelled("reset");

        let raw = just("<!")
            .ignore_then(any().filter(|c| *c != '>').repeated().collect())
            .then_ignore(just('>').labelled("closing `>`"))
            .then(content.clone())
            .then_ignore(just("</!>").labelled("closing raw tag"))
            .map(|(codes, children)| Node::Tag {
                tag: Tag::Raw(codes),
                children,
            })
            .labelled("raw");

        let fg = tag_node("f", colour(), Tag::Fg, content.clone());
        let bg = tag_node("b", colour(), Tag::Bg, content.clone());
        let mdf = tag_node("m", modifiers(), Tag::Mdf, content.clone());
        let sh = tag_node("s", wsp().ignore_then(shorthand()), |t| t, content.clone());

        choice((text_node(), reset, raw, fg, bg, mdf, sh)).labelled("node")
    })
}

// Document

fn document_parser<'src>() -> impl Parser<'src, &'src str, Document, Err<'src>> {
    node()
        .repeated()
        .collect()
        .then_ignore(end())
        .map(|root| Document { root })
}

/// A parsed AML document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Root nodes of the document.
    pub root: Vec<Node>,
}

impl Document {
    /// Parse input, panicking on failure.
    pub fn new(input: &str) -> Self {
        Document::try_new(input).unwrap()
    }

    /// Parse input, returning errors instead of panicking.
    pub fn try_new(input: &str) -> Result<Self, Vec<Rich<'_, char>>> {
        document_parser().parse(input).into_result()
    }

    /// Render this document to ANSI-escaped text.
    pub fn render(&self) -> String {
        crate::render::render(self)
    }
}
