#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueError { 
    TypeError, 
    KeyNotFoundError, 
    IndexOutOfBoundError, 
    IOError, 
    ParseError, 
} 
