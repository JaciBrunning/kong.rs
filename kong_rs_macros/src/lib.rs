use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(PluginConfig)]
pub fn plugin_config_derive(item: TokenStream) -> TokenStream {
  let DeriveInput {
    attrs: _, vis: _, ident, generics: _, data
  } = parse_macro_input!(item as DeriveInput);

  match data {
    syn::Data::Struct(st) => {
      let (without_default, with_default): (Vec<_>, Vec<_>) = st.fields.into_iter().map(|field| {
        let ident = field.ident.expect("Struct fields must be named");
        let name = ident.to_string();
        let ty = field.ty;
        
        (
          quote! { std::collections::HashMap::from([(#name.to_owned(), <#ty as kong_rs::config::PluginConfigFieldVariant>::render(None, false))]) },
          quote! { std::collections::HashMap::from([(#name.to_owned(), <#ty as kong_rs::config::PluginConfigFieldVariant>::render(Some(default.#ident), false))]) }
        )
      }).unzip();

      quote! {
        impl kong_rs::config::PluginConfigFieldVariant for #ident {
          fn ty() -> &'static str { "record" }
          fn render(default: Option<Self>, skip_required: bool) -> kong_rs::config::RenderedConfigFieldVariant {
            let mut required = Some(Self::required());
            if skip_required {
              required = None;
            }

            kong_rs::config::RenderedConfigFieldVariant {
              ty: Self::ty().to_owned(),
              required,
              default: None,
              one_of: Self::variants(),
              elements: None,
              fields: Some(match default {
                Some(default) => vec![#(#with_default),*],
                None => vec![#(#without_default),*],
              })
            }
          }
        }

        impl kong_rs::config::PluginConfig for #ident { }
      }.into()
    },
    syn::Data::Enum(en) => {
      let variants = en.variants.into_iter().map(|variant| match variant.fields {
        syn::Fields::Unit => {
          variant.ident.to_string()
        },
        _ => panic!("Enums in configs can only be of the unit variant.")
      });
      
      quote! {
        impl kong_rs::config::PluginConfigFieldVariant for #ident {
          fn ty() -> &'static str { "string" }
          fn variants() -> Option<Vec<&'static str>> { Some(vec![#(#variants),*]) }
        }

        impl kong_rs::config::PluginConfig for #ident { }
      }.into()
    },
    syn::Data::Union(_) => panic!("An enum cannot be a plugin config"),
  }
}