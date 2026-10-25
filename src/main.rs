use std::env::{self, var_os};
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process;

mod project;

macro_rules! usage {
    () => {
        ""
    };
}

fn get_config_file_path() -> Option<PathBuf> {
    let home_dir = env::var_os("HOME")?;

    let mut path = PathBuf::from(home_dir);
    path.push(".config");
    path.push("pjump");
    path.push("projects.txt");
    if let Ok(exists) = fs::exists(&path)
        && !exists
        && let Some(parent) = path.parent()
    {
        let _ = fs::create_dir_all(parent);
    }
    Some(path)
}

fn print(message: &str) {
    let message = message.replace("\"", "\\\"");
    println!("echo \"{}\"", message);
}

fn eprint(message: &str) {
    let message = message.replace("\"", "\\\"");
    println!("echo \"{}\" >&2", message);
}

fn print_pjump_function() {
    println!("function pjump() {{ eval \"$(pjump-bin $@)\"; }}");
}

fn alias(project_name: &str) {
    if should_alias() {
        alias_force(project_name);
    }
}

fn alias_force(project_name: &str) {
    println!("alias {}='pjump go {}'", project_name, project_name);
}

fn unalias(project_name: &str) {
    if should_alias() {
        println!("unalias {}", project_name);
    }
}

fn should_alias() -> bool {
    var_os("PJUMP_ALIAS").map(|v| v == "1").unwrap_or(false)
}

fn get_run_command() -> OsString {
    var_os("PJUMP_EDITOR")
        .or_else(|| var_os("EDITOR"))
        .unwrap_or_else(|| OsString::from("vi"))
}

fn main() {
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|s| s.into_string().unwrap_or_else(|_| process::exit(1)))
        .collect();
    let args_slices: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    if let Err(err) = run(&args_slices) {
        eprint(err);
        process::exit(1);
    }
}

fn run(args: &[&str]) -> Result<(), &'static str> {
    let Some(config_file_path) = get_config_file_path() else {
        return Err("HOME variable is not set.");
    };

    match args {
        ["setup"] => {
            print_pjump_function();
            Ok(())
        }
        ["setup", "-a"] => {
            print_pjump_function();
            let projects = project::get_projects(&config_file_path)?;
            for project in projects {
                alias_force(project.name());
            }
            println!("export PJUMP_ALIAS=1");
            Ok(())
        }
        ["clear"] => {
            let projects = project::get_projects(&config_file_path)?;
            for project in projects {
                unalias(project.name());
            }
            project::clear_projects(&config_file_path)?;
            print("succussfully cleared projects");
            Ok(())
        }
        ["list"] => {
            let projects = project::get_projects(&config_file_path)?;

            if !projects.is_empty() {
                for project in projects {
                    print(&project.to_string());
                }
            } else {
                print("There are no projects");
            }

            Ok(())
        }
        ["add"] => {
            let projects = project::get_projects(&config_file_path)?;

            let Ok(current_dir) = env::current_dir() else {
                return Err("Failed to get current directory");
            };

            let Some(current_dir_name) = current_dir.file_name() else {
                return Err("Could not get the current directory's name");
            };

            let Some(current_dir) = current_dir.to_str() else {
                return Err("Invalid current directory path");
            };

            let Some(current_dir_name) = current_dir_name.to_str() else {
                return Err("Invalid current directory name");
            };

            project::add_project(current_dir_name, current_dir, &config_file_path, &projects)?;
            alias(current_dir_name);

            Ok(())
        }
        ["add", name] => {
            let projects = project::get_projects(&config_file_path)?;

            let Ok(current_dir) = env::current_dir() else {
                return Err("Failed to get current directory");
            };

            let Some(current_dir) = current_dir.to_str() else {
                return Err("Invalid current directory path");
            };

            project::add_project(name, current_dir, &config_file_path, &projects)?;
            alias(name);

            Ok(())
        }
        ["remove"] => {
            let mut projects = project::get_projects(&config_file_path)?;

            let Ok(current_dir) = env::current_dir() else {
                return Err("Failed to get current directory");
            };

            let Some(current_dir_name) = current_dir.file_name() else {
                return Err("Could not get the current directory's name");
            };

            let Some(current_dir_name) = current_dir_name.to_str() else {
                return Err("Invalid current directory name");
            };

            project::remove_project(current_dir_name, &config_file_path, &mut projects)?;
            unalias(current_dir_name);

            Ok(())
        }
        ["remove", name] => {
            let mut projects = project::get_projects(&config_file_path)?;

            project::remove_project(name, &config_file_path, &mut projects)?;

            Ok(())
        }
        ["go", name] => {
            let projects = project::get_projects(&config_file_path)?;

            let Some(project) = projects.iter().find(|p| p.name() == name) else {
                return Err("Project not found");
            };

            println!("{} {}", get_run_command().to_string_lossy(), project.path());

            Ok(())
        }
        _ => Err(concat!("Invalid arguments\n", usage!())),
    }
}
