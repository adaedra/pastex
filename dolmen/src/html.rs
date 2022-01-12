//! HTML standard implementation
//!
//! Represents the HTML standard and valid tags to build documents in a valid way using the
//! Rust typing system for checks.
//!
//! All HTML tags are represented as structures in this module, and are "magically" used by the
//! [`crate::tag`] macro. They're markers to be used as type parameters and are not directly
//! buildable.

/// HTML tag marker, used in [`crate::Tag`] to build valid HTML tags
pub trait Tag {
    const NAME: &'static str;
}

macro_rules! tags {
    ($($tag:ident,)*) => {
        mod _t {
            use super::Tag;

            $(
                pub mod $tag {
                    /// Automatic implementation
                    pub struct T(());

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
    meta,
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
