use crate::compiler::generator::subroutines::*;

pub struct Subroutines{
    subroutines: Vec<String>
}

impl Subroutines {
    pub fn new() -> Subroutines {
        Subroutines{subroutines: vec![
            get_print_subroutine(),
            get_int_to_str_subroutine()
        ]}
    }
    pub fn generate(&mut self) -> String{
        self.subroutines.join("\n\n")
    }
}