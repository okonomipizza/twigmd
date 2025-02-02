#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Header,             // #
    Text,               // text
    Whitespace,         // ' '
    Eol,                // \n (End of line)
    UnorderedList,      // -
    BlockQuote,         // >
    CodeBlock,          // ```
    InlineCode,         // `
    Annotation,         // ^
    Bold,               // **
    Italic,             // *
    CarlyBracketOpen,   // {
    CarlyBracketClose,  // }
    Colon,              // :
    SemiColon,          // ;
    SquareBracketOpen,  // [
    SquareBracketClose, // ]
    ParenthesisOpen,    // (
    ParenthesisClose,   // )
    HorizontalRule,     // ---
    AlertStart,         // :::<type>
    AlertEnd,           // :::
    Exclamation, // !
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String, // actutual value in the file
    pub line: usize,   // line number in the file
}
