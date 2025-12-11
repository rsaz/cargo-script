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
/// * `filter` - Optional filter pattern to match script names or descriptions.
///
pub fn show_scripts(scripts: &Scripts, filter: Option<&str>) {
    // Filter scripts if filter pattern is provided
    let filtered_scripts: Vec<(&String, &Script)> = if let Some(pattern) = filter {
        scripts.scripts
            .iter()
            .filter(|(name, script)| {
                let name_match = name.to_lowercase().contains(&pattern.to_lowercase());
                let desc_match = match script {
                    Script::Default(_) => false,
                    Script::Inline { info, .. } | Script::CILike { info, .. } => {
                        info.as_deref()
                            .map(|i| i.to_lowercase().contains(&pattern.to_lowercase()))
                            .unwrap_or(false)
                    }
                };
                name_match || desc_match
            })
            .collect()
    } else {
        scripts.scripts.iter().collect()
    };

    if filtered_scripts.is_empty() {
        if filter.is_some() {
            println!("{}", format!("No scripts found matching '{}'", filter.unwrap()).yellow());
        } else {
            println!("{}", "No scripts found in Scripts.toml".yellow());
        }
        return;
    }

    let mut max_script_name_len = "Script".len();
    let mut max_description_len = "Description".len();

    for (name, script) in &filtered_scripts {
        max_script_name_len = max_script_name_len.max(name.len() + 2);
        let description = match script {
            Script::Default(_) => "",
            Script::Inline { info, .. } | Script::CILike { info, .. } => info.as_deref().unwrap_or(""),
        };
        max_description_len = max_description_len.max(description.len() + 2);
    }

    if filter.is_some() {
        println!("{}", format!("Found {} script(s) matching '{}':\n", filtered_scripts.len(), filter.unwrap()).cyan());
    }

    println!("{:<width1$} {:<width2$}", "Script".yellow(), "Description".yellow(), width1 = max_script_name_len, width2 = max_description_len);
    println!("{:<width1$} {:<width2$}", "-".repeat(max_script_name_len).yellow(), "-".repeat(max_description_len).yellow(), width1 = max_script_name_len, width2 = max_description_len);

    for (name, script) in &filtered_scripts {
        let description = match script {
            Script::Default(_) => "".to_string(),
            Script::Inline { info, .. } | Script::CILike { info, .. } => info.clone().unwrap_or_else(|| "".to_string()),
        };
        println!("{:<width1$} {:<width2$}", name.green(), description, width1 = max_script_name_len, width2 = max_description_len);
    }
}
