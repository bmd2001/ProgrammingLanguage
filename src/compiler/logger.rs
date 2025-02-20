pub trait Logger {
    fn new(file_name: String, code: String) -> Self;
    fn log_error(&self, message: &str, span: (usize, (usize, usize)));
}