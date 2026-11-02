use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

pub trait Compiler {
    fn compile(&mut self, source: &str);
    fn next_token(&mut self) -> String;
    fn parse(&mut self);
    fn current_token(&self) -> String;
    fn set_current_token(&mut self, tok: String);
}

pub trait LexicalAnalyzer {
    fn get_char(&mut self) -> char;
    fn add_char(&mut self, c: char);
    fn lookup(&self, s: &str) -> bool;
}

pub trait SyntaxAnalyzer {
    fn parse_lolcode(&mut self);
    fn parse_head(&mut self);
    fn parse_title(&mut self);
    fn parse_comment(&mut self);
    fn parse_body(&mut self);
    fn parse_paragraph(&mut self);
    fn parse_inner_paragraph(&mut self);
    fn parse_inner_text(&mut self);
    fn parse_variable_define(&mut self);
    fn parse_variable_use(&mut self);
    fn parse_bold(&mut self);
    fn parse_list(&mut self);
    fn parse_list_items(&mut self);
    fn parse_inner_list(&mut self);
    fn parse_link(&mut self);
    fn parse_newline(&mut self);
    fn parse_text(&mut self);
}

struct LolCompiler {
    source: String,
    cursor: usize,
    token_buffer: String,
    tokens: Vec<String>,
    token_lines: Vec<usize>,
    index: usize,
    current: String,
    line: usize,
    output: String,
    scopes: Vec<HashMap<String, String>>,
}

impl LolCompiler {
    fn new() -> Self {
        Self {
            source: String::new(),
            cursor: 0,
            token_buffer: String::new(),
            tokens: Vec::new(),
            token_lines: Vec::new(),
            index: 0,
            current: String::new(),
            line: 1,
            output: String::new(),
            scopes: Vec::new(),
        }
    }

    fn fail<T>(&self, message: impl Into<String>) -> Result<T, String> {
        Err(format!("line {}: {}", self.line, message.into()))
    }

    fn is_tag(token: &str) -> bool {
        token.starts_with('#')
    }

    fn peek(&self) -> &str {
        self.tokens
            .get(self.index)
            .map(String::as_str)
            .unwrap_or("")
    }

    fn advance(&mut self) {
        if self.index < self.tokens.len() {
            self.current = self.tokens[self.index].clone();
            self.line = self.token_lines[self.index];
            self.index += 1;
        } else {
            self.current.clear();
        }
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        if self.current.eq_ignore_ascii_case(expected) {
            self.advance();
            Ok(())
        } else {
            self.fail(format!("expected {}, found '{}'", expected, self.current))
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_variable_result(&mut self) -> Result<(), String> {
        self.expect("#IHAZ")?;
        let name = self.current.trim().to_string();
        if name.is_empty() || Self::is_tag(&name) || name.contains(char::is_whitespace) {
            return self.fail("expected one-word variable name");
        }
        self.advance();
        self.expect("#ITIZ")?;
        let value = self.current.trim().to_string();
        if value.is_empty() || Self::is_tag(&value) {
            return self.fail("expected variable value");
        }
        self.advance();
        self.expect("#MKAY")?;
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_ascii_lowercase(), value);
        }
        Ok(())
    }

