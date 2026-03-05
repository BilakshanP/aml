use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Error, LitStr, parse_macro_input};

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
                // Map your parse error offsets back to a span if possible,
                // otherwise fall back to the literal's span
                Error::new(Span::call_site(), err.reason().to_string()).to_compile_error()
            });

            quote! { #(#errors)* }.into()
        }
    }
}
