use std::fmt;

pub struct ScmErr {
    reason: String,
}

impl ScmErr {
    pub fn from(reason: &str) -> Self {
        Self {
            reason: String::from(reason),
        }
    }
}

impl fmt::Display for ScmErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.reason)
    }
}

impl fmt::Debug for ScmErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.reason)
    }
}

impl std::error::Error for ScmErr {}
