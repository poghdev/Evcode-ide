use std::borrow::Cow;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword,
    Function,
    String,
    Number,
    Comment,
    Bracket,
    Operator,
    Plain,
}

pub struct Token<'a> {
    pub text: Cow<'a, str>,
    pub kind: TokenKind,
}

#[derive(Clone, Copy)]
pub enum Lang {
    Rust,
    Python,
    JavaScript,
    Generic,
}

impl Lang {
    pub fn from_path(path: &str) -> Self {
        match path.rsplit('.').next().unwrap_or("") {
            "rs"                      => Lang::Rust,
            "py"                      => Lang::Python,
            "js" | "ts" | "jsx" | "tsx" => Lang::JavaScript,
            _                         => Lang::Generic,
        }
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            Lang::Rust => &[
                "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl",
                "trait", "for", "while", "loop", "if", "else", "match", "return",
                "self", "Self", "super", "crate", "type", "where", "async", "await",
                "move", "ref", "in", "const", "static", "unsafe", "extern", "dyn",
                "true", "false", "Some", "None", "Ok", "Err",
            ],
            Lang::Python => &[
                "def", "class", "import", "from", "return", "if", "elif", "else",
                "for", "while", "in", "not", "and", "or", "True", "False", "None",
                "pass", "break", "continue", "with", "as", "try", "except", "finally",
                "raise", "yield", "lambda", "self", "super", "async", "await",
            ],
            Lang::JavaScript => &[
                "function", "const", "let", "var", "return", "if", "else", "for",
                "while", "class", "import", "export", "default", "new", "this",
                "typeof", "instanceof", "true", "false", "null", "undefined",
                "async", "await", "try", "catch", "finally", "throw",
            ],
            Lang::Generic => &[],
        }
    }
}

pub fn highlight_line<'a>(line: &'a str, lang: Lang) -> Line<'a> {
    let trimmed = line.trim_start();
    let is_comment = match lang {
        Lang::Rust | Lang::JavaScript => trimmed.starts_with("//"),
        Lang::Python => trimmed.starts_with('#'),
        Lang::Generic => false,
    };
    if is_comment {
        return Line::from(Span::styled(
            Cow::Borrowed(line),
            Style::default().fg(Color::Rgb(98, 114, 164)).add_modifier(Modifier::ITALIC),
        ));
    }

    let tokens = tokenize(line, lang);
    let spans: Vec<Span<'a>> = tokens
        .into_iter()
        .map(|t| {
            let style = token_style(&t.kind);
            match t.text {
                Cow::Borrowed(s) => Span::styled(s, style),
                Cow::Owned(s)    => Span::styled(s, style),
            }
        })
        .collect();

    Line::from(spans)
}

fn token_style(kind: &TokenKind) -> Style {
    match kind {
        TokenKind::Keyword  => Style::default().fg(Color::Rgb(97,  175, 239)).add_modifier(Modifier::BOLD),
        TokenKind::Function => Style::default().fg(Color::Rgb(209, 154, 102)),
        TokenKind::String   => Style::default().fg(Color::Rgb(152, 195, 121)),
        TokenKind::Number   => Style::default().fg(Color::Rgb(209, 154, 102)),
        TokenKind::Comment  => Style::default().fg(Color::Rgb(98,  114, 164)).add_modifier(Modifier::ITALIC),
        TokenKind::Bracket  => Style::default().fg(Color::Rgb(198, 120, 221)),
        TokenKind::Operator => Style::default().fg(Color::Rgb(86,  182, 194)),
        TokenKind::Plain    => Style::default().fg(Color::Rgb(220, 220, 220)),
    }
}

fn tokenize<'a>(line: &'a str, lang: Lang) -> Vec<Token<'a>> {
    let keywords = lang.keywords();
    let mut tokens: Vec<Token<'a>> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        if c == '"' || c == '\'' {
            let quote = c;
            let start = i;
            i += 1;
            while i < len && chars[i] != quote {
                if chars[i] == '\\' { i += 1; }
                i += 1;
            }
            if i < len { i += 1; }
            tokens.push(Token {
                text: Cow::Borrowed(&line[byte_pos(&chars, start)..byte_pos(&chars, i)]),
                kind: TokenKind::String,
            });
            continue;
        }

        if c.is_ascii_digit() {
            let start = i;
            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '.') {
                i += 1;
            }
            tokens.push(Token {
                text: Cow::Borrowed(&line[byte_pos(&chars, start)..byte_pos(&chars, i)]),
                kind: TokenKind::Number,
            });
            continue;
        }

        if matches!(c, '(' | ')' | '[' | ']' | '{' | '}') {
            tokens.push(Token {
                text: Cow::Borrowed(&line[byte_pos(&chars, i)..byte_pos(&chars, i + 1)]),
                kind: TokenKind::Bracket,
            });
            i += 1;
            continue;
        }

        if matches!(c, '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '&' | '|' | '^' | '%' | ':') {
            tokens.push(Token {
                text: Cow::Borrowed(&line[byte_pos(&chars, i)..byte_pos(&chars, i + 1)]),
                kind: TokenKind::Operator,
            });
            i += 1;
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word = &line[byte_pos(&chars, start)..byte_pos(&chars, i)];

            let next_non_ws = chars[i..].iter().find(|&&ch| ch != ' ' && ch != '\t');
            let kind = if keywords.contains(&word) {
                TokenKind::Keyword
            } else if next_non_ws == Some(&'(') {
                TokenKind::Function
            } else {
                TokenKind::Plain
            };

            tokens.push(Token {
                text: Cow::Borrowed(word),
                kind,
            });
            continue;
        }

        let start = i;
        while i < len
            && !chars[i].is_alphanumeric()
            && !matches!(chars[i], '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' |
                         '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '&' | '|' |
                         '^' | '%' | ':' | '_')
        {
            i += 1;
        }
        if i > start {
            tokens.push(Token {
                text: Cow::Borrowed(&line[byte_pos(&chars, start)..byte_pos(&chars, i)]),
                kind: TokenKind::Plain,
            });
        }
    }

    tokens
}

fn byte_pos(chars: &[char], char_idx: usize) -> usize {
    chars[..char_idx].iter().map(|c| c.len_utf8()).sum()
}
