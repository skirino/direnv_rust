use std::env;
use std::path::{Path, PathBuf};
use std::io;
use std::fs::{self, File};
use sha1::Sha1;
use output;

#[derive(Debug, Clone)]
pub enum VarChange {
    Unset,
    Set(String),
    Append(String),
    Prepend(String),
}
pub type VarChangesVec = Vec<(String, VarChange)>;

pub fn read(dir: &PathBuf) -> io::Result<(PathBuf, bool, VarChangesVec)> {
    let path = dir.join(".env");
    let content = read_content(&path)?;
    if is_allowed(&path, &content) {
        output::log_green(&format!("direnv_rust: Loaded '{}'.\n", path.display()));
        Ok((dir.clone(), true, parse_content(&content)))
    } else {
        output::log_red(&format!("direnv_rust: Cannot load '{}' as it's not explicitly allowed. Look into the file content and if OK run `$ direnv_rust allow`.\n", path.display()));
        Ok((dir.clone(), false, Vec::new()))
    }
}

fn read_content(path: &Path) -> io::Result<String> {
    use std::io::Read;
    let mut file = File::open(&path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn parse_content(content: &str) -> VarChangesVec {
    content.lines()
        .flat_map(|l| parse_line(l).into_iter())
        .collect::<VarChangesVec>()
}

fn parse_line(l: &str) -> Option<(String, VarChange)> {
    let v = l.split_whitespace().filter(|s| !s.is_empty()).collect::<Vec<&str>>();
    match v.len() {
        2 =>
            match v[0] {
                "unset" => Some((v[1].to_string(), VarChange::Unset)),
                _       => None,
            },
        3 =>
            match v[0] {
                "set"     => Some((v[1].to_string(), VarChange::Set(v[2].to_string()))),
                "append"  => Some((v[1].to_string(), VarChange::Append(v[2].to_string()))),
                "prepend" => Some((v[1].to_string(), VarChange::Prepend(v[2].to_string()))),
                _         => None,
            },
        _ => None,
    }
}

fn is_allowed(env_file_path: &PathBuf, content: &String) -> bool {
    allow_file_path(env_file_path, content).is_file()
}

fn allow_file_path(env_file_path: &PathBuf, content: &String) -> PathBuf {
    let mut path = env::home_dir().unwrap();
    path.push(".config");
    path.push("direnv_rust");
    path.push(compute_hash(env_file_path, content));
    path
}

fn compute_hash(path: &PathBuf, content: &String) -> String {
    let path_str = path.to_string_lossy().to_owned();
    let hashed   = path_str.to_string() + content;
    let mut sha1 = Sha1::new();
    sha1.update(hashed.as_bytes());
    sha1.digest().to_string()
}

pub fn mark_as_allowed(dir: &Path) -> io::Result<()> {
    use std::io::Write;
    let env_file_path = dir.join(".env");
    let content = read_content(&env_file_path)?;
    let allow_file_path = allow_file_path(&env_file_path, &content);
    fs::create_dir_all(allow_file_path.parent().unwrap())?;
    let mut f = File::create(&allow_file_path)?;
    f.write_all(env_file_path.to_string_lossy().as_bytes())
}
