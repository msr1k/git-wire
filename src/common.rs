use serde::{Serialize, Deserialize};
use folder_compare::Error;

#[derive(Debug)]
pub enum ErrorType {
    RepositoryRootPathCommandError,
    RepositoryRootPathParseError,
    DotGitWireFileOpenError,
    DotGitWireFileParseError,
    DotGitWireFileSoundnessError,
    TempDirCreationError,
    GitCloneCommandError,
    GitCloneCommandExitStatusError,
    GitCheckoutCommandError,
    GitCheckoutCommandExitStatusError,
    GitCheckoutChangeDirectoryError,
    MoveFromTempToDestError,
    CheckDifferenceExecutionError(Error),
    CheckDifferenceStringReplaceError,
    GitLsRemoteCommandError,
    GitLsRemoteCommandExitStatusError,
    GitLsRemoteCommandStdoutDecodeError,
    GitLsRemoteCommandStdoutRegexError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Parsed {
    pub url: String,
    pub rev: String,
    pub src: String,
    pub dst: String,
}

pub mod parse;
pub mod fetch;

