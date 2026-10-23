use std::io::BufRead;

use bon::Builder;
use termcolor::WriteColor;
use zakac_span::source_map::SourceMap;

use crate::artifact::ArtifactFormat;
use crate::artifact::PhaseArtifact;
use crate::artifact::dump_artifact;
use crate::artifact::load_artifact;
use crate::config::Input;
use crate::config::Output;
use crate::config::SessionOptions;

/// A session is a collection of data, state and
/// configuration related to a compilation session.
#[derive(Builder, Default, Debug, Clone, PartialEq)]
pub struct Session<I, O, E>
where
    I: BufRead,
    O: WriteColor,
    E: WriteColor,
{
    /// The standard input that the compiler should read from.
    pub stdin: I,
    /// The standard output that the compiler should write to.
    pub stdout: O,
    /// The standard error that the compiler should write to.
    pub stderr: E,
    /// Session options.
    #[builder(default)]
    pub options: SessionOptions,
    /// Source map for the compilation session.
    #[builder(default)]
    pub source_map: SourceMap,
}

impl<I, O, E> Session<I, O, E>
where
    I: BufRead,
    O: WriteColor,
    E: WriteColor,
{
    /// Decode a phase artifact from `input`.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to load the artifact from.
    /// * `format` - The format of the artifact.
    ///
    /// # Returns
    ///
    /// The loaded artifact.
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact cannot be loaded.
    pub fn load_artifact<T: PhaseArtifact>(
        &mut self, input: &Input, format: ArtifactFormat,
    ) -> zakac_errors::Result<T>
    {
        load_artifact(input, format, &mut self.stdin, &mut self.source_map)
    }

    /// Encode `value` and write it to `output`.
    ///
    /// # Arguments
    ///
    /// * `output` - The output to write the artifact to.
    /// * `value` - The artifact to write.
    /// * `format` - The format of the artifact.
    ///
    /// # Returns
    ///
    /// The result of the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact cannot be written.
    pub fn dump_artifact<T: PhaseArtifact>(
        &mut self, output: &mut Output, value: &T, format: ArtifactFormat,
    ) -> zakac_errors::Result<()>
    {
        dump_artifact(output, &mut self.stdout, value, format)
    }
}
