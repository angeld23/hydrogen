use const_fnv1a_hash::fnv1a_hash_str_64;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

fn common_component(input: &DeriveInput) -> proc_macro2::TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics,
        data: _,
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let component_id = fnv1a_hash_str_64(&ident.to_string());
    let display_name = ident.to_string();

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub const COMPONENT_ID: hydrogen::ecs::component::ComponentId = hydrogen::ecs::component::ComponentId(#component_id);
            pub const DISPLAY_NAME: &'static str = #display_name;

            pub fn query_one(
                ecs: &hydrogen::ecs::world::World,
                entity_id: hydrogen::ecs::entity::EntityId,
            ) -> Option<&Self> {
                hydrogen::ecs::query_one!(ecs, entity_id, #ident).map(|(c,)| c)
            }

            pub fn query_one_mut(
                ecs: &mut hydrogen::ecs::world::World,
                entity_id: hydrogen::ecs::entity::EntityId,
            ) -> Option<&mut Self> {
                hydrogen::ecs::query_one_mut!(ecs, entity_id, #ident).map(|(c,)| c)
            }

            pub fn query(ecs: &hydrogen::ecs::world::World) -> impl Iterator<Item = (hydrogen::ecs::entity::EntityId, &Self)> {
                hydrogen::ecs::query!(ecs, #ident).map(|(e, (c,))| (e, c))
            }

            pub fn query_mut(ecs: &mut hydrogen::ecs::world::World) -> impl Iterator<Item = (hydrogen::ecs::entity::EntityId, &mut Self)> {
                hydrogen::ecs::query_mut!(ecs, #ident).map(|(e, (c,))| (e, c))
            }
        }
    }
}

#[proc_macro_derive(Component)]
pub fn component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let common = common_component(&input);

    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics,
        data: _,
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let component_id = fnv1a_hash_str_64(&ident.to_string());
    let display_name = ident.to_string();

    quote! {
        #common

        impl #impl_generics hydrogen::ecs::component::Component for #ident #ty_generics #where_clause {
            fn component_id(&self) -> hydrogen::ecs::component::ComponentId {
                hydrogen::ecs::component::ComponentId(#component_id)
            }
            fn display_name(&self) -> &'static str {
                #display_name
            }
            fn any_ref(&self) -> &dyn std::any::Any {
                self
            }
            fn is_serializable(&self) -> bool {
                false
            }
            fn as_serializable(&self) -> Option<&dyn hydrogen::ecs::component::SerializableComponent> {
                None
            }
            fn as_serializable_mut(&mut self) -> Option<&mut dyn hydrogen::ecs::component::SerializableComponent> {
                None
            }
        }
    }.into()
}

#[proc_macro_derive(SerializableComponent)]
pub fn serializable_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let common = common_component(&input);

    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics,
        data: _,
    } = input;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let component_id = fnv1a_hash_str_64(&ident.to_string());
    let display_name = ident.to_string();

    quote! {
        #common

        impl #impl_generics hydrogen::ecs::component::Component for #ident #ty_generics #where_clause {
            fn component_id(&self) -> hydrogen::ecs::component::ComponentId {
                hydrogen::ecs::component::ComponentId(#component_id)
            }
            fn display_name(&self) -> &'static str {
                #display_name
            }
            fn any_ref(&self) -> &dyn std::any::Any {
                self
            }
            fn is_serializable(&self) -> bool {
                true
            }
            fn as_serializable(&self) -> Option<&dyn hydrogen::ecs::component::SerializableComponent> {
                Some(self)
            }
            fn as_serializable_mut(&mut self) -> Option<&mut dyn hydrogen::ecs::component::SerializableComponent> {
                Some(self)
            }
        }

        #[typetag::serde]
        impl #impl_generics hydrogen::ecs::component::SerializableComponent for #ident #ty_generics #where_clause {
            fn clone_box(&self) -> Box<dyn hydrogen::ecs::component::SerializableComponent> {
                Box::new(self.clone())
            }
        }
    }.into()
}
