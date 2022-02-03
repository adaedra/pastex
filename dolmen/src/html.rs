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

macro_rules! tag_mods {
    ($tag:ident($name:expr), $($r:tt)*) => {
        pub mod $tag {
            #[doc = "Implementation for the `"]
            #[doc = stringify!($tag)]
            #[doc = "` tag"]
            pub struct T(());

            impl super::Tag for T {
                const NAME: &'static str = $name;
            }
        }

        tag_mods!($($r)*);
    };
    ($tag:ident, $($r:tt)*) => {
        tag_mods!($tag(stringify!($tag)),);
        tag_mods!($($r)*);
    };
    () => {}
}

macro_rules! tag_uses {
    ($tag:ident($name:expr), $($r:tt)*) => {
        pub use _t::$tag::T as $tag;

        tag_uses!($($r)*);
    };
    ($tag:ident, $($r:tt)*) => {
        tag_uses!($tag(()),);
        tag_uses!($($r)*);
    };
    () => {}
}

macro_rules! tags {
    ($($r:tt)*) => {
        mod _t {
            use super::Tag;

            tag_mods!($($r)*);
        }

        tag_uses!($($r)*);
    };
}

tags! {
    html,
    head, meta, title, link,
    body,
    a,
    p,
    div, span,
    pre,
    code,
    h1, h2, h3, h4, h5, h6,
    br,
    strong,
    nav, main, article, header, footer,
    script,
    svg, r#use("use"),
}
