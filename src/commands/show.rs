//! This module provides the functionality to display all script names and descriptions.

use crate::commands::script::{Scripts, Script};
use colored::*;

/// Show all script names and descriptions in a table format.
///
/// This function prints a table with script names and their descriptions.
/// It calculates the maximum width for the script names and descriptions
/// to format the table neatly.
///
/// # Arguments
///
/// * `scripts` - A reference to the collection of scripts.
///
pub fn show_scripts(scripts: &Scripts) {   
    let mut max_script_name_len = "Script".len();
    let mut max_description_len = "Description".len();

    for (name, script) in &scripts.scripts {
        max_script_name_len = max_script_name_len.max(name.len() + 2);
        let description = match script {
            Script::Default(_) => "",
            Script::Detailed { info, .. } => info.as_deref().unwrap_or(""),
        };
        max_description_len = max_description_len.max(description.len() + 2);
    }
   
    println!("{:<width1$} {:<width2$}", "Script".yellow(), "Description".yellow(), width1 = max_script_name_len, width2 = max_description_len);
    println!("{:<width1$} {:<width2$}", "-".repeat(max_script_name_len).yellow(), "-".repeat(max_description_len).yellow(), width1 = max_script_name_len, width2 = max_description_len);

    for (name, script) in &scripts.scripts {
        let description = match script {
            Script::Default(_) => "".to_string(),
            Script::Detailed { info, .. } => info.clone().unwrap_or_else(|| "".to_string()),
        };
        println!("{:<width1$} {:<width2$}", name.green(), description, width1 = max_script_name_len, width2 = max_description_len);
    }
}