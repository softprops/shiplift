mod api_doc;

use proc_macro::TokenStream;

pub(crate) const API_VERSION: &str = "v1.41";
pub(crate) const API_REFERENCE_URL: &str = "https://docs.docker.com/engine/api";

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
    api_doc::_api_doc(attr, item)
}
