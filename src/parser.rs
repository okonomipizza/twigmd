use crate::{
    lexer::lex,
    token::{Token, TokenType},
    tree::{
        Bold, Eol, Header, Italic, LineSpan, Node, Paragraph, Positioned, Text, UnorderedList,
        Whitespace,
    },
};

/// A structure for managing a stream of tokens.
///
/// `TokenStream` provides functionality for sequentially accessing,
/// modifying, and analyzing tokens in a token vector.
///
/// # Fields
/// - `tokens`: A mutable reference to a vector of tokens to be managed.
/// - `index`: The current position in the token stream.
///
/// This structure is commonly used in parsers to process a list of tokens
struct TokenStream<'a> {
    tokens: &'a mut Vec<Token>,
    index: usize,
}

impl<'a> TokenStream<'a> {
    /// Creates a new `TokenStream` instance.
    fn new(tokens: &'a mut Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    /// Returns the designated token.
    fn get(&self, ix: usize) -> Option<&Token> {
        self.tokens.get(ix)
    }

    /// Returns the current token.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    /// Returns the current token and advances the index to the next token in the stream.
    fn next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.index);
        self.index += 1;
        token
    }

    /// Moves the index back one token.
    fn back(&mut self) {
        self.index -= 1;
    }

    /// Replaces the current token with the given token.
    fn replace(&mut self, token: Token) {
        self.tokens[self.index] = token;
    }

    /// Determines if the next token is a list element and returns its nesting level.
    fn is_next_list(&self) -> Option<usize> {
        let mut nest = 0;
        let mut ix = self.index;

        while let Some(token) = self.get(ix) {
            if token.token_type == TokenType::Whitespace {
                nest += 1;
                ix += 1;
            } else if token.token_type == TokenType::UnorderedList {
                return Some(nest);
            } else {
                break;
            }
        }
        None
    }
}

/// Returns the position of the given node in the orginal document.
fn get_position(node: &Node) -> Option<&LineSpan> {
    match node {
        Node::UnorderedList(list) => Some(list.position()),
        _ => None,
    }
}

/// Parses a Markdown string and builds a tree structure representing its hierarchy.
///
/// This function is specifically designed to process Markdown-formatted strings.
/// It tokenizes the input, parses the tokens, and constructs a tree-like structure
/// where each `Node` corresponds to a Markdown element (e.g., headers, lists, paragraphs).
///
/// # Arguments
/// - `input`: A string slice containing the input data to be parsed.
///
/// # Returns
/// A `Vec<Node>` representing the parsed tree structure. Each `Node` in the vector
/// corresponds to an element of the tree.
///
/// # Workflow
/// 1. The input string is passed to the `lex` function to generate tokens.
/// 2. A `TokenStream` is created to manage the token sequence.
/// 3. The `parse` function processes the token stream and builds the tree-like structure.
///
/// # Notes
/// - This function is specifically intended for Markdown parsing and is not suitable
///   for other text formats.
/// - The structure of the returned tree depends on the implementation details of
///   the `parse` function and its handling of Markdown tokens.
pub fn build_tree(input: &str) -> Vec<Node> {
    let mut tokens = lex(input);
    let mut stream = TokenStream::new(&mut tokens);
    parse(&mut stream)
}

fn parse(stream: &mut TokenStream) -> Vec<Node> {
    let mut nodes: Vec<Node> = vec![];
    while let Some(token) = stream.peek() {
        match token.token_type {
            TokenType::Header => {
                let node = parse_header(stream);
                nodes.push(node);
            }
            TokenType::UnorderedList => {
                let node = parse_unordered_list(stream, 0); // root level
                nodes.push(node);
            }
            TokenType::Text | TokenType::Whitespace | TokenType::Italic | TokenType::Bold => {
                let node = parse_paragraph(stream);
                nodes.push(node);
            }
            TokenType::Eol => {
                let node = Node::Eol(Eol {
                    position: LineSpan {
                        start: token.line,
                        end: token.line,
                    },
                });
                nodes.push(node);
                stream.next();
            }
            _ => {
                let node = parse_paragraph(stream);
                nodes.push(node);
            }
        }
    }
    nodes
}

