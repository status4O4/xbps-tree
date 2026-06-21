#[derive(Debug, Clone)]
pub struct Dep {
    pub name: String,
    pub version: Option<String>,
}

impl Dep {
    pub fn new(name: impl Into<String>, version: Option<String>) -> Self {
        Dep {
            name: name.into(),
            version,
        }
    }
}
