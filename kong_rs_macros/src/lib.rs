use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, GenericArgument, Ident, TypePath};

enum FieldTy {
  String,
  Boolean,
  Integer,
  Array(Box<Self>),
}

impl FieldTy {
  pub fn render(&self, name: Ident, default: Option<Expr>) -> proc_macro2::TokenStream {
    let name_str = name.to_string();
    let inner: proc_macro2::TokenStream = match self {
      FieldTy::String => match default {
        Some(default) => quote! {
          { "type": "string", "default": #default }
        }.into(),
        None => quote! {
          { "type": "string", "required": true }
        }.into(),
      },
      FieldTy::Boolean => match default {
        Some(default) => quote! {
          { "type": "boolean", "default": #default }
        }.into(),
        None => quote! {
          { "type": "boolean", "required": true }
        }.into(),
      },
      FieldTy::Integer => match default {
        Some(default) => quote! {
          { "type": "integer", "default": #default }
        }.into(),
        None => quote! {
          { "type": "integer", "required": true }
        }.into(),
      },
      FieldTy::Array(field_ty) => {
        let element_type = match field_ty.as_ref() {
          FieldTy::String => "string",
          FieldTy::Boolean => "boolean",
          FieldTy::Integer => "integer",
          FieldTy::Array(_) => panic!("Can't have nested arrays in configuration!"),
        };

        match default {
          Some(default) => quote! {
            { "type": "array", "elements": #element_type, "default": #default }
          }.into(),
          None => quote! {
            { "type": "array", "elements": #element_type, "required": true }
          }.into()
        }
      },
    };

    quote! { { #name_str : #inner } }.into()
  }
}

fn ty_to_fieldty(ty_path: &TypePath) -> FieldTy {
  let segments = ty_path.path.segments.iter();
  let last_segment = segments.clone().last().expect("Empty type path!");
  
  match last_segment.ident.to_string().as_str() {
    "String" => FieldTy::String,
    "boolean" => FieldTy::Boolean,
    "usize" | "isize" => FieldTy::Integer,
    "Vec" => FieldTy::Array(match &last_segment.arguments {
      syn::PathArguments::AngleBracketed(angle) => {
        match angle.args.last().unwrap() {
          GenericArgument::Type(ty) => {
            match ty {
              syn::Type::Path(type_path) => {
                Box::new(ty_to_fieldty(type_path))
              },
              _ => panic!("Unknown type")
            }
          },
          _ => panic!("Unknown final generic argument")
        }
      },
      _ => panic!("Unknown Vec Generic")
    }),
    _ => panic!("Unknown field type: {:?}", last_segment.ident.to_string())
  }
}

#[proc_macro_derive(PluginConfig, attributes(default))]
pub fn plugin_config_derive(item: TokenStream) -> TokenStream {
  let DeriveInput {
    attrs: _, vis: _, ident, generics: _, data
  } = parse_macro_input!(item as DeriveInput);

  let fields: proc_macro2::TokenStream = match data {
    syn::Data::Struct(data_struct) => {
      let fields = data_struct.fields.into_iter().map(|field| {
        let name = field.ident.unwrap();
        let ty = field.ty;

        let mut default: Option<Expr> = None;

        for attr in field.attrs {
          match attr.meta {
            syn::Meta::NameValue(meta_name_value) => {
              if meta_name_value.path.is_ident("default") {
                default = Some(meta_name_value.value);
              } else {
                panic!("Unrecognised option")
              }
            },
            _ => ()
          }
        }

        let mapped_ty = match ty {
          syn::Type::Path(type_path) => {
            ty_to_fieldty(&type_path)
          },
          _ => panic!("Unknown type")
        };

        mapped_ty.render(name, default)
      });

      quote! {
        [{
          "config": {
            "type": "record",
            "fields": [#(#fields),*]
          }
        }]
      }.into()
    },
    syn::Data::Enum(_) => panic!("Expected Struct"),
    syn::Data::Union(_) => panic!("Expected Struct"),
  };

  let out = quote! {
    impl kong_rs::plugin::PluginConfig for #ident {
      fn schema_fields() -> serde_json::Value {
        serde_json::to_value(serde_json::json!( #fields )).unwrap()
      }
    }
  };

  out.into()
}