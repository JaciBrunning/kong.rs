use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct RenderedConfigFieldVariant {
  #[serde(rename = "type")]
  pub ty: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub required: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub default: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub one_of: Option<Vec<&'static str>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub elements: Option<Box<Self>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fields: Option<Vec<HashMap<String, Self>>>
}

pub trait PluginConfigFieldVariant : Sized + serde::Serialize {
  fn ty() -> &'static str;
  fn variants() -> Option<Vec<&'static str>> { None }
  fn required() -> bool { true }

  fn render_this(self) -> RenderedConfigFieldVariant {
    Self::render(Some(self), true)
  }

  fn render(default: Option<Self>, skip_required: bool) -> RenderedConfigFieldVariant {
    let mut required = Some(Self::required());
    let mut default = default.map(|x| serde_json::to_value(x).unwrap());

    if skip_required {
      required = None;
    }

    if required == Some(false) {
      default = None;
    }

    RenderedConfigFieldVariant {
      ty: Self::ty().to_owned(),
      required,
      default,
      one_of: Self::variants(),
      elements: None,
      fields: None
    }
  }
}

impl PluginConfigFieldVariant for String {
  fn ty() -> &'static str { "string" }
}

impl PluginConfigFieldVariant for bool {
  fn ty() -> &'static str { "boolean" }
}

impl PluginConfigFieldVariant for isize {
  fn ty() -> &'static str { "integer" }
}

impl<T: PluginConfigFieldVariant> PluginConfigFieldVariant for Option<T> {
  fn ty() -> &'static str { T::ty() }
  fn required() -> bool { false }
}

impl<T: PluginConfigFieldVariant> PluginConfigFieldVariant for Vec<T> {
  fn ty() -> &'static str { "array" }

  fn render(default: Option<Self>, _in_arr: bool) -> RenderedConfigFieldVariant {
      RenderedConfigFieldVariant {
        ty: Self::ty().to_owned(),
        required: Some(true),
        default: default.map(|x| serde_json::to_value(x).unwrap()),
        one_of: None,
        elements: Some(Box::new(T::render(None, true))),
        fields: None
      }
  }
}

pub trait PluginConfig : serde::de::DeserializeOwned + PluginConfigFieldVariant { }
