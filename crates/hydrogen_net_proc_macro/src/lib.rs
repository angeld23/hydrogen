use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(NetMessage)]
pub fn net_message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics,
        data: _,
    } = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        #[typetag::serde]
        impl #impl_generics hydrogen::net::comm::NetMessage for #ident #ty_generics #where_clause {

        }
    }
    .into()
}
