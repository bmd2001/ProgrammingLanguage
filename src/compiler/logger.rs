pub trait Logger {
    fn new(file_name: String, code: String) -> Self;
    fn report_error(&self, message: &str, span: (usize, (usize, usize)));
}