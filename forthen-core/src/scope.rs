use std::collections::HashMap;

#[derive(Debug)]
pub struct CompilerScope {
    variables: HashMap<String, usize>,
}

impl CompilerScope {
    pub fn new() -> Self {
        CompilerScope {
            variables: HashMap::new(),
        }
    }

    pub fn get_storage_location(&mut self, var: &str) -> usize {
        let n = self.variables.len();
        *self.variables.entry(var.to_string()).or_insert(n)
    }

    pub fn len(&self) -> usize {
        self.variables.len()
    }
}
