use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::HelperAttribute;

pub enum StructFields {
    Spans(TokenStream),
    AllFields {
        all_fields: Vec<TokenStream>,
        start_finish: Option<usize>,
        end_finish: usize,
    },
}

pub fn struct_fields(data: &syn::DataStruct, errors: &mut TokenStream) -> StructFields {
    let mut spans = None;
    let mut fields = Vec::new();

    let mut start_finish = None;
    let mut end_finish = 0;

    for (i, field) in data.fields.iter().enumerate() {
        let mut ignored = false;

        let field_name = if let Some(ident) = &field.ident {
            quote!(#ident)
        } else {
            let ident = syn::Index {
                index: i as u32,
                span: field.span(),
            };
            quote!(#ident)
        };

        for attr in &field.attrs {
            if !attr.path.is_ident("node") {
                continue;
            }

            let attr = match attr.parse_args::<HelperAttribute>() {
                Ok(attr) => attr,
                Err(err) => {
                    errors.extend(err.into_compile_error());
                    continue;
                }
            };

            match attr {
                HelperAttribute::Spans => spans = Some(field_name.clone()),
                HelperAttribute::Ignore => ignored = true,
                HelperAttribute::Always => {
                    let i = fields.len();
                    start_finish.get_or_insert(i);
                    end_finish = i;
                }
            }
        }

        if !ignored {
            fields.push(field_name)
        }
    }

    if let Some(spans) = spans {
        StructFields::Spans(spans)
    } else {
        StructFields::AllFields {
            all_fields: fields,
            start_finish,
            end_finish,
        }
    }
}
