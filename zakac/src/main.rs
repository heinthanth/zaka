mod cli;

use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::termcolor::ColorChoice;
use zakac_cli_common::termcolor::StandardStream;

fn main() -> std::process::ExitCode
{
    let mut sin = std::io::stdin().lock();
    let mut sout = StandardStream::stdout(ColorChoice::Auto);
    let mut serr = StandardStream::stderr(ColorChoice::Auto);
    let mut io = CommandIO { stdin: &mut sin, stdout: &mut sout, stderr: &mut serr };

    let command = cli::Zakac;
    command.execute(&std::env::args().collect::<Vec<String>>(), &mut io).into()
}