    fn variable_value(&self, name: &str) -> Result<String, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        self.fail(format!(
            "static semantic error: variable '{}' is undefined",
            name
        ))
    }

    fn variable_use_result(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("VAR")?;
        let name = self.current.trim().to_ascii_lowercase();
        if name.is_empty() || Self::is_tag(&name) || name.contains(char::is_whitespace) {
            return self.fail("expected one-word variable name");
        }
        self.advance();
        self.expect("#OIC")?;
        let value = self.variable_value(&name)?;
        self.output.push_str(&value);
        Ok(())
    }

    fn comment_result(&mut self) -> Result<(), String> {
        self.expect("#OBTW")?;
        self.output.push_str("<!--");
        while !self.current.eq_ignore_ascii_case("#TLDR") {
            if self.current.is_empty() {
                return self.fail("expected #TLDR, found end of file");
            }
            if Self::is_tag(&self.current) {
                return self.fail("comments may contain plain text only");
            }
            self.output.push_str(&self.current);
            self.advance();
        }
        self.expect("#TLDR")?;
        self.output.push_str("-->");
        Ok(())
    }

    fn title_result(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("TITLE")?;
        self.output.push_str("<title>");
        if self.current.is_empty() || Self::is_tag(&self.current) {
            return self.fail("expected title text");
        }
        self.output.push_str(&self.current);
        self.advance();
        self.expect("#OIC")?;
        self.output.push_str("</title>");
        Ok(())
    }

    fn head_result(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("HEAD")?;
        self.output.push_str("<head>");
        if self.current.eq_ignore_ascii_case("#GIMMEH") {
            self.title_result()?;
        }
        self.expect("#MKAY")?;
        self.output.push_str("</head>");
        Ok(())
    }

    fn newline_result(&mut self) -> Result<(), String> {
        self.expect("#NEWLINE")?;
        self.output.push_str("<br>");
        Ok(())
    }

    fn link_result(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("LINX")?;
        let address = self.current.trim().to_string();
        if address.is_empty() || Self::is_tag(&address) || address.contains(char::is_whitespace) {
            return self.fail("expected an address without spaces");
        }
        self.advance();
        self.expect("#OIC")?;
        self.output
            .push_str(&format!("<a href=\"{address}\">{address}</a>"));
        Ok(())
    }

    fn bold_result(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("BOLD")?;
        self.output.push_str("<b>");
        self.inline_result("#OIC", true)?;
        self.expect("#OIC")?;
        self.output.push_str("</b>");
        Ok(())
    }

    fn inline_result(&mut self, end: &str, allow_list: bool) -> Result<(), String> {
        while !self.current.eq_ignore_ascii_case(end) {
            if self.current.is_empty() {
                return self.fail(format!("expected {}, found end of file", end));
            }
            match self.current.to_ascii_uppercase().as_str() {
                "#GIMMEH" if self.peek().eq_ignore_ascii_case("BOLD") => {
                    self.bold_result()?;
                }
                "#GIMMEH" if self.peek().eq_ignore_ascii_case("VAR") => {
                    self.variable_use_result()?;
                }
                "#GIMMEH" if self.peek().eq_ignore_ascii_case("LINX") => {
                    self.link_result()?;
                }
                "#OBTW" => {
                    self.comment_result()?;
                }
                "#NEWLINE" => {
                    self.newline_result()?;
                }
                "#MAEK" if allow_list && self.peek().eq_ignore_ascii_case("LIST") => {
                    self.list_result()?;
                }
                "#MAEK" => return self.fail("paragraphs cannot be nested"),
                _ if Self::is_tag(&self.current) => {
                    return self.fail(format!("unexpected annotation '{}'", self.current));
                }
                _ => {
                    let text = self.current.clone();
                    self.output.push_str(&text);
                    self.advance();
                }
            };
        }
        Ok(())
    }

    fn item_result(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH")?;
        self.expect("ITEM")?;
        self.output.push_str("<li>");
        self.push_scope();
        if self.current.eq_ignore_ascii_case("#IHAZ") {
            self.define_variable_result()?;
        }
        self.inline_result("#OIC", false)?;
        self.expect("#OIC")?;
        self.pop_scope();
        self.output.push_str("</li>");
        Ok(())
    }

    fn list_result(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("LIST")?;
        self.output.push_str("<ul>");
        self.push_scope();
        if !(self.current.eq_ignore_ascii_case("#GIMMEH")
            && self.peek().eq_ignore_ascii_case("ITEM"))
        {
            return self.fail("a list must contain at least one item");
        }
        while self.current.eq_ignore_ascii_case("#GIMMEH")
            && self.peek().eq_ignore_ascii_case("ITEM")
        {
            self.item_result()?;
        }
        self.expect("#MKAY")?;
        self.pop_scope();
        self.output.push_str("</ul>");
        Ok(())
    }

    fn paragraph_result(&mut self) -> Result<(), String> {
        self.expect("#MAEK")?;
        self.expect("PARAGRAF")?;
        self.output.push_str("<p>");
        self.push_scope();
        if self.current.eq_ignore_ascii_case("#IHAZ") {
            self.define_variable_result()?;
        }
        self.inline_result("#MKAY", true)?;
        self.expect("#MKAY")?;
        self.pop_scope();
        self.output.push_str("</p>");
        Ok(())
    }

    fn body_result(&mut self) -> Result<(), String> {
        while !self.current.eq_ignore_ascii_case("#KBYE") {
            if self.current.is_empty() {
                return self.fail("expected #KBYE, found end of file");
            }
            match self.current.to_ascii_uppercase().as_str() {
                "#OBTW" => self.comment_result()?,
                "#MAEK" if self.peek().eq_ignore_ascii_case("PARAGRAF") => {
                    self.paragraph_result()?
                }
                "#MAEK" if self.peek().eq_ignore_ascii_case("LIST") => self.list_result()?,
                "#GIMMEH" if self.peek().eq_ignore_ascii_case("BOLD") => self.bold_result()?,
                "#GIMMEH" if self.peek().eq_ignore_ascii_case("LINX") => self.link_result()?,
                "#GIMMEH" if self.peek().eq_ignore_ascii_case("VAR") => {
                    self.variable_use_result()?
                }
                "#NEWLINE" => self.newline_result()?,
                "#HAI" => return self.fail("unexpected #HAI"),
                _ if Self::is_tag(&self.current) => {
                    return self.fail(format!("unexpected annotation '{}'", self.current));
                }
                _ => {
                    let text = self.current.clone();
                    self.output.push_str(&text);
                    self.advance();
                }
            }
        }
        Ok(())
    }

    fn parse_result(&mut self) -> Result<(), String> {
        self.push_scope();
        self.expect("#HAI")?;
        self.output.push_str("<html>");
        while self.current.eq_ignore_ascii_case("#OBTW") {
            self.comment_result()?;
        }
        if self.current.eq_ignore_ascii_case("#MAEK") && self.peek().eq_ignore_ascii_case("HEAD") {
            self.head_result()?;
        }
        if self.current.eq_ignore_ascii_case("#IHAZ") {
            self.define_variable_result()?;
        }
        self.body_result()?;
        self.expect("#KBYE")?;
        if !self.current.is_empty() {
            return self.fail(format!(
                "unexpected content after #KBYE: '{}'",
                self.current
            ));
        }
        self.output.push_str("</html>");
        self.pop_scope();
        Ok(())
    }

    fn lex_result(&mut self) -> Result<(), String> {
        self.tokens.clear();
        self.token_lines.clear();
        self.cursor = 0;
        self.line = 1;
        let chars: Vec<char> = self.source.chars().collect();
        let mut text = String::new();
        let mut text_line = 1;

        while self.cursor < chars.len() {
            let ch = chars[self.cursor];
            if ch != '#' {
                if text.is_empty() {
                    text_line = self.line;
                }
                text.push(ch);
                if ch == '\n' {
                    self.line += 1;
                }
                self.cursor += 1;
                continue;
            }

            if !text.trim().is_empty() {
                self.tokens.push(std::mem::take(&mut text));
                self.token_lines.push(text_line);
            } else {
                text.clear();
            }

            let tag_line = self.line;
            let start = self.cursor;
            self.cursor += 1;
            while self.cursor < chars.len()
                && !chars[self.cursor].is_whitespace()
                && chars[self.cursor] != '#'
            {
                self.cursor += 1;
            }
            let tag = chars[start..self.cursor]
                .iter()
                .collect::<String>()
                .to_ascii_uppercase();
            if !self.lookup(&tag) {
                return self.fail(format!("lexical error: unknown annotation '{}'", tag));
            }
            self.tokens.push(tag.clone());
            self.token_lines.push(tag_line);

            if tag == "#MAEK" || tag == "#GIMMEH" {
                while self.cursor < chars.len() && chars[self.cursor].is_whitespace() {
                    if chars[self.cursor] == '\n' {
                        self.line += 1;
                    }
                    self.cursor += 1;
                }
                let keyword_start = self.cursor;
                while self.cursor < chars.len()
                    && !chars[self.cursor].is_whitespace()
                    && chars[self.cursor] != '#'
                {
                    self.cursor += 1;
                }
                if keyword_start == self.cursor {
                    return self.fail(format!("lexical error: missing keyword after {}", tag));
                }
                let keyword = chars[keyword_start..self.cursor]
                    .iter()
                    .collect::<String>()
                    .to_ascii_uppercase();
                let valid = if tag == "#MAEK" {
                    ["HEAD", "PARAGRAF", "LIST"].contains(&keyword.as_str())
                } else {
                    ["TITLE", "BOLD", "ITEM", "LINX", "VAR"].contains(&keyword.as_str())
                };
                if !valid {
                    return self.fail(format!(
                        "lexical error: invalid keyword '{}' after {}",
                        keyword, tag
                    ));
                }
                self.tokens.push(keyword);
                self.token_lines.push(tag_line);
            }
        }
        if !text.trim().is_empty() {
            self.tokens.push(text);
            self.token_lines.push(text_line);
        }
        Ok(())
    }

    fn compile_result(&mut self, source: &str) -> Result<(), String> {
        self.source = source.to_string();
        self.output.clear();
        self.index = 0;
        self.current.clear();
        self.scopes.clear();
        self.lex_result()?;
        self.advance();
        self.parse_result()
    }
}

