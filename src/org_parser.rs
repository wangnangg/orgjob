extern crate regex;

use super::code_doc::*;
use regex::Regex;
use std::io::BufRead;

#[derive(PartialEq, Clone, Debug)]
pub enum DocParseError {
    BlockNotClosed { linum: usize, line: String },
    UnexpectedLevel { linum: usize, line: String },
    UnexpectedLine { linum: usize, line: String },
}

pub fn parse_org_doc<T: BufRead>(f: &mut T, docname: String) -> Result<CodeDoc, DocParseError> {
    let mut doc = CodeDoc::new();
    let begin_src_re = Regex::new(r"^#\+BEGIN_SRC(?:\s+(\w+))?(?:\s+.*)?").unwrap();
    let end_src_re = Regex::new(r"^#\+END_SRC(?:\s+.*)?").unwrap();
    let hdr_re = Regex::new(r"(\*+) (.+)").unwrap();

    #[derive(PartialEq, Copy, Clone, Debug)]
    enum State {
        TEXT,
        SRC,
    };

    let mut state = State::TEXT;

    let mut parent = DOC_NODE_ROOT_ID;
    let mut current_level = 0i32;
    let mut current_hdr = docname;
    let mut code_blocks = Vec::new();

    let mut code_hdr_line = String::from("invalid");
    let mut code_lines = Vec::new();
    let mut interpreter = String::from("invalid");

    let mut linum = 0;
    for line_res in f.lines() {
        linum += 1;
        let line = line_res.unwrap();
        match state {
            State::TEXT => {
                if let Some(caps) = begin_src_re.captures(&line) {
                    interpreter = caps[1].to_string();
                    code_lines = Vec::new();
                    state = State::SRC;
                    code_hdr_line = line.to_string();
                } else if let Some(caps) = hdr_re.captures(&line) {
                    let new_level = caps[1].len() as i32;

                    parent = doc.add(parent, current_hdr, current_level, code_blocks);
                    if new_level <= current_level + 1 {
                        for _ in 0..(current_level - new_level + 1) {
                            parent = doc.get_parent(parent).unwrap();
                        }
                    } else {
                        return Err(DocParseError::UnexpectedLevel {
                            linum: linum,
                            line: line,
                        });
                    }

                    current_level = new_level;
                    current_hdr = caps[2].to_string();
                    code_blocks = Vec::new();
                } else if end_src_re.is_match(&line) {
                    return Err(DocParseError::UnexpectedLine {
                        linum: linum,
                        line: line,
                    });
                }
            }
            State::SRC => {
                if end_src_re.is_match(&line) {
                    code_blocks.push(CodeBlock {
                        interpreter: interpreter.to_string(),
                        code: code_lines.join("\n"),
                    });
                    state = State::TEXT;
                } else {
                    code_lines.push(line);
                }
            }
        }
    }

    if state == State::SRC {
        return Err(DocParseError::BlockNotClosed {
            linum: linum - code_lines.len(),
            line: code_hdr_line,
        });
    }

    doc.add(parent, current_hdr, current_level, code_blocks);
    return Ok(doc);
}
