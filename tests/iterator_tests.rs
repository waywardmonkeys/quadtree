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

mod util; // For unordered_elements_are.

// For testing .iter(), .iter_mut(), .regions(), .values(), .values_mut().
mod iterator_tests {
    use crate::util::unordered_elements_are;
    use quadtree_impl::entry::{Entry, EntryRef};
    use quadtree_impl::Quadtree;

    fn mk_quadtree_for_iter_tests() -> Quadtree<i32, i8> {
        let mut q = Quadtree::<i32, i8>::new_with_anchor((-35, -35), 8);
        q.extend(vec![((0, -5), 10), ((-15, 20), -25), ((30, -35), 40)]);
        q
    }

    #[test]
    fn iter_all() {
        let q = mk_quadtree_for_iter_tests();

        debug_assert!(unordered_elements_are(
            q.iter().map(|e| e.inner()),
            vec![
                (&((-15, 20), (1, 1)), &-25),
                (&((0, -5), (1, 1)), &10),
                (&((30, -35), (1, 1)), &40)
            ]
        ));
    }

    // The same as iter_all(), except we mutate each value by +1.
    #[test]
    fn iter_mut_all() {
        let mut q = mk_quadtree_for_iter_tests();

        q.modify_all(|v| *v += 1);

        debug_assert!(unordered_elements_are(
            q.iter().map(|e| e.inner()),
            vec![
                (&((-15, 20), (1, 1)), &-24),
                (&((0, -5), (1, 1)), &11),
                (&((30, -35), (1, 1)), &41)
            ]
        ));
    }

    #[test]
    fn regions() {
        let q = mk_quadtree_for_iter_tests();
        debug_assert!(unordered_elements_are(
            q.regions(),
            vec![
                &((0, -5), (1, 1)),
                &((-15, 20), (1, 1)),
                &((30, -35), (1, 1))
            ],
        ));
    }

    #[test]
    fn values() {
        let q = mk_quadtree_for_iter_tests();

        debug_assert!(unordered_elements_are(q.values(), vec![&10, &-25, &40]));
    }

    #[test]
    fn into_iterator_consuming() {
        let q = mk_quadtree_for_iter_tests();
        // Entry holds by-value.
        let entries: Vec<Entry<i32, i8>> = q.into_iter().collect();
        let mut values: Vec<i8> = vec![];
        for mut e in entries {
            values.push(e.value());
        }

        debug_assert!(unordered_elements_are(values, vec![10, -25, 40],));
    }

    #[test]
    fn into_iterator_reference() {
        let mut q = mk_quadtree_for_iter_tests();
        let entries: Vec<EntryRef<i32, i8>> = (&q).into_iter().collect();
        let mut values: Vec<&i8> = vec![];
        for e in entries {
            values.push(e.value());
        }
        debug_assert!(unordered_elements_are(values, vec![&10, &-25, &40],));

        q.reset();
        debug_assert!(q.is_empty());
    }

    //// Reminder:
    //let mut q = Quadtree::<i32, i8>::new_with_anchor((-35, -35), 8);
    //q.extend(vec![((0, -5), 10), ((-15, 20), -25), ((30, -35), 40)]);
    #[test]
    fn delete_everything() {
        let mut q = mk_quadtree_for_iter_tests();
        debug_assert_eq!(q.len(), 3);
        q.delete((-35, -35), (80, 80));
        debug_assert_eq!(q.len(), 0);
    }

    #[test]
    fn delete_region() {
        let mut q = mk_quadtree_for_iter_tests();
        debug_assert_eq!(q.len(), 3);
        // Near miss.
        q.delete((29, -36), (1, 1));
        debug_assert_eq!(q.len(), 3);

        // Direct hit!
        let mut returned_entries = q.delete((30, -35), (1, 1));
        debug_assert_eq!(q.len(), 2);
        let hit = returned_entries.next().unwrap();
        debug_assert_eq!(hit.value_ref(), &40);
        debug_assert_eq!(hit.region(), ((30, -35), (1, 1)));
    }

    #[test]
    fn delete_region_two() {
        let mut q = mk_quadtree_for_iter_tests();
        debug_assert_eq!(q.len(), 3);

        // Just large enough to encompass the two points.
        let returned_entries = q.delete((-15, -5), (16, 26));
        debug_assert_eq!(q.len(), 1);
        debug_assert_eq!(returned_entries.len(), 2);

        debug_assert!(unordered_elements_are(
            returned_entries.map(|mut e| e.value()),
            vec![-25, 10,]
        ));
    }
}
