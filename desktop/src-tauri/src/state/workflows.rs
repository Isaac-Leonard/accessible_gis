use super::configurable_tools::Input;

pub struct ToolWrapper {
    tool: usize,
    inputs: Vec<WorkflowInput>,
}

pub enum WorkflowInput {
    Raw(Input),
    ToolResult { tool: usize, output: usize },
    FromUser,
}

pub struct Workflow {
    pub tool_calls: Vec<ToolWrapper>,
}