fn parse_unordered_list(stream: &mut TokenStream, cur_nest: usize) -> Node {
    let mut nodes: Vec<Node> = vec![];
    let mut children: Vec<Node> = vec![];
    let mut start: usize = 0;
    let mut end: usize = 0;

    while let Some(token) = stream.peek() {
        match token.token_type {
            TokenType::UnorderedList => {
                // If the next line contains a list element without nesting, terminate parsing the list here.
                if !nodes.is_empty() {
                    break;
                }
                // Parsing starts here.
                start = token.line;
                end = token.line;
                stream.next();
            }
            TokenType::Whitespace => {
                if let Some(nest) = {
                    let list_check = &stream.is_next_list();
                    *list_check
                } {
                    if nest > cur_nest {
                        for _ in 0..nest {
                            stream.next();
                        }
                        let child = parse_unordered_list(stream, nest);
                        if let Some(position) = get_position(&child) {
                            end = position.end
                        }
                        children.push(child);
                    } else {
                        break;
                    }
                } else {
                    end = token.line;
                    nodes.push(Node::Whitespace(Whitespace {
                        position: LineSpan {
                            start: token.line,
                            end: token.line,
                        },
                    }));
                    stream.next();
                }
            }

            // Check if the next line contains a nested UnorderedList elemet
            TokenType::Eol => {
                stream.next(); // Move one step forward from current Eol token
                if let Some(token) = stream.peek() {
                    if token.token_type == TokenType::Whitespace {
                        // If the next list is a child element, add it to children
                        if let Some(nest) = stream.is_next_list() {
                            if nest > cur_nest {
                                // Move forward by the number of whitespace tokens counted, so it becomes the root element in recursive `parse_unordered_list()`
                                for _ in 0..nest {
                                    stream.next();
                                }
                                let child = parse_unordered_list(stream, nest);
                                if let Some(position) = get_position(&child) {
                                    end = position.end
                                }
                                children.push(child);
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            // Save the content of the current list element as Text in nodes
            _ => {
                end = token.line;
                nodes.push(Node::Text(Text {
                    value: token.value.to_string(),
                    position: LineSpan {
                        start: token.line,
                        end: token.line,
                    },
                }));
                stream.next();
            }
        }
    }

    Node::UnorderedList(UnorderedList {
        level: cur_nest,
        nodes,
        children,
        position: LineSpan { start, end },
    })
}

/// Converts the tokens until the end of the line into nodes
fn parse_line(stream: &mut TokenStream) -> Vec<Node> {
    let mut nodes: Vec<Node> = vec![];

    while let Some(token) = stream.next() {
        match token.token_type {
            TokenType::Italic => {
                nodes.extend(parse_italic(stream));
            }
            TokenType::Bold => {
                nodes.extend(parse_bold(stream));
            }
            TokenType::Whitespace => nodes.push(Node::Whitespace(Whitespace {
                position: LineSpan {
                    start: token.line,
                    end: token.line,
                },
            })),
            // If the token is EOL (end of line), stop parsing
            TokenType::Eol => break,
            // For other tokens, treat them as Text nodes
            _ => nodes.push(Node::Text(Text {
                value: token.value.to_string(),
                position: LineSpan {
                    start: token.line,
                    end: token.line,
                },
            })),
        }
    }

    nodes
}

fn parse_header(stream: &mut TokenStream) -> Node {
    let mut nodes: Vec<Node> = vec![];

    // Validate the header and count header level
    let mut header_level = 0;
    let mut header_line = 0;
    let mut header_position = 0;

    while let Some(token) = stream.next() {
        match token.token_type {
            // Increment header level for each `#` token and store its line number
            TokenType::Header => {
                header_level += 1;
                header_line = token.line;
            }
            // Stop counting if the token is not a `#`
            _ => {
                stream.back();
                break;
            }
        }
    }

    if let Some(token) = stream.peek() {
        match token.token_type {
            // If the next token is Whitespace, process it as a valid Header
            TokenType::Whitespace => {
                header_position = token.line;

                // If the header level exceeds 6, treat it as a Paragraph instead
                if header_level > 6 {
                    let header_text_token = Token {
                        token_type: TokenType::Text,
                        value: "#".repeat(header_level),
                        line: header_line,
                    };
                    // Replace the last `#` token with a Text token without modifying the overall token index
                    stream.back();
                    stream.replace(header_text_token);
                    return parse_paragraph(stream);
                }
                // Process as a valid Header
                stream.next();
                nodes.push(parse_paragraph(stream));
            }
            // If the next token is not Whitespace, treat it as a Paragraph
            _ => {
                if token.token_type == TokenType::Text {
                    // Combine the `#` tokens and the text value into a single Paragraph
                    let value = format!("{}{}", "#".repeat(header_level), token.value);
                    stream.replace(Token {
                        token_type: TokenType::Text,
                        value,
                        line: header_line,
                    });
                    return parse_paragraph(stream);
                } else {
                    // If no text follows the `#`, treat it as a Paragraph
                    let header_text_token = Token {
                        token_type: TokenType::Text,
                        value: "#".repeat(header_level),
                        line: header_line,
                    };
                    stream.back();
                    stream.replace(header_text_token);
                    return parse_paragraph(stream);
                }
            }
        }
    }

    Node::Header(Header {
        level: header_level,
        nodes,
        position: LineSpan {
            start: header_position,
            end: header_position,
        },
    })
}

/// Wrap the nodes in a paragraph node.
fn parse_paragraph(stream: &mut TokenStream) -> Node {
    let nodes: Vec<Node> = parse_line(stream);
    let position = if !nodes.is_empty() {
        let start = nodes.first().unwrap().position().start;
        let end = nodes.last().unwrap().position().end;
        LineSpan { start, end }
    } else {
        // If there are no tokens to parse, refer to the previous token and use its line number
        if let Some(prev_token) = stream.get(stream.index - 1) {
            return Node::Paragraph(Paragraph {
                nodes,
                position: LineSpan {
                    start: prev_token.line,
                    end: prev_token.line,
                },
            });
        }
        LineSpan { start: 0, end: 0 }
    };
    Node::Paragraph(Paragraph { nodes, position })
}

fn parse_italic(stream: &mut TokenStream) -> Vec<Node> {
    let mut nodes: Vec<Node> = vec![];
    let mut is_closed = false;
    let mut start: usize = 0;
    let mut end: usize = 0;

    while let Some(token) = stream.peek() {
        match token.token_type {
            TokenType::Italic => {
                is_closed = true;
            }
            TokenType::Eol => {
                break;
            }
            _ => {
                nodes.push(parse_token(token));
            }
        }
        if start == 0 {
            start = token.line;
        }
        end = end.max(token.line);
        stream.next();
    }

    if !is_closed {
        let mut italic_token_line = 0;
        if let Some(prev_token) = stream.get(stream.index - 1) {
            italic_token_line = prev_token.line;
        }

        let italic_text_token = Node::Text(Text {
            value: "*".to_string(),
            position: LineSpan {
                start: italic_token_line,
                end: italic_token_line,
            },
        });
        let mut new_vec = vec![italic_text_token];
        new_vec.extend(nodes);
        return new_vec;
    }

    vec![Node::Italic(Italic {
        nodes,
        position: LineSpan { start, end },
    })]
}

fn parse_bold(stream: &mut TokenStream) -> Vec<Node> {
    let mut nodes: Vec<Node> = vec![];
    let mut is_closed = false;
    let mut start: usize = 0;
    let mut end: usize = 0;

    while let Some(token) = stream.peek() {
        match token.token_type {
            TokenType::Bold => {
                is_closed = true;
            }
            TokenType::Eol => {
                break;
            }
            _ => {
                nodes.push(parse_token(token));
            }
        }
        if start == 0 {
            start = token.line;
        }
        end = end.max(token.line);
        stream.next();
    }

    if !is_closed {
        let mut bold_token_line = 0;
        if let Some(prev_token) = stream.get(stream.index - 1) {
            bold_token_line = prev_token.line;
        }

        let bold_text_token = Node::Text(Text {
            value: "**".to_string(),
            position: LineSpan {
                start: bold_token_line,
                end: bold_token_line,
            },
        });
        let mut new_vec = vec![bold_text_token];
        new_vec.extend(nodes);
        return new_vec;
    }

    vec![Node::Bold(Bold {
        nodes,
        position: LineSpan { start, end },
    })]
}

fn parse_token(token: &Token) -> Node {
    match token.token_type {
        TokenType::Whitespace => Node::Whitespace(Whitespace {
            position: LineSpan {
                start: token.line,
                end: token.line,
            },
        }),
        _ => Node::Text(Text {
            value: token.value.to_string(),
            position: LineSpan {
                start: token.line,
                end: token.line,
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{
        Bold, Eol, Italic, LineSpan, Node, Paragraph, Text, UnorderedList, Whitespace,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_break() {
        let input = "normal\n\ntext";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![
                Node::Paragraph(Paragraph {
                    nodes: vec![Node::Text(Text {
                        value: "normal".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),],
                    position: LineSpan { start: 1, end: 1 }
                },),
                Node::Eol(Eol {
                    position: LineSpan { start: 2, end: 2 }
                }),
                Node::Paragraph(Paragraph {
                    nodes: vec![Node::Text(Text {
                        value: "text".to_string(),
                        position: LineSpan { start: 3, end: 3 }
                    }),],
                    position: LineSpan { start: 3, end: 3 }
                },),
            ],
        )
    }

    fn test_plain_text() {
        let input = "normal text";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Paragraph(Paragraph {
                nodes: vec![
                    Node::Text(Text {
                        value: "normal".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "text".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                position: LineSpan { start: 1, end: 1 }
            },)],
        )
    }

    #[test]
    fn test_multiple_text() {
        let input = "**bold**\n*italic*\nplain";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![
                Node::Paragraph(Paragraph {
                    nodes: vec![Node::Bold(Bold {
                        nodes: vec![Node::Text(Text {
                            value: "bold".to_string(),
                            position: LineSpan { start: 1, end: 1 }
                        }),],
                        position: LineSpan { start: 1, end: 1 }
                    })],
                    position: LineSpan { start: 1, end: 1 }
                },),
                Node::Paragraph(Paragraph {
                    nodes: vec![Node::Italic(Italic {
                        nodes: vec![Node::Text(Text {
                            value: "italic".to_string(),
                            position: LineSpan { start: 2, end: 2 }
                        }),],
                        position: LineSpan { start: 2, end: 2 }
                    })],
                    position: LineSpan { start: 2, end: 2 }
                },),
                Node::Paragraph(Paragraph {
                    nodes: vec![Node::Text(Text {
                        value: "plain".to_string(),
                        position: LineSpan { start: 3, end: 3 }
                    }),],
                    position: LineSpan { start: 3, end: 3 }
                },)
            ],
        )
    }

    #[test]
    fn test_closed_italic_marker() {
        let input = "*italic text*";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Paragraph(Paragraph {
                nodes: vec![Node::Italic(Italic {
                    nodes: vec![
                        Node::Text(Text {
                            value: "italic".to_string(),
                            position: LineSpan { start: 1, end: 1 }
                        }),
                        Node::Whitespace(Whitespace {
                            position: LineSpan { start: 1, end: 1 }
                        }),
                        Node::Text(Text {
                            value: "text".to_string(),
                            position: LineSpan { start: 1, end: 1 }
                        }),
                    ],
                    position: LineSpan { start: 1, end: 1 }
                })],
                position: LineSpan { start: 1, end: 1 }
            },)],
        )
    }

    #[test]
    fn test_unclosed_italic_marker() {
        let input = "*italic text";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Paragraph(Paragraph {
                nodes: vec![
                    Node::Text(Text {
                        value: "*".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "italic".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "text".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                position: LineSpan { start: 1, end: 1 }
            },)],
        )
    }

    #[test]
    fn test_unmatched_italic_marker() {
        let input = "italic text*";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Paragraph(Paragraph {
                nodes: vec![
                    Node::Text(Text {
                        value: "italic".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "text".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "*".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                position: LineSpan { start: 1, end: 1 }
            },)],
        )
    }

    #[test]
    fn test_closed_bold_marker() {
        let input = "**bold text**";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Paragraph(Paragraph {
                nodes: vec![Node::Bold(Bold {
                    nodes: vec![
                        Node::Text(Text {
                            value: "bold".to_string(),
                            position: LineSpan { start: 1, end: 1 }
                        }),
                        Node::Whitespace(Whitespace {
                            position: LineSpan { start: 1, end: 1 }
                        }),
                        Node::Text(Text {
                            value: "text".to_string(),
                            position: LineSpan { start: 1, end: 1 }
                        }),
                    ],
                    position: LineSpan { start: 1, end: 1 }
                })],
                position: LineSpan { start: 1, end: 1 }
            },)],
        )
    }

    #[test]
    fn test_unclosed_bold_marker() {
        let input = "**bold text";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Paragraph(Paragraph {
                nodes: vec![
                    Node::Text(Text {
                        value: "**".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "bold".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "text".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                position: LineSpan { start: 1, end: 1 }
            },)],
        )
    }

    #[test]
    fn test_header_marker() {
        let input = "# Header text";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Header(Header {
                level: 1,
                nodes: vec![Node::Paragraph(Paragraph {
                    nodes: vec![
                        Node::Text(Text {
                            value: "Header".to_string(),
                            position: LineSpan { start: 1, end: 1 }
                        }),
                        Node::Whitespace(Whitespace {
                            position: LineSpan { start: 1, end: 1 }
                        }),
                        Node::Text(Text {
                            value: "text".to_string(),
                            position: LineSpan { start: 1, end: 1 }
                        }),
                    ],
                    position: LineSpan { start: 1, end: 1 }
                })],
                position: LineSpan { start: 1, end: 1 }
            })]
        )
    }

    #[test]
    fn test_header_with_no_text() {
        let input = "### \ntext";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![
                Node::Header(Header {
                    level: 3,
                    nodes: vec![Node::Paragraph(Paragraph {
                        nodes: vec![],
                        position: LineSpan { start: 1, end: 1 }
                    })],
                    position: LineSpan { start: 1, end: 1 }
                }),
                Node::Paragraph(Paragraph {
                    nodes: vec![Node::Text(Text {
                        value: "text".to_string(),
                        position: LineSpan { start: 2, end: 2 }
                    }),],
                    position: LineSpan { start: 2, end: 2 }
                })
            ]
        )
    }

    #[test]
    fn test_too_long_header_marker() {
        let input = "####### Header text\n";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Paragraph(Paragraph {
                nodes: vec![
                    Node::Text(Text {
                        value: "#######".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "Header".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "text".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                position: LineSpan { start: 1, end: 1 }
            },)],
        )
    }

    #[test]
    fn test_invalid_header_marker() {
        let input = "#Header text";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Paragraph(Paragraph {
                nodes: vec![
                    Node::Text(Text {
                        value: "#Header".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "text".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                position: LineSpan { start: 1, end: 1 }
            },)],
        )
    }

    #[test]
    fn test_unordered_list() {
        let input = "- item 1\n- item 2\n- item 3\n";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![
                Node::UnorderedList(UnorderedList {
                    level: 0,
                    nodes: vec![
                        Node::Text(Text {
                            value: "item".to_string(),
                            position: LineSpan { start: 1, end: 1 }
                        }),
                        Node::Whitespace(Whitespace {
                            position: LineSpan { start: 1, end: 1 }
                        }),
                        Node::Text(Text {
                            value: "1".to_string(),
                            position: LineSpan { start: 1, end: 1 }
                        }),
                    ],
                    children: vec![],
                    position: LineSpan { start: 1, end: 1 }
                }),
                Node::UnorderedList(UnorderedList {
                    level: 0,
                    nodes: vec![
                        Node::Text(Text {
                            value: "item".to_string(),
                            position: LineSpan { start: 2, end: 2 }
                        }),
                        Node::Whitespace(Whitespace {
                            position: LineSpan { start: 2, end: 2 }
                        }),
                        Node::Text(Text {
                            value: "2".to_string(),
                            position: LineSpan { start: 2, end: 2 }
                        }),
                    ],
                    children: vec![],
                    position: LineSpan { start: 2, end: 2 }
                }),
                Node::UnorderedList(UnorderedList {
                    level: 0,
                    nodes: vec![
                        Node::Text(Text {
                            value: "item".to_string(),
                            position: LineSpan { start: 3, end: 3 }
                        }),
                        Node::Whitespace(Whitespace {
                            position: LineSpan { start: 3, end: 3 }
                        }),
                        Node::Text(Text {
                            value: "3".to_string(),
                            position: LineSpan { start: 3, end: 3 }
                        }),
                    ],
                    children: vec![],
                    position: LineSpan { start: 3, end: 3 }
                }),
            ],
        )
    }

    #[test]
    fn test_unordered_nested_list() {
        let input = "- item 1\n - item 1.1\n";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::UnorderedList(UnorderedList {
                level: 0,
                nodes: vec![
                    Node::Text(Text {
                        value: "item".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "1".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                children: vec![Node::UnorderedList(UnorderedList {
                    level: 1,
                    nodes: vec![
                        Node::Text(Text {
                            value: "item".to_string(),
                            position: LineSpan { start: 2, end: 2 }
                        }),
                        Node::Whitespace(Whitespace {
                            position: LineSpan { start: 2, end: 2 }
                        }),
                        Node::Text(Text {
                            value: "1.1".to_string(),
                            position: LineSpan { start: 2, end: 2 }
                        }),
                    ],
                    children: vec![],
                    position: LineSpan { start: 2, end: 2 }
                }),],
                position: LineSpan { start: 1, end: 2 }
            }),],
        )
    }

    #[test]
    fn test_nested_unordered_list_in_two_levels() {
        let input = "- item 1\n - item 1.1\n  - item 1.1.1";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::UnorderedList(UnorderedList {
                level: 0,
                nodes: vec![
                    Node::Text(Text {
                        value: "item".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "1".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                children: vec![Node::UnorderedList(UnorderedList {
                    level: 1,
                    nodes: vec![
                        Node::Text(Text {
                            value: "item".to_string(),
                            position: LineSpan { start: 2, end: 2 }
                        }),
                        Node::Whitespace(Whitespace {
                            position: LineSpan { start: 2, end: 2 }
                        }),
                        Node::Text(Text {
                            value: "1.1".to_string(),
                            position: LineSpan { start: 2, end: 2 }
                        }),
                    ],
                    children: vec![Node::UnorderedList(UnorderedList {
                        level: 2,
                        nodes: vec![
                            Node::Text(Text {
                                value: "item".to_string(),
                                position: LineSpan { start: 3, end: 3 }
                            }),
                            Node::Whitespace(Whitespace {
                                position: LineSpan { start: 3, end: 3 }
                            }),
                            Node::Text(Text {
                                value: "1.1.1".to_string(),
                                position: LineSpan { start: 3, end: 3 }
                            }),
                        ],
                        children: vec![],
                        position: LineSpan { start: 3, end: 3 }
                    }),],
                    position: LineSpan { start: 2, end: 3 }
                }),],
                position: LineSpan { start: 1, end: 3 }
            }),],
        )
    }

    #[test]
    fn test_two_unordered_list() {
        let input = "- item1\n - item1.1\n- item2";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![
                Node::UnorderedList(UnorderedList {
                    level: 0,
                    nodes: vec![Node::Text(Text {
                        value: "item1".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),],
                    children: vec![Node::UnorderedList(UnorderedList {
                        level: 1,
                        nodes: vec![Node::Text(Text {
                            value: "item1.1".to_string(),
                            position: LineSpan { start: 2, end: 2 }
                        }),],
                        children: vec![],
                        position: LineSpan { start: 2, end: 2 }
                    }),],
                    position: LineSpan { start: 1, end: 2 }
                }),
                Node::UnorderedList(UnorderedList {
                    level: 0,
                    nodes: vec![Node::Text(Text {
                        value: "item2".to_string(),
                        position: LineSpan { start: 3, end: 3 }
                    }),],
                    children: vec![],
                    position: LineSpan { start: 3, end: 3 }
                }),
            ],
        )
    }

    #[test]
    fn test_unordered_complexly_nested_list() {
        let input =
            "- item 1\n - item 1.1\n - item 1.2\n  - item 1.2.1\n   - item 1.2.1.1\n - item 1.3";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::UnorderedList(UnorderedList {
                level: 0,
                nodes: vec![
                    Node::Text(Text {
                        value: "item".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "1".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                children: vec![
                    Node::UnorderedList(UnorderedList {
                        level: 1,
                        nodes: vec![
                            Node::Text(Text {
                                value: "item".to_string(),
                                position: LineSpan { start: 2, end: 2 }
                            }),
                            Node::Whitespace(Whitespace {
                                position: LineSpan { start: 2, end: 2 }
                            }),
                            Node::Text(Text {
                                value: "1.1".to_string(),
                                position: LineSpan { start: 2, end: 2 }
                            }),
                        ],
                        children: vec![],
                        position: LineSpan { start: 2, end: 2 }
                    }),
                    Node::UnorderedList(UnorderedList {
                        level: 1,
                        nodes: vec![
                            Node::Text(Text {
                                value: "item".to_string(),
                                position: LineSpan { start: 3, end: 3 }
                            }),
                            Node::Whitespace(Whitespace {
                                position: LineSpan { start: 3, end: 3 }
                            }),
                            Node::Text(Text {
                                value: "1.2".to_string(),
                                position: LineSpan { start: 3, end: 3 }
                            }),
                        ],
                        children: vec![Node::UnorderedList(UnorderedList {
                            level: 2,
                            nodes: vec![
                                Node::Text(Text {
                                    value: "item".to_string(),
                                    position: LineSpan { start: 4, end: 4 }
                                }),
                                Node::Whitespace(Whitespace {
                                    position: LineSpan { start: 4, end: 4 }
                                }),
                                Node::Text(Text {
                                    value: "1.2.1".to_string(),
                                    position: LineSpan { start: 4, end: 4 }
                                }),
                            ],
                            children: vec![Node::UnorderedList(UnorderedList {
                                level: 3,
                                nodes: vec![
                                    Node::Text(Text {
                                        value: "item".to_string(),
                                        position: LineSpan { start: 5, end: 5 }
                                    }),
                                    Node::Whitespace(Whitespace {
                                        position: LineSpan { start: 5, end: 5 }
                                    }),
                                    Node::Text(Text {
                                        value: "1.2.1.1".to_string(),
                                        position: LineSpan { start: 5, end: 5 }
                                    }),
                                ],
                                children: vec![],
                                position: LineSpan { start: 5, end: 5 }
                            }),],
                            position: LineSpan { start: 4, end: 5 }
                        }),],
                        position: LineSpan { start: 3, end: 5 }
                    }),
                    Node::UnorderedList(UnorderedList {
                        level: 1,
                        nodes: vec![
                            Node::Text(Text {
                                value: "item".to_string(),
                                position: LineSpan { start: 6, end: 6 }
                            }),
                            Node::Whitespace(Whitespace {
                                position: LineSpan { start: 6, end: 6 }
                            }),
                            Node::Text(Text {
                                value: "1.3".to_string(),
                                position: LineSpan { start: 6, end: 6 }
                            }),
                        ],
                        children: vec![],
                        position: LineSpan { start: 6, end: 6 }
                    }),
                ],
                position: LineSpan { start: 1, end: 6 }
            }),],
        )
    }

    #[test]
    fn test_unordered_list_started_with_nested_content() {
        let input = " - item1";
        let nodes = build_tree(input);

        assert_eq!(
            nodes,
            vec![Node::Paragraph(Paragraph {
                nodes: vec![
                    Node::Whitespace(Whitespace {
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "- ".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                    Node::Text(Text {
                        value: "item1".to_string(),
                        position: LineSpan { start: 1, end: 1 }
                    }),
                ],
                position: LineSpan { start: 1, end: 1 }
            },)],
        )
    }

    #[test]
    fn test_fn_is_next_list() {
        // not nested
        let input = "- item1";
        let mut tokens = lex(input);
        let stream = TokenStream::new(&mut tokens);
        let next_nest = stream.is_next_list();
        assert_eq!(next_nest, Some(0));

        // nested once
        let input = " - item1";
        let mut tokens = lex(input);
        let stream = TokenStream::new(&mut tokens);
        let next_nest = stream.is_next_list();
        assert_eq!(next_nest, Some(1));
    }
}
