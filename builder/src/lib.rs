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
    let optimized = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote!{#name: std::option::Option<#ty>}
    });
    let methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote!{
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
               self.#name = Some(#name);
               self
            }
        }
    });

    let build_fields = fields.iter().map(|f|{
        let name = &f.ident;
        let ty = &f.ty;
        quote!{
            #name: self.#name.clone().ok_or(concat!(stringify!(#name), " is not set"))
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

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(
                   #name {
                        #(#build_fields?,)*
                    }
                )

            }
        }
        impl #name {
            pub fn builder() -> #bident {
                #bident{
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
    };
    println!("{:#?}", input);
    expanded.into()
}
