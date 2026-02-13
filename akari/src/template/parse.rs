use super::Value as Obj; 

#[derive(Debug, PartialEq, Clone)]
pub enum Token { 
    // Directives and Block Structure
    TemplateKeyword,         // e.g., "template"
    InsertKeyword,           // e.g., "insert" 
    BlockKeyword,            // e.g., "block"
    EndBlockKeyword,         // e.g., "endblock"
    ExportKeyword,           // e.g., "export"
    PlaceholderKeyword,      // e.g., "placeholder"
    
    // Control Flow and Loop Constructs
    LetKeyword,              // e.g., "let"
    ForKeyword,              // e.g., "for"
    InKeyword,               // e.g., "in"
    IfKeyword,               // e.g., "if"
    MatchKeyword,            // e.g., "match" 
    CaseKeyword,             // e.g., "case" 
    OutputKeyword,           // e.g., "output"
    EndIfKeyword,            // e.g., "endif"
    EndForKeyword,           // e.g., "endfor"
    EndMatchKeyword,         // e.g., "endmatch" 
    EndCaseKeyword,          // e.g., "endcase" 
    WhileKeyword,            // e.g., "while"
    EndWhileKeyword,         // e.g., "endwhile"
    DelKeyword,              // e.g., "del" 
    
    // Literals and Identifiers
    Identifier(String),      // variable names or user-defined names
    Object(Obj),          // literal number, string, boolean, list and object 
    HtmlContent(String),     // HTML content such as "<script ...>...</script>"
    
    // Operators and Punctuation 
    Dot,                  // . (dot operator for object access) 

    // Assignment Operators
    Assignment,              // =
    PlusAssignment,          // +=
    MinusAssignment,         // -=
    MultiplyAssignment,      // *=
    DivideAssignment,        // /=
    ModulusAssignment,       // %=
    
    // Increment/Decrement Operators
    Increment,               // ++
    Decrement,               // --
    
    // Arithmetic Operators
    Plus,                    // +
    Minus,                   // -
    Multiply,                // *
    Divide,                  // /
    Modulus,                 // %
    Exponent,                // ** (or ^, if you choose)
    
    // Comparison Operators
    EqualsEquals,            // ==
    NotEquals,               // !=
    LessThan,                // <
    LessThanEquals,          // <=
    GreaterThan,             // >
    GreaterThanEquals,       // >=
    
    // Logical Operators
    LogicalAnd,              // && (or "and")
    LogicalOr,               // || (or "or")
    LogicalNot,              // !  (or "not")
    
    // Grouping and Delimiters
    LeftParen,               // (
    RightParen,              // )
    LeftSquareBracket,       // [
    RightSquareBracket,      // ]
    
    // End of Statement
    EndOfStatement,          // Marks end of a directive or statement 
}

/// The Lexer struct holds the input string (our template source code)
/// and a current position pointer.
pub struct Lexer {
    input: String,
    pos: usize,
} 

impl Lexer {
    /// Creates a new Lexer instance from a given input.
    /// The input can be any type convertible to a String.
    pub fn new(input: String) -> Self {
        Lexer { input, pos: 0 }
    }

