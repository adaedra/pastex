use std::{fmt, marker::PhantomData};

mod processor;
mod tags;

pub trait Element: fmt::Display {}
pub type ElementBox = Box<dyn Element>;

struct Tag<T: tags::Tag> {
    attributes: Vec<(String, String)>,
    content: Vec<ElementBox>,
    _phantom: PhantomData<T>,
}

impl<T: tags::Tag> Default for Tag<T> {
    fn default() -> Self {
        Tag {
            attributes: Vec::new(),
            content: Vec::new(),
            _phantom: Default::default(),
        }
    }
}

impl<T: tags::Tag> fmt::Display for Tag<T> {
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

impl<T: tags::Tag> Element for Tag<T> {}
impl Element for Text {}

pub struct HtmlDocument(Tag<tags::html>);

impl fmt::Display for HtmlDocument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<!DOCTYPE html>")?;
        self.0.fmt(f)
    }
}

pub use processor::output;
