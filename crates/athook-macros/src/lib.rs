#[proc_macro]
pub fn pattern_len(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pattern = syn::parse_macro_input!(input as syn::LitStr).value();
    let len = pattern.split_ascii_whitespace().count();
    quote::quote! { #len }.into()
}
