use super::code_doc::*;
use super::*;

#[test]
fn doc_lookup_nodes1() {
    let mut doc = CodeDoc::new();
    let h1 = doc.add(DOC_NODE_ROOT_ID, "header1".to_string(), 1, Vec::new());
    let h2 = doc.add(h1, "header2".to_string(), 2, Vec::new());
    let sec = doc.add(h2, "section1".to_string(), 3, Vec::new());
    let matches = doc.lookup_nodes(DOC_NODE_ROOT_ID, &["der2", "tion"]);
    assert_eq!(matches, vec![sec]);
}

fn doc1() -> CodeDoc {
    let mut doc = CodeDoc::new();
    let h1 = doc.add(DOC_NODE_ROOT_ID, "header1".to_string(), 1, Vec::new());
    let h2 = doc.add(h1, "header2".to_string(), 2, Vec::new());
    let _h2_1 = doc.add(h1, "header2_1".to_string(), 2, Vec::new());
    let _sec1 = doc.add(h2, "section1".to_string(), 3, Vec::new());
    let _sec2 = doc.add(h2, "section1".to_string(), 3, Vec::new());
    return doc;
}

#[test]
fn doc_lookup_nodes2() {
    let doc = doc1();
    let matches = doc.lookup_nodes(DOC_NODE_ROOT_ID, &["der2", "tion"]);
    assert_eq!(matches, vec![4, 5]);
}

#[test]
fn doc_lookup_nodes3() {
    let doc = doc1();
    let matches = doc.lookup_nodes(DOC_NODE_ROOT_ID, &vec!["header"]);
    assert_eq!(matches, vec![1, 2, 3]);
}

#[test]
fn doc_lookup_nodes4() {
    let doc = doc1();
    let matches = doc.lookup_nodes(DOC_NODE_ROOT_ID, &vec![]);
    assert_eq!(matches, vec![]);
}

#[test]
fn doc_get_code1() {
    let mut doc = CodeDoc::new();
    let h1 = doc.add(
        DOC_NODE_ROOT_ID,
        "header1".to_string(),
        1,
        vec![
            CodeBlock {
                interpreter: "bash".to_string(),
                code: "h1".to_string(),
            },
            CodeBlock {
                interpreter: "bash".to_string(),
                code: "code".to_string(),
            },
        ],
    );
    let h2 = doc.add(
        h1,
        "header2".to_string(),
        2,
        vec![
            CodeBlock {
                interpreter: "python".to_string(),
                code: "h2".to_string(),
            },
            CodeBlock {
                interpreter: "bash".to_string(),
                code: "code".to_string(),
            },
        ],
    );
    let sec = doc.add(
        h2,
        "section1".to_string(),
        3,
        vec![
            CodeBlock {
                interpreter: "bash".to_string(),
                code: "sec1".to_string(),
            },
            CodeBlock {
                interpreter: "bash".to_string(),
                code: "body".to_string(),
            },
        ],
    );
    let code = doc.get_runnable_code(sec);
    assert_eq!(code.len(), 1);
    assert_eq!(code[0].code.join(""), "h1codecodesec1body");
}

#[test]
fn parse_test1() {
    let doc_str = r###"
#+TITLE: Test Doc

this is intro.
#+BEGIN_SRC bash
intro src
#+END_SRC

* header 1
h1 body
** header 2.1
h2.1 body
#+begin_src bash
bash code
#+end_src
hehe
** header 2.2
h2.2 body

"###;
    let doc = parse_org_doc(&mut doc_str.as_bytes(), "doc_root".to_string(), "bash").unwrap();
    assert_eq!(doc.len(), 4);

    let nodes = doc.lookup_nodes(DOC_NODE_ROOT_ID, &["2.1"]);
    assert_eq!(nodes.len(), 1);
    let codes = doc.get_runnable_code(nodes[0]);
    assert_eq!(codes.len(), 1);

    assert_eq!(
        codes[0].fullname,
        vec!["doc_root", "header 1", "header 2.1"]
    );

    assert_eq!(
        codes[0].code.join("\n"),
        r###"intro src
bash code"###
    );
}

#[test]
fn parse_test2() {
    let doc_str = r###"
#+TITLE: Test Doc

this is intro.
#+BEGIN_SRC
intro src
#+END_SRC

* header 1
h1 body
** header 2.1
h2.1 body
#+begin_src
shell code
#+end_src
hehe
** header 2.2
h2.2 body

"###;
    let doc = parse_org_doc(&mut doc_str.as_bytes(), "doc_root".to_string(), "bash").unwrap();
    assert_eq!(doc.len(), 4);

    let nodes = doc.lookup_nodes(DOC_NODE_ROOT_ID, &["2.1"]);
    assert_eq!(nodes.len(), 1);
    let codes = doc.get_runnable_code(nodes[0]);
    assert_eq!(codes.len(), 1);

    assert_eq!(
        codes[0].fullname,
        vec!["doc_root", "header 1", "header 2.1"]
    );

    assert_eq!(
        codes[0].code.join("\n"),
        r###"intro src
shell code"###
    );
}

#[test]
fn run_code1() {
    let code = r###"
echo hello world from run_code1
exit 42
"###;
    match run_code("bash", code).unwrap().code() {
        Some(code) => assert_eq!(code, 42),
        None => assert!(false),
    };
}
