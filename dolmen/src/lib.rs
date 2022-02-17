//! Rust rapid creation of in-memory HTML documents
//!
//! Dolmen allows client programs to create, store and generate HTML documents on the fly,
//! based on a macro to create HTML elements in a Rust-like way, and generate HTML into text
//! lazilly.
//!
//! See the [`tag`] macro for details.

use std::{fmt, marker::PhantomData};

pub mod html;

/// Any kind of HTML document element
pub trait Element: fmt::Display {}

/// Simple alias for any [`Element`] in a [`Box`], but used everywhere in this crate for generic
/// Element and Tag handling.
pub type ElementBox = Box<dyn Element>;

/// A in-memory HTML tag, that you can build with [`tag`] or manually with [`Tag::build`]
pub struct Tag<T: html::Tag> {
    attributes: Vec<(String, String)>,
    content: Vec<ElementBox>,
    _phantom: PhantomData<T>,
}

impl<T: html::Tag> Tag<T> {
    /// Manually builds a HTML tag
    pub fn build(attributes: Vec<(String, String)>, content: Vec<ElementBox>) -> Tag<T> {
        Tag {
            attributes,
            content,
            _phantom: Default::default(),
        }
    }
}

impl<T: html::Tag> fmt::Display for Tag<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}", T::NAME)?;

        for (name, value) in &self.attributes {
            write!(f, r#" {}="{}""#, name, value)?;
        }

        if self.content.is_empty() {
            return write!(f, " />");
        }

        write!(f, ">")?;

        for span in &self.content {
            span.fmt(f)?;
        }

        write!(f, "</{}>", T::NAME)
    }
}

struct Text(String);

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        html_escape::encode_text(&self.0).fmt(f)
    }
}

/// A fragment is an ensemble of elements
pub struct Fragment(Vec<ElementBox>);

impl Fragment {
    /// Creates an empty fragment
    pub fn empty() -> Fragment {
        Fragment(Vec::new())
    }

    /// Creates a fragment from a list of element
    pub fn from(inner: Vec<ElementBox>) -> Fragment {
        Fragment(inner)
    }
}

impl fmt::Display for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for element in &self.0 {
            element.fmt(f)?;
        }

        Ok(())
    }
}

pub struct RawHTML(String);

impl RawHTML {
    pub unsafe fn from(s: String) -> RawHTML {
        RawHTML(s)
    }
}

impl fmt::Display for RawHTML {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: html::Tag> Element for Tag<T> {}
impl Element for Text {}
impl Element for Fragment {}
impl Element for RawHTML {}

#[doc(hidden)]
#[macro_export]
macro_rules! attr {
    ($v:ident, $name:ident = $value:expr) => {
        $v.push((stringify!($name).to_owned(), $value.to_owned()));
    };
    ($v:ident, $name:ident = $value:expr , $($r:tt)*) => {
        $v.push((stringify!($name).to_owned(), $value.to_owned()));
        $crate::attr!($v, $($r)*);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! attrs {
    ($($r:tt)*) => {
        {
            let mut v = Vec::new();
            $crate::attr!(v, $($r)*);
            v
        }
    };
}

/// Creates a tag in memory
///
/// The `tag!` macro creates a new memory representation of a tag, using a Rustified syntax for
/// the tag.
#[macro_export]
macro_rules! tag {
    (box $($r:tt)*) => {
        tag!($($r)*).into_element_box()
    };
    ($tag:ident) => {
        $crate::Tag::<$crate::html::$tag>::build(Default::default(), Default::default())
    };
    ($tag:ident { $($t:expr ;)* }) => {
        $crate::Tag::<$crate::html::$tag>::build(Default::default(), [$($crate::IntoElementBox::into_element_box($t)),*].into_iter().collect::<Vec<_>>())
    };
    ($tag:ident => $content:expr) => {
        $crate::Tag::<$crate::html::$tag>::build(Default::default(), $content)
    };
    ($tag:ident($($r:tt)*)) => {
        $crate::Tag::<$crate::html::$tag>::build($crate::attrs!($($r)*), Default::default())
    };
    ($tag:ident($($r:tt)*) { $($t:expr ;)* }) => {
        $crate::Tag::<$crate::html::$tag>::build($crate::attrs!($($r)*), [$($crate::IntoElementBox::into_element_box($t)),*].into_iter().collect::<Vec<_>>())
    };
    ($tag:ident($($r:tt)*) => $content:expr) => {
        $crate::Tag::<$crate::html::$tag>::build($crate::attrs!($($r)*), $content)
    };
}

/// Trait to convert into boxed generic element
pub trait IntoElementBox {
    fn into_element_box(self) -> ElementBox;
}

impl<T: 'static + html::Tag> IntoElementBox for Tag<T> {
    fn into_element_box(self) -> ElementBox {
        Box::new(self)
    }
}

impl<T: 'static + Element> IntoElementBox for Box<T> {
    fn into_element_box(self) -> ElementBox {
        self
    }
}

impl IntoElementBox for Box<dyn Element> {
    fn into_element_box(self) -> ElementBox {
        self
    }
}

impl IntoElementBox for Fragment {
    fn into_element_box(self) -> ElementBox {
        Box::new(self)
    }
}

impl<T: AsRef<str> + ?Sized> IntoElementBox for &T {
    fn into_element_box(self) -> ElementBox {
        Box::new(Text(self.as_ref().to_string()))
    }
}

impl<T: 'static + IntoElementBox> IntoElementBox for Option<T> {
    fn into_element_box(self) -> ElementBox {
        self.map(IntoElementBox::into_element_box)
            .unwrap_or_else(|| Box::new(Fragment::empty()))
    }
}

/// A complete HTML document
///
/// Represents a comblete, final HTML document ready to be generated. The main difference between
/// this and a `html` tag element is that this will also generate a `DOCTYPE`.
pub struct HtmlDocument(pub Tag<html::html>);

impl fmt::Display for HtmlDocument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "<!DOCTYPE html>")?;
        self.0.fmt(f)
    }
}
