use const_fnv1a_hash::fnv1a_hash_str_64;
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
    let net_message_id = fnv1a_hash_str_64(&ident.to_string());
    let display_name = ident.to_string();

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub const NET_MESSAGE_ID: hydrogen::net::comm::NetMessageId = hydrogen::net::comm::NetMessageId(#net_message_id);
            pub const DISPLAY_NAME: &'static str = #display_name;
        }

        #[typetag::serde]
        impl #impl_generics hydrogen::net::comm::NetMessage for #ident #ty_generics #where_clause {
            fn net_id(&self) -> hydrogen::net::comm::NetMessageId {
                hydrogen::net::comm::NetMessageId(#net_message_id)
            }
            fn display_name(&self) -> &'static str {
                #display_name
            }
        }
    }
    .into()
}
