use std::collections::HashMap;

#[derive(Debug)]
#[derive(PartialEq)]
struct Variable{
    m_name: String,
    m_type: String,
    m_scope_depth: usize,
    m_stack_loc: usize
}

impl Variable{
    fn new(name: String, r#type: String, m_scope_depth: usize,  m_stack_loc: usize) -> Self{
        Variable{
            m_name: name,
            m_type: r#type,
            m_scope_depth,
            m_stack_loc,
        }
    }
}

pub struct StackHandler {
    m_variables: HashMap<String, Vec<Variable>>,
    m_stack_size: usize,
    m_scope_depth: usize
}

impl StackHandler{
    pub fn new() -> Self{
        StackHandler {
            m_variables: HashMap::new(),
            m_stack_size: 0,
            m_scope_depth: 0
        }
    }
    
    pub fn add_variable(&mut self, name: String, r#type: String){
        self.m_stack_size += 8;
        let variable = Variable::new(name.clone(), r#type, self.m_scope_depth, self.m_stack_size);
        self.m_variables.entry(name).or_insert(vec![]).push(variable);
    }
    
    pub fn get_offset(&mut self, name: String) -> usize{
        let variable = self.m_variables.get(&name).expect("No variable found");
        let variable_pos = variable.last().expect("The Stack Handler should have deleted this entry").m_stack_loc;
        self.m_stack_size.checked_sub(variable_pos).expect("Stack size logic not working")
    }
    
    pub fn increase_scope_depth(&mut self){
        self.m_scope_depth += 1;
    }
    
    pub fn decrease_scope_depth(&mut self){
        self.pop_scope_variables();
        self.m_scope_depth -= 1;
    }

    fn pop_scope_variables(&mut self){
        for (_, variable) in self.m_variables.iter_mut(){
            while variable.last().is_some_and(|var| var.m_scope_depth == self.m_scope_depth){
                variable.pop();
            }
        }
    }
}



#[cfg(test)]
mod test_stack_handler{
    use std::panic;
    use std::panic::AssertUnwindSafe;
    use super::*;

    #[test]
    fn test_add_variable(){
        let mut stack = StackHandler::new();
        stack.add_variable("Test".to_string(), "Test".to_string());
        assert_eq!(stack.m_stack_size, 8);
        assert_eq!(stack.m_scope_depth, 0);

        assert!(stack.m_variables.get(&"Test".to_string()).is_some());
        let test_variables = stack.m_variables.get(&"Test".to_string()).expect("There should have been a check that there was a variable in the map");
        assert_eq!(test_variables, &vec![Variable::new("Test".to_string(), "Test".to_string(), 0, 8)])
    }

    #[test]
    fn test_get_offset(){
        let mut stack = StackHandler::new();
        stack.add_variable("Test".to_string(), "Test".to_string());
        assert_eq!(stack.get_offset("Test".to_string()), 0);

        stack.add_variable("Test2".to_string(), "Test2".to_string());
        assert_eq!(stack.get_offset("Test2".to_string()), 0);
        assert_eq!(stack.get_offset("Test".to_string()), 8);
    }

    #[test]
    fn test_increase_scope_depth(){
        let mut stack = StackHandler::new();
        stack.increase_scope_depth();
        assert_eq!(stack.m_scope_depth, 1)
    }

    #[test]
    fn test_decrease_scope_depth(){
        let mut stack = StackHandler::new();
        stack.add_variable("Scope".to_string(), "Scope".to_string());
        stack.increase_scope_depth();
        stack.add_variable("Scope".to_string(), "Scope".to_string());
        stack.add_variable("Scope".to_string(), "Scope".to_string());
        stack.increase_scope_depth();

        stack.decrease_scope_depth();
        let vars = stack.m_variables.get(&"Scope".to_string()).expect("There should be stored variables");
        assert_eq!(vars.len(), 3);
        assert_eq!(stack.m_scope_depth, 1);
        stack.decrease_scope_depth();
        let vars = stack.m_variables.get(&"Scope".to_string()).expect("There should be stored variables");
        assert_eq!(vars.len(), 1);
        assert_eq!(stack.m_scope_depth, 0);
        // To prevent rust to print the panic message in the terminal, we replace the panic before calling decrease_scope_depth
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let failure = panic::catch_unwind(AssertUnwindSafe(|| stack.decrease_scope_depth()));
        panic::set_hook(prev_hook);
        assert!(failure.is_err())
    }
}