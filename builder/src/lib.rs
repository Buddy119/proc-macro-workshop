extern crate syn;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;


#[proc_macro_derive(Builder)]
pub fn builder(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = if let syn::Data::Struct(syn::DataStruct{ fields: syn::Fields::Named(syn::FieldsNamed{ ref named,.. }), .. }) = input.data{
        named
    } else{
        unimplemented!()
    };
    fn ty_is_option(ty : &syn::Type) -> Option<&syn::Type> {
        if let syn::Type::Path(ref p) = ty {
            if p.path.segments.len() != 1 || p.path.segments[0].ident != "Option" {
                return None;
            }

            if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments{
                if inner_ty.args.len() != 1{
                    return None;
                }
                let inner_ty = inner_ty.args.first().unwrap();
                if let syn::GenericArgument::Type(ref t) = inner_ty {
                    return Some(t);
                }
            }
        }
        None
    }

    let optimized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if ty_is_option(ty).is_some(){
            quote!{#name: #ty}
        } else{
            quote!{#name: std::option::Option<#ty>}
        }

    });
    let methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if let Some(inner_ty) = ty_is_option(ty) {
            quote!{
                pub fn #name(&mut self, #name: #inner_ty) -> &mut Self {
                self.#name = Some(#name);
                self
                }
            }
        } else {
            quote!{
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
                }
            }
        }

    });
    let build_fields = fields.iter().map(|f|{
        let name = &f.ident;
        let ty = &f.ty;
        if ty_is_option(ty).is_some(){
            quote!{
                #name: self.#name.clone()
            }
        } else{
            quote!{
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))?
            }
        }

    });
    let build_none = fields.iter().map(|f|{
        let name = &f.ident;
        quote!{
            #name: None,
        }
    });
    let bname = format!("{}Builder", name);
    let bident = syn::Ident::new(&bname, name.span());
    let expanded = quote!{
        pub struct #bident {
            #(#optimized,)*
        }
        impl #bident {
            #(#methods)*

            pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(
                   #name {
                        #(#build_fields,)*
                    }
                )

            }
        }
        impl #name {
            pub fn builder() -> #bident {
                #bident{
                    #(#build_none)*
                }
            }
        }
    };
    println!("{:#?}", input);
    expanded.into()
}
