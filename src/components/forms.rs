use super::states::Forms;
use std::{
    usize,
    rc::Rc,
    cell::RefCell,
};


pub struct Form {
    pub kind:           Forms,
    pub inputs:         Vec<Rc<RefCell<Vec<String>>>>,
    pub options:        Vec<String>,
    pub selected_input: usize,
    pub is_public:      bool,
}

impl Form {
    pub fn new(k: Option<Forms>, n_inputs: Option<usize>, options: Option<Vec<String>>) -> Self {
        let n = match n_inputs {
            Some(value) => value,
            None        => 1,
        };
        let inps = (0..n)
            .map(|_| Rc::new(RefCell::new(vec![String::from("")])))
            .collect();

        let opts: Vec<String> = match options {
            Some(vec) => vec,
            None      => vec!["".to_string()],
        };

        Self {
            kind:           match k {
                Some(value) => value,
                None        => Forms::Typing,
            },
            inputs:            inps,
            options:           opts,
            selected_input:    0,
            is_public:         true,
        }
    }

    pub fn switch_pub(&mut self) {
        self.is_public = !self.is_public;
    }
}
