use crate::imp;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub(crate) fn derive_ast_node(ty: syn::DeriveInput) -> TokenStream {
    match ty.data {
        syn::Data::Struct(_) => derive_ast_node_struct(ty),
        syn::Data::Enum(_) => derive_ast_node_enum(ty),
        syn::Data::Union(_) => todo!(),
    }
}

fn derive_ast_node_struct(ty: syn::DeriveInput) -> TokenStream {
    let data = match ty.data {
        syn::Data::Struct(data) => data,
        _ => unreachable!(),
    };

    let mut errors = TokenStream::new();

    let fields = imp::struct_fields(&data, &mut errors);

    if !errors.is_empty() {
        return errors;
    }

    let name = ty.ident;
    let (impl_generics, ty_generics, where_clause) = ty.generics.split_for_impl();

    match fields {
        imp::StructFields::Spans(spans) => quote! {
            #[automatically_derived]
            impl #impl_generics AstNode for #name #ty_generics #where_clause  {
                fn span<SpanPos: Position>(&self) -> Span<SpanPos>  {
                    SpanPos::span(&self.#spans)
                }

                fn start<SpanPos: Position>(&self) -> SpanPos {
                    SpanPos::start(&self.#spans)
                }

                fn end<SpanPos: Position>(&self) -> SpanPos {
                    SpanPos::end(&self.#spans)
                }
            }
        },
        imp::StructFields::AllFields {
            start_finish: None, ..
        } => {
            quote! {
                compile_error! {
                    "at least one field must be marked #{node(always)]"
                }
            }
        }
        imp::StructFields::AllFields {
            all_fields: fields,
            start_finish: Some(start_finish),
            end_finish,
        } => {
            let mut rev_fields = fields.clone();
            let rev_fields = &mut rev_fields[end_finish..];
            rev_fields.reverse();
            let fields = &fields[..=start_finish];

            let (start_finish, fields) = fields.split_last().unwrap();
            let (end_finish, rev_fields) = rev_fields.split_first().unwrap();

            quote! {
                #[automatically_derived]
                impl #impl_generics AstNode for #name #ty_generics #where_clause  {
                    fn start<SpanPos: Position>(&self) -> SpanPos {
                        #(
                            if let Some(start) = MaybeAstNode::try_start(&self.#fields) {
                                return start
                            }
                        )*

                        AstNode::start(&self.#start_finish)
                    }

                    fn end<SpanPos: Position>(&self) -> SpanPos {
                        #(
                            if let Some(end) = MaybeAstNode::try_end(&self.#rev_fields) {
                                return end
                            }
                        )*

                        AstNode::end(&self.#end_finish)
                    }
                }
            }
        }
    }
}

fn derive_ast_node_enum(ty: syn::DeriveInput) -> TokenStream {
    let data = match ty.data {
        syn::Data::Enum(data) => data,
        _ => unreachable!(),
    };

    let mut variant_names = Vec::new();

    let mut errors = TokenStream::new();

    for variant in data.variants {
        match variant.fields {
            syn::Fields::Named(_) => errors.extend(quote_spanned! {
                variant.span() => compile_error!{ "Enums with named fields are not supported" }
            }),
            syn::Fields::Unnamed(ref fields) => {
                if fields.unnamed.is_empty() {
                    errors.extend(quote_spanned! {
                        variant.span() => compile_error!{ "Enums with no fields are not supported" }
                    })
                } else if fields.unnamed.len() != 1 {
                    errors.extend(quote_spanned! {
                        variant.span() => compile_error!{ "Enums with multiple fields are not supported" }
                    })
                }

                variant_names.push(variant.ident);
            }
            syn::Fields::Unit => errors.extend(quote_spanned! {
                variant.span() => compile_error!{ "Enums with no fields are not supported" }
            }),
        }
    }

    if !errors.is_empty() {
        return errors;
    }

    let name = ty.ident;
    let (impl_generics, ty_generics, where_clause) = ty.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics AstNode for #name #ty_generics #where_clause {
            fn span<T: Position>(&self) -> Span<T> {
                match self {
                    #(Self::#variant_names(inner) => AstNode::span(inner),)*
                }
            }

            fn start<T: Position>(&self) -> T {
                match self {
                    #(Self::#variant_names(inner) => AstNode::start(inner),)*
                }
            }

            fn end<T: Position>(&self) -> T {
                match self {
                    #(Self::#variant_names(inner) => AstNode::end(inner),)*
                }
            }
        }
    }
}
