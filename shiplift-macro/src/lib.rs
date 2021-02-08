use core::iter::Extend;
use proc_macro::TokenStream;
use proc_macro2::{Group, Punct};
use quote::{quote, ToTokens};
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
        } else if inp.peek(Token![struct]) {
            Ok(Item::Struct(input.parse()?))
        } else {
            Err(syn::Error::new(inp.span(), "unsupported token type"))
        }
    }
}

/// Extracts all `doc` attributes and returns them as first argument and
/// returning rest of attributes as second argument
fn get_doc_attrs(attrs: Vec<Attribute>) -> (Vec<TokenStream>, Vec<Attribute>) {
    let mut docs = Vec::<TokenStream>::new();
    let mut new_attrs = Vec::new();

    for attr in attrs {
        if let Some(path) = attr.path.segments.first() {
            if &path.ident.to_string() == "doc" {
                docs.push(quote! { #attr }.into());
            } else {
                new_attrs.push(attr);
            }
        }
    }

    (docs, new_attrs)
}

///  Produces a final TokenStream from extracted comments, url and item.
fn doc_token_stream<T: ToTokens>(
    url: &str,
    docs: Vec<TokenStream>,
    item: T,
) -> TokenStream {
    let mut tokens = TokenStream::new();
    tokens.extend(docs);
    tokens.extend::<Vec<TokenStream>>(vec![quote! {
        #[doc = "\n"]
        #[doc = #url]
        #item
    }
    .into()]);

    tokens
}

#[proc_macro_attribute]
/// Annotates a function or struct with a doc comment hyperlink placed
/// at the end of comments.
///
/// When both arguments are supplied f.e. `#[api_doc("tag", "Image")]` the url
/// points to this specific category and section.
///
/// If not arguments are supplied like so `#[api_doc]`, an absolute link to api reference is added
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
            item.attrs = attrs; // replace attrs so that extracted comments are not there
            doc_token_stream(&url, docs, item)
        }
        Item::Struct(mut item) => {
            let (docs, attrs) = get_doc_attrs(item.attrs);
            item.attrs = attrs;
            doc_token_stream(&url, docs, item)
        }
    }
}
