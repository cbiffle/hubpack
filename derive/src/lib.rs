extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote_spanned;
use syn;
use syn::spanned::Spanned;

#[proc_macro_derive(SerializedSize)]
pub fn size_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let dispatch = gen_dispatch(&name, &input.data);

    let expanded = quote_spanned! {name.span()=>
        impl #impl_generics ::hubpack::SerializedSize for #name #ty_generics
        #where_clause {
            const MAX_SIZE: usize = #dispatch;
        }
    };

    TokenStream::from(expanded)
}

/// Naively slaps a bound on every generic type parameter. This leads to
/// overconstrained impls but it's sure easy -- and it's essentially what the
/// built in derives do.
fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            type_param
                .bounds
                .push(syn::parse_quote!(::hubpack::SerializedSize));
        }
    }
    generics
}

fn gen_dispatch(ty: &syn::Ident, data: &syn::Data) -> proc_macro2::TokenStream {
    match data {
        syn::Data::Struct(data) => gen_fields(ty, &data.fields),
        syn::Data::Enum(data) => {
            let variants = data.variants.iter().map(|v| gen_fields(ty, &v.fields));

            // We now need to take the maximum of the variant sizes, and
            // then add one for the variant index.
            quote_spanned! {ty.span() =>
                {
                    let mut __max__ = 0usize;

                    #(
                        let __next__ = #variants;
                        if __next__ > __max__ {
                            __max__ = __next__;
                        }
                    )*

                        __max__ + 1
                }
            }
        }
        syn::Data::Union(_) => {
            unimplemented!("Unions are not supported")
        }
    }
}

/// Generates size expression for a sequence of fields.
fn gen_fields_size<'a>(
    ty: &syn::Ident,
    fields: impl IntoIterator<Item = &'a syn::Field>,
) -> proc_macro2::TokenStream {

    let mut stmts = fields
        .into_iter()
        .map(|f| {
            let ty = &f.ty;
            quote_spanned! {f.span()=>
                <#ty as ::hubpack::SerializedSize>::MAX_SIZE
            }
        })
        .peekable();

    if stmts.peek().is_some() {
        quote_spanned! {ty.span()=> #( #stmts )+* }
    } else {
        gen_unit(ty)
    }
}

fn gen_fields(
    ty: &syn::Ident,
    fields: &syn::Fields,
) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => gen_named_struct(ty, fields),
        syn::Fields::Unnamed(fields) => gen_tuple_struct(ty, fields),
        syn::Fields::Unit => gen_unit(ty),
    }
}

fn gen_unit(
    ty: &syn::Ident,
) -> proc_macro2::TokenStream {
    quote_spanned! {ty.span()=> 0 }
}

/// Generates size expression for a struct with named fields.
fn gen_named_struct(
    ty: &syn::Ident,
    fields: &syn::FieldsNamed,
) -> proc_macro2::TokenStream {
    gen_fields_size(ty, &fields.named)
}

fn gen_tuple_struct(
    ty: &syn::Ident,
    fields: &syn::FieldsUnnamed,
) -> proc_macro2::TokenStream {
    gen_fields_size(ty, &fields.unnamed)
}
