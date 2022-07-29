use crate::imp;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub(crate) fn derive_maybe_ast_node(ty: syn::DeriveInput) -> TokenStream {
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
            impl #impl_generics MaybeAstNode for #name #ty_generics #where_clause  {
                fn try_span<SpanPos: Position>(&self) -> Option<Span<SpanPos>>  {
                    Some(SpanPos::span(&self.#spans))
                }

                fn try_start<SpanPos: Position>(&self) -> Option<SpanPos> {
                    Some(SpanPos::start(&self.#spans))
                }

                fn try_end<SpanPos: Position>(&self) -> Option<SpanPos> {
                    Some(SpanPos::end(&self.#spans))
                }
            }
        },
        imp::StructFields::AllFields {
            all_fields: fields,
            start_finish: None,
            ..
        } => {
            let mut rev_fields = fields.clone();
            rev_fields.reverse();

            quote! {
                #[automatically_derived]
                impl #impl_generics MaybeAstNode for #name #ty_generics #where_clause  {
                    fn try_start<SpanPos: Position>(&self) -> Option<SpanPos> {
                        #(
                            if let Some(start) = MaybeAstNode::try_start(&self.#fields) {
                                return start
                            }
                        )*

                        None
                    }

                    fn try_end<SpanPos: Position>(&self) -> Option<SpanPos> {
                        #(
                            if let Some(end) = MaybeAstNode::try_end(&self.#rev_fields) {
                                return end
                            }
                        )*

                        None
                    }
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
                impl #impl_generics MaybeAstNode for #name #ty_generics #where_clause  {
                    fn try_start<SpanPos: Position>(&self) -> Option<SpanPos> {
                        #(
                            if let Some(start) = MaybeAstNode::try_start(&self.#fields) {
                                return Some(start)
                            }
                        )*

                        Some(AstNode::start(&self.#start_finish))
                    }

                    fn try_end<SpanPos: Position>(&self) -> Option<SpanPos> {
                        #(
                            if let Some(end) = MaybeAstNode::try_end(&self.#rev_fields) {
                                return Some(end)
                            }
                        )*

                        Some(AstNode::end(&self.#end_finish))
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
        impl #impl_generics MaybeAstNode for #name #ty_generics #where_clause {
            fn try_span<T: Position>(&self) -> Option<Span<T>> {
                match self {
                    #(Self::#variant_names(inner) => MaybeAstNode::try_span(inner),)*
                }
            }

            fn try_start<T: Position>(&self) -> Option<T> {
                match self {
                    #(Self::#variant_names(inner) => MaybeAstNode::try_start(inner),)*
                }
            }

            fn try_end<T: Position>(&self) -> Option<T> {
                match self {
                    #(Self::#variant_names(inner) => MaybeAstNode::try_end(inner),)*
                }
            }
        }
    }
}
