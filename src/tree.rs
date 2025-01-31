use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
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
    Eol(Eol),
}

impl Node {
    pub fn position(&self) -> &LineSpan {
        match self {
            Node::Header(header) => header.position(),
            Node::Paragraph(paragraph) => paragraph.position(),
            Node::UnorderedList(unordered_list) => unordered_list.position(),
            Node::Text(text) => text.position(),
            Node::Italic(italic) => italic.position(),
            Node::Bold(bold) => bold.position(),
            Node::Whitespace(whitespace) => whitespace.position(),
            Node::Eol(eol) => eol.position(),
        }
    }
}

pub trait Positioned {
    fn position(&self) -> &LineSpan;
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct LineSpan {
    pub start: usize,
    pub end: usize,
}

macro_rules! impl_positioned {
    ($struct_name:ident) => {
        impl Positioned for $struct_name {
            fn position(&self) -> &LineSpan {
                &self.position
            }
        }
    };
}
impl_positioned!(Header);
impl_positioned!(Paragraph);
impl_positioned!(UnorderedList);
impl_positioned!(Text);
impl_positioned!(Italic);
impl_positioned!(Bold);
impl_positioned!(Whitespace);
impl_positioned!(Eol);

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Header {
    pub level: usize,
    pub nodes: Vec<Node>,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Paragraph {
    pub nodes: Vec<Node>,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct UnorderedList {
    pub level: usize, // 0 for root
    pub nodes: Vec<Node>,
    pub children: Vec<Node>,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Text {
    pub value: String,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Italic {
    pub nodes: Vec<Node>,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Bold {
    pub nodes: Vec<Node>,
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Whitespace {
    pub position: LineSpan,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Eol {
    pub position: LineSpan,
}
