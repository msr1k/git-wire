use serde::{Serialize, Deserialize};
use folder_compare::Error;

#[derive(Debug)]
pub enum ErrorType {
    RepositoryRootPathCommandError,
    RepositoryRootPathParseError,
    DotGitWireFileOpenError,
    DotGitWireFileParseError,
    DotGitWireFileSoundnessError,
    DotGitWireFileIdNotUniqueError,
    TempDirCreationError,
    GitCloneCommandError,
    GitCloneCommandExitStatusError,
    GitCheckoutCommandError,
    GitCheckoutCommandExitStatusError,
    GitCheckoutChangeDirectoryError,
    GitFetchCommandError,
    GitFetchCommandExitStatusError,
    MoveFromTempToDestError,
    NoItemToOperateError,
    CheckDifferenceExecutionError(Error),
    CheckDifferenceStringReplaceError,
    GitLsRemoteCommandError,
    GitLsRemoteCommandExitStatusError,
    GitLsRemoteCommandStdoutDecodeError,
    GitLsRemoteCommandStdoutRegexError,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Method {
    #[serde(rename = "shallow")]
    Shallow,

    #[serde(rename = "shallow_no_sparse")]
    ShallowNoSparse,

    #[serde(rename = "partial")]
    Partial,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Parsed {
    pub id: Option<String>,
    pub url: String,
    pub rev: String,
    pub src: String,
    pub dst: String,
    pub mtd: Option<Method>,
}

pub mod parse;
pub mod fetch;

