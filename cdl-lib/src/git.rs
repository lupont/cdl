pub use git2::{Branch, Repository};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs, io,
    path::Path,
    process::{Command, Stdio},
};

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug)]
pub enum GitError {
    DirCreationError(io::Error),
    RepositoryError(git2::Error),
    InvalidBranchError,
}

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

pub fn checkout(repo: &mut Repository, branch_name: &str) -> Result<()> {
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

pub fn execute_gradlew(repo: &Repository) -> Result<()> {
    match repo.workdir() {
        Some(dir) => {
            let _ = Command::new("chmod")
                .current_dir(dir)
                .arg("+x")
                .arg("gradlew")
                .output()?;
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

pub fn get_compiled_jars(repo: &Repository) -> Result<Vec<fs::DirEntry>> {
    if let Some(dir) = repo.workdir() {
        let build_dir = Path::join(dir, "build/libs");
        if build_dir.is_dir() {
            let files = fs::read_dir(build_dir)?
                .filter_map(std::result::Result::ok)
                .collect::<Vec<_>>();
            return Ok(files);
        }
    }

    Ok(vec![])
}

#[allow(dead_code)]
fn pull(repo: &Repository, branch_name: &str) -> Result<()> {
    let mut origin_remote = repo.find_remote("origin")?;
    origin_remote.fetch(&[branch_name], None, None)?;
    let oid = repo.refname_to_id(&format!("refs/remotes/origin/{}", branch_name))?;
    let object = repo.find_object(oid, None)?;
    repo.reset(&object, git2::ResetType::Hard, None)?;
    Ok(())
}

pub fn clone(url: &str) -> Result<Repository> {
    // let full_url = format!("https://github.com/{}", url);
    let url = if !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url.into()
    };
    let local_dir = Path::join(
        Path::new("/tmp/cdl/"),
        &url.replace("https://", "").replace('/', "__"),
    );

    if let Ok(repo) = Repository::open(&local_dir) {
        println!("opening repo");
        // let branch_name = repo.find_branch
        // pull(repo, branch_name)?;
        return Ok(repo);
    }

    Repository::clone(&url, &local_dir).map_err(|e| {
        println!("caught error");
        GitError::from(e)
    })
}
