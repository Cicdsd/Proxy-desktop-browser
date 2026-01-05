//! Automation System Module - V1000 Experimental Category 13
//!
//! Provides browser automation capabilities including:
//! - Visual automation builder
//! - Natural language automation
//! - Record and replay
//! - Conditional workflows
//! - Distributed automation

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// =============================================================================
// EXP-13001: Visual Automation Builder
// =============================================================================

/// Visual automation builder for creating browser automations
pub struct VisualAutomationBuilder {
    workflows: HashMap<String, Workflow>,
    active_workflow: Option<String>,
}

/// A workflow is a sequence of automation steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<AutomationStep>,
    pub variables: HashMap<String, String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Individual automation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationStep {
    pub id: String,
    pub step_type: StepType,
    pub selector: Option<String>,
    pub value: Option<String>,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub condition: Option<StepCondition>,
}

/// Types of automation steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    Navigate { url: String },
    Click,
    Type { text: String },
    Wait { duration_ms: u64 },
    WaitForElement,
    WaitForNavigation,
    Screenshot { filename: String },
    ExtractText,
    ExtractAttribute { attribute: String },
    Scroll { direction: ScrollDirection, amount: i32 },
    Hover,
    Select { option: String },
    KeyPress { key: String },
    ExecuteScript { script: String },
    Conditional { condition: StepCondition, then_steps: Vec<AutomationStep>, else_steps: Vec<AutomationStep> },
    Loop { count: u32, steps: Vec<AutomationStep> },
    LoopWhile { condition: StepCondition, steps: Vec<AutomationStep>, max_iterations: u32 },
}

/// Scroll direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Condition for conditional steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCondition {
    pub condition_type: ConditionType,
    pub selector: Option<String>,
    pub value: Option<String>,
}

/// Types of conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    ElementExists,
    ElementVisible,
    ElementContainsText,
    UrlContains,
    UrlEquals,
    VariableEquals,
    VariableContains,
    Custom { script: String },
}

impl VisualAutomationBuilder {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            active_workflow: None,
        }
    }

    /// Create a new workflow
    pub fn create_workflow(&mut self, name: &str, description: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let workflow = Workflow {
            id: id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            steps: Vec::new(),
            variables: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        self.workflows.insert(id.clone(), workflow);
        info!("Created workflow: {} ({})", name, id);
        id
    }

    /// Add step to workflow
    pub fn add_step(&mut self, workflow_id: &str, step: AutomationStep) -> Result<()> {
        let workflow = self.workflows.get_mut(workflow_id)
            .ok_or_else(|| anyhow!("Workflow not found"))?;
        workflow.steps.push(step);
        Ok(())
    }

    /// Get workflow
    pub fn get_workflow(&self, id: &str) -> Option<&Workflow> {
        self.workflows.get(id)
    }

    /// List all workflows
    pub fn list_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }

    /// Delete workflow
    pub fn delete_workflow(&mut self, id: &str) -> bool {
        self.workflows.remove(id).is_some()
    }

    /// Set active workflow
    pub fn set_active(&mut self, id: &str) {
        self.active_workflow = Some(id.to_string());
    }

    /// Get active workflow
    pub fn get_active(&self) -> Option<&Workflow> {
        self.active_workflow.as_ref().and_then(|id| self.workflows.get(id))
    }
}

impl Default for VisualAutomationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// EXP-13002: Natural Language Automation
// =============================================================================

/// Natural language automation interpreter
pub struct NaturalLanguageAutomation {
    command_patterns: Vec<CommandPattern>,
    context: AutomationContext,
}

/// Command pattern for NL matching
#[derive(Debug, Clone)]
pub struct CommandPattern {
    pub pattern: String,
    pub step_type: StepType,
    pub extract_params: Vec<String>,
}

/// Context for automation
#[derive(Debug, Clone, Default)]
pub struct AutomationContext {
    pub current_url: Option<String>,
    pub variables: HashMap<String, String>,
    pub last_result: Option<String>,
}

