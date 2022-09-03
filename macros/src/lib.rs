// lint me harder
#![forbid(non_ascii_idents)]
#![deny(
    future_incompatible,
    keyword_idents,
    elided_lifetimes_in_paths,
    meta_variable_misuse,
    noop_method_call,
    pointer_structural_match,
    unused_lifetimes,
    unused_qualifications,
    clippy::wildcard_dependencies,
    clippy::debug_assert_with_mut_call,
    clippy::empty_line_after_outer_attr,
    clippy::panic,
    clippy::unwrap_used,
    clippy::redundant_field_names,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::unneeded_field_pattern,
    clippy::useless_let_if_seq
)]
#![warn(clippy::pedantic)]

mod name_it;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn name_it(attr: TokenStream, func: TokenStream) -> TokenStream {
    name_it::name_it( attr,  func)
}



mod expand;
mod parse;
use expand::expand;

use syn::{parse_macro_input};
use parse::Item;
use quote::quote;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn async_trait(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as Item);
    expand(item)
}
