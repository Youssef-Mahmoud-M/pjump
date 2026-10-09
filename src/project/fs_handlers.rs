use std::fmt::Write;
use std::io::ErrorKind;
use std::path::Path;
use std::{fs, io};

use super::Project;

pub fn get_projects(config_file_path: &Path) -> Result<Vec<Project>, &'static str> {
    let config_file = match fs::read_to_string(config_file_path) {
        Ok(s) => s,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => String::from(""),
            _ => return Err(format_io_error(e, true)),
        },
    };
    config_file
        .lines()
        .map(|s| {
            let Some((name, path)) = s.split_once('|') else {
                return Err("Invalid config file");
            };

            Ok(Project {
                name: String::from(name),
                path: String::from(path),
            })
        })
        .collect()
}

pub fn add_project(
    project_name: &str,
    project_path: &str,
    config_file_path: &Path,
    projects: &[Project],
) -> Result<Project, &'static str> {
    if projects.iter().any(|p| p.name == project_name) {
        return Err("A project with that name already exists");
    }
    if project_name.contains('|') {
        return Err("Projects cannot contain the character '|'");
    }
    if project_name == "pjump" {
        return Err("Project name cannot be \"pjump\"");
    }
    if project_name == "pjump-bin" {
        return Err("Project name cannot be \"pjump-bin\"");
    }
    let separator_len = "|".len();
    let base_capacity = projects.iter().fold(0, |acc, p| {
        acc + p.name.len() + separator_len + p.path.len() + 1
    });

    let total_capacity = base_capacity + project_name.len() + separator_len + project_path.len();

    let mut config_data = String::with_capacity(total_capacity);

    for project in projects {
        let _ = writeln!(config_data, "{}|{}", project.name, project.path);
    }
    let _ = write!(config_data, "{}|{}", project_name, project_path);

    fs::write(config_file_path, config_data).map_err(|e| format_io_error(e, false))?;

    Ok(Project {
        name: project_name.to_string(),
        path: project_path.to_string(),
    })
}

pub fn remove_project(
    project_name: &str,
    config_file_path: &Path,
    projects: &mut Vec<Project>,
) -> Result<(), &'static str> {
    if !projects.iter().any(|p| p.name == project_name) {
        return Err("No project with this name exists");
    }
    projects.retain(|p| p.name != project_name);
    let separator_len = "|".len();
    let total_capacity = projects.iter().fold(0, |acc, p| {
        acc + p.name.len() + separator_len + p.path.len() + 1
    });

    let mut config_data = String::with_capacity(total_capacity);

    for project in projects {
        let _ = writeln!(config_data, "{}|{}", project.name, project.path);
    }

    fs::write(config_file_path, config_data).map_err(|e| format_io_error(e, false))
}

pub fn clear_projects(config_file_path: &Path) -> Result<(), &'static str> {
    match fs::write(config_file_path, "") {
        Ok(_) => Ok(()),
        Err(e) => Err(format_io_error(e, false)),
    }
}

fn format_io_error(e: io::Error, is_read: bool) -> &'static str {
    match e.kind() {
        ErrorKind::NotFound => "The config file does not exist",
        ErrorKind::PermissionDenied => "You do not have the permission to edit the config file",
        _ => {
            if is_read {
                "Could not read the file"
            } else {
                "Could not write to the file"
            }
        }
    }
}
