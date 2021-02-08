use core::iter::Extend;
use proc_macro::TokenStream;
use proc_macro2::{Group, Punct};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Paren,
    Attribute, ItemFn, ItemStruct, LitStr, Result, Token,
};

const API_VERSION: &str = "v1.41";
const API_REFERENCE_URL: &str = "https://docs.docker.com/engine/api";

struct Args {
    category: Option<String>,
    endpoint: Option<String>,
}

impl Args {
    fn url(self) -> String {
        let mut url = String::new();
        if let Some(category) = self.category {
            if let Some(ep) = self.endpoint {
                url = format!(
                    "[API Reference]({}/{}/#{}/{})",
                    API_REFERENCE_URL, API_VERSION, category, ep
                );
            }
        }
        if url.is_empty() {
            url = format!("[API Reference]({}/{})", API_REFERENCE_URL, API_VERSION);
        }

        url
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<LitStr, Token![,]>::parse_terminated(input)?;
        let mut vars = vars.iter();

        Ok(Args {
            category: vars.next().map(|lit| lit.value()),
            endpoint: vars.next().map(|lit| lit.value()),
        })
    }
}

enum Item {
    Fn(ItemFn),
    Struct(ItemStruct),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        //println!("{:#?}", input);
        let inp = input.fork();

        while let (Ok(_), Ok(_)) = (inp.parse::<Punct>(), inp.parse::<Group>()) {} //skip attributes

        if inp.peek(Token![pub]) {
            inp.parse::<Token![pub]>()?;
        }

        if inp.peek(Paren) {
            inp.parse::<Group>()?; // (crate)
        }

        if inp.peek(Token![async]) {
            inp.parse::<Token![async]>()?;
        }

        if inp.peek(Token![fn]) {
            Ok(Item::Fn(input.parse()?))
        } else {
            Ok(Item::Struct(input.parse()?))
        }
    }
}

fn get_doc_attrs(attrs: Vec<Attribute>) -> (Vec<TokenStream>, Vec<Attribute>) {
    let mut docs = Vec::<TokenStream>::new();
    let mut new_attrs = Vec::new();

    for attr in attrs {
        if let Some(path) = attr.path.segments.first() {
            println!("{}", path.ident.to_string());
            if &path.ident.to_string() == "doc" {
                docs.push(quote! { #attr }.into());
            } else {
                new_attrs.push(attr);
            }
        }
    }

    (docs, new_attrs)
}

#[proc_macro_attribute]
pub fn api_doc(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let args = parse_macro_input!(attr as Args);
    let url = args.url();
    let item = parse_macro_input!(item as Item);
    match item {
        Item::Fn(mut item) => {
            let (docs, attrs) = get_doc_attrs(item.attrs);
            item.attrs = attrs;

            let mut out = TokenStream::new();
            out.extend(docs);
            out.extend::<Vec<TokenStream>>(vec![quote! {
                #[doc = "\n"]
                #[doc = #url]
                #item
            }
            .into()]);

            out
        }
        Item::Struct(mut item) => {
            let (docs, attrs) = get_doc_attrs(item.attrs);
            item.attrs = attrs;

            let mut out = TokenStream::new();
            out.extend(docs);
            out.extend::<Vec<TokenStream>>(vec![quote! {
                #[doc = "\n"]
                #[doc = #url]
                #item
            }
            .into()]);

            out
        }
    }
}
