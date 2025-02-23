#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Span {
    pub m_line: usize,
    pub m_start: usize,
    pub m_end: usize
}

impl Span{
    pub fn new(m_line: usize, m_start: usize, m_end: usize) -> Self{
        Span{m_line, m_start, m_end}
    }
}