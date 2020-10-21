use proc_macro::TokenStream;

mod symbol;

#[proc_macro]
pub fn symbols(input: TokenStream) -> TokenStream {
    symbol::symbols(input)
}
