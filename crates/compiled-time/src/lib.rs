use proc_macro::{Literal, TokenStream, TokenTree};
use std::time::{SystemTime, UNIX_EPOCH};

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
