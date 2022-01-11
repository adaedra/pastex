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
