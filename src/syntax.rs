use crossterm::style::Color;

#[derive(Clone, Debug)]
pub struct HighlightSpan {
    pub text: String,
    pub color: Color,
    pub is_bold: bool,
}

#[derive(Clone, Debug)]
pub struct HighlightedLine {
    pub spans: Vec<HighlightSpan>,
}

const COLOR_KEYWORD: Color = Color::Rgb { r: 255, g: 120, b: 150 };
const COLOR_TYPE: Color = Color::Rgb { r: 120, g: 210, b: 255 };
const COLOR_FUNCTION: Color = Color::Rgb { r: 140, g: 230, b: 180 };
const COLOR_STRING: Color = Color::Rgb { r: 240, g: 210, b: 130 };
const COLOR_NUMBER: Color = Color::Rgb { r: 210, g: 160, b: 255 };
const COLOR_COMMENT: Color = Color::Rgb { r: 110, g: 115, b: 135 };
const COLOR_HEADING: Color = Color::Rgb { r: 120, g: 200, b: 255 };
const COLOR_DEFAULT: Color = Color::Rgb { r: 220, g: 220, b: 230 };

pub fn highlight_line(line: &str, ext: &str) -> HighlightedLine {
    if line.trim().is_empty() {
        return HighlightedLine {
            spans: vec![HighlightSpan {
                text: line.to_string(),
                color: COLOR_DEFAULT,
                is_bold: false,
            }],
        };
    }

    match ext {
        "rs" | "rust" => highlight_rust(line),
        "py" | "python" => highlight_python(line),
        "js" | "jsx" | "ts" | "tsx" => highlight_js(line),
        "json" | "toml" | "yaml" | "yml" => highlight_config(line),
        "md" | "markdown" => highlight_markdown(line),
        "sh" | "bash" | "zsh" | "ps1" | "bat" | "cmd" => highlight_shell(line),
        "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" | "go" | "java" | "cs" => highlight_c_like(line),
        _ => HighlightedLine {
            spans: vec![HighlightSpan {
                text: line.to_string(),
                color: COLOR_DEFAULT,
                is_bold: false,
            }],
        },
    }
}

fn highlight_rust(line: &str) -> HighlightedLine {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return single_span(line, COLOR_COMMENT, false);
    }
    highlight_generic_code(
        line,
        &[
            "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "type", "use",
            "mod", "const", "static", "if", "else", "match", "for", "while", "loop", "return",
            "break", "continue", "async", "await", "where", "move", "unsafe", "ref", "in",
            "as", "dyn", "true", "false", "Some", "None", "Ok", "Err",
        ],
        &[
            "String", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32",
            "i64", "i128", "isize", "f32", "f64", "bool", "char", "Vec", "Option", "Result",
            "Box", "Rc", "Arc", "Self", "self",
        ],
        "//",
    )
}

fn highlight_python(line: &str) -> HighlightedLine {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return single_span(line, COLOR_COMMENT, false);
    }
    highlight_generic_code(
        line,
        &[
            "def", "class", "import", "from", "as", "return", "if", "elif", "else", "for",
            "while", "try", "except", "finally", "with", "yield", "lambda", "global", "nonlocal",
            "pass", "break", "continue", "raise", "async", "await", "in", "is", "not", "and", "or",
            "True", "False", "None", "self",
        ],
        &[
            "int", "float", "str", "list", "dict", "set", "tuple", "bool", "bytes", "object",
            "Exception", "type",
        ],
        "#",
    )
}

fn highlight_js(line: &str) -> HighlightedLine {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return single_span(line, COLOR_COMMENT, false);
    }
    highlight_generic_code(
        line,
        &[
            "function", "const", "let", "var", "class", "interface", "type", "import", "export",
            "from", "default", "return", "if", "else", "for", "while", "do", "switch", "case",
            "break", "continue", "try", "catch", "finally", "throw", "new", "this", "super",
            "extends", "implements", "async", "await", "yield", "typeof", "instanceof", "in", "of",
            "true", "false", "null", "undefined", "NaN",
        ],
        &[
            "string", "number", "boolean", "any", "void", "never", "unknown", "Promise",
            "Array", "Object", "Map", "Set", "Record",
        ],
        "//",
    )
}

fn highlight_c_like(line: &str) -> HighlightedLine {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return single_span(line, COLOR_COMMENT, false);
    }
    highlight_generic_code(
        line,
        &[
            "if", "else", "for", "while", "do", "switch", "case", "break", "continue", "return",
            "struct", "class", "enum", "typedef", "public", "private", "protected", "static",
            "const", "constexpr", "virtual", "override", "import", "package", "func", "go",
            "chan", "defer", "true", "false", "nullptr", "nil", "NULL",
        ],
        &[
            "int", "long", "short", "char", "float", "double", "void", "bool", "size_t",
            "uint8_t", "uint16_t", "uint32_t", "uint64_t", "int8_t", "int16_t", "int32_t",
            "int64_t", "string", "auto",
        ],
        "//",
    )
}

