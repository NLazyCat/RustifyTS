//! Dominator tree calculation.
//!
//! Implements dominator tree computation for control flow graphs.

use super::{BasicBlockId, ControlFlowGraph};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

/// Represents a dominator tree for a control flow graph.
///
/// The dominator tree is a tree where each node's parent is its immediate
/// dominator. A node A dominates node B if every path from the entry block
/// to B must pass through A.
#[derive(Debug, Clone, PartialEq)]
pub struct DominatorTree {
    /// Map from each basic block to its immediate dominator
    idom: FxHashMap<BasicBlockId, BasicBlockId>,
    /// Map from each basic block to its children in the dominator tree
    children: FxHashMap<BasicBlockId, Vec<BasicBlockId>>,
    /// Depth of each node in the dominator tree (entry block has depth 0)
    depth: FxHashMap<BasicBlockId, usize>,
}

impl DominatorTree {
    /// Compute the dominator tree for the given control flow graph.
    ///
    /// Uses the iterative algorithm by Cooper, Harvey, and Kennedy:
    /// "A Simple, Fast Dominance Algorithm"
    pub fn compute(cfg: &ControlFlowGraph) -> Self {
        let mut postorder = postorder_traversal(cfg);
        let entry = cfg.entry;

        // Remove entry from postorder (we process it separately)
        if let Some(pos) = postorder.iter().position(|&id| id == entry) {
            postorder.remove(pos);
        }

        let mut idom = FxHashMap::default();
        idom.insert(entry, entry);

        let mut changed = true;
        while changed {
            changed = false;

            for &block in &postorder {
                // Find the first predecessor that has an idom
                let mut new_idom = None;
                for &pred in &cfg.get_block(block).unwrap().predecessors {
                    if idom.contains_key(&pred) {
                        new_idom = Some(pred);
                        break;
                    }
                }

                let Some(mut new_idom) = new_idom else {
                    continue; // No processed predecessors, skip
                };

                // Intersect with all other processed predecessors
                for &pred in &cfg.get_block(block).unwrap().predecessors {
                    if pred == new_idom {
                        continue;
                    }
                    if idom.contains_key(&pred) {
                        new_idom = intersect(&idom, pred, new_idom);
                    }
                }

                // Update idom if changed
                if idom.get(&block) != Some(&new_idom) {
                    idom.insert(block, new_idom);
                    changed = true;
                }
            }
        }

        // Build children map
        let mut children: FxHashMap<BasicBlockId, Vec<BasicBlockId>> = FxHashMap::default();
        for (&node, &parent) in &idom {
            if node != parent { // Don't add entry as child of itself
                children.entry(parent).or_default().push(node);
            }
        }

        // Compute depths
        let mut depth = FxHashMap::default();
        let mut queue = VecDeque::new();
        depth.insert(entry, 0);
        queue.push_back(entry);

        while let Some(node) = queue.pop_front() {
            let node_depth = depth[&node];
            if let Some(children) = children.get(&node) {
                for &child in children {
                    depth.insert(child, node_depth + 1);
                    queue.push_back(child);
                }
            }
        }

        Self {
            idom,
            children,
            depth,
        }
    }

    /// Get the immediate dominator of a block.
    pub fn idom(&self, block: BasicBlockId) -> Option<BasicBlockId> {
        self.idom.get(&block).copied()
    }

    /// Get the children of a block in the dominator tree.
    pub fn children(&self, block: BasicBlockId) -> &[BasicBlockId] {
        self.children.get(&block).map_or(&[], |v| v.as_slice())
    }

    /// Get the depth of a block in the dominator tree.
    pub fn depth(&self, block: BasicBlockId) -> Option<usize> {
        self.depth.get(&block).copied()
    }

    /// Check if block `a` dominates block `b`.
    pub fn dominates(&self, a: BasicBlockId, mut b: BasicBlockId) -> bool {
        if a == b {
            return true;
        }

        // Walk up the dominator tree from b to see if we reach a
        while let Some(&idom) = self.idom.get(&b) {
            if idom == a {
                return true;
            }
            if idom == b {
                break; // Reached entry node
            }
            b = idom;
        }

        false
    }

    /// Check if block `a` strictly dominates block `b`.
    pub fn strictly_dominates(&self, a: BasicBlockId, b: BasicBlockId) -> bool {
        a != b && self.dominates(a, b)
    }

    /// Find the nearest common dominator of two blocks.
    pub fn common_dominator(&self, mut a: BasicBlockId, mut b: BasicBlockId) -> Option<BasicBlockId> {
        // Make sure a is deeper in the tree
        while self.depth.get(&a)? > self.depth.get(&b)? {
            a = *self.idom.get(&a)?;
        }

        // Make sure b is deeper in the tree
        while self.depth.get(&b)? > self.depth.get(&a)? {
            b = *self.idom.get(&b)?;
        }

        // Now move both up until they meet
        while a != b {
            a = *self.idom.get(&a)?;
            b = *self.idom.get(&b)?;
        }

        Some(a)
    }
}

