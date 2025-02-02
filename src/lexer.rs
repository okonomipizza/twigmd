use crate::token::{Token, TokenType};

struct CharStream<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> CharStream<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    // Reads the next character without advancing the position.
    pub fn peek_next(&self) -> Option<char> {
        let mut chars = self.input[self.position..].chars();
        chars.next()
    }

    // Advances by one character and returns it.
    pub fn next(&mut self) -> Option<char> {
        let mut chars = self.input[self.position..].chars();
        let next_char = chars.next();
        if let Some(c) = next_char {
            self.position += c.len_utf8() // UTF-8対応
        }
        next_char
    }

    // Retrives the character `n` steps back from the current position.
    pub fn prev(&mut self, n: usize) -> Option<char> {
        if self.position >= n {
            let mut chars = self.input[0..self.position].chars().rev(); // Reverse iterator

            for _ in 0..n - 1 {
                chars.next()?;
            }

            chars.next()
        } else {
            None
        }
    }

    // Consumes and returns a string until a separator (whitespace or newline) is found.
    pub fn consume_until_separator(&mut self) -> String {
        let mut result = String::new();

        // Retrive the previous character (before calling this function) to ensure proper handling.
        if let Some(c) = self.prev(1) {
            if c.is_whitespace() || c == '\n' {
                return result;
            }
            result.push(c);
        }

        while let Some(c) = self.next() {
            if c.is_whitespace() || c == '\n' || c == '*' {
                // Move the position back if a separator is found.
                self.position -= c.len_utf8();
                break;
            }
            result.push(c);
        }
        result
    }
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut stream = CharStream::new(input);
    let mut tokens: Vec<Token> = Vec::new();
    let mut line = 1;

    // Process the input one character at a time.
    while let Some(c) = stream.next() {
        match c {
            '\n' => {
                tokens.push(Token {
                    token_type: TokenType::Eol,
                    value: c.to_string(),
                    line,
                });
                line += 1; // Increment the line count on a newline.
            }
            ' ' => tokens.push(Token {
                token_type: TokenType::Whitespace,
                value: c.to_string(),
                line,
            }),
            '#' => tokens.push(Token {
                token_type: TokenType::Header,
                value: c.to_string(),
                line,
            }),
            '-' => {
                if let Some(next) = stream.peek_next() {
                    if next.is_whitespace() {
                        tokens.push(Token {
                            token_type: TokenType::UnorderedList,
                            value: "- ".to_string(),
                            line,
                        });
                        stream.next();
                    } else {
                        let text = stream.consume_until_separator();
                        if text.is_empty() {
                            continue;
                        }

                        tokens.push(Token {
                            token_type: TokenType::Text,
                            value: text,
                            line,
                        });
                    }
                }
            }
            '>' => tokens.push(Token {
                token_type: TokenType::BlockQuote,
                value: c.to_string(),
                line,
            }),
            '`' => tokens.push(Token {
                token_type: TokenType::InlineCode,
                value: c.to_string(),
                line,
            }),
            '*' => {
                if let Some(prev) = stream.prev(2) {
                    if prev == '*' {
                        if let Some(last) = tokens.last_mut() {
                            *last = Token {
                                token_type: TokenType::Bold,
                                value: "**".to_string(),
                                line,
                            };
                            continue;
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Italic,
                            value: c.to_string(),
                            line,
                        })
                    }
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Italic,
                        value: c.to_string(),
                        line,
                    })
                }
            },
            '!' => tokens.push(Token {
                token_type: TokenType::Exclamation,
                value: c.to_string(),
                line,
            }),
            '{' => tokens.push(Token {
                token_type: TokenType::CarlyBracketOpen,
                value: c.to_string(),
                line,
            }),
            '}' => tokens.push(Token {
                token_type: TokenType::CarlyBracketClose,
                value: c.to_string(),
                line,
            }),
            '[' => tokens.push(Token {
                token_type: TokenType::SquareBracketOpen,
                value: c.to_string(),
                line,
            }),
            ']' => tokens.push(Token {
                token_type: TokenType::SquareBracketClose,
                value: c.to_string(),
                line,
            }),
            '(' => tokens.push(Token {
                token_type: TokenType::ParenthesisOpen,
                value: c.to_string(),
                line,
            }),
            ')' => tokens.push(Token {
                token_type: TokenType::ParenthesisClose,
                value: c.to_string(),
                line,
            }),
            ';' => tokens.push(Token {
                token_type: TokenType::SemiColon,
                value: c.to_string(),
                line,
            }),
            ':' => tokens.push(Token {
                token_type: TokenType::Colon,
                value: c.to_string(),
                line,
            }),
            _ => {
                let text = stream.consume_until_separator();
                if text.is_empty() {
                    continue;
                }

                tokens.push(Token {
                    token_type: TokenType::Text,
                    value: text,
                    line,
                });
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenType};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_header_marker() {
        let input = "#";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::Header,
                value: '#'.to_string(),
                line: 1
            }]
        )
    }

    #[test]
    fn test_multiple_markers() {
        let input = "# > ` * !";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Header,
                    value: '#'.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Whitespace,
                    value: ' '.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::BlockQuote,
                    value: '>'.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Whitespace,
                    value: ' '.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::InlineCode,
                    value: '`'.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Whitespace,
                    value: ' '.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Italic,
                    value: '*'.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Whitespace,
                    value: ' '.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Exclamation,
                    value: '!'.to_string(),
                    line: 1,
                }
            ]
        );
    }

    #[test]
    fn test_unordered_list() {
        let input = "- list\n - list1-1";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::UnorderedList,
                    value: "- ".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Text,
                    value: "list".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Eol,
                    value: "\n".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Whitespace,
                    value: " ".to_string(),
                    line: 2
                },
                Token {
                    token_type: TokenType::UnorderedList,
                    value: "- ".to_string(),
                    line: 2,
                },
                Token {
                    token_type: TokenType::Text,
                    value: "list1-1".to_string(),
                    line: 2,
                },
            ]
        );
    }

    #[test]
    fn test_invalid_unordered_list() {
        let input = "-list";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![Token {
                token_type: TokenType::Text,
                value: "-list".to_string(),
                line: 1,
            },]
        );
    }

    #[test]
    fn test_text_and_symbols() {
        let input = "Hello, world! #Markdown";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Text,
                    value: "Hello,".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Whitespace,
                    value: ' '.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Text,
                    value: "world!".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Whitespace,
                    value: ' '.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Header,
                    value: '#'.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Text,
                    value: "Markdown".to_string(),
                    line: 1,
                },
            ]
        );
    }

    #[test]
    fn test_italic_markers() {
        let input = "*italic*";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Italic,
                    value: "*".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Text,
                    value: "italic".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Italic,
                    value: "*".to_string(),
                    line: 1,
                },
            ]
        );
    }

    #[test]
    fn test_bold_markers() {
        let input = "**bold**";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Bold,
                    value: "**".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Text,
                    value: "bold".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Bold,
                    value: "**".to_string(),
                    line: 1,
                },
            ]
        );
    }

    #[test]
    fn test_multiline_input() {
        let input = "# Header\n- List Item\n";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Header,
                    value: '#'.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Whitespace,
                    value: ' '.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Text,
                    value: "Header".to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::Eol,
                    value: '\n'.to_string(),
                    line: 1,
                },
                Token {
                    token_type: TokenType::UnorderedList,
                    value: "- ".to_string(),
                    line: 2,
                },
                Token {
                    token_type: TokenType::Text,
                    value: "List".to_string(),
                    line: 2,
                },
                Token {
                    token_type: TokenType::Whitespace,
                    value: ' '.to_string(),
                    line: 2,
                },
                Token {
                    token_type: TokenType::Text,
                    value: "Item".to_string(),
                    line: 2,
                },
                Token {
                    token_type: TokenType::Eol,
                    value: '\n'.to_string(),
                    line: 2,
                },
            ]
        );
    }
    
    
}
