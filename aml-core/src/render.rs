use std::collections::HashSet;

use crate::parser::{Colour, Document, Mdf, Node, Tag};

// ANSI escape helpers

pub const CSI: &str = "\x1b[";
pub const RESET: &str = "\x1b[0m";

pub(crate) fn wrap(codes: &[u8]) -> String {
    let inner = codes
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(";");

    format!("{CSI}{inner}m")
}

// Terminal state

/// The SGR attributes currently active in the terminal, inferred from emitted codes.
/// Used to compute minimal state transitions.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct TermState {
    fg: Option<Colour>,
    bg: Option<Colour>,
    mdf: HashSet<Mdf>,
}

impl TermState {
    fn is_default(&self) -> bool {
        self.fg.is_none() && self.bg.is_none() && self.mdf.is_empty()
    }
}

/// Compute the minimal CSI sequence to move the terminal from `from` to `to`.
/// Returns `None` if no transition is needed.
///
/// Strategy:
/// - If `to` is default, emit a bare ESC[0m reset (always cheapest).
/// - If something must be removed (fg/bg cleared or a modifier dropped),
///   emit ESC[0;...m - reset then re-apply everything, since SGR provides
///   no reliable per-attribute off codes across all terminals.
/// - If only additions are needed, emit only new codes and let the terminal's
///   cumulative SGR behavior handle the rest.
fn transition(from: &TermState, to: &TermState) -> Option<String> {
    if from == to {
        return None;
    }

    if to.is_default() {
        return Some(RESET.to_string());
    }

    let needs_reset = (from.fg.is_some() && to.fg.is_none())
        || (from.bg.is_some() && to.bg.is_none())
        || from.mdf.difference(&to.mdf).next().is_some();

    if needs_reset {
        let mut codes: Vec<u8> = vec![0];

        codes.extend(to.fg.as_ref().map_or(vec![], Colour::fg_codes));
        codes.extend(to.bg.as_ref().map_or(vec![], Colour::bg_codes));
        codes.extend(sorted_mdf_codes(&to.mdf));

        return Some(wrap(&codes));
    }

    // Pure additions - include unchanged fg/bg too so the sequence is
    // self-contained and survives future modifier additions.
    let mut codes: Vec<u8> = Vec::new();

    codes.extend(to.fg.as_ref().map_or(vec![], Colour::fg_codes));
    codes.extend(to.bg.as_ref().map_or(vec![], Colour::bg_codes));
    codes.extend(sorted_mdf_codes(
        &to.mdf.difference(&from.mdf).copied().collect(),
    ));

    (!codes.is_empty()).then(|| wrap(&codes))
}

fn sorted_mdf_codes(mdf: &HashSet<Mdf>) -> Vec<u8> {
    let mut v: Vec<u8> = mdf.iter().map(|&m| m as u8).collect();
    v.sort_unstable();
    v
}

// Style stack

#[derive(Debug, Default)]
struct StyleStack(Vec<Tag>);

impl StyleStack {
    fn push(&mut self, tag: Tag) {
        self.0.push(tag);
    }
    fn pop(&mut self) {
        self.0.pop();
    }

    /// Resolve the current style by walking layers innermost-first,
    /// accumulating foreground, background, and modifiers.
    /// Stops at the first Reset tag encountered.
    fn resolve(&self) -> TermState {
        let mut state = TermState::default();

        for tag in self.0.iter().rev() {
            match tag {
                Tag::Reset => break,

                Tag::Fg(c) => {
                    state.fg.get_or_insert(*c);
                }
                Tag::Bg(c) => {
                    state.bg.get_or_insert(*c);
                }
                Tag::Mdf(m) => state.mdf.extend(&m.0),

                // Raw codes are opaque - they don't participate in stack resolution.
                Tag::Raw(_) => {}

                Tag::Shorthand { fg, bg, mdf } => {
                    if let Some(c) = fg {
                        state.fg.get_or_insert(*c);
                    }
                    if let Some(c) = bg {
                        state.bg.get_or_insert(*c);
                    }
                    if let Some(m) = mdf {
                        state.mdf.extend(&m.0);
                    }
                }
            }
        }

        state
    }
}

// Renderer

fn render_nodes(nodes: &[Node], stack: &mut StyleStack, out: &mut String, current: &mut TermState) {
    for node in nodes {
        match node {
            Node::Text(text) => {
                let desired = stack.resolve();
                if let Some(seq) = transition(current, &desired) {
                    out.push_str(&seq);
                    *current = desired;
                }
                out.push_str(text);
            }

            Node::Tag { tag, children } => {
                let is_reset = matches!(tag, Tag::Reset);
                let is_raw = matches!(tag, Tag::Raw(_));

                if is_reset {
                    // Explicit reset tag - emit immediately regardless of state.
                    out.push_str(RESET);
                    *current = TermState::default();
                }

                if is_raw {
                    // Emit raw CSI sequence immediately. Don't update current state
                    // since raw codes are transparent. Children will emit their own
                    // transitions on top.
                    if let Tag::Raw(codes) = tag {
                        out.push_str(&format!("{CSI}{codes}"));
                    }
                }

                stack.push(tag.clone());
                render_nodes(children, stack, out, current);
                stack.pop();

                if is_reset {
                    // Restore parent context after closing reset tag.
                    let desired = stack.resolve();

                    if let Some(seq) = transition(current, &desired) {
                        out.push_str(&seq);
                        *current = desired;
                    }
                }

                if is_raw {
                    // Emit universal reset after raw tag, then re-apply parent context.
                    out.push_str(RESET);
                    *current = TermState::default();
                    let desired = stack.resolve();

                    if let Some(seq) = transition(current, &desired) {
                        out.push_str(&seq);
                        *current = desired;
                    }
                }

                // Non-reset/raw tags emit nothing on close - the next Text node
                // diffs lazily and emits only what changed.
            }
        }
    }
}

/// Render a document to ANSI-escaped text.
///
/// The output is an ANSI-formatted string ready for terminal display. If any
/// style attributes are left active at the end, a trailing reset is added.
pub fn render(doc: &Document) -> String {
    let mut out = String::new();
    let mut stack = StyleStack::default();
    let mut current = TermState::default();

    render_nodes(&doc.root, &mut stack, &mut out, &mut current);

    if !current.is_default() {
        out.push_str(RESET);
    }

    out
}
