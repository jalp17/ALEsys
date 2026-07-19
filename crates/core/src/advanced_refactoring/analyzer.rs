use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub id: String,
    pub block_type: BlockType,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub complexity: usize,
    pub dependencies: Vec<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BlockType {
    Function,
    Method,
    Struct,
    Enum,
    Impl,
    Module,
    Closure,
    Loop,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
    pub circular_deps: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOpportunity {
    pub opportunity_type: OpportunityType,
    pub description: String,
    pub confidence: f64,
    pub affected_blocks: Vec<String>,
    pub estimated_impact: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OpportunityType {
    ExtractFunction,
    ExtractModule,
    RenameSymbol,
    InlineFunction,
    SimplifyConditional,
    RemoveDeadCode,
    DeduplicateCode,
    ImproveNaming,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
}

pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_code(&self, code: &str, language: &str) -> Vec<CodeBlock> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        match language {
            "rust" => self.analyze_rust(&lines, &mut blocks),
            "python" => self.analyze_python(&lines, &mut blocks),
            "typescript" | "javascript" => self.analyze_javascript(&lines, &mut blocks),
            _ => self.analyze_generic(&lines, &mut blocks),
        }

        blocks
    }

    fn analyze_rust(&self, lines: &[&str], blocks: &mut Vec<CodeBlock>) {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                let name = self.extract_function_name(trimmed);
                let end = self.find_block_end(lines, i);
                blocks.push(CodeBlock {
                    id: format!("fn_{}_{}", name, i),
                    block_type: BlockType::Function,
                    name,
                    start_line: i + 1,
                    end_line: end + 1,
                    content: lines[i..=end].join("\n"),
                    complexity: self.estimate_complexity(lines[i..=end].join("\n").as_str()),
                    dependencies: self.extract_dependencies(lines[i..=end].join("\n").as_str()),
                    children: vec![],
                });
            } else if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
                let name = self.extract_struct_name(trimmed);
                let end = self.find_block_end(lines, i);
                blocks.push(CodeBlock {
                    id: format!("struct_{}_{}", name, i),
                    block_type: BlockType::Struct,
                    name,
                    start_line: i + 1,
                    end_line: end + 1,
                    content: lines[i..=end].join("\n"),
                    complexity: 0,
                    dependencies: vec![],
                    children: vec![],
                });
            }
        }
    }

    fn analyze_python(&self, lines: &[&str], blocks: &mut Vec<CodeBlock>) {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("def ") {
                let name = self.extract_python_function_name(trimmed);
                let end = self.find_python_block_end(lines, i);
                blocks.push(CodeBlock {
                    id: format!("fn_{}_{}", name, i),
                    block_type: BlockType::Function,
                    name,
                    start_line: i + 1,
                    end_line: end + 1,
                    content: lines[i..=end].join("\n"),
                    complexity: self.estimate_complexity(lines[i..=end].join("\n").as_str()),
                    dependencies: self.extract_dependencies(lines[i..=end].join("\n").as_str()),
                    children: vec![],
                });
            } else if trimmed.starts_with("class ") {
                let name = self.extract_class_name(trimmed);
                let end = self.find_python_block_end(lines, i);
                blocks.push(CodeBlock {
                    id: format!("class_{}_{}", name, i),
                    block_type: BlockType::Struct,
                    name,
                    start_line: i + 1,
                    end_line: end + 1,
                    content: lines[i..=end].join("\n"),
                    complexity: 0,
                    dependencies: vec![],
                    children: vec![],
                });
            }
        }
    }

    fn analyze_javascript(&self, lines: &[&str], blocks: &mut Vec<CodeBlock>) {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("function ") || trimmed.starts_with("async function ") {
                let name = self.extract_js_function_name(trimmed);
                let end = self.find_block_end(lines, i);
                blocks.push(CodeBlock {
                    id: format!("fn_{}_{}", name, i),
                    block_type: BlockType::Function,
                    name,
                    start_line: i + 1,
                    end_line: end + 1,
                    content: lines[i..=end].join("\n"),
                    complexity: self.estimate_complexity(lines[i..=end].join("\n").as_str()),
                    dependencies: self.extract_dependencies(lines[i..=end].join("\n").as_str()),
                    children: vec![],
                });
            }
        }
    }

    fn analyze_generic(&self, lines: &[&str], blocks: &mut Vec<CodeBlock>) {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("fn ") || trimmed.contains("function ") {
                let name = format!("function_{}", i);
                let end = (i + 10).min(lines.len() - 1);
                blocks.push(CodeBlock {
                    id: format!("fn_{}_{}", name, i),
                    block_type: BlockType::Function,
                    name,
                    start_line: i + 1,
                    end_line: end + 1,
                    content: lines[i..=end].join("\n"),
                    complexity: self.estimate_complexity(lines[i..=end].join("\n").as_str()),
                    dependencies: vec![],
                    children: vec![],
                });
            }
        }
    }

    pub fn build_dependency_graph(&self, blocks: &[CodeBlock]) -> DependencyGraph {
        let nodes: Vec<String> = blocks.iter().map(|b| b.id.clone()).collect();
        let mut edges = Vec::new();

        for block in blocks {
            for dep in &block.dependencies {
                if let Some(dep_block) = blocks.iter().find(|b| b.name == *dep) {
                    edges.push((block.id.clone(), dep_block.id.clone()));
                }
            }
        }

        let circular_deps = self.detect_circular_deps(&edges);

        DependencyGraph {
            nodes,
            edges,
            circular_deps,
        }
    }

    pub fn find_opportunities(&self, blocks: &[CodeBlock]) -> Vec<RefactoringOpportunity> {
        let mut opportunities = Vec::new();

        for block in blocks {
            if block.complexity > 10 {
                opportunities.push(RefactoringOpportunity {
                    opportunity_type: OpportunityType::ExtractFunction,
                    description: format!("Function '{}' has high complexity ({}). Consider extracting parts into smaller functions.", block.name, block.complexity),
                    confidence: 0.8,
                    affected_blocks: vec![block.id.clone()],
                    estimated_impact: ImpactLevel::High,
                });
            }

            if block.end_line - block.start_line > 50 {
                opportunities.push(RefactoringOpportunity {
                    opportunity_type: OpportunityType::ExtractModule,
                    description: format!("Block '{}' is very long ({} lines). Consider extracting into a separate module.", block.name, block.end_line - block.start_line),
                    confidence: 0.7,
                    affected_blocks: vec![block.id.clone()],
                    estimated_impact: ImpactLevel::Medium,
                });
            }
        }

        let duplicates = self.find_duplicate_blocks(blocks);
        for (block1, block2) in duplicates {
            opportunities.push(RefactoringOpportunity {
                opportunity_type: OpportunityType::DeduplicateCode,
                description: format!("Blocks '{}' and '{}' have similar content. Consider deduplication.", block1.name, block2.name),
                confidence: 0.9,
                affected_blocks: vec![block1.id.clone(), block2.id.clone()],
                estimated_impact: ImpactLevel::High,
            });
        }

        opportunities
    }

    fn extract_function_name(&self, line: &str) -> String {
        line.split("fn ")
            .nth(1)
            .unwrap_or("unknown")
            .split('(')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    fn extract_struct_name(&self, line: &str) -> String {
        line.split("struct ")
            .nth(1)
            .unwrap_or("unknown")
            .split('{')
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string()
    }

    fn extract_python_function_name(&self, line: &str) -> String {
        line.split("def ")
            .nth(1)
            .unwrap_or("unknown")
            .split('(')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    fn extract_class_name(&self, line: &str) -> String {
        line.split("class ")
            .nth(1)
            .unwrap_or("unknown")
            .split('(')
            .next()
            .unwrap_or("unknown")
            .split(':')
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string()
    }

    fn extract_js_function_name(&self, line: &str) -> String {
        let without_async = line.replace("async ", "");
        without_async
            .split("function ")
            .nth(1)
            .unwrap_or("unknown")
            .split('(')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    fn find_block_end(&self, lines: &[&str], start: usize) -> usize {
        let mut depth = 0;
        for (i, line) in lines.iter().enumerate().skip(start) {
            for c in line.chars() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            return i;
                        }
                    }
                    _ => {}
                }
            }
        }
        lines.len() - 1
    }

    fn find_python_block_end(&self, lines: &[&str], start: usize) -> usize {
        let base_indent = lines[start].len() - lines[start].trim_start().len();
        for i in (start + 1)..lines.len() {
            let line = lines[i];
            if !line.trim().is_empty() {
                let indent = line.len() - line.trim_start().len();
                if indent <= base_indent {
                    return i - 1;
                }
            }
        }
        lines.len() - 1
    }

    fn estimate_complexity(&self, code: &str) -> usize {
        let mut complexity = 1;
        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("if ")
                || trimmed.starts_with("else if ")
                || trimmed.contains("else {")
                || trimmed.starts_with("match ")
                || trimmed.contains("=> ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("loop ")
            {
                complexity += 1;
            }
            if trimmed.contains("&&") || trimmed.contains("||") {
                complexity += 1;
            }
        }
        complexity
    }

    fn extract_dependencies(&self, code: &str) -> Vec<String> {
        let mut deps = Vec::new();
        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.contains("use ") {
                if let Some(dep) = trimmed.split("use ").nth(1) {
                    let cleaned = dep.replace(";", "").replace("{", "").replace("}", "");
                    for part in cleaned.split(',') {
                        let name = part.trim().split("::").last().unwrap_or("").trim();
                        if !name.is_empty() {
                            deps.push(name.to_string());
                        }
                    }
                }
            }
        }
        deps
    }

    fn detect_circular_deps(&self, edges: &[(String, String)]) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut path = Vec::new();

        for (from, _) in edges {
            if !visited.contains(from) {
                self.dfs_cycle(from, edges, &mut visited, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: &str,
        edges: &[(String, String)],
        visited: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if path.contains(&node.to_string()) {
            let cycle_start = path.iter().position(|n| n == node).unwrap_or(0);
            cycles.push(path[cycle_start..].to_vec());
            return;
        }

        if visited.contains(node) {
            return;
        }

        path.push(node.to_string());
        visited.insert(node.to_string());

        for (from, to) in edges {
            if from == node {
                self.dfs_cycle(to, edges, visited, path, cycles);
            }
        }

        path.pop();
    }

    fn find_duplicate_blocks<'a>(&self, blocks: &'a [CodeBlock]) -> Vec<(&'a CodeBlock, &'a CodeBlock)> {
        let mut duplicates = Vec::new();
        for i in 0..blocks.len() {
            for j in (i + 1)..blocks.len() {
                if blocks[i].content == blocks[j].content && blocks[i].content.len() > 50 {
                    duplicates.push((&blocks[i], &blocks[j]));
                }
            }
        }
        duplicates
    }
}

