use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Error, LitStr, parse_macro_input};

/// Compile an AML style string into ANSI escape codes at compile time.
///
/// Takes a string literal containing a style specification and generates code
/// that creates a CompiledStyle with the corresponding ANSI escape codes.
///
/// # Example
///
/// ```ignore
/// let style = style!("f#ff0000 mbi");  // red foreground, bold, italic
/// println!("{}{}{}", style, "Hello", aml::render::RESET);
/// ```
///
/// The style string uses the same format as Style::new():
/// - `f<color>` - foreground color
/// - `b<color>` - background color
/// - `m<mods>` - text modifiers
///
/// Colors can be:
/// - Hex: #f, #ff, #abc, #aabbcc
/// - RGB: 255,0,0
/// - ANSI names: r, g, b, c, m, y, w, k (bright: R, G, B, etc.)
/// - Fixed: 0-255
///
/// Modifiers: b=bold, d=dim, i=italic, u=underline, k=blink, r=rapid,
/// v=invert, h=hide, s=strike, l=double-underline, o=overline
#[proc_macro]
pub fn style(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let src = lit.value();

    match aml_core::styler::Style::new(&src) {
        Ok(style) => {
            let compiled = style.compile();
            quote! { #compiled }.into()
        }
        Err(errs) => {
            let errors = errs.iter().map(|err| {
                Error::new(Span::call_site(), err.reason().to_string()).to_compile_error()
            });

            quote! { #(#errors)* }.into()
        }
    }
}
