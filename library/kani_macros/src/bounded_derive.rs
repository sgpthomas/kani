use quote::quote;

pub(crate) fn expand_derive_bounded_arbitrary(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(item as syn::DeriveInput);

    match &parsed.data {
        syn::Data::Struct(data_struct) => derive_struct(&parsed, data_struct),
        syn::Data::Enum(data_enum) => derive_enum(data_enum),
        syn::Data::Union(data_union) => derive_union(data_union),
    }
}

fn derive_struct(
    input: &syn::DeriveInput,
    data_struct: &syn::DataStruct,
) -> proc_macro::TokenStream {
    let fields = data_struct.fields.iter().map(|field| {
        let ty = &field.ty;
        let any_call = if let Some(attr) = field.attrs.first() {
            let _path = attr.meta.require_path_only().unwrap();
            // TODO: actually check the path
            // (rn this relies in bounded being the only attr declared)
            quote!(bounded_arbitrary::bounded_any::<#ty, N>())
        } else {
            quote!(kani::any::<#ty>())
        };

        if let Some(ident) = &field.ident { quote!(#ident: #any_call) } else { quote!(#any_call) }
    });

    // parse generics
    let (generics, clauses) = quote_generics(&input.generics);

    let name = &input.ident;
    let named_fields = data_struct.fields.iter().all(|field| field.ident.is_some());
    if named_fields {
        quote! {
            impl #generics bounded_arbitrary::BoundedArbitrary for #name #generics
            #clauses
            {
                fn bounded_any<const N: usize>() -> Self {
                    Self {
                        #(#fields),*
                    }
                }
            }
        }
        .into()
    } else {
        quote! {
            impl bounded_arbitrary::BoundedArbitrary for #name {
                fn bounded_any<const N: usize>() -> Self {
                    Self (
                        #(#fields),*
                    )
                }
            }
        }
        .into()
    }
}

fn quote_generics(
    generics: &syn::Generics,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let params = generics.type_params().map(|param| quote!(#param)).collect::<Vec<_>>();
    let where_clauses = generics.type_params().map(|param| quote!(#param : kani::Arbitrary));
    if !params.is_empty() {
        (quote!(<#(#params),*>), quote!(where #(#where_clauses),*))
    } else {
        (quote!(), quote!())
    }
}

fn derive_enum(_data_enum: &syn::DataEnum) -> proc_macro::TokenStream {
    todo!()
}

fn derive_union(_data_union: &syn::DataUnion) -> proc_macro::TokenStream {
    todo!()
}
