use std::rc::Rc;
use model::Attribute;
use std::fmt::{Display, Formatter};
use std::result::Result as StdResult;
use std::fmt::Error as FmtError;
use std::iter;

#[derive(Default, Debug)]
pub struct Element {
    tag: Rc<String>,
    attrs: Vec<Attribute>,
    children: Vec<Element>,
    level: u32,
}

impl Element {
    pub fn new(tag: Rc<String>, attrs: Vec<Attribute>) -> Self {
        Element {
            tag: tag,
            attrs: attrs,
            children: Vec::new(),
            level: 0,
        }
    }

    pub fn append(&mut self, element: Element) {
        self.children.push(element)
    }

    pub fn set_level(&mut self, level: u32) {
        self.level = level;
    }

    pub fn get_attributes(&self) -> &Vec<Attribute> {
        &self.attrs
    }

    pub fn get_tag(&self) -> Rc<String> {
        self.tag.clone()
    }

    pub fn get_children(&self) -> &Vec<Element> {
        &self.children
    }
}

impl Display for Element {
    fn fmt(&self, formatter: &mut Formatter) -> StdResult<(), FmtError> {
        let tabs = iter::repeat("\t").take(self.level as usize).collect::<String>();
        write!(formatter, "{}Element: {}\n", tabs, self.tag)?;

        for c in &self.children {
            write!(formatter, "{}", c)?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct ElementContainer {
    stack: Vec<Element>,
    root: Option<Element>,
}

impl ElementContainer {
    pub fn start_element(&mut self, mut element: Element) {
        element.set_level(self.stack.len() as u32);
        self.stack.push(element);
    }

    pub fn end_element(&mut self) {
        self.stack
            .pop()
            .and_then(|element| {
                if self.stack.is_empty() {
                    self.root = Some(element);
                } else {
                    // Append child to current element
                    let last_element = self.stack.len();
                    self.stack[last_element - 1].append(element);
                }

                Some(())
            })
            .unwrap_or_else(|| {
                error!("Received an end element event with an empty stack");
            });
    }

    pub fn get_root(&self) -> &Option<Element> {
        &self.root
    }
}
