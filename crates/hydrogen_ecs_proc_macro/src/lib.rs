use const_fnv1a_hash::fnv1a_hash_str_64;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Component, attributes(local_component))]
pub fn component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics,
        data: _,
    } = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let component_id = fnv1a_hash_str_64(&ident.to_string());
    let display_name = ident.to_string();

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub const COMPONENT_ID: hydrogen_ecs::component::ComponentId = hydrogen_ecs::component::ComponentId(#component_id);
            pub const DISPLAY_NAME: &'static str = #display_name;
        }

        impl #impl_generics hydrogen_ecs::component::Component for #ident #ty_generics #where_clause {
            fn component_id(&self) -> hydrogen_ecs::component::ComponentId {
                hydrogen_ecs::component::ComponentId(#component_id)
            }
            fn display_name(&self) -> &'static str {
                #display_name
            }
        }
    }.into()
}

#[proc_macro_derive(SerializableComponent, attributes(local_component))]
pub fn serializable_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
        impl #impl_generics hydrogen_ecs::component::SerializableComponent for #ident #ty_generics #where_clause {}
    }.into()
}
