use git2::{Branch, BranchType, Branches, Repository};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs::{self, DirEntry},
    num::ParseIntError,
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug)]
pub enum GitError {
    DirCreationError(std::io::Error),
    RepositoryError(git2::Error),
    InvalidBranchError(ParseIntError),
}

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Error for GitError {}

impl From<std::io::Error> for GitError {
    fn from(e: std::io::Error) -> Self {
        Self::DirCreationError(e)
    }
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        Self::RepositoryError(e)
    }
}

pub fn checkout(repo: &mut Repository, branch_name: &str) -> Result<(), GitError> {
    let head = repo.head()?;
    let oid = match head.target() {
        Some(target) => target,
        None => panic!("error getting the old target"),
    };

    let commit = repo.find_commit(oid)?;

    let _ = repo.branch(branch_name, &commit, false);

    let obj = repo.revparse_single(&("refs/heads/".to_owned() + branch_name))?;

    repo.checkout_tree(&obj, None)?;

    repo.set_head(&("refs/heads/".to_owned() + branch_name))?;
    Ok(())
}

pub fn execute_gradlew(repo: &Repository) -> Result<(), GitError> {
    let dir = repo.workdir();
    if let Some(dir) = dir {
        let _ = Command::new("sh")
            .current_dir(dir)
            .arg("-c")
            .arg("./gradlew build")
            .stdout(Stdio::inherit())
            .output()?;
        println!("Succes!!!!!!!!!!");
    }

    Ok(())
}

pub fn copy_compiled_mod(repo: &Repository) -> Result<(), GitError> {
    let dir = repo.workdir();

    if let Some(dir) = dir {
        let build_dir = Path::join(dir, "build/libs");
        if build_dir.is_dir() {
            println!("Found these jars:");
            let files = fs::read_dir(build_dir)?
                .filter_map(Result::ok)
                .collect::<Vec<_>>();

            for (i, file) in files.iter().enumerate() {
                println!("{}: {}", i + 1, file.file_name().to_string_lossy());
            }

            let input = {
                let mut tmp = String::new();
                std::io::stdin().read_line(&mut tmp)?;
                tmp.trim().to_string()
            };
            let input = crate::parse_input(&input);

            if let Some(input) = input {
                let f = files
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| input.contains(&(i + 1)))
                    .map(|(_, f)| f)
                    .collect::<Vec<_>>();

                for file in f {
                    fs::copy(&mut file.path(), &mut file.file_name())?;
                }
            } else {
                println!("There's nothing to do.");
            }
        }
    }

    Ok(())
}

pub fn clone(url: &str) -> Result<Repository, GitError> {
    let full_url = format!("https://github.com/{}", url);
    let local_dir = Path::join(Path::new("/tmp/cdl/"), url);
    Repository::clone(&full_url, &local_dir).map_err(|e| GitError::RepositoryError(e))
}

pub fn choose_branch(repo: &Repository) -> Result<String, GitError> {
    let branches = repo
        .branches(None)?
        .filter_map(Result::ok)
        .filter(|(b, _)| b.name().is_ok())
        .map(|(b, _)| b)
        .enumerate()
        .collect::<Vec<(usize, Branch)>>();

    for (i, branch) in &branches {
        let name = branch.name().unwrap();
        if let Some(name) = name {
            println!("{}: {}", i + 1, name);
        }
    }

    let input = {
        let mut tmp = String::new();
        std::io::stdin().read_line(&mut tmp)?;
        tmp.trim().to_string()
    };

    match input.parse::<usize>() {
        Ok(n) => {
            if n > 0 && n <= branches.len() {
                let branch = &branches[n - 1].1;
                return Ok(branch.name().unwrap().unwrap().into());
            } else {
                panic!("invalid branch index");
            }
        }

        Err(e) => {
            println!("There's nothing to do.");
            return Err(GitError::InvalidBranchError(e));
        }
    }
}
