// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use {
    crate::{geometry::area::Area, qtinner::QTInner, traversal::Traversal},
    num::PrimInt,
    std::{collections::HashSet, iter::FusedIterator, ops::Deref},
    uuid::Uuid,
};

// db    db db    db d888888b d8888b. d888888b d888888b d88888b d8888b.
// 88    88 88    88   `88'   88  `8D   `88'   `~~88~~' 88'     88  `8D
// 88    88 88    88    88    88   88    88       88    88ooooo 88oobY'
// 88    88 88    88    88    88   88    88       88    88~~~~~ 88`8b
// 88b  d88 88b  d88   .88.   88  .8D   .88.      88    88.     88 `88.
// ~Y8888P' ~Y8888P' Y888888P Y8888D' Y888888P    YP    Y88888P 88   YD

#[derive(Clone, Debug)]
pub(crate) struct UuidIter<'a, U>
where
    U: PrimInt,
{
    uuid_stack: Vec<&'a Uuid>,
    qt_stack: Vec<&'a QTInner<U>>,
    visited: HashSet<Uuid>,
}

impl<'a, U> UuidIter<'a, U>
where
    U: PrimInt,
{
    pub(crate) fn new(qt: &'a QTInner<U>) -> UuidIter<'a, U> {
        UuidIter {
            uuid_stack: vec![],
            qt_stack: vec![qt],
            visited: HashSet::new(),
        }
    }

    // Descent is an optimization for queries. We don't want to traverse the entire tree searching
    // for uuids which (mostly) correspond to regions our @req doesn't intersect with.
    //
    // Instead, we can make a beeline for the lowest region which totally contains the @req (but no
    // lower). We then have to actually evaluate every uuid below that node.
    //
    // Along the way, if our query is meant to be of type Traversal::Overlapping, we collect the
    // uuids we meet along the way. They are guaranteed to intersect @req.
    pub(crate) fn query_optimization(&mut self, req: Area<U>, traversal_method: Traversal) {
        // This method expects to be called at a point in time when the UuidIter has just been
        // created but has not yet been called.
        assert!(self.qt_stack.len() == 1);
        assert!(self.uuid_stack.is_empty());
        assert!(self.visited.is_empty());

        self.descend_recurse_step(req, traversal_method);
    }

    fn descend_recurse_step(&mut self, req: Area<U>, traversal_method: Traversal) {
        assert!(self.qt_stack.len() == 1);
        // Peek into the stack. We have to peek rather than pop, because if we are about to go too
        // far down we'd rather stop and return the UuidIter as-is.
        if let Some(qt) = self.qt_stack.last() {
            // If the region doesn't contain our @req, we're already too far down. Return here.
            if !qt.region.contains(req) {
                return;
            }
            assert!(qt.region.contains(req));

            if let Some(sqs) = qt.subquadrants.as_ref() {
                for sq in sqs.iter() {
                    // If we find a subquadrant which totally contains the @req, we want to make
                    // that our new sole qt.
                    if sq.region.contains(req) {
                        if traversal_method == Traversal::Overlapping {
                            self.uuid_stack.extend(&qt.kept_uuids);
                        }

                        // TODO(ambuc): Could this be done with Vec::swap() or std::mem::replace()?
                        assert!(self.qt_stack.len() == 1);
                        self.qt_stack = vec![sq];

                        // Recurse on this step. It will naturally return, but we want to propogate
                        // that return rather than continue to search the other subquadrants.
                        return self.descend_recurse_step(req, traversal_method);
                    }
                }
            }
            // If there aren't any subquadrants, we're probably done.
            return;
        }
    }
}

impl<U> Iterator for UuidIter<'_, U>
where
    U: PrimInt,
{
    type Item = Uuid;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Check the uuid_stack.
        if let Some(uuid) = self.uuid_stack.pop() {
            if !self.visited.insert(uuid.clone()) {
                return self.next();
            }
            return Some(uuid.clone());
        }

        // Then check the qt_stack.
        if let Some(qt) = self.qt_stack.pop() {
            // Push my regions onto the region stack
            self.uuid_stack.extend(&qt.kept_uuids);

            // Push my subquadrants onto the qt_stack too.
            if let Some(sqs) = qt.subquadrants.as_ref() {
                self.qt_stack.extend(sqs.iter().map(|x| x.deref()));
            }
            return self.next();
        }

        // Else there's nothing left to search.
        return None;
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<U> FusedIterator for UuidIter<'_, U> where U: PrimInt {}
