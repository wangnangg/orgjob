pub struct CodeBlock {
    pub interpreter: String,
    pub code: String,
}

pub struct RunnableCode {
    pub interpreter: String,
    pub fullname: Vec<String>,
    pub code: Vec<String>,
}

pub type DocNodeId = usize;

pub struct DocNode {
    name: String,
    level: i32,
    code_blocks: Vec<CodeBlock>,
}

impl DocNode {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn level(&self) -> i32 {
        self.level
    }
}

pub struct CodeDoc {
    nodes: Vec<DocNode>,
    parent: Vec<DocNodeId>,
}

pub const DOC_NODE_ROOT_ID: DocNodeId = 0;

impl CodeDoc {
    pub fn new() -> CodeDoc {
        let mut doc = CodeDoc {
            nodes: Vec::new(),
            parent: Vec::new(),
        };
        doc.nodes.push(DocNode {
            level: -1,
            name: String::new(),
            code_blocks: Vec::new(),
        });
        doc.parent.push(DOC_NODE_ROOT_ID);
        return doc;
    }
    pub fn add(
        &mut self,
        parent: DocNodeId,
        name: String,
        level: i32,
        code_blocks: Vec<CodeBlock>,
    ) -> DocNodeId {
        assert!(parent < self.nodes.len());
        assert!(self.nodes[parent].level < level);
        let id = self.nodes.len();
        self.nodes.push(DocNode {
            name,
            level,
            code_blocks,
        });
        self.parent.push(parent);
        return id;
    }

    pub fn len(&self) -> usize {
        return self.nodes.len() - 1;
    }

    pub fn get_parent(&self, node: DocNodeId) -> Option<DocNodeId> {
        assert!(node < self.nodes.len());
        if node == DOC_NODE_ROOT_ID {
            return None;
        } else {
            return Some(self.parent[node]);
        }
    }

    pub fn get_node(&self, node: DocNodeId) -> &DocNode {
        assert!(node < self.nodes.len());
        assert!(node != DOC_NODE_ROOT_ID);
        return &self.nodes[node];
    }

    ///lookup matching nodes
    pub fn lookup_nodes(&self, start_node: DocNodeId, query: &[&str]) -> Vec<DocNodeId> {
        assert!(start_node < self.nodes.len());
        let mut result = Vec::new();

        if query.len() == 0 {
            if start_node != DOC_NODE_ROOT_ID {
                result.push(start_node);
            }
        } else {
            let key = &query[0];

            for child in 1..self.nodes.len() {
                if self.parent[child] == start_node {
                    if self.nodes[child].name.contains(key) {
                        result.extend(self.lookup_nodes(child, &query[1..]));
                        result.extend(self.lookup_nodes(child, query));
                    } else {
                        result.extend(self.lookup_nodes(child, query));
                    }
                }
            }
        }

        return result;
    }

    pub fn get_runnable_code(&self, node: DocNodeId) -> Vec<RunnableCode> {
        let mut langs = Vec::new();

        for cb in self.nodes[node].code_blocks.iter() {
            if !langs.contains(&cb.interpreter) {
                langs.push(cb.interpreter.clone());
            }
        }

        let mut result = Vec::new();

        for l in langs.iter() {
            let mut blocks = Vec::new();
            let mut current = node;
            let mut fullname = Vec::new();
            while current != DOC_NODE_ROOT_ID {
                blocks.extend(
                    self.nodes[current]
                        .code_blocks
                        .iter()
                        .rev()
                        .filter(|x| &x.interpreter == l)
                        .map(|x| x.code.clone()),
                );
                fullname.push(self.nodes[current].name.clone());
                current = self.parent[current];
            }
            fullname.reverse();
            blocks.reverse();
            result.push(RunnableCode {
                interpreter: l.to_string(),
                fullname: fullname,
                code: blocks,
            })
        }

        return result;
    }
}
