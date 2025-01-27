#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    // Block contents
    Header(Header),
    Paragraph(Paragraph),
    UnorderedList(UnorderedList),
    // Inline contents
    Text(Text),
    Italic(Italic),
    Bold(Bold),
    Whitespace(Whitespace),
}

pub trait Positioned {
    fn position(&self) -> &LineSpan;
}

#[derive(Debug, PartialEq, Eq)]
pub struct LineSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub level: usize,
    pub nodes: Vec<Node>,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Paragraph {
    pub nodes: Vec<Node>,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnorderedList {
    pub level: usize, // 0 for root
    pub nodes: Vec<Node>,
    pub children: Vec<Node>,
    pub position: LineSpan,
}

impl Positioned for UnorderedList {
    fn position(&self) -> &LineSpan {
        &self.position
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Text {
    pub value: String,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Italic {
    pub nodes: Vec<Node>,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Bold {
    pub nodes: Vec<Node>,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Whitespace {
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Eol {
    pub position: LineSpan,
}
