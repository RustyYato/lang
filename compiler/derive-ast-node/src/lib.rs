use proc_macro::TokenStream;

mod imp;
mod imp_ast_node;
mod imp_maybe_ast_node;
mod imp_serialize_test;

#[proc_macro_derive(AstNode, attributes(node))]
pub fn derive_ast_node(ty: TokenStream) -> TokenStream {
    let ty = syn::parse_macro_input!(ty as syn::DeriveInput);
    imp_ast_node::derive_ast_node(ty).into()
}

#[proc_macro_derive(MaybeAstNode, attributes(node))]
pub fn derive_maybe_ast_node(ty: TokenStream) -> TokenStream {
    let ty = syn::parse_macro_input!(ty as syn::DeriveInput);
    imp_maybe_ast_node::derive_maybe_ast_node(ty).into()
}

#[proc_macro_derive(SerializeTest)]
pub fn derive_serialize_test(ty: TokenStream) -> TokenStream {
    let ty = syn::parse_macro_input!(ty as syn::DeriveInput);
    imp_serialize_test::derive_serialize_test(ty).into()
}

enum HelperAttribute {
    Spans,
    Ignore,
    Always,
}

impl syn::parse::Parse for HelperAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        syn::custom_keyword!(spans);

        if input.peek(spans) {
            input.parse::<spans>()?;

            return Ok(Self::Spans);
        }
        syn::custom_keyword!(ignore);

        if input.peek(ignore) {
            input.parse::<ignore>()?;

            return Ok(Self::Ignore);
        }
        syn::custom_keyword!(always);

        if input.peek(always) {
            input.parse::<always>()?;

            return Ok(Self::Always);
        }

        Err(syn::Error::new(input.span(), "Unknown node attribute"))
    }
}
