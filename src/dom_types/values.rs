// ------------- CSSValue -------------

/// CSS property value.
///
/// # Example
///
/// ```rust,no_run
///style! {
///    "padding" => px(12),
///    "background-color" => if disabled { CSSValue::Ignored } else { "green".into() },
///    "display" => CSSValue::Some("block".to_string()),
///}
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSSValue {
    /// The whole CSS property is ignored (i.e. not rendered).
    Ignored,
    /// Rendered CSS property value.
    Some(String),
}

impl<T: ToString> From<T> for CSSValue {
    fn from(value: T) -> Self {
        CSSValue::Some(value.to_string())
    }
}

// `&` because `style!` macro automatically adds prefix `&` before values for more ergonomic API
// (otherwise it would fail when you use for example a Model's property in View functions as `CSSValue`)
impl From<&CSSValue> for CSSValue {
    fn from(value: &CSSValue) -> Self {
        value.clone()
    }
}

// ----------- ToCssValue impls ------------

// impl ToCssValue for CSSValue
#[doc(hidden)]
pub trait ToCssValue {
    fn to_css_value(self) -> CSSValue;
}

impl ToCssValue for CSSValue {
    fn to_css_value(self) -> CSSValue {
        self
    }
}

// impl<T: ToString> ToCssValue for T
#[doc(hidden)]
pub trait ToCssValueForToString {
    fn to_css_value(&self) -> CSSValue;
}

impl<T: ToString> ToCssValueForToString for T {
    fn to_css_value(&self) -> CSSValue {
        CSSValue::Some(self.to_string())
    }
}

// impl<T: ToString> ToCssValue for Option<T>
#[doc(hidden)]
pub trait ToCssValueForOptionToString {
    fn to_css_value(&self) -> CSSValue;
}

impl<T: ToString> ToCssValueForOptionToString for Option<T> {
    fn to_css_value(&self) -> CSSValue {
        self.as_ref()
            .map_or(CSSValue::Ignored, |t| CSSValue::Some(t.to_string()))
    }
}

// TODO: Should we impl ToCssValue for more types? such as Vec<T>, Box<T>.. etc

// ------------- AtValue -------------

/// Attribute value.
///
/// # Example
///
/// ```rust,no_run
///attrs! {
///    At::Disabled => false.as_at_value(),  // same as `=> AtValue::Ignored`
///    At::Value => model.message,
///    At::AutoFocus => AtValue::None,
///}
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtValue {
    /// The whole attribute is ignored (i.e. not rendered).
    Ignored,
    /// Attribute value is not used (i.e. rendered as empty string).
    None,
    /// Rendered attribute value.
    Some(String),
}

impl<T: ToString> From<T> for AtValue {
    fn from(value: T) -> Self {
        AtValue::Some(value.to_string())
    }
}

// `&` because `attrs!` macro automatically adds prefix `&` before values for more ergonomic API
// (otherwise it would fail when you use for example a Model's property in View functions as `AtValue`)
impl From<&AtValue> for AtValue {
    fn from(value: &AtValue) -> Self {
        value.clone()
    }
}

// -- AsAtValue --

pub trait AsAtValue {
    fn as_at_value(&self) -> AtValue;
}

impl AsAtValue for bool {
    fn as_at_value(&self) -> AtValue {
        match self {
            true => AtValue::None,
            false => AtValue::Ignored,
        }
    }
}
