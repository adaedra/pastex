use std::{fmt, marker::PhantomData};

mod processor;
mod tags;

pub type AnyTag = Box<dyn fmt::Display>;

struct Tag<T: tags::Tag> {
    attributes: Vec<(String, String)>,
    content: Vec<AnyTag>,
    _phantom: PhantomData<T>,
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

pub struct HtmlDocument(Tag<tags::html>);

impl fmt::Display for HtmlDocument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub use processor::output;
