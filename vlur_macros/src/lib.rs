use proc_macro::{Delimiter, Group, Literal, Punct, Spacing, TokenStream, TokenTree};
use std::time::{SystemTime, UNIX_EPOCH};

#[proc_macro]
pub fn unique_bytes(_: TokenStream) -> TokenStream {
    let bytes = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
        .to_le_bytes();

    let mut stream = Vec::new();
    for byte in bytes {
        stream.push(TokenTree::Literal(Literal::u8_suffixed(byte)));
        stream.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
    }
    let stream = TokenStream::from_iter(stream);

    TokenStream::from(TokenTree::Group(Group::new(Delimiter::Bracket, stream)))
}
