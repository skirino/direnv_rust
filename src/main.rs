#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate ansi_term;
extern crate sha1;

use std::process;
use std::env;
use std::path::{Path, PathBuf};
use std::collections::btree_map::BTreeMap;

mod stack;
mod output;
mod envfile;
use stack::{Stack, Entry, VarsMap};
use envfile::{VarChange, VarChangesVec};

fn split_undo_and_kept<'a>(stack: &'a Stack, current_dir: &Path) -> (&'a [Entry], &'a [Entry]) {
    let n_kept = stack.iter().take_while(|e| current_dir.starts_with(&e.dir)).count();
    stack.split_at(n_kept)
}

fn merge_undoes(undo: &[Entry], var_changes: &mut VarsMap) -> () {
    for entry in undo.iter().rev() {
        if entry.allowed {
            output::log_green(&format!("direnv_rust: Unloaded '{}'.\n", Path::new(&entry.dir).join(".env").display()));
            for (key, val) in &entry.before {
                var_changes.insert(key.clone(), val.clone());
            }
        }
    }
}

fn enumerate_parent_dirs(d: &Path) -> Vec<PathBuf> {
    let mut path = d;
    let mut v = vec![path.to_path_buf()];
    while let Some(p) = path.parent() {
        path = p;
        v.push(path.to_path_buf());
    }
    v.into_iter().rev().collect()
}

fn read_target_files(current_dir: &Path, start_dir: &Option<&Path>) -> Vec<(PathBuf, bool, VarChangesVec)> {
    let dirs = enumerate_parent_dirs(current_dir);
    let files = match *start_dir {
        Some(ref d) => dirs.iter().skip_while(|p| d.starts_with(&p)).collect::<Vec<_>>(),
        None        => dirs.iter()                                  .collect::<Vec<_>>(),
    };

    files.into_iter().flat_map(|ref p| envfile::read(p).into_iter()).collect::<Vec<_>>()
}

fn get_current_var(var_changes_so_far: &VarsMap, key: &str) -> Option<String> {
    match var_changes_so_far.get(key) {
        Some(opt) => opt.clone(),
        None      => env::var(key).ok(),
    }
}

fn append_to_option_string(o: &Option<String>, s2: &str) -> String {
    match *o {
        Some(ref s1) => s1.to_string() + s2,
        None         => s2.to_string(),
    }
}

fn prepend_to_option_string(o: &Option<String>, s2: &str) -> String {
    match *o {
        Some(ref s1) => s2.to_string() + &s1,
        None         => s2.to_string(),
    }
}

fn apply_change(var: &Option<String>, ch: &VarChange) -> Option<String> {
    match *ch {
        VarChange::Unset            => None,
        VarChange::Set(ref new_var) => Some(new_var.to_string()),
        VarChange::Append(ref s)    => Some(append_to_option_string(&var, s)),
        VarChange::Prepend(ref s)   => Some(prepend_to_option_string(&var, s)),
    }
}

fn add_changes(dir: &PathBuf, changes: &VarChangesVec, var_changes_so_far: &mut VarsMap) -> Entry {
    // First we need to classify `changes` by name, as multiple changes can occur with the same name
    let mut changes_by_name = BTreeMap::new();
    for &(ref key, ref change) in changes {
        changes_by_name.entry(key.clone()).or_insert(Vec::new()).push(change.clone());
    }
    let mut map = BTreeMap::new();
    for (key, changes_vec) in &changes_by_name {
        let var_before = get_current_var(var_changes_so_far, &key);
        let var_after  = changes_vec.iter().fold(var_before.clone(), |acc, ref ch| apply_change(&acc, ch));
        var_changes_so_far.insert(key.clone(), var_after);
        map.insert(key.clone(), var_before.clone());
    }
    Entry { dir: dir.to_string_lossy().to_owned().to_string(), allowed: true, before: map }
}

const DIRENV_RUST: &'static str = "DIRENV_RUST";

fn load(current_dir: &Path) -> () {
    let mut var_changes: VarsMap = BTreeMap::new();

    // read DIRENV_RUST and find vars to be undone
    let stack_before = match env::var(DIRENV_RUST) {
        Ok(v)  => stack::decode(v),
        Err(_) => Vec::new(),
    };
    let (stack_kept, stack_undo) = split_undo_and_kept(&stack_before, &current_dir);
    merge_undoes(&stack_undo, &mut var_changes);

    // find/read/parse .env files
    let start_dir = stack_kept.last().map(|e| Path::new(&e.dir));
    let dir_changes_pairs = read_target_files(current_dir, &start_dir);

    // apply changes defined in .env files
    let mut new_stack = stack_kept.to_vec();
    for &(ref dir, allowed, ref changes) in dir_changes_pairs.iter() {
        let entry =
            if allowed {
                add_changes(dir, changes, &mut var_changes)
            } else {
                Entry { dir: dir.to_string_lossy().to_owned().to_string(), allowed: false, before: BTreeMap::new() }
            };
        new_stack.push(entry);
    }

    // output
    if !var_changes.is_empty() {
        for (k, o) in &var_changes {
            output::print_instruction_with_log(k, o);
        }
    }
    let direnv_var = stack::encode(&new_stack);
    output::print_instruction_without_log(DIRENV_RUST, &Some(direnv_var));
}

fn mark_as_allowed(dir: &Path) -> () {
    match envfile::mark_as_allowed(dir) {
        Ok(_)  => (),
        Err(e) => {
            println!("Error while marking .env file as allowed: {}.", e);
            process::exit(1)
        },
    }
}

fn main() {
    let current_dir_pathbuf = env::current_dir().unwrap();
    let current_dir = current_dir_pathbuf.as_path();
    match env::args().nth(1) {
        Some(ref s) if s == "allow" => mark_as_allowed(&current_dir),
        _                           => load(&current_dir),
    }
}
