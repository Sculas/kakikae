use quote::quote;

#[proc_macro]
pub fn pattern_len(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let pattern = syn::parse_macro_input!(input as syn::LitStr).value();
    let len = pattern.split_ascii_whitespace().count();
    quote! { #len }.into()
}

#[derive(derive_syn_parse::Parse)]
struct ExecutePatchParams {
    patch_pattern: syn::LitStr,
    _comma_token: syn::Token![,],
    patch_addr: syn::Ident,
}

#[proc_macro]
pub fn execute_patch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ExecutePatchParams {
        patch_pattern,
        patch_addr,
        ..
    } = syn::parse_macro_input!(input as ExecutePatchParams);

    let mut patch_index = 0isize;
    let patches = patch_pattern
        .value()
        .split_ascii_whitespace()
        .filter_map(|v| {
            patch_index += 1;
            if v == "?" || v == "??" {
                None
            } else {
                let idx = patch_index - 1;
                let byte = read_hex(v);
                Some(quote! { *#patch_addr.offset(#idx) = #byte; })
            }
        })
        .collect::<Vec<_>>();

    quote! { #(#patches)* }.into()
}

fn read_hex(v: &str) -> u8 {
    let mut buf = [0u8];
    hex::decode_to_slice(v, &mut buf).unwrap();
    buf[0]
}