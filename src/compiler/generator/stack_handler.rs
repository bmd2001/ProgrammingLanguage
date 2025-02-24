use std::collections::HashMap;

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