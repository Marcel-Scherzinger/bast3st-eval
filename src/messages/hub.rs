use std::collections::BTreeSet;

use crate::messages::message::Message;

#[derive(Debug, Clone)]
pub struct Messages<Level>(pub(super) BTreeSet<Message<Level>>);

impl<Level> IntoIterator for Messages<Level> {
    type Item = Message<Level>;
    type IntoIter = std::collections::btree_set::IntoIter<Message<Level>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<Level, A> Extend<Message<A>> for Messages<Level>
where
    Level: From<A>,
{
    fn extend<T: IntoIterator<Item = Message<A>>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(Message::change_level))
    }
}

impl<Level, I> FromIterator<Message<I>> for Messages<Level>
where
    Level: From<I>,
{
    fn from_iter<T: IntoIterator<Item = Message<I>>>(iter: T) -> Self {
        Self(iter.into_iter().map(Message::change_level).collect())
    }
}

impl<Level> Messages<Level> {
    pub fn drain(&mut self) -> std::collections::btree_set::IntoIter<Message<Level>> {
        std::mem::take(&mut self.0).into_iter()
    }
    pub fn cast_drain<O>(&mut self) -> impl Iterator<Item = Message<O>>
    where
        O: From<Level>,
    {
        self.drain().map(|x| x.change_level())
    }
}
