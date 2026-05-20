use tree_sitter::{Node, Parser};

#[derive(Debug, Clone)]
pub struct AstContext {
    pub kind: String,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub fn get_ast_context(source: &str, line: usize, file_ext: &str) -> Option<AstContext> {
    let language = match file_ext {
        "rs" => tree_sitter_rust::language(),
        "py" => tree_sitter_python::language(),
        "js" | "ts" | "jsx" | "tsx" => tree_sitter_javascript::language(),
        _ => return None,
    };

    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;

    let tree = parser.parse(source, None)?;
    let root = tree.root_node();

    let target_line = line.saturating_sub(1);
    find_enclosing_node(root, source, target_line)
}

fn find_enclosing_node(node: Node, source: &str, target_line: usize) -> Option<AstContext> {
    let start = node.start_position().row;
    let end = node.end_position().row;

    if target_line < start || target_line > end {
        return None;
    }

    let is_relevant = matches!(
        node.kind(),
        "function_item"
        | "struct_item"
        | "impl_item"
        | "enum_item"
        | "trait_item"
        | "function_definition"
        | "class_definition"
        | "function_declaration"
        | "method_definition"
        | "class_declaration"
        | "arrow_function"
    );

    let mut best_child: Option<AstContext> = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(ctx) = find_enclosing_node(child, source, target_line) {
            best_child = Some(ctx);
        }
    }

    if best_child.is_some() {
        return best_child;
    }

    if is_relevant {
        let name = extract_name(&node, source).unwrap_or_else(|| "<anonim>".to_string());
        let kind = friendly_kind(node.kind());
        return Some(AstContext {
            kind,
            name,
            start_line: start + 1,
            end_line: end + 1,
        });
    }

    None
}

fn extract_name(node: &Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "type_identifier" {
            let name = &source[child.byte_range()];
            return Some(name.to_string());
        }
    }
    None
}

fn friendly_kind(kind: &str) -> String {
    match kind {
        "function_item" => "fn".to_string(),
        "struct_item" => "struct".to_string(),
        "impl_item" => "impl".to_string(),
        "enum_item" => "enum".to_string(),
        "trait_item" => "trait".to_string(),
        "function_definition" => "def".to_string(),
        "class_definition" => "class".to_string(),
        "function_declaration" => "function".to_string(),
        "method_definition" => "method".to_string(),
        "class_declaration" => "class".to_string(),
        "arrow_function" => "=>".to_string(),
        _ => kind.to_string(),
    }
}