use std::{
    borrow::Cow,
    collections::{HashMap, hash_map::Entry},
    usize,
};

use crate::{scopenizer::Break, *};

#[derive(Debug, Clone)]
pub enum Retokenized<'s> {
    Text(Cow<'s, str>),
    Odoriji(Odoriji),
    Break(Break),
    Figure(Figure<'s>),
    DecoBegin(Deco<'s>),
    DecoEnd(Deco<'s>),
}

pub struct DecoQueue<'s>(HashMap<usize, Vec<Deco<'s>>>);

impl<'s> DecoQueue<'s> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn push(&mut self, index: usize, deco: Deco<'s>) {
        self.0.entry(index).or_insert_with(|| Vec::new()).push(deco)
    }

    pub fn pop(&mut self, index: usize) -> Option<Deco<'s>> {
        match self.0.entry(index) {
            Entry::Occupied(mut entry) => {
                let vec = entry.get_mut();
                let val = vec.pop();
                if vec.is_empty() {
                    entry.remove();
                }
                val
            }
            Entry::Vacant(_) => None,
        }
    }
}
