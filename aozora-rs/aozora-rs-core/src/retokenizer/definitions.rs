use std::{
    collections::{HashMap, hash_map::Entry},
    usize,
};

use crate::{scopenizer::Break, *};

#[derive(Debug, Clone)]
pub enum Retokenized<'s> {
    Text(&'s str),
    Odoriji(Odoriji),
    Kunten(&'s str),
    Okurigana(&'s str),
    Break(Break),
    Figure(Figure<'s>),
    DecoBegin(Deco<'s>),
    DecoEnd(Deco<'s>),
}

#[derive(Default, Debug)]
pub enum RetokenizeError {
    #[default]
    InvalidEndOfToken,
    InvalidEndOfScope,
}

impl std::fmt::Display for RetokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidEndOfScope => "スコープの終了地点が不正です。これは内部的なエラーです",
                Self::InvalidEndOfToken => "トークンの終了地点が不正です。これは内部的なエラーです",
            }
        )
    }
}

impl Retokenized<'_> {
    pub fn is_visible(&self) -> bool {
        match self {
            Self::Kunten(k) => !k.is_empty(),
            Self::Okurigana(o) => !o.is_empty(),
            Self::Break(_) => false,
            Self::DecoBegin(_) => false,
            Self::DecoEnd(_) => false,
            _ => true,
        }
    }
}

#[derive(Default)]
pub struct IndexedStacks<T>(HashMap<usize, Vec<T>>);

impl<T> IndexedStacks<T> {
    pub fn push(&mut self, index: usize, deco: T) {
        self.0.entry(index).or_default().push(deco)
    }

    pub fn pop(&mut self, index: usize) -> Option<T> {
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

    pub fn into_iter(self) -> impl Iterator<Item = (usize, Vec<T>)> {
        self.0.into_iter()
    }
}

pub type DecoQueue<'s> = IndexedStacks<Deco<'s>>;