impl Compiler for LolCompiler {
    fn compile(&mut self, source: &str) {
        if let Err(error) = self.compile_result(source) {
            eprintln!("Compilation error: {error}");
            std::process::exit(1);
        }
    }

    fn next_token(&mut self) -> String {
        self.advance();
        self.current.clone()
    }

    fn parse(&mut self) {
        if let Err(error) = self.parse_result() {
            eprintln!("Syntax error: {error}");
            std::process::exit(1);
        }
    }

    fn current_token(&self) -> String {
        self.current.clone()
    }

    fn set_current_token(&mut self, tok: String) {
        self.current = tok;
    }
}

impl LexicalAnalyzer for LolCompiler {
    fn get_char(&mut self) -> char {
        let ch = self.source.chars().nth(self.cursor).unwrap_or('\0');
        self.cursor += 1;
        ch
    }

    fn add_char(&mut self, c: char) {
        self.token_buffer.push(c);
    }

    fn lookup(&self, s: &str) -> bool {
        [
            "#HAI", "#KBYE", "#OBTW", "#TLDR", "#MAEK", "#MKAY", "#OIC", "#GIMMEH", "#IHAZ",
            "#ITIZ", "#NEWLINE",
        ]
        .contains(&s.to_ascii_uppercase().as_str())
    }
}

