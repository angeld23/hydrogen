use quote::quote;
use syn::{parse_macro_input, DeriveInput, Type};

fn has_empty_attribute(field: &syn::Field, expected_ident: &str) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path().segments.len() == 1 && attr.path().segments[0].ident == expected_ident
    })
}

#[proc_macro_derive(DependencyProvider, attributes(dep, dep_mut))]
pub fn dependency(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics,
        data,
    } = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = data
    {
        named
    } else {
        unimplemented!()
    };

    let impls = fields.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;

        let (is_reference, inner_ty) = if let Type::Reference(ty_ref) = ty {
            let inner_ty = ty_ref.elem.as_ref();
            (true, quote! { #inner_ty })
        } else {
            (false, quote! { #ty })
        };

        let has_dep = has_empty_attribute(field, "dep");
        let has_dep_mut = has_empty_attribute(field, "dep_mut");

        let dep_fn_body = if is_reference {
            quote! { self.#name }
        } else {
            quote! { &self.#name }
        };
        let dep_mut_fn_body = if is_reference {
            quote! { self.#name }
        } else {
            quote! { &mut self.#name }
        };

        let dep_impl = quote! {
            impl #impl_generics hydrogen::core::dependency::Dependency<#inner_ty> for #ident #ty_generics #where_clause {
                fn dep(&self) -> &#inner_ty {
                    #dep_fn_body
                }
            }
        };

        let dep_mut_impl = quote! {
            impl #impl_generics hydrogen::core::dependency::DependencyMut<#inner_ty> for #ident #ty_generics #where_clause {
                fn dep_mut(&mut self) -> &mut #inner_ty {
                    #dep_mut_fn_body
                }
            }
        };

        match (has_dep, has_dep_mut) {
            (false, false) => quote! {},
            (true, false) => dep_impl,
            (true, true) | (false, true) => quote! {
                #dep_impl
                #dep_mut_impl
            },
        }
    });

    quote! {
        #(#impls)*
    }
    .into()
}
