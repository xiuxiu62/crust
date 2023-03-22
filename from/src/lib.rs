use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, parse_macro_input, Field};
// use syn::Type

#[proc_macro_attribute]
pub fn from(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let field = parse_macro_input!(input with Field::parse_named);

    let id = field.ident.unwrap();
    let ty = field.ty;

    TokenStream::from(quote! {
        impl From<#ty> for #id {
            fn from(value: char) -> Self {
                Self::#id
            }
        }
    })
}

struct Temp;

impl From<char> for Temp {
    fn from(value: char) -> Self {
        todo!()
    }
}

trait Tempers {
    fn temp();
}

#[cfg(test)]
mod test {
    #[test]
    fn works() {
        // let temp = H {};
    }
}