/// Perform a postorder traversal of the CFG.
fn postorder_traversal(cfg: &ControlFlowGraph) -> Vec<BasicBlockId> {
    let mut visited = FxHashMap::default();
    let mut stack = Vec::new();
    let mut result = Vec::new();

    stack.push((cfg.entry, false));

    while let Some((node, processed)) = stack.pop() {
        if processed {
            result.push(node);
            continue;
        }

        if visited.contains_key(&node) {
            continue;
        }

        visited.insert(node, true);
        stack.push((node, true));

        // Push successors in reverse order to maintain order
        if let Some(block) = cfg.get_block(node) {
            for &succ in block.successors.iter().rev() {
                if !visited.contains_key(&succ) {
                    stack.push((succ, false));
                }
            }
        }
    }

    result
}

/// Intersect two dominator sets.
///
/// Finds the common dominator of two nodes by walking up the dominator tree
/// until they meet.
fn intersect(
    idom: &FxHashMap<BasicBlockId, BasicBlockId>,
    mut b1: BasicBlockId,
    mut b2: BasicBlockId,
) -> BasicBlockId {
    while b1 != b2 {
        // This simplistic implementation assumes the idom map is complete
        // and we can just walk up until we find a common node
        b1 = idom[&b1];
        b2 = idom[&b2];
    }
    b1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::flow::ControlFlowGraph;

    #[test]
    fn test_dominator_tree_simple() {
        let mut cfg = ControlFlowGraph::new();
        let entry = cfg.entry;
        let exit = cfg.exit;

        // Create a simple diamond CFG:
        // entry -> bb1 -> bb2 -> exit
        //       -> bb3 ->
        let bb1 = cfg.create_block();
        let bb2 = cfg.create_block();
        let bb3 = cfg.create_block();

        cfg.add_edge(entry, bb1);
        cfg.add_edge(entry, bb3);
        cfg.add_edge(bb1, bb2);
        cfg.add_edge(bb3, bb2);
        cfg.add_edge(bb2, exit);

        let dt = DominatorTree::compute(&cfg);

        // Entry dominates everything
        assert!(dt.dominates(entry, entry));
        assert!(dt.dominates(entry, bb1));
        assert!(dt.dominates(entry, bb3));
        assert!(dt.dominates(entry, bb2));
        assert!(dt.dominates(entry, exit));

        // bb1 only dominates itself
        assert!(dt.dominates(bb1, bb1));
        assert!(!dt.dominates(bb1, bb3));
        assert!(!dt.dominates(bb1, bb2));

        // bb3 only dominates itself
        assert!(dt.dominates(bb3, bb3));
        assert!(!dt.dominates(bb3, bb1));
        assert!(!dt.dominates(bb3, bb2));

        // bb2 dominates exit
        assert!(dt.dominates(bb2, bb2));
        assert!(dt.dominates(bb2, exit));

        // Exit only dominates itself
        assert!(dt.dominates(exit, exit));
    }

    #[test]
    fn test_dominator_tree_loop() {
        let mut cfg = ControlFlowGraph::new();
        let entry = cfg.entry;
        let exit = cfg.exit;

        // Create a CFG with a loop:
        // entry -> header -> body -> latch -> header
        //                          -> exit
        let header = cfg.create_block();
        let body = cfg.create_block();
        let latch = cfg.create_block();

        cfg.add_edge(entry, header);
        cfg.add_edge(header, body);
        cfg.add_edge(header, exit);
        cfg.add_edge(body, latch);
        cfg.add_edge(latch, header);
        cfg.add_edge(latch, exit);

        let dt = DominatorTree::compute(&cfg);

        // Entry dominates everything
        assert!(dt.dominates(entry, header));
        assert!(dt.dominates(entry, body));
        assert!(dt.dominates(entry, latch));
        assert!(dt.dominates(entry, exit));

        // Header dominates body, latch, and exit
        assert!(dt.dominates(header, body));
        assert!(dt.dominates(header, latch));
        assert!(dt.dominates(header, exit));

        // Body dominates latch
        assert!(dt.dominates(body, latch));
        assert!(!dt.dominates(body, header));
        assert!(!dt.dominates(body, exit));

        // Latch doesn't dominate anything except itself
        assert!(dt.dominates(latch, latch));
        assert!(!dt.dominates(latch, header));
        assert!(!dt.dominates(latch, exit));
    }
}
