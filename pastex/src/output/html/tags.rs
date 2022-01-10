use std::fmt;

pub trait Tag {
    const NAME: &'static str;
}

macro_rules! tags {
    ($($tag:ident,)*) => {
        mod _t {
            use super::Tag;

            $(
                pub mod $tag {
                    pub struct T;

                    impl super::Tag for T {
                        const NAME: &'static str = stringify!($tag);
                    }
                }
            )*
        }

        $(
            pub use _t::$tag::T as $tag;
        )*
    };
}

tags! {
    html,
    head,
    body,
    title,
    p,
    pre,
    code,
    h1,
    h2,
    h3,
    h4,
    h5,
    h6,
    br,
    strong,
}

pub struct Fragment(pub Vec<super::AnyTag>);

impl fmt::Display for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for tag in &self.0 {
            tag.fmt(f)?;
        }

        Ok(())
    }
}
