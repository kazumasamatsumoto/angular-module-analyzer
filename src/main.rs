use anyhow::{ Context, Result };
use clap::{ Parser, Subcommand };
use colored::*;
use petgraph::{ Graph, Undirected };
use serde::{ Deserialize, Serialize };
use std::collections::{ HashMap, HashSet };
use std::fs;
use std::path::{ Path, PathBuf };
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "angular-analyzer")]
#[command(about = "Angular module architecture analyzer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze module dependencies
    Analyze {
        /// Path to Angular project
        #[arg(short, long)]
        path: String,
        /// Output format (json, console)
        #[arg(short, long, default_value = "console")]
        output: String,
    },
    /// Generate dependency graph
    Graph {
        /// Path to Angular project
        #[arg(short, long)]
        path: String,
        /// Output file for graph
        #[arg(short, long, default_value = "dependency-graph.dot")]
        output: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub path: PathBuf,
    pub name: String,
    pub module_type: ModuleType,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub providers: Vec<String>,
    pub declarations: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModuleType {
    Core,
    Shared,
    Feature,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub modules: Vec<ModuleInfo>,
    pub dependency_violations: Vec<DependencyViolation>,
    pub circular_dependencies: Vec<Vec<String>>,
    pub metrics: ArchitectureMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyViolation {
    pub from_module: String,
    pub to_module: String,
    pub violation_type: ViolationType,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ViolationType {
    CoreDependsOnFeature,
    SharedDependsOnFeature,
    FeatureToFeatureDirect,
    CircularDependency,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchitectureMetrics {
    pub total_modules: usize,
    pub core_modules: usize,
    pub shared_modules: usize,
    pub feature_modules: usize,
    pub average_dependencies_per_module: f32,
    pub max_dependency_depth: usize,
    pub coupling_factor: f32,
}

pub struct AngularAnalyzer {
    project_path: PathBuf,
}

impl AngularAnalyzer {
    pub fn new(project_path: &str) -> Self {
        Self {
            project_path: PathBuf::from(project_path),
        }
    }

    pub fn analyze(&self) -> Result<AnalysisResult> {
        let modules = self.discover_modules()?;
        let dependency_violations = self.check_dependency_violations(&modules);
        let circular_dependencies = self.detect_circular_dependencies(&modules);
        let metrics = self.calculate_metrics(&modules);

        Ok(AnalysisResult {
            modules,
            dependency_violations,
            circular_dependencies,
            metrics,
        })
    }

    fn discover_modules(&self) -> Result<Vec<ModuleInfo>> {
        let mut modules = Vec::new();

        for entry in WalkDir::new(&self.project_path)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let path = entry.path();
            if
                path.extension().map_or(false, |ext| ext == "ts") &&
                path
                    .file_name()
                    .map_or(false, |name| name.to_string_lossy().ends_with(".module.ts"))
            {
                if let Ok(module_info) = self.parse_module_file(path) {
                    modules.push(module_info);
                }
            }
        }

        Ok(modules)
    }

    fn parse_module_file(&self, path: &Path) -> Result<ModuleInfo> {
        let content = fs
            ::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;

        let name = self.extract_module_name(path, &content);
        let module_type = self.determine_module_type(path, &content);
        let imports = self.extract_imports(&content);
        let exports = self.extract_exports(&content);
        let providers = self.extract_providers(&content);
        let declarations = self.extract_declarations(&content);
        let dependencies = self.extract_dependencies(&content);

        Ok(ModuleInfo {
            path: path.to_path_buf(),
            name,
            module_type,
            imports,
            exports,
            providers,
            declarations,
            dependencies,
        })
    }

    fn extract_module_name(&self, path: &Path, content: &str) -> String {
        // NgModuleクラス名を抽出
        let class_regex = regex::Regex::new(r"export\s+class\s+(\w+Module)").unwrap();
        if let Some(captures) = class_regex.captures(content) {
            captures.get(1).unwrap().as_str().to_string()
        } else {
            path.file_stem().unwrap_or_default().to_string_lossy().to_string()
        }
    }

    fn determine_module_type(&self, path: &Path, _content: &str) -> ModuleType {
        let path_str = path.to_string_lossy().to_lowercase();

        if path_str.contains("/core/") || path_str.contains("core.module") {
            ModuleType::Core
        } else if path_str.contains("/shared/") || path_str.contains("shared.module") {
            ModuleType::Shared
        } else if
            path_str.contains("/feature/") ||
            path_str.contains("/features/") ||
            (!path_str.contains("/core/") && !path_str.contains("/shared/"))
        {
            ModuleType::Feature
        } else {
            ModuleType::Unknown
        }
    }

    fn extract_imports(&self, content: &str) -> Vec<String> {
        self.extract_ngmodule_array(content, "imports")
    }

    fn extract_exports(&self, content: &str) -> Vec<String> {
        self.extract_ngmodule_array(content, "exports")
    }

    fn extract_providers(&self, content: &str) -> Vec<String> {
        self.extract_ngmodule_array(content, "providers")
    }

    fn extract_declarations(&self, content: &str) -> Vec<String> {
        self.extract_ngmodule_array(content, "declarations")
    }

    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let import_regex = regex::Regex
            ::new(r#"import\s*\{[^}]*\}\s*from\s*["']([^"']*)["']\s*;"#)
            .unwrap();
        import_regex
            .captures_iter(content)
            .map(|cap| cap.get(1).unwrap().as_str().to_string())
            .filter(|import| !import.starts_with(".") && !import.starts_with("@angular/"))
            .collect()
    }

    fn extract_ngmodule_array(&self, content: &str, field: &str) -> Vec<String> {
        let pattern = format!(r"{}:\s*\[(.*?)\]", field);
        let regex = regex::Regex::new(&pattern).unwrap();

        if let Some(captures) = regex.captures(content) {
            let array_content = captures.get(1).unwrap().as_str();
            array_content
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    fn check_dependency_violations(&self, modules: &[ModuleInfo]) -> Vec<DependencyViolation> {
        let mut violations = Vec::new();
        let module_map: HashMap<String, &ModuleInfo> = modules
            .iter()
            .map(|m| (m.name.clone(), m))
            .collect();

        for module in modules {
            for dep in &module.dependencies {
                if let Some(dep_module) = module_map.get(dep) {
                    // Core modules should not depend on Feature modules
                    if
                        module.module_type == ModuleType::Core &&
                        dep_module.module_type == ModuleType::Feature
                    {
                        violations.push(DependencyViolation {
                            from_module: module.name.clone(),
                            to_module: dep.clone(),
                            violation_type: ViolationType::CoreDependsOnFeature,
                            description: "Core module depends on Feature module".to_string(),
                        });
                    }

                    // Shared modules should not depend on Feature modules
                    if
                        module.module_type == ModuleType::Shared &&
                        dep_module.module_type == ModuleType::Feature
                    {
                        violations.push(DependencyViolation {
                            from_module: module.name.clone(),
                            to_module: dep.clone(),
                            violation_type: ViolationType::SharedDependsOnFeature,
                            description: "Shared module depends on Feature module".to_string(),
                        });
                    }
                }
            }
        }

        violations
    }

    fn detect_circular_dependencies(&self, modules: &[ModuleInfo]) -> Vec<Vec<String>> {
        let mut graph = Graph::<String, (), Undirected>::new_undirected();
        let mut node_indices = HashMap::new();

        // グラフのノードを作成
        for module in modules {
            let idx = graph.add_node(module.name.clone());
            node_indices.insert(module.name.clone(), idx);
        }

        // エッジを追加
        for module in modules {
            if let Some(&from_idx) = node_indices.get(&module.name) {
                for dep in &module.dependencies {
                    if let Some(&to_idx) = node_indices.get(dep) {
                        graph.add_edge(from_idx, to_idx, ());
                    }
                }
            }
        }

        // 循環依存の検出（簡易版）
        Vec::new() // 実装を簡略化
    }

    fn calculate_metrics(&self, modules: &[ModuleInfo]) -> ArchitectureMetrics {
        let total_modules = modules.len();
        let core_modules = modules
            .iter()
            .filter(|m| m.module_type == ModuleType::Core)
            .count();
        let shared_modules = modules
            .iter()
            .filter(|m| m.module_type == ModuleType::Shared)
            .count();
        let feature_modules = modules
            .iter()
            .filter(|m| m.module_type == ModuleType::Feature)
            .count();

        let total_dependencies: usize = modules
            .iter()
            .map(|m| m.dependencies.len())
            .sum();
        let average_dependencies_per_module = if total_modules > 0 {
            (total_dependencies as f32) / (total_modules as f32)
        } else {
            0.0
        };

        // 結合度の計算（依存関係の密度）
        let possible_connections = if total_modules > 1 {
            total_modules * (total_modules - 1)
        } else {
            1
        };
        let coupling_factor = (total_dependencies as f32) / (possible_connections as f32);

        ArchitectureMetrics {
            total_modules,
            core_modules,
            shared_modules,
            feature_modules,
            average_dependencies_per_module,
            max_dependency_depth: 0, // 実装を簡略化
            coupling_factor,
        }
    }

    pub fn generate_dot_graph(&self, modules: &[ModuleInfo]) -> String {
        let mut dot = String::from("digraph AngularModules {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box];\n\n");

        // ノードの定義
        for module in modules {
            let color = match module.module_type {
                ModuleType::Core => "lightblue",
                ModuleType::Shared => "lightgreen",
                ModuleType::Feature => "lightyellow",
                ModuleType::Unknown => "lightgray",
            };
            dot.push_str(&format!("  \"{}\" [fillcolor={} style=filled];\n", module.name, color));
        }

        dot.push('\n');

        // エッジの定義
        let module_names: HashSet<String> = modules
            .iter()
            .map(|m| m.name.clone())
            .collect();

        for module in modules {
            for dep in &module.dependencies {
                if module_names.contains(dep) {
                    dot.push_str(&format!("  \"{}\" -> \"{}\";\n", module.name, dep));
                }
            }
        }

        dot.push_str("}\n");
        dot
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Analyze { path, output } => {
            let analyzer = AngularAnalyzer::new(path);
            let result = analyzer.analyze()?;

            match output.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&result)?;
                    println!("{}", json);
                }
                _ => {
                    print_analysis_result(&result);
                }
            }
        }
        Commands::Graph { path, output } => {
            let analyzer = AngularAnalyzer::new(path);
            let result = analyzer.analyze()?;
            let dot_graph = analyzer.generate_dot_graph(&result.modules);

            fs::write(output, dot_graph)?;
            println!("Dependency graph written to: {}", output);
        }
    }

    Ok(())
}

fn print_analysis_result(result: &AnalysisResult) {
    println!("{}", "=== Angular Module Analysis Report ===".bold().cyan());
    println!();

    // メトリクス表示
    println!("{}", "📊 Architecture Metrics".bold().green());
    println!("Total Modules: {}", result.metrics.total_modules);
    println!("Core Modules: {}", result.metrics.core_modules);
    println!("Shared Modules: {}", result.metrics.shared_modules);
    println!("Feature Modules: {}", result.metrics.feature_modules);
    println!(
        "Average Dependencies per Module: {:.2}",
        result.metrics.average_dependencies_per_module
    );
    println!("Coupling Factor: {:.2}", result.metrics.coupling_factor);
    println!();

    // 依存関係違反
    if !result.dependency_violations.is_empty() {
        println!("{}", "⚠️  Dependency Violations".bold().red());
        for violation in &result.dependency_violations {
            println!(
                "  {} -> {}: {}",
                violation.from_module.red(),
                violation.to_module.red(),
                violation.description
            );
        }
        println!();
    }

    // モジュール一覧
    println!("{}", "📦 Modules by Type".bold().blue());

    let mut modules_by_type: HashMap<&ModuleType, Vec<&ModuleInfo>> = HashMap::new();
    for module in &result.modules {
        modules_by_type.entry(&module.module_type).or_default().push(module);
    }

    for (module_type, modules) in modules_by_type {
        let type_name = match module_type {
            ModuleType::Core => "Core",
            ModuleType::Shared => "Shared",
            ModuleType::Feature => "Feature",
            ModuleType::Unknown => "Unknown",
        };

        println!("  {}:", type_name.bold());
        for module in modules {
            println!("    - {} ({} dependencies)", module.name, module.dependencies.len());
        }
        println!();
    }

    if result.dependency_violations.is_empty() {
        println!("{}", "✅ No dependency violations found!".green());
    }
}
