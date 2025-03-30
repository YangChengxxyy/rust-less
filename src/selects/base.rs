#[derive(Debug)]
pub struct Attr(pub String, pub String);

impl Clone for Attr {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

#[derive(Debug)]
pub struct Variable(pub String, pub String);

impl Clone for Variable {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}