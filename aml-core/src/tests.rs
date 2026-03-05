use crate::parser::Document;
use crate::render::{CSI, RESET, render};

// ── Helpers ───────────────────────────────────────────────────────────────

fn csi(s: &str) -> String {
    format!("{CSI}{s}m")
}

fn rendered(input: &str) -> String {
    render(&Document::new(input))
}

// ── Baseline ──────────────────────────────────────────────────────────────

#[test]
fn plain_text_passes_through_unchanged() {
    assert_eq!(rendered("hello world"), "hello world");
}

#[test]
fn empty_input_produces_empty_output() {
    assert_eq!(rendered(""), "");
}

#[test]
fn escaped_angle_bracket_passes_through() {
    assert_eq!(rendered("a\\<b"), "a<b");
}

#[test]
fn unstyled_text_between_tags_has_no_escape_sequences() {
    let out = rendered("before<fr>styled</f>after");
    assert!(out.starts_with("before"), "prefix should be plain: {out:?}");
    let after_idx = out.rfind("after").unwrap();
    assert_eq!(
        &out[after_idx..],
        "after",
        "'after' should have no trailing escape: {out:?}"
    );
}

// ── Single tags ───────────────────────────────────────────────────────────

#[test]
fn single_fg_tag() {
    assert_eq!(rendered("<fr>red</f>"), format!("{}red{RESET}", csi("31")));
}

#[test]
fn single_bg_tag() {
    assert_eq!(
        rendered("<bb>text</b>"),
        format!("{}text{RESET}", csi("44"))
    );
}

#[test]
fn single_modifier_bold() {
    assert_eq!(rendered("<mb>text</m>"), format!("{}text{RESET}", csi("1")));
}

// ── Pure additions: self-contained sequences ──────────────────────────────

#[test]
fn adding_modifier_to_fg_includes_fg_in_sequence() {
    // <fr> red <mi> red italic </m> red </f>
    //
    // Transition trace:
    //   {} → {fg:red}               = ESC[31m]
    //   {fg:red} → {fg:red,italic}  = ESC[31;3m]   ← fg always included
    //   {fg:red,italic} → {fg:red}  = ESC[0;31m]   ← italic removed: reset+reapply
    //   {fg:red} → {}               = ESC[0m]
    let out = rendered("<fr> red <mi> red italic </m> red </f>");
    assert_eq!(
        out,
        format!(
            "{} red {} red italic {} red {RESET}",
            csi("31"),
            csi("31;3"),
            csi("0;31")
        )
    );
}

#[test]
fn adding_bg_to_fg_includes_fg_in_sequence() {
    // <fr> red <bb> red blue </b> red </f>
    //
    // Transition trace:
    //   {} → {fg:red}                 = ESC[31m]
    //   {fg:red} → {fg:red, bg:blue}  = ESC[31;44m]  ← fg always included
    //   {fg:red,bg:blue} → {fg:red}   = ESC[0;31m]   ← bg removed: reset+reapply
    //   {fg:red} → {}                 = ESC[0m]
    let out = rendered("<fr> red <bb> red blue </b> red </f>");
    assert_eq!(
        out,
        format!(
            "{} red {} red blue {} red {RESET}",
            csi("31"),
            csi("31;44"),
            csi("0;31")
        )
    );
}

#[test]
fn adding_modifiers_to_fg_includes_fg_in_sequence() {
    // <fr><fr> A <mb><mo> B </m></m> C </f></f>
    //
    // Transition trace:
    //   {} → {fg:red}                        = ESC[31m]
    //   {fg:red} → {fg:red, bold, overline}  = ESC[31;1;53m]  ← fg always included
    //   {fg:red,bold,overline} → {fg:red}    = ESC[0;31m]     ← modifiers removed
    //   {fg:red} → {}                         = ESC[0m]
    let out = rendered("<fr><fr> A <mb><mo> B </m></m> C </f></f>");
    assert_eq!(
        out,
        format!(
            "{} A {} B {} C {RESET}",
            csi("31"),
            csi("31;1;53"),
            csi("0;31")
        )
    );
}

#[test]
fn changing_fg_colour_emits_only_new_fg_code() {
    // <fr><fg>green</f>red</f>
    //
    // Transition trace:
    //   {} → {fg:green}            = ESC[32m]  ← innermost fg wins
    //   {fg:green} → {fg:red}     = ESC[31m]  ← fg swap, pure addition path
    //   {fg:red} → {}              = ESC[0m]
    let out = rendered("<fr><fg>green</f>red</f>");
    assert_eq!(out, format!("{}green{}red{RESET}", csi("32"), csi("31")));
    // No reset should appear before the final one
    let last_reset = out.rfind(RESET).unwrap();
    assert!(
        !out[..last_reset].contains(RESET),
        "unexpected mid-sequence RESET: {out:?}"
    );
}

