use genawaiter::rc::gen;
use genawaiter::yield_;

use std::collections::VecDeque;

use crate::xotdata::{Node, Xot};

/// Node in level order
///
/// Used by [`Xot::level_order`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelOrder {
    /// A node
    Node(Node),
    /// The end of a sequence of nodes.
    End,
}

// traverse the tree in level order, meaning subsequent children are traversed first
// when a sequence of children comes to an end, a LevelOrder::End is yielded
// #[generator(yield(LevelOrder))]
pub(crate) fn level_order_traverse(xot: &Xot, node: Node) -> impl Iterator<Item = LevelOrder> + '_ {
    gen!({
        let mut queue = VecDeque::new();
        queue.push_back(node);
        // we make last node the current node; that's okay as it's
        // popped right away and thus has the same parent as the current node
        let mut last_node = node;
        while let Some(node) = queue.pop_front() {
            if xot.parent(last_node) != xot.parent(node) {
                yield_!(LevelOrder::End);
            }
            yield_!(LevelOrder::Node(node));

            last_node = node;
            for child in xot.children(node) {
                queue.push_back(child);
            }
        }
        yield_!(LevelOrder::End);
    })
    .into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_order_traverse() {
        let mut xot = Xot::new();
        let root = xot
            .parse("<doc><a><b/><b/></a><a><b><c/><c/><c/></b><b/></a></doc>")
            .unwrap();
        let doc = xot.document_element(root).unwrap();
        let a0 = xot.first_child(doc).unwrap();
        let a1 = xot.next_sibling(a0).unwrap();
        let b0 = xot.first_child(a0).unwrap();
        let b1 = xot.next_sibling(b0).unwrap();
        let b2 = xot.first_child(a1).unwrap();
        let b3 = xot.next_sibling(b2).unwrap();
        let c0 = xot.first_child(b2).unwrap();
        let c1 = xot.next_sibling(c0).unwrap();
        let c2 = xot.next_sibling(c1).unwrap();

        let mut iter = level_order_traverse(&xot, root).into_iter();

        let v = iter.collect::<Vec<_>>();
        assert_eq!(
            v,
            vec![
                LevelOrder::Node(root),
                LevelOrder::End,
                LevelOrder::Node(doc),
                LevelOrder::End,
                LevelOrder::Node(a0),
                LevelOrder::Node(a1),
                LevelOrder::End,
                LevelOrder::Node(b0),
                LevelOrder::Node(b1),
                LevelOrder::End,
                LevelOrder::Node(b2),
                LevelOrder::Node(b3),
                LevelOrder::End,
                LevelOrder::Node(c0),
                LevelOrder::Node(c1),
                LevelOrder::Node(c2),
                LevelOrder::End,
            ]
        );
    }

    #[test]
    fn test_level_order_traverse_with_text() {
        let mut xot = Xot::new();
        let root = xot.parse("<doc><a>X</a><a>X<b/>X</a></doc>").unwrap();
        let doc = xot.document_element(root).unwrap();
        let a0 = xot.first_child(doc).unwrap();
        let a1 = xot.next_sibling(a0).unwrap();
        let x0 = xot.first_child(a0).unwrap();
        let x1 = xot.first_child(a1).unwrap();
        let b0 = xot.next_sibling(x1).unwrap();
        let x2 = xot.next_sibling(b0).unwrap();

        let mut iter = level_order_traverse(&xot, root);

        let v = iter.collect::<Vec<_>>();
        assert_eq!(
            v,
            vec![
                LevelOrder::Node(root),
                LevelOrder::End,
                LevelOrder::Node(doc),
                LevelOrder::End,
                LevelOrder::Node(a0),
                LevelOrder::Node(a1),
                LevelOrder::End,
                LevelOrder::Node(x0),
                LevelOrder::End,
                LevelOrder::Node(x1),
                LevelOrder::Node(b0),
                LevelOrder::Node(x2),
                LevelOrder::End,
            ]
        );
    }
}