impl NaturalLanguageAutomation {
    pub fn new() -> Self {
        let patterns = vec![
            CommandPattern {
                pattern: "go to (.+)".to_string(),
                step_type: StepType::Navigate { url: String::new() },
                extract_params: vec!["url".to_string()],
            },
            CommandPattern {
                pattern: "click on (.+)".to_string(),
                step_type: StepType::Click,
                extract_params: vec!["selector".to_string()],
            },
            CommandPattern {
                pattern: "type (.+) in (.+)".to_string(),
                step_type: StepType::Type { text: String::new() },
                extract_params: vec!["text".to_string(), "selector".to_string()],
            },
            CommandPattern {
                pattern: "wait (\\d+) seconds".to_string(),
                step_type: StepType::Wait { duration_ms: 0 },
                extract_params: vec!["duration".to_string()],
            },
            CommandPattern {
                pattern: "take screenshot as (.+)".to_string(),
                step_type: StepType::Screenshot { filename: String::new() },
                extract_params: vec!["filename".to_string()],
            },
            CommandPattern {
                pattern: "scroll (up|down|left|right)".to_string(),
                step_type: StepType::Scroll { direction: ScrollDirection::Down, amount: 100 },
                extract_params: vec!["direction".to_string()],
            },
        ];
        
        Self {
            command_patterns: patterns,
            context: AutomationContext::default(),
        }
    }

    /// Parse natural language command
    pub fn parse_command(&self, input: &str) -> Option<AutomationStep> {
        let input_lower = input.to_lowercase();
        
        // Simple pattern matching
        if input_lower.starts_with("go to ") || input_lower.starts_with("navigate to ") {
            let url = input_lower
                .trim_start_matches("go to ")
                .trim_start_matches("navigate to ")
                .trim()
                .to_string();
            return Some(AutomationStep {
                id: uuid::Uuid::new_v4().to_string(),
                step_type: StepType::Navigate { url },
                selector: None,
                value: None,
                timeout_ms: 30000,
                retry_count: 3,
                condition: None,
            });
        }
        
        if input_lower.starts_with("click ") || input_lower.starts_with("click on ") {
            let selector = input_lower
                .trim_start_matches("click on ")
                .trim_start_matches("click ")
                .trim()
                .to_string();
            return Some(AutomationStep {
                id: uuid::Uuid::new_v4().to_string(),
                step_type: StepType::Click,
                selector: Some(selector),
                value: None,
                timeout_ms: 10000,
                retry_count: 3,
                condition: None,
            });
        }
        
        if input_lower.contains("wait") {
            // Extract number from "wait X seconds"
            let duration_ms = input_lower
                .split_whitespace()
                .find_map(|s| s.parse::<u64>().ok())
                .unwrap_or(1) * 1000;
            return Some(AutomationStep {
                id: uuid::Uuid::new_v4().to_string(),
                step_type: StepType::Wait { duration_ms },
                selector: None,
                value: None,
                timeout_ms: duration_ms,
                retry_count: 0,
                condition: None,
            });
        }
        
        if input_lower.starts_with("screenshot") || input_lower.starts_with("take screenshot") {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let filename = format!("screenshot_{}.png", timestamp);
            return Some(AutomationStep {
                id: uuid::Uuid::new_v4().to_string(),
                step_type: StepType::Screenshot { filename },
                selector: None,
                value: None,
                timeout_ms: 5000,
                retry_count: 0,
                condition: None,
            });
        }
        
        None
    }

    /// Parse multiple commands from text
    pub fn parse_script(&self, script: &str) -> Vec<AutomationStep> {
        script
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .filter_map(|line| self.parse_command(line))
            .collect()
    }

    /// Set context variable
    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.context.variables.insert(name.to_string(), value.to_string());
    }

    /// Get context variable
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.context.variables.get(name)
    }
}

impl Default for NaturalLanguageAutomation {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// EXP-13003: Record and Replay Pro
// =============================================================================

/// Browser action recorder
pub struct ActionRecorder {
    recording: bool,
    recorded_actions: Vec<RecordedAction>,
    start_time: Option<Instant>,
}

/// Recorded browser action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedAction {
    pub action_type: ActionType,
    pub timestamp_ms: u128,
    pub url: Option<String>,
    pub selector: Option<String>,
    pub value: Option<String>,
    pub coordinates: Option<(i32, i32)>,
}

