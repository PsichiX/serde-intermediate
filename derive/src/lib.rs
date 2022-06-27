extern crate proc_macro;

mod reflect;
mod schema;

use proc_macro::TokenStream;

#[proc_macro_derive(ReflectIntermediate, attributes(reflect_intermediate))]
pub fn derive_reflect_intermediate(input: TokenStream) -> TokenStream {
    crate::reflect::derive_intermediate(input)
}

#[proc_macro_derive(SchemaIntermediate, attributes(schema_intermediate))]
pub fn derive_schema_intermediate(input: TokenStream) -> TokenStream {
    crate::schema::derive_intermediate(input)
}
