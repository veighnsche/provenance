use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Block {
    Heading { level: u8, text: String },
    Component(Component),
    Paragraph(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub children: Vec<Block>,
    pub self_closing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attr {
    pub key: String,
    pub value: String,
}

pub fn parse(input: &str) -> Result<Document> {
    // Extremely small parser that can handle the example front_page.pml
    let mut blocks = Vec::new();
    let mut i = 0;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        // skip whitespace/newlines
        while i < bytes.len() && (bytes[i] == b'\n' || bytes[i] == b'\r') { i += 1; }
        if i >= bytes.len() { break; }

        if bytes[i] == b'#' {
            // heading: count #
            let mut level = 0u8; let mut j = i;
            while j < bytes.len() && bytes[j] == b'#' && level < 4 { level += 1; j += 1; }
            if j < bytes.len() && bytes[j] == b' ' {
                j += 1;
            }
            let start = j;
            while j < bytes.len() && bytes[j] != b'\n' { j += 1; }
            let text = input[start..j].trim().to_string();
            blocks.push(Block::Heading { level, text });
            i = j + 1;
            continue;
        }

        if bytes[i] == b'<' {
            // component (open or self-close)
            let (comp, consumed) = parse_component(&input[i..])?;
            blocks.push(Block::Component(comp));
            i += consumed;
            continue;
        }

        // paragraph until blank line or tag
        let start = i;
        while i < bytes.len() && bytes[i] != b'\n' && bytes[i] != b'<' { i += 1; }
        let text = input[start..i].trim().to_string();
        if !text.is_empty() { blocks.push(Block::Paragraph(text)); }
        // consume newline
        if i < bytes.len() && bytes[i] == b'\n' { i += 1; }
    }

    Ok(Document { blocks })
}

fn parse_component(src: &str) -> Result<(Component, usize)> {
    // Expect something like <name ...> ... </name> or <name ... />
    if !src.starts_with('<') {
        return Err(anyhow!("expected component"));
    }
    // Find end of tag '>'
    let close = src.find('>').ok_or_else(|| anyhow!("unterminated tag"))?;
    let tag_inner = &src[1..close];
    let self_close = tag_inner.trim_end().ends_with("/");
    let tag_inner = tag_inner.trim_end_matches('/').trim();

    // Split name and attrs
    let mut parts = tag_inner.split_whitespace();
    let name = parts.next().ok_or_else(|| anyhow!("missing component name"))?.to_string();
    let rest = &tag_inner[name.len()..].trim();
    let attrs = parse_attrs(rest)?;

    let mut consumed = close + 1; // include '>'

    let mut children = Vec::new();
    if !self_close {
        // parse until closing tag </name>
        let close_tag = format!("</{}>", name);
        let mut remain = &src[consumed..];
        while !remain.starts_with(&close_tag) {
            if remain.is_empty() {
                return Err(anyhow!("unterminated component <{}>", name));
            }
            // skip newlines
            if remain.starts_with('\n') || remain.starts_with('\r') { consumed += 1; remain = &src[consumed..]; continue; }
            if remain.starts_with('<') {
                let (child, c) = parse_component(remain)?;
                children.push(Block::Component(child));
                consumed += c;
                remain = &src[consumed..];
            } else if remain.starts_with('#') {
                // heading inside
                // find end of line
                let end = remain.find('\n').unwrap_or(remain.len());
                let line = &remain[..end];
                let mut level = 0u8; let mut idx = 0;
                for b in line.as_bytes() { if *b == b'#' && level < 4 { level += 1; idx += 1; } else { break; } }
                let text = line[idx..].trim().to_string();
                children.push(Block::Heading { level, text });
                consumed += end + 1; remain = &src[consumed..];
            } else {
                // paragraph/text until next tag or newline
                let mut end = remain.find('<').unwrap_or(remain.len());
                if let Some(nl) = remain.find('\n') { end = end.min(nl); }
                let text = remain[..end].trim().to_string();
                if !text.is_empty() { children.push(Block::Paragraph(text)); }
                consumed += end;
                remain = &src[consumed..];
            }
        }
        consumed += close_tag.len();
    }

    Ok((Component { name, attrs, children, self_closing: self_close }, consumed))
}

fn parse_attrs(src: &str) -> Result<Vec<Attr>> {
    let mut attrs = Vec::new();
    let mut s = src.trim();
    while !s.is_empty() {
        // key=
        let eq = match s.find('=') { Some(i) => i, None => break };
        let key = s[..eq].trim().trim_matches(|c: char| c.is_whitespace()).to_string();
        s = &s[eq+1..];
        if s.starts_with('"') {
            // quoted
            s = &s[1..];
            let end = s.find('"').ok_or_else(|| anyhow!("unterminated quoted attr"))?;
            let val = s[..end].to_string();
            attrs.push(Attr { key, value: val });
            s = &s[end+1..].trim_start();
        } else {
            // bare until whitespace
            let mut end = s.find(' ').unwrap_or(s.len());
            // if ">" or "/>" begins, stop
            if let Some(gt) = s.find('>') { end = end.min(gt); }
            let val = s[..end].trim().trim_end_matches('/').to_string();
            attrs.push(Attr { key, value: val });
            s = &s[end..].trim_start();
        }
    }
    Ok(attrs)
}

pub fn find_attr<'a>(attrs: &'a [Attr], key: &str) -> Option<&'a str> {
    attrs.iter().find(|a| a.key == key).map(|a| a.value.as_str())
}
