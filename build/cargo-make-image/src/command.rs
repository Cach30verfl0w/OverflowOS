use crate::error::Error;
use log::debug;
use std::{
    path::Path,
    process::{
        Command,
        Stdio,
    },
};

pub(crate) fn run_command(
    executable: &Path, working_folder: Option<&Path>, arguments: &[&str], redirect_stdout: bool,
) -> Result<(), Error> {
    // Create command and set working folder if defined
    let mut command = Command::new(&executable);
    if let Some(working_folder) = working_folder {
        command.current_dir(working_folder);
    }

    // Block stdout and stderr if needed
    if !redirect_stdout {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    // Insert arguments and notify user about execution
    let mut process_arguments = String::from(executable.to_str().unwrap());
    arguments.iter().map(|arg| arg.as_ref()).for_each(|arg| {
        process_arguments.push(' ');
        process_arguments.push_str(arg);
        command.arg(arg);
    });
    debug!("Running process '{}' => {}", executable.to_str().unwrap(), process_arguments);

    // Execute process and validate exit code
    let status = command.status()?;
    if !status.success() {
        return Err(Error::FailedProcess(
            String::from(executable.to_str().unwrap()),
            status.code().unwrap_or(-1),
        ));
    }
    Ok(())
}
