use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn derive_serialize_test(ty: syn::DeriveInput) -> TokenStream {
    match ty.data {
        syn::Data::Struct(_) => derive_serialize_test_struct(ty),
        syn::Data::Enum(_) => derive_serialize_test_enum(ty),
        syn::Data::Union(_) => todo!(),
    }
}

fn derive_serialize_test_struct(ty: syn::DeriveInput) -> TokenStream {
    let data = match ty.data {
        syn::Data::Struct(data) => data,
        _ => unreachable!(),
    };

    let name = ty.ident;
    let (impl_generics, ty_generics, where_clause) = ty.generics.split_for_impl();
    let field_names = data.fields.iter().map(|field| &field.ident);

    let type_name = name.to_string();

    quote!(
        impl #impl_generics SerializeTest for #name #ty_generics #where_clause {
            fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, #type_name)?;
                write!(f, "(")?;
                #(
                    self.#field_names.serialize(f)?;
                    write!(f, ",")?;
                )*
                write!(f, ")")?;
                Ok(())
            }
        }
    )
}

fn derive_serialize_test_enum(ty: syn::DeriveInput) -> TokenStream {
    let data = match ty.data {
        syn::Data::Enum(data) => data,
        _ => unreachable!(),
    };

    let mut variant_names = Vec::new();

    for variant in &data.variants {
        variant_names.push(&variant.ident);
    }

    let name = ty.ident;
    let (impl_generics, ty_generics, where_clause) = ty.generics.split_for_impl();
    let type_name = name.to_string();

    quote! {
        impl #impl_generics SerializeTest for #name #ty_generics #where_clause {
            fn serialize(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, #type_name)?;
                let inner: &dyn SerializeTest = match self {
                    #(Self::#variant_names(inner) => inner,)*
                };
                write!(f, "(")?;
                inner.serialize(f)?;
                write!(f, ")")
            }
        }
    }
}
