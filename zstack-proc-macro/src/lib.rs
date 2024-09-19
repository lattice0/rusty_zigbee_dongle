use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput};
use quote::quote;

#[proc_macro_derive(DeriveWriter, attributes(my_attribute))]
pub fn derive_writer(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let struct_data = match input.data {
        Data::Struct(data) => data,
        _ => panic!("MyMacro só pode ser usado em structs"),
    };

    let fields = struct_data.fields;
    for field in fields.iter() {
        if let syn::Type::Path(type_path) = &field.ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "u8" {
                    println!("Found a u8 field: {:?}", field.ident);
                }
            }
        }
    }

    let expanded = quote! {
        impl IntoBytes for #name {
            type Output = Result<usize, IntoBytesError>;

            fn into_bytes(output: &mut [u8]) -> Self::Output {
                let writer = SliceWriter::new(output);
                
                todo!()
            }
        }
    };

    TokenStream::from(expanded)
}
