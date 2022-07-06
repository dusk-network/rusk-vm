// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::borrow::BorrowMut;

use bytecheck::CheckBytes;
use microkelvin::{
    Annotation, ArchivedChild, ArchivedCompound, ArchivedLink, Child, ChildMut,
    Compound, Link, MutableLeaves, StoreProvider, StoreRef, StoreSerializer,
};
// use rend::LittleEndian;
use rkyv::{
    validation::validators::DefaultValidator, Archive, Deserialize, Serialize,
};

#[derive(Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[archive(bound(serialize = "
  T: Clone + Serialize<StoreSerializer<I>>,
  A: Clone + Clone + Annotation<T>,
  I: Clone,
  __S: Sized + BorrowMut<StoreSerializer<I>>"))]
#[archive(bound(deserialize = "
  T::Archived: Deserialize<T, StoreRef<I>>,
  ArchivedLink<Self, A, I>: Deserialize<Link<Self, A, I>, __D>,
  A: Clone + Annotation<T>,
  I: Clone,
  __D: StoreProvider<I>,"))]
pub enum LinkedList<T, A, I> {
    Empty,
    Node {
        val: T,
        #[omit_bounds]
        next: Link<Self, A, I>,
    },
}

impl<T, A, I> Default for LinkedList<T, A, I> {
    fn default() -> Self {
        LinkedList::Empty
    }
}

impl<T, A, I> ArchivedCompound<LinkedList<T, A, I>, A, I>
    for ArchivedLinkedList<T, A, I>
where
    T: Archive,
{
    fn child(&self, ofs: usize) -> ArchivedChild<LinkedList<T, A, I>, A, I> {
        match (ofs, self) {
            (0, ArchivedLinkedList::Node { val, .. }) => {
                ArchivedChild::Leaf(val)
            }
            (1, ArchivedLinkedList::Node { next, .. }) => {
                ArchivedChild::Link(next)
            }
            (
                _,
                ArchivedLinkedList::Node { .. } | ArchivedLinkedList::Empty,
            ) => ArchivedChild::End,
        }
    }
}

impl<T, A, I> Compound<A, I> for LinkedList<T, A, I>
where
    T: Archive,
{
    type Leaf = T;

    fn child(&self, ofs: usize) -> Child<Self, A, I> {
        match (ofs, self) {
            (0, LinkedList::Node { val, .. }) => Child::Leaf(val),
            (1, LinkedList::Node { next, .. }) => Child::Link(next),
            (_, LinkedList::Node { .. }) => Child::End,
            (_, LinkedList::Empty) => Child::End,
        }
    }

    fn child_mut(&mut self, ofs: usize) -> ChildMut<Self, A, I> {
        match (ofs, self) {
            (0, LinkedList::Node { val, .. }) => ChildMut::Leaf(val),
            (1, LinkedList::Node { next, .. }) => ChildMut::Link(next),
            (_, LinkedList::Node { .. }) => ChildMut::End,
            (_, LinkedList::Empty) => ChildMut::End,
        }
    }
}

impl<T, A, I> MutableLeaves for LinkedList<T, A, I> {}

impl<T, A, I> LinkedList<T, A, I>
where
    T: Archive,
    T::Archived: for<'any> CheckBytes<DefaultValidator<'any>>,
    A: Archive,
    A::Archived: for<'any> CheckBytes<DefaultValidator<'any>>,
    I: Clone + for<'any> CheckBytes<DefaultValidator<'any>>,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, t: T) {
        match core::mem::take(self) {
            LinkedList::Empty => {
                *self = LinkedList::Node {
                    val: t,
                    next: Link::new(LinkedList::Empty),
                }
            }
            old @ LinkedList::Node { .. } => {
                *self = LinkedList::Node {
                    val: t,
                    next: Link::new(old),
                };
            }
        }
    }

    pub fn pop(&mut self) -> Option<T>
    where
        T: Archive + Clone,
        T::Archived: Deserialize<T, StoreRef<I>>,
        A: Archive + Clone + Annotation<T>,
        A::Archived: Deserialize<A, StoreRef<I>>,
    {
        match core::mem::take(self) {
            LinkedList::Empty => None,
            LinkedList::Node { val: t, next } => {
                *self = next.unlink();
                Some(t)
            }
        }
    }
}
