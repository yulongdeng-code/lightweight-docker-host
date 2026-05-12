use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub code_spec: Option<CodeSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSpec {
    pub language: String,
    pub requirements: String,
    pub test_cases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub subtasks: Vec<Subtask>,
    pub dag: DAG,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAG {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPath {
    pub order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub generated_code: Option<GeneratedCode>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    pub files: HashMap<String, String>,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub passed: bool,
    pub failures: Vec<Failure>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    pub message: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub total: usize,
    pub passed: usize,
    pub failed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionHints {
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalDelivery {
    pub artifacts: HashMap<String, String>,
    pub summary: String,
}

#[derive(Debug, Error)]
pub enum PlanError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Planning failed: {0}")]
    PlanningFailed(String),
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Code generation failed: {0}")]
    CodeGenFailed(String),
    #[error("Tool call failed: {0}")]
    ToolCallFailed(String),
}

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Test execution failed: {0}")]
    TestFailed(String),
}

#[derive(Debug, Error)]
pub enum PEVError {
    #[error("Planning error: {0}")]
    Plan(#[from] PlanError),
    #[error("Execution error: {0}")]
    Execution(#[from] ExecutionError),
    #[error("Verification error: {0}")]
    Verify(#[from] VerifyError),
}

pub trait PlanningLayer: Send + Sync {
    fn decompose_task(&self, user_input: &str, context: &str) -> Result<ExecutionPlan, PlanError>;
    fn build_dependency_graph(&self, subtasks: &[Subtask]) -> Result<DAG, PlanError>;
    fn optimize_execution_path(&self, dag: &DAG) -> Result<ExecutionPath, PlanError>;
}

pub trait ExecutionLayer: Send + Sync {
    async fn execute_subtask(&self, subtask: &Subtask) -> Result<ExecutionResult, ExecutionError>;
    async fn generate_code(&self, spec: &CodeSpec) -> Result<GeneratedCode, ExecutionError>;
    async fn call_tool(&self, tool_call: &ToolCall) -> Result<ToolResult, ExecutionError>;
}

pub trait VerificationLayer: Send + Sync {
    fn verify_result(&self, result: &ExecutionResult, task: &str) -> Result<VerificationReport, VerifyError>;
    async fn run_tests(&self, code: &GeneratedCode) -> Result<TestReport, VerifyError>;
    fn suggest_corrections(&self, failures: &[Failure]) -> Result<CorrectionHints, VerifyError>;
    async fn auto_fix(&self, code: &GeneratedCode, report: &TestReport) -> Result<GeneratedCode, VerifyError>;
}

pub struct SimplePlanner;

impl PlanningLayer for SimplePlanner {
    fn decompose_task(&self, user_input: &str, context: &str) -> Result<ExecutionPlan, PlanError> {
        let subtasks = vec![Subtask {
            id: "task_1".to_string(),
            description: user_input.to_string(),
            dependencies: vec![],
            code_spec: Some(CodeSpec {
                language: "rust".to_string(),
                requirements: user_input.to_string(),
                test_cases: vec!["Test 1".to_string()],
            }),
        }];
        let dag = self.build_dependency_graph(&subtasks)?;
        Ok(ExecutionPlan { subtasks, dag })
    }

    fn build_dependency_graph(&self, subtasks: &[Subtask]) -> Result<DAG, PlanError> {
        let nodes: Vec<String> = subtasks.iter().map(|s| s.id.clone()).collect();
        let mut edges = Vec::new();
        for subtask in subtasks {
            for dep in &subtask.dependencies {
                edges.push((dep.clone(), subtask.id.clone()));
            }
        }
        Ok(DAG { nodes, edges })
    }

    fn optimize_execution_path(&self, dag: &DAG) -> Result<ExecutionPath, PlanError> {
        Ok(ExecutionPath {
            order: dag.nodes.clone(),
        })
    }
}

pub struct SimpleExecutor;

impl ExecutionLayer for SimpleExecutor {
    async fn execute_subtask(&self, subtask: &Subtask) -> Result<ExecutionResult, ExecutionError> {
        Ok(ExecutionResult {
            success: true,
            output: format!("Executed: {}", subtask.description),
            generated_code: None,
            duration_ms: 100,
        })
    }

    async fn generate_code(&self, spec: &CodeSpec) -> Result<GeneratedCode, ExecutionError> {
        let mut files = HashMap::new();
        files.insert(
            "main.rs".to_string(),
            "fn main() { println!(\"Hello, NexusAgent!\"); }".to_string(),
        );
        Ok(GeneratedCode {
            files,
            language: spec.language.clone(),
        })
    }

    async fn call_tool(&self, tool_call: &ToolCall) -> Result<ToolResult, ExecutionError> {
        Ok(ToolResult {
            success: true,
            data: serde_json::json!({"status": "ok"}),
        })
    }
}

pub struct SimpleVerifier;

impl VerificationLayer for SimpleVerifier {
    fn verify_result(&self, result: &ExecutionResult, _task: &str) -> Result<VerificationReport, VerifyError> {
        Ok(VerificationReport {
            passed: result.success,
            failures: vec![],
            score: if result.success { 1.0 } else { 0.0 },
        })
    }

    async fn run_tests(&self, _code: &GeneratedCode) -> Result<TestReport, VerifyError> {
        Ok(TestReport {
            total: 1,
            passed: 1,
            failed: vec![],
        })
    }

    fn suggest_corrections(&self, failures: &[Failure]) -> Result<CorrectionHints, VerifyError> {
        Ok(CorrectionHints {
            suggestions: failures.iter().map(|f| f.message.clone()).collect(),
        })
    }

    async fn auto_fix(&self, code: &GeneratedCode, _report: &TestReport) -> Result<GeneratedCode, VerifyError> {
        Ok(code.clone())
    }
}

pub struct PEVEngine {
    planner: Box<dyn PlanningLayer>,
    executor: Box<dyn ExecutionLayer>,
    verifier: Box<dyn VerificationLayer>,
}

impl PEVEngine {
    pub fn new() -> Self {
        Self {
            planner: Box::new(SimplePlanner),
            executor: Box::new(SimpleExecutor),
            verifier: Box::new(SimpleVerifier),
        }
    }

    pub async fn end_to_end_execute(&self, user_request: &str) -> Result<FinalDelivery, PEVError> {
        let plan = self.planner.decompose_task(user_request, "")?;
        let mut results = Vec::new();

        for subtask in plan.subtasks {
            let result = self.executor.execute_subtask(&subtask).await?;
            let verification = self.verifier.verify_result(&result, &subtask.description)?;
            if verification.passed {
                results.push(result);
            }
        }

        Ok(FinalDelivery {
            artifacts: HashMap::new(),
            summary: "Task completed successfully".to_string(),
        })
    }
}

impl Default for PEVEngine {
    fn default() -> Self {
        Self::new()
    }
}
