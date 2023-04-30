use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DataEnum, DeriveInput, Field, FieldsNamed, Lit,
    LitStr, Type, TypePath,
};

#[proc_macro_derive(Parse)]
pub fn parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let variants = if let Data::Enum(DataEnum { variants, .. }) = &input.data {
        variants
    } else {
        panic!("Must be an enum");
    };
    let match_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let var_ident = &variant.ident;
            let cmd_name = Lit::Str(LitStr::new(&camel_case(&var_ident)[..], variant.span()));
            match &variant.fields {
                syn::Fields::Named(FieldsNamed { named, .. }) => {
                    let fields: Vec<_> = named
                        .iter()
                        .map(|field| {
                            let field_ident = &field.ident;
                            let valid_type = ValidType::from_field(&field)
                                .unwrap_or_else(|| panic!("unexpected type"));
                            let f = match valid_type {
                                ValidType::String => quote! { consume_name },
                                ValidType::Integer => quote! { consume_integer },
                                ValidType::Bytes => quote! { consume_bytes },
                            };
                            quote! {
                                #field_ident: parser.#f()?
                            }
                        })
                        .collect();
                    quote! {
                        #cmd_name => Self::#var_ident {
                            #(#fields),*
                        }
                    }
                }
                syn::Fields::Unit => {
                    quote! {
                        #cmd_name => Self::#var_ident
                    }
                }
                syn::Fields::Unnamed(_) => panic!("must have named fields"),
            }
        })
        .collect();

    let struct_ident = &input.ident;
    let expanded = quote! {
        impl TryFrom<Vec<crate::codec::Data>> for #struct_ident {
            type Error = anyhow::Error;

            fn try_from(data: Vec<crate::codec::Data>) -> anyhow::Result<Self> {
                let mut parser = crate::parser::Parser::new(data);
                let command_name = parser.consume_name()?;
                let cmd = match &command_name[..] {
                    #(#match_arms,)*
                    _ => anyhow::bail!("BAD_FORMAT"),
                };
                parser.finish()?;

                Ok(cmd)
            }
        }
    };

    TokenStream::from(expanded)
}

fn camel_case(ident: impl ToString) -> String {
    ident
        .to_string()
        .chars()
        .flat_map(|c| {
            if c.is_ascii_uppercase() {
                vec!['_', ((c as u8) + 32) as char]
            } else {
                vec![c]
            }
        })
        .skip(1)
        .collect()
}

enum ValidType {
    String,
    Integer,
    Bytes,
}

impl ValidType {
    fn from_field(field: &Field) -> Option<Self> {
        if let Type::Path(TypePath { path, .. }) = &field.ty {
            let ty_ident = &path.segments.last()?.ident;
            match &ty_ident.to_string()[..] {
                "u32" => Some(Self::Integer),
                "String" => Some(Self::String),
                "Bytes" => Some(Self::Bytes),
                _ => None,
            }
        } else {
            None
        }
    }
}
