extern crate proc_macro;

use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Index, Lit, Meta, NestedMeta,
};

#[derive(Debug, Default)]
struct TypeAttribs {
    before_patch_change: Option<Ident>,
    after_patch_change: Option<Ident>,
}

#[derive(Debug, Default)]
struct FieldAttribs {
    pub ignore: bool,
}

#[proc_macro_derive(ReflectIntermediate, attributes(reflect_intermediate))]
pub fn derive_reflect_intermediate(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let attribs = parse_type_attribs(&ast.attrs);
    let before_patch_change = match attribs.before_patch_change {
        Some(name) => {
            quote! {
                fn before_patch_change(&mut self) {
                    self.#name();
                }
            }
        }
        None => Default::default(),
    };
    let after_patch_change = match attribs.after_patch_change {
        Some(name) => {
            quote! {
                fn after_patch_change(&mut self) {
                    self.#name();
                }
            }
        }
        None => Default::default(),
    };
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    match ast.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let fields = fields.named.iter().filter_map(|field| {
                    let attribs = parse_field_attribs(&field.attrs);
                    if attribs.ignore {
                        return None;
                    }
                    let name = field.ident.as_ref().unwrap();
                    let key = name.to_string();
                    Some(quote! {
                        #key => {
                            self.#name.patch_change(change);
                        }
                    })
                }).collect::<Vec<_>>();
                quote! {
                    impl #impl_generics serde_reflect_intermediate::ReflectIntermediate for #name #ty_generics #where_clause {
                        fn patch_change(&mut self, change: &Change) {
                            self.before_patch_change();
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
                            self.after_patch_change();
                        }

                        #before_patch_change

                        #after_patch_change
                    }
                }.into()
            }
            Fields::Unnamed(fields) => {
                let fields = fields.unnamed.iter().enumerate().filter_map(|(index,field)| {
                    let attribs = parse_field_attribs(&field.attrs);
                    if attribs.ignore {
                        return None;
                    }
                    let tuple_index = Index::from(index);
                    Some(quote! {
                        #index => {
                            self.#tuple_index.patch_change(change);
                        }
                    })
                }).collect::<Vec<_>>();
                quote! {
                    impl #impl_generics serde_reflect_intermediate::ReflectIntermediate for #name #ty_generics #where_clause {
                        fn patch_change(&mut self, change: &Change) {
                            self.before_patch_change();
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
                            self.after_patch_change();
                        }

                        #before_patch_change

                        #after_patch_change
                    }
                }.into()
            }
            Fields::Unit => quote! {
                impl #impl_generics serde_reflect_intermediate::ReflectIntermediate for #name #ty_generics #where_clause {}
            }
            .into(),
        },
        Data::Enum(data) => {
            let new_type_variants = data.variants.iter().filter_map(|variant| {
                let name = &variant.ident;
                if let Fields::Unnamed(_) = &variant.fields {
                    if variant.fields.len() == 1 {
                        let field = variant.fields.iter().next().unwrap();
                        let attribs = parse_field_attribs(&field.attrs);
                        if attribs.ignore {
                            return None;
                        }
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
                let attribs = parse_field_attribs(&variant.attrs);
                if attribs.ignore {
                    return None;
                }
                let name = &variant.ident;
                if let Fields::Named(fields) = &variant.fields {
                    let field_names = fields
                        .named
                        .iter()
                        .filter_map(|field| {
                            let attribs = parse_field_attribs(&field.attrs);
                            if attribs.ignore {
                                return None;
                            }
                            Some(field.ident.as_ref().unwrap())
                        })
                        .collect::<Vec<_>>();
                    let fields = fields.named.iter().filter_map(|field| {
                        let attribs = parse_field_attribs(&field.attrs);
                        if attribs.ignore {
                            return None;
                        }
                        let name = field.ident.as_ref().unwrap();
                        let key = name.to_string();
                        Some(quote! {
                            #key => {
                                #name.patch_change(change);
                            }
                        })
                    }).collect::<Vec<_>>();
                    Some(quote! {
                        Self::#name { #( #field_names , )* .. } => {
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
                        self.before_patch_change();
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
                        self.after_patch_change();
                    }

                    #before_patch_change

                    #after_patch_change
                }
            }.into()
        }
        _ => panic!("ReflectIntermediate can be derived only for structs and enums"),
    }
}

fn parse_type_attribs(attrs: &[Attribute]) -> TypeAttribs {
    let mut result = TypeAttribs::default();
    for attrib in attrs {
        match attrib.parse_meta() {
            Err(error) => panic!(
                "Could not parse attribute `{}`: {:?}",
                attrib.to_token_stream(),
                error
            ),
            Ok(Meta::List(meta)) => {
                if meta.path.is_ident("reflect_intermediate") {
                    for meta in meta.nested {
                        if let NestedMeta::Meta(Meta::NameValue(meta)) = &meta {
                            if meta.path.is_ident("before_patch_change") {
                                if let Lit::Str(value) = &meta.lit {
                                    result.before_patch_change =
                                        Some(Ident::new(&value.value(), Span::call_site().into()));
                                }
                            } else if meta.path.is_ident("after_patch_change") {
                                if let Lit::Str(value) = &meta.lit {
                                    result.after_patch_change =
                                        Some(Ident::new(&value.value(), Span::call_site().into()));
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    result
}

fn parse_field_attribs(attrs: &[Attribute]) -> FieldAttribs {
    let mut result = FieldAttribs::default();
    for attrib in attrs {
        match attrib.parse_meta() {
            Err(error) => panic!(
                "Could not parse attribute `{}`: {:?}",
                attrib.to_token_stream(),
                error
            ),
            Ok(Meta::List(meta)) => {
                if meta.path.is_ident("reflect_intermediate") {
                    for meta in meta.nested {
                        if let NestedMeta::Meta(Meta::Path(path)) = &meta {
                            if path.is_ident("ignore") {
                                result.ignore = true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    result
}
