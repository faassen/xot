use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::error::Error;
use crate::xmlvalue::Text;
use crate::xotdata::{Node, Xot};

struct Magic {
    xot: Rc<RefCell<Xot>>,
    node: Node,
}

impl Magic {
    fn new(xot: Rc<RefCell<Xot>>, node: Node) -> Self {
        Magic { xot, node }
    }

    // fn as_mut(&self) -> MagicMut<'a> {
    //     MagicMut::new(xot, self.node)
    // }

    fn document_element(&self) -> Result<Magic, Error> {
        Ok(Magic::new(
            self.xot.clone(),
            self.xot.borrow().document_element(self.node)?,
        ))
    }

    fn top_element(&self) -> Magic {
        Magic::new(self.xot.clone(), self.xot.borrow().top_element(self.node))
    }

    fn is_removed(&self) -> bool {
        self.xot.borrow().is_removed(self.node)
    }

    fn parent(&self) -> Option<Magic> {
        self.xot
            .borrow()
            .parent(self.node)
            .map(|node| Magic::new(self.xot.clone(), node))
    }

    fn first_child(&self) -> Option<Magic> {
        self.xot
            .borrow()
            .first_child(self.node)
            .map(|node| Magic::new(self.xot.clone(), node))
    }

    fn ancestors(&self) -> impl Iterator<Item = Magic> + '_ {
        self.xot
            .borrow()
            .ancestors(self.node)
            .map(move |node| Magic::new(self.xot.clone(), node))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn children(&self) -> impl Iterator<Item = Magic> + '_ {
        self.xot
            .borrow()
            .children(self.node)
            .map(move |node| Magic::new(self.xot.clone(), node))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn text_mut(&self) -> Option<TextWrap> {
        let mut xot = self.xot.borrow_mut();
        xot.text_mut(self.node).map(|_t| TextWrap {
            xot: self.xot.clone(),
            node: self.node,
        })
    }

    fn to_string(&self) -> Result<String, Error> {
        self.xot.borrow().to_string(self.node)
    }
}

struct TextWrap {
    xot: Rc<RefCell<Xot>>,
    node: Node,
}

impl TextWrap {
    fn new(xot: Rc<RefCell<Xot>>, node: Node) -> Self {
        TextWrap { xot, node }
    }

    fn set<S: Into<String>>(&self, text: S) {
        let mut xot = self.xot.borrow_mut();
        let txt = xot.text_mut(self.node).unwrap();
        txt.set(text);
    }

    fn get(&self) -> String {
        let xot = self.xot.borrow();
        xot.text(self.node).unwrap().get().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic() {
        let mut xot = Xot::new();
        let root = xot.parse("<p>Example</p>").unwrap();
        let magic = Magic::new(Rc::new(RefCell::new(xot)), root);
        let doc_el = magic.document_element().unwrap();
        let txt = doc_el.first_child().unwrap();
        // doesn't need to be text_mut, due to interior mutability...
        let txt_value = txt.text_mut().unwrap();
        txt_value.set("Hello, world!");

        assert_eq!(magic.to_string().unwrap(), "<p>Hello, world!</p>");
    }
}
