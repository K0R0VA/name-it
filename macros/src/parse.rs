use proc_macro2::{Span, TokenStream};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Attribute, ItemImpl, ItemTrait, Token};
use quote::{ToTokens};

pub enum Item {
    Trait(ItemTrait),
    Impl(ItemImpl),
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Item::Trait(item) => item.to_tokens(tokens),
            Item::Impl(item) => item.to_tokens(tokens),
        }
    }
}

impl Parse for Item {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let mut lookahead = input.lookahead1();
        if lookahead.peek(Token![unsafe]) {
            let ahead = input.fork();
            ahead.parse::<Token![unsafe]>()?;
            lookahead = ahead.lookahead1();
        }
        if lookahead.peek(Token![pub]) || lookahead.peek(Token![trait]) {
            let mut item: ItemTrait = input.parse()?;
            item.attrs = attrs;
            Ok(Item::Trait(item))
        } else if lookahead.peek(Token![impl]) {
            let mut item: ItemImpl = input.parse()?;
            if item.trait_.is_none() {
                return Err(Error::new(Span::call_site(), "expected a trait impl"));
            }
            item.attrs = attrs;
            Ok(Item::Impl(item))
        } else {
            Err(lookahead.error())
        }
    }
}