fn highlight_config(line: &str) -> HighlightedLine {
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return single_span(line, COLOR_COMMENT, false);
    }
    if (trimmed.starts_with('[') && trimmed.ends_with(']')) || (trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return single_span(line, COLOR_HEADING, true);
    }

    if let Some((key, val)) = line.split_once(':').or_else(|| line.split_once('=')) {
        let mut spans = Vec::new();
        spans.push(HighlightSpan {
            text: key.to_string(),
            color: COLOR_TYPE,
            is_bold: true,
        });
        spans.push(HighlightSpan {
            text: if line.contains(':') { ":".to_string() } else { " = ".to_string() },
            color: COLOR_DEFAULT,
            is_bold: false,
        });
        spans.push(HighlightSpan {
            text: val.to_string(),
            color: if val.trim().starts_with('"') || val.trim().starts_with('\'') {
                COLOR_STRING
            } else if val.trim().parse::<f64>().is_ok() {
                COLOR_NUMBER
            } else {
                COLOR_KEYWORD
            },
            is_bold: false,
        });
        return HighlightedLine { spans };
    }

    single_span(line, COLOR_DEFAULT, false)
}

fn highlight_markdown(line: &str) -> HighlightedLine {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return single_span(line, COLOR_HEADING, true);
    }
    if trimmed.starts_with("```") || trimmed.starts_with("---") {
        return single_span(line, COLOR_COMMENT, false);
    }
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("1. ") {
        let mut spans = Vec::new();
        let bullet_len = line.len() - trimmed.len() + 2;
        spans.push(HighlightSpan {
            text: line[..bullet_len].to_string(),
            color: COLOR_KEYWORD,
            is_bold: true,
        });
        spans.push(HighlightSpan {
            text: line[bullet_len..].to_string(),
            color: COLOR_DEFAULT,
            is_bold: false,
        });
        return HighlightedLine { spans };
    }
    single_span(line, COLOR_DEFAULT, false)
}

fn highlight_shell(line: &str) -> HighlightedLine {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return single_span(line, COLOR_COMMENT, false);
    }
    single_span(line, COLOR_DEFAULT, false)
}

fn single_span(text: &str, color: Color, is_bold: bool) -> HighlightedLine {
    HighlightedLine {
        spans: vec![HighlightSpan {
            text: text.to_string(),
            color,
            is_bold,
        }],
    }
}

fn highlight_generic_code(
    line: &str,
    keywords: &[&str],
    types: &[&str],
    comment_prefix: &str,
) -> HighlightedLine {
    let mut spans = Vec::new();
    let mut current_token = String::new();
    let mut in_string = false;
    let mut string_char = ' ';

    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Check for comment start
        if !in_string && line[i..].starts_with(comment_prefix) {
            if !current_token.is_empty() {
                flush_token(&mut spans, &current_token, keywords, types);
                current_token.clear();
            }
            spans.push(HighlightSpan {
                text: line[i..].to_string(),
                color: COLOR_COMMENT,
                is_bold: false,
            });
            return HighlightedLine { spans };
        }

        // Handle string literals
        if !in_string && (ch == '"' || ch == '\'') {
            if !current_token.is_empty() {
                flush_token(&mut spans, &current_token, keywords, types);
                current_token.clear();
            }
            in_string = true;
            string_char = ch;
            current_token.push(ch);
            i += 1;
            continue;
        } else if in_string {
            current_token.push(ch);
            if ch == string_char && (i == 0 || chars[i - 1] != '\\') {
                in_string = false;
                spans.push(HighlightSpan {
                    text: current_token.clone(),
                    color: COLOR_STRING,
                    is_bold: false,
                });
                current_token.clear();
            }
            i += 1;
            continue;
        }

        // Word separators
        if !ch.is_alphanumeric() && ch != '_' {
            if !current_token.is_empty() {
                flush_token(&mut spans, &current_token, keywords, types);
                current_token.clear();
            }
            spans.push(HighlightSpan {
                text: ch.to_string(),
                color: COLOR_DEFAULT,
                is_bold: false,
            });
        } else {
            current_token.push(ch);
        }

        i += 1;
    }

    if !current_token.is_empty() {
        if in_string {
            spans.push(HighlightSpan {
                text: current_token,
                color: COLOR_STRING,
                is_bold: false,
            });
        } else {
            flush_token(&mut spans, &current_token, keywords, types);
        }
    }

    HighlightedLine { spans }
}

fn flush_token(spans: &mut Vec<HighlightSpan>, token: &str, keywords: &[&str], types: &[&str]) {
    if keywords.contains(&token) {
        spans.push(HighlightSpan {
            text: token.to_string(),
            color: COLOR_KEYWORD,
            is_bold: true,
        });
    } else if types.contains(&token) {
        spans.push(HighlightSpan {
            text: token.to_string(),
            color: COLOR_TYPE,
            is_bold: false,
        });
    } else if token.parse::<f64>().is_ok() {
        spans.push(HighlightSpan {
            text: token.to_string(),
            color: COLOR_NUMBER,
            is_bold: false,
        });
    } else {
        spans.push(HighlightSpan {
            text: token.to_string(),
            color: COLOR_DEFAULT,
            is_bold: false,
        });
    }
}
