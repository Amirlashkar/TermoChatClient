use std::usize;

use super::states::Forms;


pub struct Form {
    pub kind:           Option<Forms>,
    pub inputs:         Vec<String>,
    pub selected_input: usize,
}

impl Form {
    pub fn new(k: Option<Forms>, n_inputs: Option<usize>) -> Self {
        Self {
            kind:           k,
            inputs:         vec![String::from("");
                                    match n_inputs {
                                        Some(value) => value,
                                        None        => 0,
                                    }
                                ],
            selected_input: 0,
        }
    }
}
