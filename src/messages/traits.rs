use crate::{Messages, messages::Message};

impl<Level> PartialEq for Message<Level> {
    fn eq(&self, other: &Self) -> bool {
        self.severity == other.severity && self.text == other.text
    }
}
impl<Level> PartialEq for Messages<Level> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<Level> Eq for Message<Level> {}
impl<Level> Eq for Messages<Level> {}

impl<Level> PartialOrd for Message<Level> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<Level> PartialOrd for Messages<Level> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<Level> Ord for Message<Level> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.severity
            .cmp(&other.severity)
            .then(self.text.cmp(&other.text))
    }
}
impl<Level> Ord for Messages<Level> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<Level> Default for Messages<Level> {
    fn default() -> Self {
        Self(Default::default())
    }
}
