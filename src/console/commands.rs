//! Console command registry.
//!
//! Commands are deliberately small and independent of the console UI or the
//! engine. Adding a command means adding a [`Command`] to [`COMMANDS`] and a
//! handler function; the parser and console state do not need to change.

/// Result of executing one command line.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CommandResult {
    /// The command ran successfully and has no engine side effect.
    #[allow(dead_code)] // reserved for future no-effect commands
    Handled,
    /// Give credits to the local player. The console applies the effect after
    /// parsing so command tests never touch game memory.
    GiveMoney(i32),
    /// The command name is unknown.
    Unknown,
    /// The command exists but its arguments are invalid.
    Invalid,
}

/// One console command.
pub(crate) struct Command {
    /// Command name, matched case-insensitively.
    pub(crate) name: &'static str,
    /// Handler receiving the arguments after the command name.
    pub(crate) handler: fn(&[&str]) -> CommandResult,
}

/// Commands currently exposed by the developer console.
pub(crate) static COMMANDS: &[Command] = &[Command {
    name: "cash",
    handler: cash,
}];

/// Parses and executes a console line.
pub(crate) fn execute(line: &str) -> CommandResult {
    let mut words = line.split_whitespace();
    let Some(name) = words.next() else {
        return CommandResult::Unknown;
    };
    let args: Vec<&str> = words.collect();

    let Some(command) = COMMANDS
        .iter()
        .find(|command| command.name.eq_ignore_ascii_case(name))
    else {
        return CommandResult::Unknown;
    };
    (command.handler)(&args)
}

fn cash(args: &[&str]) -> CommandResult {
    if !args.is_empty() {
        return CommandResult::Invalid;
    }
    CommandResult::GiveMoney(5000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_line_is_unknown() {
        assert_eq!(execute(""), CommandResult::Unknown);
        assert_eq!(execute("   "), CommandResult::Unknown);
    }

    #[test]
    fn unknown_command_is_unknown() {
        assert_eq!(execute("nope"), CommandResult::Unknown);
    }

    #[test]
    fn cash_is_known_and_case_insensitive() {
        // Parsing never dereferences the engine; the console applies the
        // effect, so this test is safe on Windows as well as Linux.
        assert_eq!(execute("cash"), CommandResult::GiveMoney(5000));
        assert_eq!(execute("CaSh"), CommandResult::GiveMoney(5000));
        assert_eq!(execute("cash now"), CommandResult::Invalid);
    }
}
