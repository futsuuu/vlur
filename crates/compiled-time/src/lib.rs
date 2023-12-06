use proc_macro::{TokenStream, TokenTree, Literal};
use std::time::{UNIX_EPOCH, SystemTime};

#[proc_macro]
pub fn bytes(_: TokenStream) -> TokenStream {
    let bytes = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
        .to_le_bytes();

    let literal = Literal::byte_string(&bytes);
    TokenStream::from(TokenTree::Literal(literal))
}