// ── Modifier accumulation ─────────────────────────────────────────────────

#[test]
fn modifier_layers_accumulate() {
    // <mb><mi>text</m></m>  — two layers, both active when "text" is reached
    let out = rendered("<mb><mi>text</m></m>");
    assert_eq!(out, format!("{}text{RESET}", csi("1;3")));
}

#[test]
fn multiple_modifiers_in_single_tag() {
    // <mbi>text</m>  — bold+italic in one tag
    let out = rendered("<mbi>text</m>");
    assert_eq!(out, format!("{}text{RESET}", csi("1;3")));
}

#[test]
fn duplicate_modifier_across_layers_deduplicated() {
    // <mb><mb>text</m></m>  — bold only, not doubled
    let out = rendered("<mb><mb>text</m></m>");
    assert_eq!(out, format!("{}text{RESET}", csi("1")));
}

// ── Nested identical tags ─────────────────────────────────────────────────

#[test]
fn nested_identical_fg_tags_no_redundant_sequences() {
    // <fr><fr>X<by> Hi </b>X</f></f>
    //
    // Both fg layers resolve to red — inner open is a no-op transition.
    //
    // Transition trace:
    //   {} → {fg:red}                   = ESC[31m]
    //   {fg:red} → {fg:red, bg:yellow}  = ESC[31;43m]  ← fg included
    //   {fg:red,bg:yellow} → {fg:red}   = ESC[0;31m]   ← bg removed
    //   {fg:red} → {}                    = ESC[0m]
    let out = rendered("<fr><fr>X<by> Hi </b>X</f></f>");
    assert_eq!(
        out,
        format!("{}X{} Hi {}X{RESET}", csi("31"), csi("31;43"), csi("0;31"))
    );
    // ESC[31m appears exactly once — identical inner fg is a no-op
    assert_eq!(
        out.matches(&csi("31")).count(),
        1,
        "ESC[31m emitted more than once: {out:?}"
    );
}

#[test]
fn no_duplicate_csi_on_tag_close_when_parent_same_style() {
    let out = rendered("<fr><fr>text</f>more</f>");
    assert_eq!(
        out.matches(&csi("31")).count(),
        1,
        "ESC[31m emitted more than once: {out:?}"
    );
}

// ── Reset tag (<> … </>) ──────────────────────────────────────────────────

#[test]
fn reset_tag_always_emits_reset_at_entry_even_with_no_outer_style() {
    // Explicit user reset — must emit RESET regardless of tracked state.
    let out = rendered("<>hello</>");
    assert_eq!(out, format!("{RESET}hello"));
}

#[test]
fn reset_tag_inside_styled_context() {
    // <fr> red <bb> red blue <> normal </> red blue </b> red </f>
    //
    // Transition trace:
    //   {} → {fg:red}                      = ESC[31m]
    //   {fg:red} → {fg:red, bg:blue}       = ESC[31;44m]   ← fg included
    //   reset open                         → ESC[0m]        ← eager explicit reset
    //   {} → {}                              = (nothing)      ← " normal " unstyled
    //   reset close: {} → {fg:red, bg:blue} = ESC[31;44m]   ← full re-apply from default
    //   {fg:red,bg:blue} → same            = (nothing)      ← " red blue " already current
    //   {fg:red,bg:blue} → {fg:red}        = ESC[0;31m]    ← bg removed
    //   {fg:red} → {}                       = ESC[0m]
    let out = rendered("<fr> red <bb> red blue <> normal </> red blue </b> red </f>");
    assert_eq!(
        out,
        format!(
            "{} red {} red blue {RESET} normal {} red blue {} red {RESET}",
            csi("31"),
            csi("31;44"),
            csi("31;44"),
            csi("0;31")
        )
    );
}

#[test]
fn reset_tag_restores_full_context_on_close() {
    // After </> the full parent style must be re-emitted before "after".
    let out = rendered("<fr><bb>before<>mid</>after</b></f>");
    let after_idx = out.find("after").expect("'after' not in output");
    assert!(
        out[..after_idx].ends_with(&csi("31;44")),
        "expected ESC[31;44m before 'after': {out:?}"
    );
}

// ── Shorthand tags ────────────────────────────────────────────────────────

#[test]
fn shorthand_fg_and_modifier() {
    let out = rendered("<s fr mbi>text</s>");
    assert_eq!(out, format!("{}text{RESET}", csi("31;1;3")));
}

#[test]
fn shorthand_fg_bg_mdf() {
    let out = rendered("<s fr bb mbi>text</s>");
    assert_eq!(out, format!("{}text{RESET}", csi("31;44;1;3")));
}

