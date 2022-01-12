use std::{fmt, marker::PhantomData};

pub mod html;

pub trait Element: fmt::Display {}
pub type ElementBox = Box<dyn Element>;

pub struct Tag<T: html::Tag> {
    pub attributes: Vec<(String, String)>,
    pub content: Vec<ElementBox>,
    pub _phantom: PhantomData<T>,
}

impl<T: html::Tag> Default for Tag<T> {
    fn default() -> Self {
        Tag {
            attributes: Vec::new(),
            content: Vec::new(),
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
        self.0.fmt(f)
    }
}

struct Empty;

impl fmt::Display for Empty {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<T: html::Tag> Element for Tag<T> {}
impl Element for Text {}
impl Element for Empty {}

#[macro_export]
macro_rules! attr {
    ($v:ident, $name:ident = $value:expr) => {
        $v.push((stringify!($name).to_owned(), $value.to_owned()));
    };
    ($v:ident, $name:ident = $value:expr , $($r:tt)*) => {
        $v.push((stringify!($name).to_owned(), $value.to_owned()));
        attr!($v, $($r)*);
    }
}

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

#[macro_export]
macro_rules! tag {
    (box $($r:tt)*) => {
        tag!($($r)*).into_element_box()
    };
    ($tag:ident) => {
        $crate::Tag::<$crate::html::$tag>::default()
    };
    ($tag:ident { $($t:expr ;)* }) => {
        $crate::Tag::<$crate::html::$tag> {
            content: [$($crate::IntoElementBox::into_element_box($t)),*].into_iter().collect::<Vec<_>>(),
            .. Default::default()
        }
    };
    ($tag:ident => $content:expr) => {
        $crate::Tag::<$crate::html::$tag> {
            content: $content,
            .. Default::default()
        }
    };
    ($tag:ident($($r:tt)*)) => {
        $crate::Tag::<$crate::html::$tag> {
            attributes: $crate::attrs!($($r)*),
            .. Default::default()
        }
    };
    ($tag:ident($($r:tt)*) => $content:expr) => {
        $crate::Tag::<$crate::html::$tag> {
            content: $content,
            attributes: $crate::attrs!($($r)*),
            .. Default::default()
        }
    };
}

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

impl IntoElementBox for String {
    fn into_element_box(self) -> ElementBox {
        Box::new(Text(self))
    }
}

impl<T: 'static + IntoElementBox> IntoElementBox for Option<T> {
    fn into_element_box(self) -> ElementBox {
        self.map(IntoElementBox::into_element_box)
            .unwrap_or_else(|| Box::new(Empty))
    }
}

pub struct HtmlDocument(pub Tag<html::html>);

impl fmt::Display for HtmlDocument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "<!DOCTYPE html>")?;
        self.0.fmt(f)
    }
}
