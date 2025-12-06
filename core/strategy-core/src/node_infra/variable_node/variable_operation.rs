use serde::{Deserialize, Serialize};
use strum::Display;

/**
 * Variable value update operation type
 */
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
pub enum UpdateVarValueOperation {
    #[serde(rename = "set")]
    #[strum(serialize = "set")]
    Set, // Set variable value
    #[serde(rename = "add")]
    #[strum(serialize = "add")]
    Add, // Add to variable value
    #[serde(rename = "subtract")]
    #[strum(serialize = "subtract")]
    Subtract, // Subtract from variable value
    #[serde(rename = "multiply")]
    #[strum(serialize = "multiply")]
    Multiply, // Multiply variable value
    #[serde(rename = "divide")]
    #[strum(serialize = "divide")]
    Divide, // Divide variable value
    #[serde(rename = "max")]
    #[strum(serialize = "max")]
    Max, // Maximum value
    #[serde(rename = "min")]
    #[strum(serialize = "min")]
    Min, // Minimum value
    #[serde(rename = "toggle")]
    #[strum(serialize = "toggle")]
    Toggle, // Toggle variable value
    #[serde(rename = "append")]
    #[strum(serialize = "append")]
    Append, // Append to variable value
    #[serde(rename = "remove")]
    #[strum(serialize = "remove")]
    Remove, // Remove from variable value
    #[serde(rename = "clear")]
    #[strum(serialize = "clear")]
    Clear, // Clear variable value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableOperation {
    Get,    // Get variable value
    Update, // Update variable value
    Reset,  // Reset variable value
}
