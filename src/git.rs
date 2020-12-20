use git2::{Branch, Repository};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs,
    io::{self, stdin, stdout, Write},
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug)]
pub enum GitError {
    DirCreationError(io::Error),
    RepositoryError(git2::Error),
    InvalidBranchError,
}

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Error for GitError {}

impl From<io::Error> for GitError {
    fn from(e: io::Error) -> Self {
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
    match dir {
        Some(dir) => {
            let _ = Command::new("sh")
                .current_dir(dir)
                .arg("-c")
                .arg("./gradlew build")
                .stdout(Stdio::inherit())
                .output()?;
            Ok(())
        }
        None => Ok(()),
    }
}

pub fn copy_compiled_mod(repo: &Repository) -> Result<(), GitError> {
    let dir = repo.workdir();

    if let Some(dir) = dir {
        let build_dir = Path::join(dir, "build/libs");
        if build_dir.is_dir() {
            let files = fs::read_dir(build_dir)?
                .filter_map(Result::ok)
                .collect::<Vec<_>>();

            println!("  INDEX  FILE");
            for (i, file) in files.iter().enumerate() {
                println!(
                    "> {}     {}{}",
                    i + 1,
                    if i + 1 < 10 { " " } else { "" },
                    file.file_name().to_string_lossy()
                );
            }

            print!("==> ");
            stdout().flush()?;

            let input = {
                let mut tmp = String::new();
                stdin().read_line(&mut tmp)?;
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

#[allow(dead_code)]
fn pull(repo: &Repository, branch_name: &str) -> Result<(), GitError> {
    let mut origin_remote = repo.find_remote("origin")?;
    origin_remote.fetch(&[branch_name], None, None)?;
    let oid = repo.refname_to_id(&format!("refs/remotes/origin/{}", branch_name))?;
    let object = repo.find_object(oid, None)?;
    repo.reset(&object, git2::ResetType::Hard, None)?;
    Ok(())
}

pub fn clone(url: &str) -> Result<Repository, GitError> {
    let full_url = format!("https://github.com/{}", url);
    let local_dir = Path::join(Path::new("/tmp/cdl/"), url);

    if let Ok(repo) = Repository::open(&local_dir) {
        // let branch_name = repo.find_branch
        // pull(repo, branch_name)?;
        return Ok(repo);
    }

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

    println!("  INDEX  BRANCH");
    for (i, branch) in &branches {
        let name = branch.name().unwrap();
        if let Some(name) = name {
            println!(
                "> {}     {}{}",
                i + 1,
                if i + 1 < 10 { " " } else { "" },
                name
            );
        }
    }

    print!("==> ");
    stdout().flush()?;

    let input = {
        let mut tmp = String::new();
        stdin().read_line(&mut tmp)?;
        tmp.trim().to_string()
    };

    match input.parse::<usize>() {
        Ok(n) if n > 0 && n <= branches.len() => {
            let branch = &branches[n - 1].1;
            return Ok(branch.name().unwrap().unwrap().into());
        }

        _ => {
            println!("There's nothing to do.");
            return Err(GitError::InvalidBranchError);
        }
    }
}
