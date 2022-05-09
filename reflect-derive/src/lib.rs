extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Index};

#[proc_macro_derive(ReflectIntermediate)]
pub fn derive_reflect_intermediate(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    match ast.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let fields = fields.named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let key = name.to_string();
                    quote! {
                        #key => {
                            self.#name.patch_change(change);
                        }
                    }
                }).collect::<Vec<_>>();
                quote! {
                    impl #impl_generics serde_reflect_intermediate::ReflectIntermediate for #name #ty_generics #where_clause {
                        fn patch_change(&mut self, change: &Change) {
                            match change {
                                Change::Changed(v) => {
                                    if let Ok(v) = serde_intermediate::from_intermediate(v) {
                                        *self = v;
                                    }
                                }
                                Change::PartialStruct(v) => {
                                    for (name, change) in v {
                                        match name.as_str() {
                                            #( #fields )*
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }.into()
            }
            Fields::Unnamed(fields) => {
                let fields = fields.unnamed.iter().enumerate().map(|(index,_)| {
                    let tuple_index = Index::from(index);
                    quote! {
                        #index => {
                            self.#tuple_index.patch_change(change);
                        }
                    }
                }).collect::<Vec<_>>();
                quote! {
                    impl #impl_generics serde_reflect_intermediate::ReflectIntermediate for #name #ty_generics #where_clause {
                        fn patch_change(&mut self, change: &Change) {
                            match change {
                                Change::Changed(v) => {
                                    if let Ok(v) = serde_intermediate::from_intermediate(v) {
                                        *self = v;
                                    }
                                }
                                Change::PartialSeq(v) => {
                                    for (index, change) in v {
                                        match *index {
                                            #( #fields )*
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }.into()
            }
            Fields::Unit => quote! {
                impl #impl_generics serde_reflect_intermediate::ReflectIntermediate for #name #ty_generics #where_clause {}
            }
            .into(),
        },
        Data::Enum(data) => {
            // let variants = data
            //     .variants
            //     .iter()
            //     .map(|variant| {
            //         let name = variant.ident.to_string();
            //         match &variant.fields {
            //             Fields::Named(fields) => {
            //                 let fields = fields
            //                     .named
            //                     .iter()
            //                     .filter_map(|field| {
            //                         let attribs = parse_field_attribs(&field.attrs);
            //                         if attribs.ignore {
            //                             return None;
            //                         }
            //                         let name = field.ident.as_ref().unwrap().to_string();
            //                         let type_ = parse_type(&field.ty);
            //                         Some(IgniteNamedField {
            //                             name,
            //                             typename: type_,
            //                             mapping: attribs.mapping,
            //                             meta: attribs.meta,
            //                         })
            //                     })
            //                     .collect::<Vec<_>>();
            //                 IgniteVariant::Named(IgniteNamed { name, fields })
            //             }
            //             Fields::Unnamed(fields) => {
            //                 let fields = fields
            //                     .unnamed
            //                     .iter()
            //                     .filter_map(|field| {
            //                         let attribs = parse_field_attribs(&field.attrs);
            //                         if attribs.ignore {
            //                             return None;
            //                         }
            //                         let type_ = parse_type(&field.ty);
            //                         Some(IgniteUnnamedField {
            //                             typename: type_,
            //                             mapping: attribs.mapping,
            //                             meta: attribs.meta,
            //                         })
            //                     })
            //                     .collect::<Vec<_>>();
            //                 IgniteVariant::Unnamed(IgniteUnnamed { name, fields })
            //             }
            //             Fields::Unit => IgniteVariant::Unit(name),
            //         }
            //     })
            //     .collect::<Vec<_>>();
            // IgniteTypeVariant::Enum(IgniteEnum { name, variants })

            let new_type_variants = data.variants.iter().filter_map(|variant| {
                let name = &variant.ident;
                if let Fields::Unnamed(_) = &variant.fields {
                    if variant.fields.len() == 1 {
                        Some(quote! {
                            Self::#name(content) => {
                                content.patch_change(change);
                            }
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            let struct_variants = data.variants.iter().filter_map(|variant| {
                let name = &variant.ident;
                if let Fields::Named(fields) = &variant.fields {
                    let field_names = fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();
                    let fields = fields.named.iter().map(|field| {
                        let name = field.ident.as_ref().unwrap();
                        let key = name.to_string();
                        quote! {
                            #key => {
                                #name.patch_change(change);
                            }
                        }
                    }).collect::<Vec<_>>();
                    Some(quote! {
                        Self::#name { #( #field_names ),* } => {
                            for (name, change) in v {
                                match name.as_str() {
                                    #( #fields )*
                                    _ => {}
                                }
                            }
                        }
                    })
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            quote! {
                impl #impl_generics serde_reflect_intermediate::ReflectIntermediate for #name #ty_generics #where_clause {
                    fn patch_change(&mut self, change: &Change) {
                        match change {
                            Change::Changed(v) => {
                                if let Ok(v) = serde_intermediate::from_intermediate(v) {
                                    *self = v;
                                }
                            }
                            Change::PartialChange(change) => {
                                match self {
                                    #( #new_type_variants )*
                                    _ => {}
                                }
                            }
                            Change::PartialStruct(v) => {
                                match self {
                                    #( #struct_variants )*
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }.into()
        }
        _ => panic!("ReflectIntermediate can be derived only for structs and enums"),
    }
}
