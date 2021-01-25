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

    fn is_anscestor(&self, anscestor: DocNodeId, mut child: DocNodeId) -> bool {
        assert!(anscestor < self.nodes.len());
        assert!(anscestor != DOC_NODE_ROOT_ID);
        assert!(child < self.nodes.len());
        assert!(child != DOC_NODE_ROOT_ID);

        while child != DOC_NODE_ROOT_ID && child != anscestor {
            child = self.parent[child];
        }
        return child == anscestor;
    }

    fn get_ancestors(&self, node: DocNodeId) -> Vec<DocNodeId> {
        assert!(node < self.nodes.len());
        assert!(node != DOC_NODE_ROOT_ID);
        let mut res = Vec::new();
        for ances in (DOC_NODE_ROOT_ID + 1)..node {
            if self.is_anscestor(ances, node) {
                res.push(ances);
            }
        }
        return res;
    }

    fn get_descendants(&self, node: DocNodeId) -> Vec<DocNodeId> {
        assert!(node < self.nodes.len());
        assert!(node != DOC_NODE_ROOT_ID);
        let mut res = Vec::new();
        for child in (node + 1)..self.nodes.len() {
            if self.is_anscestor(node, child) {
                res.push(child);
            }
        }
        return res;
    }

    pub fn get_fullname(&self, node: DocNodeId) -> Vec<String> {
        let mut fullname = Vec::new();
        for n in self.get_ancestors(node) {
            fullname.push(self.get_node(n).name.clone());
        }
        fullname.push(self.get_node(node).name.clone());
        return fullname;
    }

    pub fn get_runnable_code(&self, node: DocNodeId) -> Vec<RunnableCode> {
        let mut nodes = Vec::new();
        nodes.extend(self.get_ancestors(node));
        nodes.push(node);
        nodes.extend(self.get_descendants(node));

        let mut langs = Vec::new();
        for n in &nodes {
            for cb in self.get_node(*n).code_blocks.iter() {
                if !langs.contains(&cb.interpreter) {
                    langs.push(cb.interpreter.clone());
                }
            }
        }

        let mut result = Vec::new();

        for l in langs.iter() {
            let mut blocks = Vec::new();
            for n in &nodes {
                blocks.extend(
                    self.get_node(*n)
                        .code_blocks
                        .iter()
                        .filter(|x| &x.interpreter == l)
                        .map(|x| x.code.clone()),
                )
            }
            result.push(RunnableCode {
                interpreter: l.to_string(),
                fullname: self.get_fullname(node),
                code: blocks,
            })
        }

        return result;
    }
}