/// Type of recorded action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    PageLoad,
    Click,
    DblClick,
    RightClick,
    Type,
    KeyDown,
    KeyUp,
    Scroll,
    Hover,
    Focus,
    Blur,
    Select,
    Submit,
    Resize,
    Custom { name: String },
}

impl ActionRecorder {
    pub fn new() -> Self {
        Self {
            recording: false,
            recorded_actions: Vec::new(),
            start_time: None,
        }
    }

    /// Start recording
    pub fn start(&mut self) {
        self.recording = true;
        self.start_time = Some(Instant::now());
        self.recorded_actions.clear();
        info!("Action recording started");
    }

    /// Stop recording
    pub fn stop(&mut self) -> Vec<RecordedAction> {
        self.recording = false;
        info!("Action recording stopped. {} actions recorded", self.recorded_actions.len());
        self.recorded_actions.clone()
    }

    /// Is recording active
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Record an action
    pub fn record_action(
        &mut self,
        action_type: ActionType,
        url: Option<&str>,
        selector: Option<&str>,
        value: Option<&str>,
        coordinates: Option<(i32, i32)>,
    ) {
        if !self.recording {
            return;
        }
        
        let timestamp_ms = self.start_time.map(|t| t.elapsed().as_millis()).unwrap_or(0);
        
        let action = RecordedAction {
            action_type,
            timestamp_ms,
            url: url.map(|s| s.to_string()),
            selector: selector.map(|s| s.to_string()),
            value: value.map(|s| s.to_string()),
            coordinates,
        };
        
        self.recorded_actions.push(action);
    }

    /// Convert recorded actions to automation steps
    pub fn to_workflow(&self, name: &str) -> Workflow {
        let steps: Vec<AutomationStep> = self.recorded_actions.iter().map(|action| {
            let step_type = match &action.action_type {
                ActionType::PageLoad => StepType::Navigate { url: action.url.clone().unwrap_or_default() },
                ActionType::Click | ActionType::DblClick => StepType::Click,
                ActionType::Type => StepType::Type { text: action.value.clone().unwrap_or_default() },
                ActionType::Scroll => StepType::Scroll { direction: ScrollDirection::Down, amount: 100 },
                ActionType::Hover => StepType::Hover,
                _ => StepType::Wait { duration_ms: 100 },
            };
            
            AutomationStep {
                id: uuid::Uuid::new_v4().to_string(),
                step_type,
                selector: action.selector.clone(),
                value: action.value.clone(),
                timeout_ms: 10000,
                retry_count: 3,
                condition: None,
            }
        }).collect();
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Workflow {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: format!("Recorded workflow with {} steps", steps.len()),
            steps,
            variables: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get recorded actions count
    pub fn action_count(&self) -> usize {
        self.recorded_actions.len()
    }
}

impl Default for ActionRecorder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// EXP-13004: Conditional Workflows
// =============================================================================

/// Workflow condition evaluator
pub struct ConditionEvaluator {
    custom_evaluators: HashMap<String, Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl ConditionEvaluator {
    pub fn new() -> Self {
        Self {
            custom_evaluators: HashMap::new(),
        }
    }

    /// Evaluate a condition
    pub fn evaluate(&self, condition: &StepCondition, context: &AutomationContext) -> bool {
        match &condition.condition_type {
            ConditionType::ElementExists => {
                // Would check DOM - placeholder
                condition.selector.is_some()
            }
            ConditionType::ElementVisible => {
                condition.selector.is_some()
            }
            ConditionType::ElementContainsText => {
                true // Placeholder
            }
            ConditionType::UrlContains => {
                if let (Some(current), Some(expected)) = (&context.current_url, &condition.value) {
                    current.contains(expected)
                } else {
                    false
                }
            }
            ConditionType::UrlEquals => {
                context.current_url.as_ref() == condition.value.as_ref()
            }
            ConditionType::VariableEquals => {
                if let (Some(var_name), Some(expected)) = (&condition.selector, &condition.value) {
                    context.variables.get(var_name).map(|v| v == expected).unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionType::VariableContains => {
                if let (Some(var_name), Some(expected)) = (&condition.selector, &condition.value) {
                    context.variables.get(var_name).map(|v| v.contains(expected)).unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionType::Custom { script } => {
                // Would execute JS - placeholder
                !script.is_empty()
            }
        }
    }

    /// Register custom evaluator
    pub fn register_custom<F>(&mut self, name: &str, evaluator: F)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.custom_evaluators.insert(name.to_string(), Box::new(evaluator));
    }
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// EXP-13005: Distributed Automation
// =============================================================================

/// Distributed automation coordinator
pub struct DistributedAutomation {
    nodes: HashMap<String, AutomationNode>,
    task_queue: Vec<DistributedTask>,
    completed_tasks: Vec<TaskResult>,
}

/// Automation node (worker)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationNode {
    pub id: String,
    pub name: String,
    pub status: NodeStatus,
    pub capabilities: Vec<String>,
    pub current_task: Option<String>,
    pub completed_tasks: u64,
}

/// Node status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Available,
    Busy,
    Offline,
    Error,
}

/// Distributed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedTask {
    pub id: String,
    pub workflow_id: String,
    pub priority: u8,
    pub assigned_node: Option<String>,
    pub status: TaskStatus,
    pub created_at: u64,
}

/// Task status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub node_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl DistributedAutomation {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            task_queue: Vec::new(),
            completed_tasks: Vec::new(),
        }
    }

    /// Register a node
    pub fn register_node(&mut self, node: AutomationNode) {
        info!("Registering automation node: {}", node.name);
        self.nodes.insert(node.id.clone(), node);
    }

    /// Unregister a node
    pub fn unregister_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
    }

