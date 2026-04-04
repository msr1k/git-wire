use serde::{Serialize, Deserialize};
use folder_compare::Error;

#[derive(Debug)]
pub enum ErrorType {
    RepositoryRootPathCommandError,
    RepositoryRootPathParseError,
    CurrentDirRetrieveError,
    CurrentDirConvertError,
    DotGitWireFileOpenError,
    DotGitWireFileParseError,
    DotGitWireFileSoundnessError,
    DotGitWireFileNameNotUniqueError,
    TempDirCreationError,
    GitCloneCommandError,
    GitCloneCommandExitStatusError,
    GitCheckoutCommandError,
    GitCheckoutCommandExitStatusError,
    GitFetchCommandError,
    GitFetchCommandExitStatusError,
    MoveFromTempToDestError,
    NoItemToOperateError,
    #[allow(dead_code)]
    CheckDifferenceExecutionError(Error),
    CheckDifferenceStringReplaceError,
    GitLsRemoteCommandError,
    GitLsRemoteCommandExitStatusError,
    GitLsRemoteCommandStdoutDecodeError,
    GitLsRemoteCommandStdoutRegexError,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Method {
    #[serde(rename = "shallow")]
    Shallow,

    #[serde(rename = "shallow_no_sparse")]
    ShallowNoSparse,

    #[serde(rename = "partial")]
    Partial,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parsed {
    pub name: Option<String>,
    pub dsc: Option<String>,
    pub url: String,
    pub rev: String,
    pub src: String,
    pub dst: String,
    pub mtd: Option<Method>,
}

pub enum Target {
    Declared(Option<String>),
    Local(Option<String>),
    Direct(Parsed),
}

pub(super) fn target_string(target: &Target) -> &str {
    match target {
        Target::Declared(_) => "",
        Target::Local(_) => "local",
        Target::Direct(_) => "direct",
    }
}

pub mod parse;
pub mod fetch;
pub mod sequence;