impl SyntaxAnalyzer for LolCompiler {
    fn parse_lolcode(&mut self) {
        let _ = self.parse_result();
    }
    fn parse_head(&mut self) {
        let _ = self.head_result();
    }
    fn parse_title(&mut self) {
        let _ = self.title_result();
    }
    fn parse_comment(&mut self) {
        let _ = self.comment_result();
    }
    fn parse_body(&mut self) {
        let _ = self.body_result();
    }
    fn parse_paragraph(&mut self) {
        let _ = self.paragraph_result();
    }
    fn parse_inner_paragraph(&mut self) {
        let _ = self.inline_result("#MKAY", true);
    }
    fn parse_inner_text(&mut self) {
        let _ = self.inline_result("#MKAY", true);
    }
    fn parse_variable_define(&mut self) {
        let _ = self.define_variable_result();
    }
    fn parse_variable_use(&mut self) {
        let _ = self.variable_use_result();
    }
    fn parse_bold(&mut self) {
        let _ = self.bold_result();
    }
    fn parse_list(&mut self) {
        let _ = self.list_result();
    }
    fn parse_list_items(&mut self) {
        let _ = self.item_result();
    }
    fn parse_inner_list(&mut self) {
        let _ = self.inline_result("#OIC", false);
    }
    fn parse_link(&mut self) {
        let _ = self.link_result();
    }
    fn parse_newline(&mut self) {
        let _ = self.newline_result();
    }
    fn parse_text(&mut self) {
        self.output.push_str(&self.current);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: lolcompiler input.lol");
        return;
    }
    let input = Path::new(&args[1]);
    if input.extension().and_then(|ext| ext.to_str()) != Some("lol") {
        eprintln!("Usage error: input file must end in .lol");
        return;
    }
    let source = match fs::read_to_string(input) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Could not read '{}': {error}", input.display());
            return;
        }
    };
    let mut compiler = LolCompiler::new();
    compiler.compile(&source);
    let output_path = input.with_extension("html");
    if let Err(error) = fs::write(&output_path, &compiler.output) {
        eprintln!("Could not write '{}': {error}", output_path.display());
        return;
    }
    let _ = Command::new("open").arg(output_path).status();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_for_test(source: &str) -> Result<String, String> {
        let mut compiler = LolCompiler::new();
        compiler.compile_result(source)?;
        Ok(compiler.output)
    }

    #[test]
    fn accepts_case_insensitive_annotations_and_two_word_tags() {
        let output = compile_for_test("#hai\n#maek paragraf\nHello#mkay\n#kbye").unwrap();
        assert_eq!(output, "<html><p>\nHello</p></html>");
    }

    #[test]
    fn rejects_unknown_annotations() {
        let error = compile_for_test("#HAI #HEY #KBYE").unwrap_err();
        assert!(error.contains("unknown annotation '#HEY'"));
    }

    #[test]
    fn translates_design_features() {
        let source = r#"#HAI
#OBTW A comment #TLDR
#MAEK HEAD
#GIMMEH TITLE Home #OIC
#MKAY
#MAEK PARAGRAF
Hello #GIMMEH BOLD world #OIC#NEWLINE
#GIMMEH LINX example.com #OIC
#MAEK LIST
#GIMMEH ITEM One #OIC
#GIMMEH ITEM Two #OIC
#MKAY
#MKAY
#KBYE"#;
        let html = compile_for_test(source).unwrap();
        assert!(html.starts_with("<html><!-- A comment -->"));
        assert!(html.contains("<head><title> Home </title></head>"));
        assert!(html.contains("<b> world </b><br>"));
        assert!(html.contains("<a href=\"example.com\">example.com</a>"));
        assert!(html.contains("<ul><li> One </li><li> Two </li></ul>"));
        assert!(html.ends_with("</p></html>"));
    }

    #[test]
    fn resolves_shadowing_and_rejects_undefined_variables() {
        let source = r#"#HAI
#IHAZ name #ITIZ outer #MKAY
Before: #GIMMEH VAR name #OIC
#MAEK PARAGRAF
#IHAZ name #ITIZ inner #MKAY
Inside: #GIMMEH VAR name #OIC
#MKAY
After: #GIMMEH VAR name #OIC
#KBYE"#;
        let html = compile_for_test(source).unwrap();
        assert_eq!(html.matches("outer").count(), 2);
        assert_eq!(html.matches("inner").count(), 1);
        let error = compile_for_test("#HAI\n#GIMMEH VAR missing #OIC\n#KBYE").unwrap_err();
        assert!(error.contains("variable 'missing' is undefined"));
    }

    #[test]
    fn rejects_invalid_structure() {
        let nested =
            compile_for_test("#HAI\n#MAEK PARAGRAF\n#MAEK PARAGRAF\ntext\n#MKAY\n#MKAY\n#KBYE")
                .unwrap_err();
        assert!(nested.contains("paragraphs cannot be nested"));
        let empty_list = compile_for_test("#HAI\n#MAEK LIST\n#MKAY\n#KBYE").unwrap_err();
        assert!(empty_list.contains("must contain at least one item"));
    }
}
