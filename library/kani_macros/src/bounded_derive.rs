use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn expand_derive_bounded_arbitrary(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(item as syn::DeriveInput);

    let constructor = match &parsed.data {
        syn::Data::Struct(data_struct) => {
            generate_type_constructor(quote!(Self), &data_struct.fields)
        }
        syn::Data::Enum(data_enum) => enum_constructor(&parsed.ident, data_enum),
        syn::Data::Union(data_union) => union_constructor(data_union),
    };

    // parse generics
    let (generics, clauses) = quote_generics(&parsed.generics);
    let name = &parsed.ident;

    // generate the implementation
    quote! {
        impl #generics kani::BoundedArbitrary for #name #generics
            #clauses
        {
            fn bounded_any<const N: usize>() -> Self {
                #constructor
            }
        }
    }
    .into()
}

fn generate_type_constructor(type_name: TokenStream, fields: &syn::Fields) -> TokenStream {
    let field_calls = fields.iter().map(generate_any_call);
    if fields.iter().all(|f| f.ident.is_some()) {
        quote!(#type_name { #(#field_calls),* })
    } else {
        quote!(#type_name( #(#field_calls),* ))
    }
}

fn enum_constructor(ident: &syn::Ident, data_enum: &syn::DataEnum) -> TokenStream {
    let variant_constructors = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        generate_type_constructor(quote!(#ident::#variant_name), &variant.fields)
    });
    let n_variants = data_enum.variants.len();
    let cases =
        variant_constructors.enumerate().map(|(idx, var_constr)| quote!(#idx => #var_constr));

    quote! {
        let which_variant: usize = kani::any_where(|n| *n < #n_variants);
        match which_variant {
            #(#cases),* ,
            _ => unreachable!()
        }
    }
}

fn union_constructor(_data_union: &syn::DataUnion) -> TokenStream {
    todo!()
}

fn quote_generics(generics: &syn::Generics) -> (Option<TokenStream>, Option<TokenStream>) {
    let params = generics.type_params().map(|param| quote!(#param)).collect::<Vec<_>>();
    let where_clauses = generics.type_params().map(|param| quote!(#param : kani::BoundedArbitrary));
    if !params.is_empty() {
        (Some(quote!(<#(#params),*>)), Some(quote!(where #(#where_clauses),*)))
    } else {
        (None, None)
    }
}

fn generate_any_call(field: &syn::Field) -> TokenStream {
    let ty = &field.ty;
    let any_call = if let Some(attr) = field.attrs.first() {
        if let Ok(_path) = attr.meta.require_path_only() {
            quote!(kani::bounded_any::<#ty, N>())
        } else {
            quote!(kani::any::<#ty>())
        }
    } else {
        quote!(kani::any::<#ty>())
    };

    let ident_tok = field.ident.as_ref().map(|ident| quote!(#ident: ));
    quote!(#ident_tok #any_call)
}