#[test]
fn shorthand_nested_inside_fg_uses_minimal_transitions() {
    // <fr>a<s bb mbi>b</s>c</f>
    //
    // Transition trace:
    //   {} → {fg:red}                               = ESC[31m]
    //   {fg:red} → {fg:red, bg:blue, bold, italic}  = ESC[31;44;1;3m]  ← fg included
    //   {fg:red,bg:blue,bold,italic} → {fg:red}     = ESC[0;31m]       ← bg+mods removed
    //   {fg:red} → {}                                = ESC[0m]
    let out = rendered("<fr>a<s bb mbi>b</s>c</f>");
    let a_idx = out.find('a').unwrap();
    assert!(
        out[..a_idx].ends_with(&csi("31")),
        "expected ESC[31m before 'a': {out:?}"
    );
    let b_idx = out.find('b').unwrap();
    assert!(
        out[..b_idx].ends_with(&csi("31;44;1;3")),
        "expected ESC[31;44;1;3m before 'b': {out:?}"
    );
    let c_idx = out.find('c').unwrap();
    assert!(
        out[..c_idx].ends_with(&csi("0;31")),
        "expected ESC[0;31m before 'c': {out:?}"
    );
}

// ── Invariants ────────────────────────────────────────────────────────────

#[test]
fn standalone_reset_never_appears_mid_sequence_outside_reset_tag() {
    // A bare ESC[0m only appears at document end. Mid-sequence removals use
    // ESC[0;...m (reset+reapply), which is distinct from a bare ESC[0m.
    let inputs = [
        "<fr> red <mi> red italic </m> red </f>",
        "<fr> red <bb> red blue </b> red </f>",
        "<fr><fr>X<by> Hi </b>X</f></f>",
        "<fr><fr> A <mb><mo> B </m></m> C </f></f>",
        "<fr>a<s bb mbi>b</s>c</f>",
    ];
    for input in inputs {
        let out = rendered(input);
        let last_reset = out.rfind(RESET).expect("should end with RESET");
        assert!(
            !out[..last_reset].contains(RESET),
            "standalone RESET found mid-sequence in {input:?}: {out:?}"
        );
    }
}

// ── Raw tag (<! …> … </!>) ────────────────────────────────────────────────────

#[test]
fn raw_tag_emits_codes_verbatim() {
    // <! 53> — raw overline (code 53), which the type system doesn't model
    let out = rendered("<!53m>overlined</!>");
    assert_eq!(out, format!("{}overlined{RESET}", csi("53")));
}

#[test]
fn raw_tag_multiple_codes() {
    let out = rendered("<!1;3;53;a;bcm>text</!>");
    assert_eq!(out, format!("{}text{RESET}", csi("1;3;53;a;bc")));
}

#[test]
fn raw_tag_always_resets_on_close() {
    // Even with no outer style, </!> must emit a bare RESET.
    let out = rendered("<! 53>text</!>");
    assert!(out.ends_with(RESET), "expected trailing RESET: {out:?}");
}

#[test]
fn raw_tag_restores_parent_context_on_close() {
    // <fr> red <! 53> overlined red </!> red </f>
    //
    // Transition trace:
    //   {} → {fg:red}      = ESC[31m]
    //   raw open           → ESC[53m]   ← verbatim, current unchanged
    //   {fg:red} → {fg:red} = (nothing) ← child text, no transition needed
    //   raw close          → ESC[0m]    ← universal wipe
    //   {} → {fg:red}      = ESC[31m]   ← parent context restored
    //   {fg:red} → {}       = ESC[0m]
    let out = rendered("<fr> red <!53m> overlined red </!> red </f>");
    assert_eq!(
        out,
        format!(
            "{} red {} overlined red {RESET}{} red {RESET}",
            csi("31"),
            csi("53"),
            csi("31")
        )
    );
}

#[test]
fn raw_tag_is_transparent_to_outer_style_resolution() {
    // The raw tag must not affect how siblings resolve their styles.
    // Text after </!> should see exactly the same state as before <!>.
    let out = rendered("<fr><! 53>x</!>y</f>");
    let y_idx = out.find('y').unwrap();
    // After raw close: RESET then ESC[31m re-applied; 'y' must follow ESC[31m].
    assert!(
        out[..y_idx].ends_with(&csi("31")),
        "expected ESC[31m before 'y': {out:?}"
    );
}

#[test]
fn raw_tag_nested_inside_reset_tag() {
    // <> <! 53>text</!> </> — raw inside an explicit reset context
    let out = rendered("<><!53m>text</!></>");
    assert!(out.contains(&csi("53")), "raw code missing: {out:?}");
}

#[test]
fn outer_style_does_not_bleed_into_post_raw_close() {
    // After </!> the renderer resets and re-applies — so there should be
    // exactly one ESC[31m before content following the raw span.
    let out = rendered("<fr>a<! 53>b</!>c</f>");
    let c_idx = out.find('c').unwrap();
    assert!(
        out[..c_idx].ends_with(&csi("31")),
        "expected ESC[31m directly before 'c': {out:?}"
    );
}