    /// Returns the next character from the current position without consuming it.
    pub fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Checks if the input (from the current position) starts with the given substring.
    pub fn peek_str(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    /// Consumes and returns the next character, advancing the position.
    pub fn next_char(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    /// Advances the position while the next character is a whitespace.
    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    /// Checks if the character following the current one is a digit.
    pub fn peek_next_is_digit(&self) -> bool {
        let mut iter = self.input[self.pos..].chars();
        iter.next(); // skip the current char
        if let Some(next_ch) = iter.next() {
            next_ch.is_digit(10)
        } else {
            false
        }
    }

    /// This function lexes a directive block.
    ///
    /// A directive block starts with the marker "-[" (already consumed in the main loop)
    /// and ends with the marker "]-". It tokenizes the content inside (keywords, identifiers,
    /// literals, operators, etc.) and finally appends an `EndOfStatement` token.
    pub fn lex_directive(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        self.skip_whitespace();
        while self.pos < self.input.len() && !self.peek_str("]-") {
            self.skip_whitespace();
            if self.peek_str("]-") {
                break;
            }
            // Lex one token at a time from within the directive
            let token = self.lex_directive_token();
            tokens.push(token);
            self.skip_whitespace();
        }
        // Consume the closing marker "]-" if present.
        if self.peek_str("]-") {
            self.pos += 2;
        }
        // Mark the end of the directive statement.
        tokens.push(Token::EndOfStatement);
        tokens
    }

    /// Lexes a single token inside a directive block.
    ///
    /// It distinguishes between string literals, numeric literals (also handling negatives),
    /// identifiers (or keywords), and various operators/punctuation.
    pub fn lex_directive_token(&mut self) -> Token {
        self.skip_whitespace();
        // If we have reached the directive closing marker, return an EndOfStatement.
        if self.peek_str("]-") {
            return Token::EndOfStatement;
        }
        if let Some(ch) = self.peek() {
            // Handle string literals (delimited by double quotes)
            if ch == '"' {
                return self.lex_string();
            }
            // Handle numeric literals: if the current char is a digit or a minus sign
            // followed by a digit, then treat it as a number.
            if ch.is_digit(10) || (ch == '-' && self.peek_next_is_digit()) {
                return self.lex_number();
            }
            // Handle identifiers and keywords: they start with an alphabetic character or underscore.
            if ch.is_alphabetic() || ch == '_' {
                return self.lex_identifier_or_keyword();
            }
            // Handle multi-character operators and punctuation.
            // Check two-character sequences first.
            if self.peek_str("==") {
                self.pos += 2;
                return Token::EqualsEquals;
            }
            if self.peek_str("!=") {
                self.pos += 2;
                return Token::NotEquals;
            }
            if self.peek_str("<=") {
                self.pos += 2;
                return Token::LessThanEquals;
            }
            if self.peek_str(">=") {
                self.pos += 2;
                return Token::GreaterThanEquals;
            }
            if self.peek_str("+=") {
                self.pos += 2;
                return Token::PlusAssignment;
            }
            if self.peek_str("-=") {
                self.pos += 2;
                return Token::MinusAssignment;
            }
            if self.peek_str("*=") {
                self.pos += 2;
                return Token::MultiplyAssignment;
            }
            if self.peek_str("/=") {
                self.pos += 2;
                return Token::DivideAssignment;
            }
            if self.peek_str("%=") {
                self.pos += 2;
                return Token::ModulusAssignment;
            }
            if self.peek_str("++") {
                self.pos += 2;
                return Token::Increment;
            }
            if self.peek_str("--") {
                self.pos += 2;
                return Token::Decrement;
            }
            if self.peek_str("**") {
                self.pos += 2;
                return Token::Exponent;
            }
            if self.peek_str("&&") {
                self.pos += 2;
                return Token::LogicalAnd;
            }
            if self.peek_str("||") {
                self.pos += 2;
                return Token::LogicalOr;
            }
            // If no two-character operator matches, check for single-character tokens.
            let ch = self.next_char().unwrap(); // safe because peek() returned Some(ch)
            match ch {
                '=' => Token::Assignment,
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Multiply,
                '/' => Token::Divide,
                '%' => Token::Modulus,
                '<' => Token::LessThan,
                '>' => Token::GreaterThan,
                '!' => Token::LogicalNot,
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '[' => Token::LeftSquareBracket,
                ']' => Token::RightSquareBracket,
                '.' => Token::Dot, 
                // For any unrecognized character, we simply return it as an identifier.
                _ => Token::Identifier(ch.to_string()),
            }
        } else {
            Token::EndOfStatement
        }
    }

    /// Lexes a string literal within a directive.
    ///
    /// It handles escape sequences such as \n, \t, \\ and \".
    /// The returned token wraps the resulting string into an Object (i.e. Obj::Str).
    pub fn lex_string(&mut self) -> Token {
        // Consume the opening double quote.
        self.next_char();
        let mut s = String::new();
        while let Some(ch) = self.next_char() {
            if ch == '"' {
                break;
            }
            if ch == '\\' {
                if let Some(escaped) = self.next_char() {
                    match escaped {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        other => s.push(other),
                    }
                }
            } else {
                s.push(ch);
            }
        }
        // Wrap the literal string into an Object token.
        Token::Object(Obj::Str(s))
    }

    /// Lexes a numeric literal (which may be an integer or a floating point number).
    ///
    /// The numeric literal is then wrapped into an Object token (i.e. Obj::Numerical).
    pub fn lex_number(&mut self) -> Token {
        let start = self.pos;
        let mut dot_encountered = false;
        if self.peek() == Some('-') {
            self.next_char();
        }
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                self.next_char();
            } else if ch == '.' && !dot_encountered {
                dot_encountered = true;
                self.next_char();
            } else {
                break;
            }
        }
        let number_str = &self.input[start..self.pos];
        if let Ok(num) = number_str.parse::<f64>() {
            Token::Object(Obj::Numerical(num))
        } else {
            // If parsing fails, return the raw string as an identifier.
            Token::Identifier(number_str.to_string())
        }
    }

