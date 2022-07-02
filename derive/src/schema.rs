use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, GenericArgument, Ident, Lit, Meta,
    MetaNameValue, NestedMeta, PathArguments, Type,
};

#[derive(Debug, Default)]
struct TypeAttribs {
    package_remote: Vec<String>,
    docs: String,
}

#[derive(Debug, Default)]
struct FieldAttribs {
    ignore: bool,
    package: bool,
    package_traverse: Vec<Ident>,
    docs: String,
}

pub fn derive_intermediate(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let attribs = parse_type_attribs(&ast.attrs);
    let package_remote = attribs
        .package_remote
        .iter()
        .map(|content| {
            let ty = syn::parse_str::<Type>(content).unwrap();
            quote! {
                #ty::schema(package);
            }
        })
        .collect::<Vec<_>>();
    let description = &attribs.docs;
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
                    let description = &attribs.docs;
                    let name = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    let id = if attribs.package {
                        quote! {
                            #ty::schema(package)
                        }
                    } else {
                        quote! {
                            SchemaIdContainer::new::<#ty>(package.prefer_tree_id)
                        }
                    };
                    let mut package_traverse = vec![];
                    traverse_type(ty, &attribs.package_traverse, &mut package_traverse);
                    Some(quote! {
                        #( #package_traverse )*
                        content = content.field(
                            stringify!(#name),
                            SchemaTypeInstance::new(#id).description(#description),
                        );
                    })
                }).collect::<Vec<_>>();
                quote! {
                    impl #impl_generics serde_intermediate::SchemaIntermediate for #name #ty_generics #where_clause {
                        fn schema(package: &mut serde_intermediate::SchemaPackage) -> serde_intermediate::SchemaIdContainer {
                            use serde_intermediate::schema::*;
                            let id = SchemaIdContainer::new::<Self>(package.prefer_tree_id);
                            let mut content = SchemaTypeStruct::default();
                            #( #package_remote )*
                            #( #fields )*
                            package.with(
                                id.to_owned(),
                                Schema::new(SchemaType::new_struct(content)).description(#description),
                            );
                            id
                        }
                    }
                }.into()
            }
            Fields::Unnamed(fields) => {
                let fields = fields.unnamed.iter().filter_map(|field| {
                    let attribs = parse_field_attribs(&field.attrs);
                    if attribs.ignore {
                        return None;
                    }
                    let description = &attribs.docs;
                    let ty = &field.ty;
                    let id = if attribs.package {
                        quote! {
                            #ty::schema(package)
                        }
                    } else {
                        quote! {
                            SchemaIdContainer::new::<#ty>(package.prefer_tree_id)
                        }
                    };
                    Some(quote! {
                        content = content.item(SchemaTypeInstance::new(#id).description(#description));
                    })
                }).collect::<Vec<_>>();
                quote! {
                    impl #impl_generics serde_intermediate::SchemaIntermediate for #name #ty_generics #where_clause {
                        fn schema(package: &mut serde_intermediate::SchemaPackage) -> serde_intermediate::SchemaIdContainer {
                            use serde_intermediate::schema::*;
                            let id = SchemaIdContainer::new::<Self>(package.prefer_tree_id);
                            let mut content = SchemaTypeTuple::default();
                            #( #package_remote )*
                            #( #fields )*
                            package.with(
                                id.to_owned(),
                                Schema::new(SchemaType::new_tuple_struct(content)).description(#description),
                            );
                            id
                        }
                    }
                }.into()
            }
            Fields::Unit => quote! {
                impl #impl_generics serde_intermediate::SchemaIntermediate for #name #ty_generics #where_clause {
                    fn schema(package: &mut serde_intermediate::SchemaPackage) -> serde_intermediate::SchemaIdContainer {
                        use serde_intermediate::schema::*;
                        let id = SchemaIdContainer::new::<Self>(package.prefer_tree_id);
                        let mut content = SchemaTypeStruct::default();
                        #( #package_remote )*
                        package.with(
                            id.to_owned(),
                            Schema::new(SchemaType::new_struct(content)).description(#description),
                        );
                        id
                    }
                }
            }
            .into(),
        },
        Data::Enum(data) => {
            let variants = data.variants.iter().filter_map(|variant| {
                let attribs = parse_field_attribs(&variant.attrs);
                if attribs.ignore {
                    return None;
                }
                let name = &variant.ident;
                match &variant.fields {
                    Fields::Named(fields) => {
                        let fields = fields.named.iter().filter_map(|field| {
                            let attribs = parse_field_attribs(&field.attrs);
                            if attribs.ignore {
                                return None;
                            }
                            let description = &attribs.docs;
                            let name = field.ident.as_ref().unwrap();
                            let ty = &field.ty;
                            let id = if attribs.package {
                                quote! {
                                    #ty::schema(package)
                                }
                            } else {
                                quote! {
                                    SchemaIdContainer::new::<#ty>(package.prefer_tree_id)
                                }
                            };
                            Some(quote! {
                                content = content.field(
                                    stringify!(#name),
                                    SchemaTypeInstance::new(#id).description(#description),
                                );
                            })
                        }).collect::<Vec<_>>();
                        Some(quote! {
                            let mut variant_content = {
                                let mut content = SchemaTypeStruct::default();
                                #( #fields )*
                                content
                            };
                            content = content.variant(stringify!(#name), SchemaTypeEnumVariant::Struct(variant_content));
                        })
                    }
                    Fields::Unnamed(fields) => {
                        let fields = fields.unnamed.iter().filter_map(|field| {
                            let attribs = parse_field_attribs(&field.attrs);
                            if attribs.ignore {
                                return None;
                            }
                            let description = &attribs.docs;
                            let ty = &field.ty;
                            let id = if attribs.package {
                                quote! {
                                    #ty::schema(package)
                                }
                            } else {
                                quote! {
                                    SchemaIdContainer::new::<#ty>(package.prefer_tree_id)
                                }
                            };
                            Some(quote! {
                                content = content.item(SchemaTypeInstance::new(#id).description(#description));
                            })
                        }).collect::<Vec<_>>();
                        Some(quote! {
                            let mut variant_content = {
                                let mut content = SchemaTypeTuple::default();
                                #( #fields )*
                                content
                            };
                            content = content.variant(stringify!(#name), SchemaTypeEnumVariant::Tuple(variant_content));
                        })
                    }
                    Fields::Unit => Some(quote! {
                        content = content.variant(stringify!(#name), SchemaTypeEnumVariant::Empty);
                    }),
                }
            }).collect::<Vec<_>>();
            quote! {
                impl #impl_generics serde_intermediate::SchemaIntermediate for #name #ty_generics #where_clause {
                    fn schema(package: &mut serde_intermediate::SchemaPackage) -> serde_intermediate::SchemaIdContainer {
                        use serde_intermediate::schema::*;
                        let id = SchemaIdContainer::new::<Self>(package.prefer_tree_id);
                        let mut content = SchemaTypeEnum::default();
                        #( #package_remote )*
                        #( #variants )*
                        package.with(
                            id.to_owned(),
                            Schema::new(SchemaType::new_enum(content)).description(#description),
                        );
                        id
                    }
                }
            }.into()
        }
        _ => panic!("SchemaIntermediate can be derived only for structs and enums"),
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
            Ok(Meta::NameValue(MetaNameValue { path, lit, .. })) => {
                if path.is_ident("doc") {
                    if let Lit::Str(lit) = lit {
                        if !result.docs.is_empty() {
                            result.docs.push('\n');
                        }
                        result.docs.push_str(lit.value().trim());
                    }
                }
            }
            Ok(Meta::List(meta)) => {
                if meta.path.is_ident("schema_intermediate") {
                    for meta in meta.nested {
                        if let NestedMeta::Meta(Meta::List(meta)) = &meta {
                            if meta.path.is_ident("package_remote") {
                                for meta in &meta.nested {
                                    if let NestedMeta::Lit(Lit::Str(lit)) = meta {
                                        result.package_remote.push(lit.value());
                                    }
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
            Ok(Meta::NameValue(MetaNameValue { path, lit, .. })) => {
                if path.is_ident("doc") {
                    if let Lit::Str(lit) = lit {
                        if !result.docs.is_empty() {
                            result.docs.push('\n');
                        }
                        result.docs.push_str(lit.value().trim());
                    }
                }
            }
            Ok(Meta::List(meta)) => {
                if meta.path.is_ident("schema_intermediate") {
                    for meta in meta.nested {
                        if let NestedMeta::Meta(meta) = &meta {
                            match meta {
                                Meta::Path(path) => {
                                    if path.is_ident("ignore") {
                                        result.ignore = true;
                                    } else if path.is_ident("package") {
                                        result.package = true;
                                    }
                                }
                                Meta::List(meta) => {
                                    if meta.path.is_ident("package_traverse") {
                                        for meta in &meta.nested {
                                            if let NestedMeta::Meta(Meta::Path(path)) = meta {
                                                if let Some(ident) = path.get_ident() {
                                                    result.package_traverse.push(ident.to_owned());
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
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

fn traverse_type(ty: &Type, filters: &[Ident], result: &mut Vec<proc_macro2::TokenStream>) {
    match ty {
        Type::Array(array) => {
            traverse_type(&array.elem, filters, result);
        }
        Type::Group(group) => {
            traverse_type(&group.elem, filters, result);
        }
        Type::Paren(paren) => {
            traverse_type(&paren.elem, filters, result);
        }
        Type::Path(path) => {
            if let Some(segment) = path.path.segments.last() {
                if filters.iter().any(|filter| &segment.ident == filter) {
                    result.push(quote! {
                        #ty::schema(package);
                    });
                } else if let PathArguments::AngleBracketed(generics) = &segment.arguments {
                    for arg in &generics.args {
                        if let GenericArgument::Type(ty) = arg {
                            traverse_type(ty, filters, result);
                        }
                    }
                }
            }
        }
        Type::Ptr(ptr) => {
            traverse_type(&ptr.elem, filters, result);
        }
        Type::Reference(reference) => {
            traverse_type(&reference.elem, filters, result);
        }
        Type::Slice(slice) => {
            traverse_type(&slice.elem, filters, result);
        }
        Type::Tuple(tuple) => {
            for elem in &tuple.elems {
                traverse_type(elem, filters, result);
            }
        }
        _ => {}
    }
}
