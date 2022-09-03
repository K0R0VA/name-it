use quote::{format_ident, quote, ToTokens};
use std::collections::BTreeSet as Set;
use syn::{FnArg, ImplItem, PatType, Receiver, ReturnType, TraitItem};
use proc_macro::TokenStream;
use quote::__private::ext::RepToTokensExt;
use syn::__private::TokenStream2;
use syn::Pat::Type;
use crate::name_it::name_it;

use crate::parse::Item;

pub fn expand(mut input: Item) -> TokenStream {
    match &mut input {
        Item::Trait(input) => {
            let trait_name = &input.ident;
            let mut types = Set::new();
            let items = &mut input.items;
            for inner in items.iter_mut() {
                if let TraitItem::Method(method) = inner {
                    let sig = &mut method.sig;
                    if sig.asyncness.take().is_some() {
                        let method_name = sig.ident.clone();
                        types.insert(method_name.clone());
                        let output = match sig.output {
                            ReturnType::Type(arrow, ref t) => {
                                let output = quote!{ #arrow Self::#method_name };
                                let output = syn::parse(output.into()).unwrap();
                                output
                            },
                            ReturnType::Default => {
                                let output = quote! { -> Self::#method_name };
                                let output = syn::parse(output.into()).unwrap();
                                output
                            }
                        };
                        sig.output = output;
                    }
                }
            }
            let types = types.iter()
                .map(|ident| quote! (type #ident;).into())
                .filter_map(|t| syn::parse(t).ok());
            for t in types {
                input.items.push(t);
            }
            TokenStream::from(quote!(#input))
        }
        Item::Impl(input) => {
            let mut types = Vec::new();
            let ident = input.self_ty.clone();
            let mut funcs = Vec::new();
            for inner in &mut input.items {
                if let ImplItem::Method(method) = inner {
                    let mut fun = method.clone();
                    let sig = &mut method.sig;
                    if sig.asyncness.take().is_some() {
                        let method_name = sig.ident.clone();
                        fun.sig.inputs.iter_mut().for_each(|i| match i {
                            FnArg::Receiver(Receiver {mutability, reference, ..}) => {
                                let mutability = mutability.clone().map(|m| quote!(#m)).unwrap_or(quote!());
                                let reference = reference.clone().map(|(a, l)| {
                                    let l  = l.map(|l| quote!(#l)).unwrap_or(quote!());
                                    quote!(#a #l)
                                }).unwrap_or(quote!());
                                let this = quote!(this: #reference #mutability #ident);
                                *i = syn::parse(this.clone().into()).expect(&this.to_string());
                            },
                            _ => {}
                        });
                        let fun_name = sig.ident.clone();
                        let fun_fut_name = format_ident!("{}_future", fun.sig.ident);
                        types.push(quote!(type #fun_name = #fun_fut_name;));
                        let future: TokenStream2 = name_it(quote!(#fun_fut_name).into(), fun.into_token_stream().into()).into();
                        funcs.push(future);
                        let block = &mut method.block;
                        let inputs: Vec<TokenStream2> = sig.inputs.iter().map(|i| match i {
                            FnArg::Receiver(r) => quote!(#r),
                            FnArg::Typed(PatType {pat, ..}) => quote!(#pat)
                        }).collect();
                        let inputs = quote!( #(#inputs,) * );
                        *block = syn::parse(quote!({ #fun_name( #inputs ) }).into()).unwrap();
                        sig.output = syn::parse(quote!(-> Self::#fun_name).into()).unwrap();
                    }
                }
            }
            for t in types {
                let t = syn::parse(t.into()).unwrap();
                input.items.push(t);
            }
            TokenStream::from(quote!(
                #(#funcs), *
                #input
            ))
        }
    }
}