    /// Lexes an identifier or keyword.
    ///
    /// This function collects a contiguous string of alphanumeric characters or underscores.
    /// It then checks if the word matches a reserved keyword (such as "template", "block", etc.)
    /// or one of the boolean literals ("true", "false"). If not, it returns it as an Identifier token.
    pub fn lex_identifier_or_keyword(&mut self) -> Token {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.next_char();
            } else {
                break;
            }
        }
        let word = &self.input[start..self.pos];
        // Handle boolean literals.
        if word == "true" {
            return Token::Object(Obj::Boolean(true));
        } else if word == "false" {
            return Token::Object(Obj::Boolean(false));
        }
        // Map reserved words to their corresponding token variants.
        match word {
            "template" => Token::TemplateKeyword, 
            "insert" => Token::InsertKeyword, 
            "block" => Token::BlockKeyword,
            "endblock" => Token::EndBlockKeyword,
            "export" => Token::ExportKeyword,
            "placeholder" => Token::PlaceholderKeyword,
            "let" => Token::LetKeyword,
            "for" => Token::ForKeyword,
            "in" => Token::InKeyword,
            "if" => Token::IfKeyword,
            "output" => Token::OutputKeyword,
            "endif" => Token::EndIfKeyword,
            "endfor" => Token::EndForKeyword,
            "while" => Token::WhileKeyword,
            "endwhile" => Token::EndWhileKeyword,
            "del" => Token::DelKeyword,
            "match" => Token::MatchKeyword, 
            "endmatch" => Token::EndMatchKeyword, 
            "case" => Token::CaseKeyword, 
            "endcase" => Token::EndCaseKeyword, 
            _ => Token::Identifier(word.to_string()),
        }
    }
}

