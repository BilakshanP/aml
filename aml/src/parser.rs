use std::collections::HashSet;

use chumsky::error::Rich;
use chumsky::prelude::*;

const BRIGHT_OFFSET: u8 = 60;
const BACKGROUND_OFFSET: u8 = 10;

type Err<'src> = extra::Err<Rich<'src, char>>;

// ── Colour ────────────────────────────────────────────────────────────────────

#[repr(u8)]
enum Variant {
    Fg = 0,
    Bg = BACKGROUND_OFFSET,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    /// Standard 8/16-colour ANSI palette.
    Ansi { clr: Clr, bright: bool },
    /// 24-bit true colour.
    Rgb { r: u8, g: u8, b: u8 },
    /// 256-colour fixed-palette index.
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

    pub fn fg_codes(&self) -> Vec<u8> {
        self.codes(Variant::Fg)
    }
    pub fn bg_codes(&self) -> Vec<u8> {
        self.codes(Variant::Bg)
    }
}

// ── Modifiers ─────────────────────────────────────────────────────────────────

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Modifiers(pub HashSet<Mdf>);

// ── Tag ───────────────────────────────────────────────────────────────────────

/// A parsed style tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    /// `<>…</>` — hard SGR reset.
    Reset,
    Fg(Colour),
    Bg(Colour),
    Mdf(Modifiers),
    /// Combined shorthand tag, e.g. `<s fg br mbi>`.
    Shorthand {
        fg: Option<Colour>,
        bg: Option<Colour>,
        mdf: Option<Modifiers>,
    },
    /// `<! 0 123 255>…</!>` — raw SGR codes, emitted verbatim.
    /// Transparent to the style stack; always followed by a reset on close.
    Raw(String),
}

// ── AST Node ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Tag { tag: Tag, children: Vec<Node> },
    Text(String),
}

impl Node {
    pub fn doc(self) -> Document {
        Document { root: vec![self] }
    }
}

// ── Parser helpers ────────────────────────────────────────────────────────────

/// One or more ASCII spaces.
fn wsp<'src>() -> impl Parser<'src, &'src str, (), Err<'src>> + Clone {
    just(' ')
        .repeated()
        .at_least(1)
        .ignored()
        .labelled("whitespace")
}

/// A decimal integer in `0..=255`.
fn byte<'src>() -> impl Parser<'src, &'src str, u8, Err<'src>> + Clone {
    text::int(10)
        .from_str()
        .unwrapped()
        .labelled("byte (0–255)")
}

/// Plain text; `\<` is an escape for a literal `<`.
fn text_node<'src>() -> impl Parser<'src, &'src str, Node, Err<'src>> + Clone {
    choice((just("\\<").to('<'), any().filter(|c: &char| *c != '<')))
        .repeated()
        .at_least(1)
        .collect()
        .map(Node::Text)
        .labelled("text")
}

// ── Colour parsers ────────────────────────────────────────────────────────────

fn fixed_colour<'src>() -> impl Parser<'src, &'src str, Colour, Err<'src>> + Clone {
    byte().map(Colour::Fixed).labelled("Fixed colour")
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
    .labelled("ANSI colour")
}

/// Decimal `R,G,B` triple, e.g. `0,128,255`.
fn rgb_colour<'src>() -> impl Parser<'src, &'src str, Colour, Err<'src>> + Clone {
    byte()
        .then_ignore(just(','))
        .then(byte())
        .then_ignore(just(','))
        .then(byte())
        .map(|((r, g), b)| Colour::Rgb { r, g, b })
        .labelled("RGB colour")
}

/// Hex colour — `#f`, `#ff`, `#abc`, `#aabbcc` (expanded automatically).
fn hex_colour<'src>() -> impl Parser<'src, &'src str, Colour, Err<'src>> + Clone {
    let hex_digit = any()
        .filter(|c: &char| c.is_ascii_hexdigit())
        .labelled("hex digit (0-9, a-f, A-F)");

    // Longest match first so `#abcdef` isn't parsed as `#abc`.
    let digits = choice((
        hex_digit.repeated().exactly(6).collect(),
        hex_digit.repeated().exactly(3).collect(),
        hex_digit.repeated().exactly(2).collect(),
        hex_digit.repeated().exactly(1).collect(),
    ));

    just('#')
        .ignore_then(digits)
        .map(expand_hex)
        .labelled("HEX colour")
}

fn expand_hex(s: String) -> Colour {
    let full: String = match s.len() {
        1 => s.repeat(6),
        2 => s.repeat(3),
        3 => s.chars().flat_map(|c| [c, c]).collect(),
        6 => s,
        _ => unreachable!("hex parser only yields 1/2/3/6 hex digits"),
    };

    Colour::Rgb {
        r: u8::from_str_radix(&full[0..2], 16).unwrap(),
        g: u8::from_str_radix(&full[2..4], 16).unwrap(),
        b: u8::from_str_radix(&full[4..6], 16).unwrap(),
    }
}

/// Any supported colour: hex → rgb → ansi → fixed-palette.
fn colour<'src>() -> impl Parser<'src, &'src str, Colour, Err<'src>> + Clone {
    choice((hex_colour(), rgb_colour(), ansi_colour(), fixed_colour()))
}

// ── Modifier parsers ──────────────────────────────────────────────────────────

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

// ── Shorthand arg parsers ─────────────────────────────────────────────────────
//
// A shorthand tag bundles fg / bg / mdf into one `<s …>` tag:
//   `<s fg br mbi>…</s>`  — foreground green, background red, bold + italic

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

// ── Node parser ───────────────────────────────────────────────────────────────

/// Builds a `<name …>…</name>` tag node.
///
/// - `attr_parser` — parses the attribute body between the name and `>`.
/// - `into_tag`    — converts the parsed attribute into a [`Tag`].
/// - `content`     — parses the child nodes.
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

pub fn node<'src>() -> impl Parser<'src, &'src str, Node, Err<'src>> + Clone {
    recursive(|node| {
        let content = node.repeated().collect();

        // `<>…</>` — hard SGR reset
        let reset = just("<>")
            .ignore_then(content.clone())
            .then_ignore(just("</>"))
            .map(|children| Node::Tag {
                tag: Tag::Reset,
                children,
            })
            .labelled("reset");

        // `<! 1 31 42>…</!>` — raw SGR codes, space-separated bytes
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

// ── Document ──────────────────────────────────────────────────────────────────

fn document_parser<'src>() -> impl Parser<'src, &'src str, Document, Err<'src>> {
    node()
        .repeated()
        .collect()
        .then_ignore(end())
        .map(|root| Document { root })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub root: Vec<Node>,
}

impl Document {
    /// Panics on failure.
    pub fn new(input: &str) -> Self {
        Document::try_new(input).unwrap()
    }

    /// Parse `input`, returning errors instead of panicking.
    pub fn try_new(input: &str) -> Result<Self, Vec<Rich<'_, char>>> {
        document_parser().parse(input).into_result()
    }

    pub fn render(&self) -> String {
        crate::render::render(self)
    }
}
