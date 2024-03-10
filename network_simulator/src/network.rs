#![allow(dead_code)]

use std::{cell::RefCell, collections::HashMap, rc::Rc};

// Note that NodeId can only
#[derive(Debug, Clone)]
struct NodeId(usize);

#[derive(Default)]
pub struct NodeSet {
    inner: Rc<RefCell<Inner>>,
}

#[derive(Default)]
struct Inner {
    count: usize,
    sym_tabl: HashMap<String, NodeId>,
    node_ids: HashMap<String, NodeId>,
}

impl NodeSet {
    fn new() -> Self {
        Default::default()
    }

    fn n(&self) -> impl Fn(&str) -> NodeHandle {
        let inner = self.inner.clone();
        move |sym: &str| {
            let inner = inner.clone();
            let sym: String = sym.into();
            let mut state = inner.as_ref().borrow_mut();
            if state.sym_tabl.get(&sym).is_none() {
                let count = state.count;
                state.sym_tabl.insert(sym.clone(), NodeId(count));
                state.count = count + 1;
            }
            std::mem::drop(state);
            NodeHandle { sym, state: inner }
        }
    }
}

pub struct NodeHandle {
    sym: String,
    state: Rc<RefCell<Inner>>,
}

impl NodeHandle {
    fn get(self) -> Option<NodeId> {
        self.state
            .as_ref()
            .borrow()
            .node_ids
            .get(&self.sym)
            .cloned()
    }

    fn get_or_create(self) -> NodeId {
        let node_id = self
            .state
            .as_ref()
            .borrow()
            .sym_tabl
            .get(&self.sym)
            .cloned()
            .unwrap();
        self.state
            .as_ref()
            .borrow_mut()
            .node_ids
            .entry(self.sym.clone())
            .or_insert(node_id)
            .clone()
    }
}