/// Tokenizes the entire input into a list of tokens for the template language.
///
/// This function automatically handles both HTML content (outside directive markers)
/// and directive blocks (inside "-[" and "]-"). It accepts any input type that can be
/// converted into a String (such as &str, String, or even Vec<u8> after conversion).
///
/// # Example
///
/// ```rust
/// use akari::{tokenize, Token}; 
/// use akari::Value; 
/// let input = r#"
/// -[ template "template.html" ]-
/// -[ block header ]-
///     <script src="pmine.org"></script>
/// -[ endblock ]-
///
/// -[ block body ]-
///     -[ let a = 1 ]-
///     -[ for str in list ]-
///         -[ if (a % 2 == 0) ]-
///             -[ output str ]-
///         -[ endif ]-
///         -[ a = a + 1 ]-
///     -[ endfor ]-
/// -[ endblock ]-
/// "#;
///
/// let tokens = tokenize(input); 
/// println!("{:?}", tokens); 
/// // `tokens` now contains a mixture of HtmlContent tokens and directive tokens,
/// // with each directive ending with an EndOfStatement token.
/// ```
pub fn tokenize<S: Into<String>>(input: S) -> Vec<Token> {
    let input_str = input.into();
    let mut lexer = Lexer::new(input_str);
    let mut tokens = Vec::new();

    // The main loop alternates between HTML mode and directive mode.
    while lexer.pos < lexer.input.len() {
        // When we see the directive start marker "-[", enter directive mode.
        if lexer.peek_str("-[") {
            lexer.pos += 2; // Consume the "-[" marker.
            let directive_tokens = lexer.lex_directive();
            tokens.extend(directive_tokens);
        } else {
            // Otherwise, we are in HTML mode: collect text until the next "-[".
            let start = lexer.pos;
            while lexer.pos < lexer.input.len() && !lexer.peek_str("-[") {
                lexer.next_char();
            }
            let html_content = lexer.input[start..lexer.pos].to_string();
            if !html_content.is_empty() {
                tokens.push(Token::HtmlContent(html_content));
            }
        }
    }
    tokens
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = r#"
        -[ template "template.html" ]-
        -[ block header ]-
            <script src="pmine.org"></script>
        -[ endblock ]-

        -[ block body ]-
            -[ let a = 1 ]-
            -[ for str in list ]-
                -[ if (a % 2 == 0) ]-
                    -[ output str ]-
                -[ endif ]-
                -[ a = a + 1 ]-
            -[ endfor ]-
        -[ endblock ]-
        "#;
        let tokens = tokenize(input);
        println!("{:?}", tokens);
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_html_only_no_directives() {
        let tokens = tokenize("<h1>Hello World</h1>");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::HtmlContent(s) if s == "<h1>Hello World</h1>"));
    }

    #[test]
    fn test_empty_directive() {
        let tokens = tokenize("-[]-");
        // Empty directive should produce just an EndOfStatement
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::EndOfStatement));
    }

    #[test]
    fn test_directive_whitespace_only() {
        let tokens = tokenize("-[   ]-");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::EndOfStatement));
    }

    #[test]
    fn test_unclosed_directive() {
        // -[ without ]- — lexer should consume to end of input
        let tokens = tokenize("-[ block header");
        // Should still produce tokens, just no closing marker consumed
        assert!(tokens.iter().any(|t| matches!(t, Token::BlockKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::EndOfStatement)));
    }

    #[test]
    fn test_unterminated_string_in_directive() {
        // String literal without closing quote
        let tokens = tokenize(r#"-[ template "unclosed ]-"#);
        // The lexer consumes until it hits a quote or end of input
        // Should still produce tokens without panicking
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_escape_sequences_in_string() {
        let tokens = tokenize(r#"-[ let x = "hello\nworld\t\"quoted\"" ]-"#);
        let has_string = tokens.iter().any(|t| {
            if let Token::Object(Obj::Str(s)) = t {
                s.contains('\n') && s.contains('\t') && s.contains('"')
            } else {
                false
            }
        });
        assert!(has_string, "Expected string with escape sequences");
    }

    #[test]
    fn test_negative_number() {
        let tokens = tokenize("-[ let x = -42 ]-");
        let has_neg = tokens.iter().any(|t| {
            matches!(t, Token::Object(Obj::Numerical(n)) if *n == -42.0)
        });
        assert!(has_neg, "Expected negative number token");
    }

    #[test]
    fn test_float_number() {
        let tokens = tokenize("-[ let x = 3.14 ]-");
        let has_float = tokens.iter().any(|t| {
            matches!(t, Token::Object(Obj::Numerical(n)) if (*n - 3.14).abs() < f64::EPSILON)
        });
        assert!(has_float, "Expected float number token");
    }

    #[test]
    fn test_boolean_literals() {
        let tokens = tokenize("-[ if true ]- -[ if false ]-");
        let bools: Vec<_> = tokens.iter().filter(|t| matches!(t, Token::Object(Obj::Boolean(_)))).collect();
        assert_eq!(bools.len(), 2);
    }

    #[test]
    fn test_all_comparison_operators() {
        let tokens = tokenize("-[ if a == b ]- -[ if a != b ]- -[ if a <= b ]- -[ if a >= b ]- -[ if a < b ]- -[ if a > b ]-");
        assert!(tokens.iter().any(|t| matches!(t, Token::EqualsEquals)));
        assert!(tokens.iter().any(|t| matches!(t, Token::NotEquals)));
        assert!(tokens.iter().any(|t| matches!(t, Token::LessThanEquals)));
        assert!(tokens.iter().any(|t| matches!(t, Token::GreaterThanEquals)));
        assert!(tokens.iter().any(|t| matches!(t, Token::LessThan)));
        assert!(tokens.iter().any(|t| matches!(t, Token::GreaterThan)));
    }

    #[test]
    fn test_all_arithmetic_operators() {
        let tokens = tokenize("-[ let x = a + b - c * d / e % f ** g ]-");
        assert!(tokens.iter().any(|t| matches!(t, Token::Plus)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Minus)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Multiply)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Divide)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Modulus)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Exponent)));
    }

    #[test]
    fn test_assignment_operators() {
        let tokens = tokenize("-[ a = 1 ]- -[ a += 1 ]- -[ a -= 1 ]- -[ a *= 2 ]- -[ a /= 2 ]- -[ a %= 3 ]-");
        assert!(tokens.iter().any(|t| matches!(t, Token::Assignment)));
        assert!(tokens.iter().any(|t| matches!(t, Token::PlusAssignment)));
        assert!(tokens.iter().any(|t| matches!(t, Token::MinusAssignment)));
        assert!(tokens.iter().any(|t| matches!(t, Token::MultiplyAssignment)));
        assert!(tokens.iter().any(|t| matches!(t, Token::DivideAssignment)));
        assert!(tokens.iter().any(|t| matches!(t, Token::ModulusAssignment)));
    }

    #[test]
    fn test_increment_decrement() {
        let tokens = tokenize("-[ a++ ]- -[ b-- ]-");
        assert!(tokens.iter().any(|t| matches!(t, Token::Increment)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Decrement)));
    }

    #[test]
    fn test_logical_operators() {
        let tokens = tokenize("-[ if a && b || !c ]-");
        assert!(tokens.iter().any(|t| matches!(t, Token::LogicalAnd)));
        assert!(tokens.iter().any(|t| matches!(t, Token::LogicalOr)));
        assert!(tokens.iter().any(|t| matches!(t, Token::LogicalNot)));
    }

    #[test]
    fn test_grouping_and_index() {
        let tokens = tokenize("-[ output (a + b) ]- -[ output list[0] ]-");
        assert!(tokens.iter().any(|t| matches!(t, Token::LeftParen)));
        assert!(tokens.iter().any(|t| matches!(t, Token::RightParen)));
        assert!(tokens.iter().any(|t| matches!(t, Token::LeftSquareBracket)));
        assert!(tokens.iter().any(|t| matches!(t, Token::RightSquareBracket)));
    }

    #[test]
    fn test_dot_operator() {
        let tokens = tokenize("-[ output obj.field ]-");
        assert!(tokens.iter().any(|t| matches!(t, Token::Dot)));
    }

    #[test]
    fn test_all_keywords() {
        let tokens = tokenize(
            r#"-[ template "t" ]- -[ insert "i" ]- -[ block b ]- -[ endblock ]- -[ export ]- -[ placeholder ]- -[ let x = 1 ]- -[ for i in list ]- -[ endfor ]- -[ if x ]- -[ endif ]- -[ while x ]- -[ endwhile ]- -[ del x ]- -[ output x ]- -[ match x ]- -[ endmatch ]- -[ case 1 ]- -[ endcase ]-"#
        );
        assert!(tokens.iter().any(|t| matches!(t, Token::TemplateKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::InsertKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::BlockKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::EndBlockKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::ExportKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::PlaceholderKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::LetKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::ForKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::InKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::EndForKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::IfKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::EndIfKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::WhileKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::EndWhileKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::DelKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::OutputKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::MatchKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::EndMatchKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::CaseKeyword)));
        assert!(tokens.iter().any(|t| matches!(t, Token::EndCaseKeyword)));
    }

    #[test]
    fn test_unrecognized_char_becomes_identifier() {
        // Characters like @ # $ etc. become single-char identifiers
        let tokens = tokenize("-[ @ ]-");
        let has_at = tokens.iter().any(|t| {
            matches!(t, Token::Identifier(s) if s == "@")
        });
        assert!(has_at, "Unrecognized char should become an Identifier");
    }

    #[test]
    fn test_adjacent_directives() {
        let tokens = tokenize("-[ if x ]--[ endif ]-");
        let end_of_stmts: Vec<_> = tokens.iter().filter(|t| matches!(t, Token::EndOfStatement)).collect();
        assert_eq!(end_of_stmts.len(), 2);
    }

    #[test]
    fn test_html_between_directives() {
        let tokens = tokenize("-[ if x ]-<p>hello</p>-[ endif ]-");
        assert!(tokens.iter().any(|t| matches!(t, Token::HtmlContent(s) if s == "<p>hello</p>")));
    }

    #[test]
    fn test_number_parse_failure_becomes_identifier() {
        // A minus not followed by a digit is treated as Minus operator
        let tokens = tokenize("-[ - ]-");
        assert!(tokens.iter().any(|t| matches!(t, Token::Minus)));
    }
}