    /// Submit a task
    pub fn submit_task(&mut self, workflow_id: &str, priority: u8) -> String {
        let task_id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let task = DistributedTask {
            id: task_id.clone(),
            workflow_id: workflow_id.to_string(),
            priority,
            assigned_node: None,
            status: TaskStatus::Pending,
            created_at: now,
        };
        
        self.task_queue.push(task);
        info!("Task {} submitted for workflow {}", task_id, workflow_id);
        task_id
    }

    /// Assign pending tasks to available nodes
    pub fn assign_tasks(&mut self) {
        let available_nodes: Vec<String> = self.nodes.iter()
            .filter(|(_, n)| n.status == NodeStatus::Available)
            .map(|(id, _)| id.clone())
            .collect();
        
        for node_id in available_nodes {
            if let Some(task) = self.task_queue.iter_mut()
                .find(|t| t.status == TaskStatus::Pending)
            {
                task.assigned_node = Some(node_id.clone());
                task.status = TaskStatus::Assigned;
                
                if let Some(node) = self.nodes.get_mut(&node_id) {
                    node.status = NodeStatus::Busy;
                    node.current_task = Some(task.id.clone());
                }
                
                info!("Task {} assigned to node {}", task.id, node_id);
            }
        }
    }

    /// Complete a task
    pub fn complete_task(&mut self, task_id: &str, result: TaskResult) {
        if let Some(task) = self.task_queue.iter_mut().find(|t| t.id == task_id) {
            task.status = if result.success { TaskStatus::Completed } else { TaskStatus::Failed };
            
            if let Some(node_id) = &task.assigned_node {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.status = NodeStatus::Available;
                    node.current_task = None;
                    node.completed_tasks += 1;
                }
            }
        }
        
        self.completed_tasks.push(result);
    }

    /// Get pending task count
    pub fn pending_count(&self) -> usize {
        self.task_queue.iter().filter(|t| t.status == TaskStatus::Pending).count()
    }

    /// Get available node count
    pub fn available_nodes(&self) -> usize {
        self.nodes.values().filter(|n| n.status == NodeStatus::Available).count()
    }

    /// Get all nodes
    pub fn list_nodes(&self) -> Vec<&AutomationNode> {
        self.nodes.values().collect()
    }
}