impl Default for CodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_rust() {
        let analyzer = CodeAnalyzer::new();
        let code = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\npub fn subtract(a: i32, b: i32) -> i32 {\n    a - b\n}";
        let blocks = analyzer.analyze_code(code, "rust");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, BlockType::Function);
        assert_eq!(blocks[0].name, "add");
    }

    #[test]
    fn test_analyze_python() {
        let analyzer = CodeAnalyzer::new();
        let code = "def add(a, b):\n    return a + b\n\ndef subtract(a, b):\n    return a - b";
        let blocks = analyzer.analyze_code(code, "python");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, "add");
    }

    #[test]
    fn test_complexity_estimation() {
        let analyzer = CodeAnalyzer::new();
        let simple = "fn simple() {\n    let x = 1;\n}";
        let complex = "fn complex() {\n    if x > 0 {\n        for i in 0..10 {\n            if i % 2 == 0 {\n                do_something();\n            }\n        }\n    }\n}";
        assert!(analyzer.estimate_complexity(complex) > analyzer.estimate_complexity(simple));
    }

    #[test]
    fn test_dependency_graph() {
        let analyzer = CodeAnalyzer::new();
        let blocks = vec![
            CodeBlock {
                id: "fn_a".to_string(),
                block_type: BlockType::Function,
                name: "a".to_string(),
                start_line: 1,
                end_line: 3,
                content: String::new(),
                complexity: 1,
                dependencies: vec!["b".to_string()],
                children: vec![],
            },
            CodeBlock {
                id: "fn_b".to_string(),
                block_type: BlockType::Function,
                name: "b".to_string(),
                start_line: 5,
                end_line: 7,
                content: String::new(),
                complexity: 1,
                dependencies: vec![],
                children: vec![],
            },
        ];
        let graph = analyzer.build_dependency_graph(&blocks);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_find_opportunities() {
        let analyzer = CodeAnalyzer::new();
        let blocks = vec![CodeBlock {
            id: "fn_complex".to_string(),
            block_type: BlockType::Function,
            name: "complex_func".to_string(),
            start_line: 1,
            end_line: 100,
            content: String::new(),
            complexity: 15,
            dependencies: vec![],
            children: vec![],
        }];
        let opportunities = analyzer.find_opportunities(&blocks);
        assert!(!opportunities.is_empty());
        assert!(opportunities.iter().any(|o| o.opportunity_type == OpportunityType::ExtractFunction));
    }
}
