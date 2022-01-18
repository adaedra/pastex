pub trait Field {
    fn is_set(&self) -> bool;
    fn from(s: &str) -> Self;
}

impl Field for Option<String> {
    fn is_set(&self) -> bool {
        self.is_some()
    }

    fn from(s: &str) -> Self {
        Some(s.to_owned())
    }
}

impl Field for bool {
    fn is_set(&self) -> bool {
        false
    }

    fn from(_: &str) -> Self {
        true
    }
}

impl Field for Vec<String> {
    fn is_set(&self) -> bool {
        !self.is_empty()
    }

    fn from(s: &str) -> Self {
        s.split(',').map(str::trim).map(str::to_owned).collect()
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub keywords: Vec<String>,
    pub draft: bool,
    pub r#abstract: Option<Vec<super::Block>>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            title: None,
            author: None,
            date: None,
            keywords: Vec::new(),
            draft: false,
            r#abstract: None,
        }
    }
}