impl Default for DistributedAutomation {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Unified Automation Manager
// =============================================================================

/// Unified automation system manager
pub struct AutomationManager {
    pub visual_builder: VisualAutomationBuilder,
    pub nl_automation: NaturalLanguageAutomation,
    pub recorder: ActionRecorder,
    pub condition_evaluator: ConditionEvaluator,
    pub distributed: DistributedAutomation,
    start_time: Instant,
}

impl AutomationManager {
    pub fn new() -> Self {
        info!("Initializing Automation Manager");
        Self {
            visual_builder: VisualAutomationBuilder::new(),
            nl_automation: NaturalLanguageAutomation::new(),
            recorder: ActionRecorder::new(),
            condition_evaluator: ConditionEvaluator::new(),
            distributed: DistributedAutomation::new(),
            start_time: Instant::now(),
        }
    }

    /// Get uptime
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Create workflow from natural language
    pub fn workflow_from_text(&mut self, name: &str, script: &str) -> String {
        let steps = self.nl_automation.parse_script(script);
        let workflow_id = self.visual_builder.create_workflow(name, "Created from natural language");
        
        for step in steps {
            let _ = self.visual_builder.add_step(&workflow_id, step);
        }
        
        workflow_id
    }

    /// Get automation statistics
    pub fn get_stats(&self) -> AutomationStats {
        AutomationStats {
            workflows_count: self.visual_builder.list_workflows().len(),
            recorded_actions: self.recorder.action_count(),
            distributed_nodes: self.distributed.list_nodes().len(),
            pending_tasks: self.distributed.pending_count(),
            uptime_seconds: self.uptime_seconds(),
        }
    }
}

impl Default for AutomationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Automation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationStats {
    pub workflows_count: usize,
    pub recorded_actions: usize,
    pub distributed_nodes: usize,
    pub pending_tasks: usize,
    pub uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_builder() {
        let mut builder = VisualAutomationBuilder::new();
        let id = builder.create_workflow("Test", "Test workflow");
        
        let step = AutomationStep {
            id: "step1".to_string(),
            step_type: StepType::Navigate { url: "https://example.com".to_string() },
            selector: None,
            value: None,
            timeout_ms: 10000,
            retry_count: 3,
            condition: None,
        };
        
        assert!(builder.add_step(&id, step).is_ok());
        assert_eq!(builder.get_workflow(&id).expect("Operation should succeed in test").steps.len(), 1);
    }

    #[test]
    fn test_nl_automation() {
        let nl = NaturalLanguageAutomation::new();
        
        let step = nl.parse_command("go to https://example.com");
        assert!(step.is_some());
        
        let step = nl.parse_command("click on #button");
        assert!(step.is_some());
        
        let step = nl.parse_command("wait 5 seconds");
        assert!(step.is_some());
    }

    #[test]
    fn test_action_recorder() {
        let mut recorder = ActionRecorder::new();
        
        recorder.start();
        assert!(recorder.is_recording());
        
        recorder.record_action(ActionType::Click, Some("https://example.com"), Some("#btn"), None, None);
        recorder.record_action(ActionType::Type, None, Some("#input"), Some("test"), None);
        
        let actions = recorder.stop();
        assert_eq!(actions.len(), 2);
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_distributed_automation() {
        let mut dist = DistributedAutomation::new();
        
        let node = AutomationNode {
            id: "node1".to_string(),
            name: "Worker 1".to_string(),
            status: NodeStatus::Available,
            capabilities: vec!["chrome".to_string()],
            current_task: None,
            completed_tasks: 0,
        };
        
        dist.register_node(node);
        assert_eq!(dist.available_nodes(), 1);
        
        dist.submit_task("workflow1", 5);
        assert_eq!(dist.pending_count(), 1);
        
        dist.assign_tasks();
        assert_eq!(dist.pending_count(), 0);
        assert_eq!(dist.available_nodes(), 0);
    }

    #[test]
    fn test_automation_manager() {
        let mut manager = AutomationManager::new();
        
        let workflow_id = manager.workflow_from_text(
            "Test Script",
            "go to https://example.com\nwait 2 seconds\nclick on #button"
        );
        
        let workflow = manager.visual_builder.get_workflow(&workflow_id);
        assert!(workflow.is_some());
        assert_eq!(workflow.expect("Operation should succeed in test").steps.len(), 3);
    }
}
