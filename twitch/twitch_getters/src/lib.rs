//! A proc macro for generating getters for `UnsafeSlice` fields. See the
//! documentation on [`twitch_getters`] for more info.
//!
//! [`twitch_getters`]: crate::twitch_getters
#![feature(proc_macro_diagnostic)]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{spanned::Spanned, ItemStruct};

const UNSAFE_SLICE_TYPE_NAME: &str = "UnsafeSlice";

#[derive(Debug, Clone, Copy)]
enum GetterType {
    Bare,
    Option,
    Vec,
}

/// Generates getters for `UnsafeSlice` fields contained in the struct. Only
/// bare, Option, and Vec fields are supported.
///
///
/// ```ignore
//  use crate::util::UnsafeSlice;
/// use twitch_getters::twitch_getters;
///
///  #[twitch_getters]
///  struct TwitchStruct {
///     // UnsafeSlice fields
///     nick: UnsafeSlice,
///     sub: Option<UnsafeSlice>,
///     badges: Vec<UnsafeSlice>,
///     // Any other fields
///     some_other_vec: Vec<i32>,
///     some_option: Option<String>
/// }
///
/// // Expands into this:
/// impl TwitchStruct {
///     #[inline]
///     pub fn nick(&self) -> &str {
///         self.nick.as_str()
///     }
///     #[inline]
///     pub fn sub(&self) -> Option<&str> {
///         self.sub.as_ref().map(|v| v.as_str())
///     }
///     #[inline]
///     pub fn badges(&self) -> impl Iterator<Item = &str> + '_ {
///         self.badges.iter().map(|v| v.as_str())
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn twitch_getters(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item: syn::Item = syn::parse(input).expect("This macro can only be used with structs.");

    let (name, fields) = match &item {
        syn::Item::Struct(i) => (i.ident.clone(), collect_unsafe_slice_fields(i, UNSAFE_SLICE_TYPE_NAME)),
        _ => {
            item.span()
                .unstable()
                .error("Can only generate twitch getters for structs, and this is not a struct.");
            return item.into_token_stream().into();
        }
    };

    let mut getters = Vec::with_capacity(fields.len());
    for (name, getter_kind) in fields {
        let name = syn::Ident::new(&name[..], proc_macro2::Span::call_site());
        match getter_kind {
            GetterType::Bare => getters.push(quote! {
                #[inline]
                pub fn #name(&self) -> &str {
                    self.#name.as_str()
                }
            }),
            GetterType::Option => getters.push(quote! {
                #[inline]
                pub fn #name(&self) -> Option<&str> {
                    self.#name.as_ref().map(|v| v.as_str())
                }
            }),
            GetterType::Vec => getters.push(quote! {
                #[inline]
                pub fn #name(&self) -> impl Iterator<Item=&str> + '_ {
                    self.#name.iter().map(|v| v.as_str())
                }
            }),
        }
    }

    let output = quote! {
        #item

        impl #name {
            #(#getters)*
        }
    };
    output.into()
}

fn collect_unsafe_slice_fields(i: &ItemStruct, type_name: &str) -> Vec<(String, GetterType)> {
    let mut getters = vec![];

    match &i.fields {
        syn::Fields::Named(fields) => {
            fields
                .named
                .iter()
                .filter(|field| field.ident.is_some())
                .for_each(|field| {
                    match field.ty {
                        // The guard is for skipping self-qualified types like <Vec<T>>::Iter
                        syn::Type::Path(ref path) if path.qself.is_none() => {
                            if let Some(ty) = determine_getter_type(&path.path, type_name) {
                                getters.push((field.ident.as_ref().unwrap().to_string(), ty));
                            }
                        }
                        _ => {}
                    }
                })
        }
        _ => (),
    }

    getters
}

fn determine_getter_type(path: &syn::Path, type_name: &str) -> Option<GetterType> {
    // Get the last segment of the path, e.g. std::option::Option
    let ty = path.segments.iter().last()?;

    let name = ty.ident.to_string();
    let args = &ty.arguments;
    // If the first type is the type we're looking for, check if it doesn't have any
    // generics
    if name == type_name && args.is_empty() {
        Some(GetterType::Bare)
    // If the first type is Vec or Option, check that it has only one generic
    // argument, and that argument is our type
    } else if name == "Vec" || name == "Option" {
        match args {
            // Vec/Option doesn't have an argument.
            syn::PathArguments::None => None,
            // This can only happen in function types; we do not are about them
            syn::PathArguments::Parenthesized(_) => None,
            syn::PathArguments::AngleBracketed(generics) => {
                let generics = generics.args.iter().collect::<Vec<_>>();
                match &generics[..] {
                    &[syn::GenericArgument::Type(syn::Type::Path(ty))]
                        if ty
                            .path
                            .segments
                            .iter()
                            .next()
                            .map(|i| i.ident.to_string())
                            .as_ref()
                            .map(|s| &s[..])
                            == Some(type_name) =>
                    {
                        match &name[..] {
                            "Vec" => Some(GetterType::Vec),
                            "Option" => Some(GetterType::Option),
                            _ => unreachable!(),
                        }
                    }
                    _ => None,
                }
            }
        }
    } else {
        None
    }
}
