//! A flexible REPL (Read-Eval-Print Loop) framework for building interactive CLI applications.
//!
//! # Quick Start
//!
//! ```no_run
//! use mycli::repl::{Repl, CommandHandler};
//!
//! struct Calculator;
//!
//! impl CommandHandler for Calculator {
//!     fn handle(&mut self, command: &str) -> bool {
//!         match command {
//!             "quit" | "exit" => false,
//!             cmd => {
//!                 // Process command here
//!                 println!("Processing: {}", cmd);
//!                 true
//!             }
//!         }
//!     }
//! }
//!
//! fn main() -> rustyline::Result<()> {
//!     let mut repl = Repl::new("calc> ", Calculator)?;
//!     let _ = repl.load_history(".calc_history");
//!     repl.run()?;
//!     let _ = repl.save_history(".calc_history");
//!     Ok(())
//! }
//! ```

use rustyline::{error::ReadlineError, DefaultEditor, Result};


/// A Read-Eval-Print Loop (REPL) implementation with customizable command handling.
///
/// The REPL handles:
/// - Reading lines with editing capabilities (arrow keys, history navigation)
/// - Empty line filtering (pressing Enter on empty input is ignored)
/// - Signal handling (Ctrl+C continues, Ctrl+D exits)
/// - Command history management
///
/// # Type Parameters
///
/// * `H` - A type that implements the `CommandHandler` trait to process commands
///
/// # Examples
///
/// ```
/// use mycli::repl::{Repl, CommandHandler};
///
/// pub struct MyApp;
///
/// impl CommandHandler for MyApp {
///     fn handle(&mut self, command: &str) -> bool {
///         println!("Received: {}", command);
///         command != "exit"
///     }
/// }
///
/// fn main() -> rustyline::Result<()> {
///     let app = MyApp;
///     let mut repl = Repl::new("MyApp> ", app)?;
///     let _ = repl.load_history(".history");
///     repl.run()?;
///     let _ = repl.save_history(".history");
///     Ok(())
/// }
/// ```
pub struct Repl<H>
where H: CommandHandler {
    prompt: String,
    handler: H,
    editor: DefaultEditor,
}

/// Trait for handling commands in the REPL.
///
/// Implementors of this trait define how commands are processed when entered
/// by the user in the REPL.
///
/// # Examples
///
/// ```
/// use mycli::repl::CommandHandler;
///
/// struct EchoHandler;
///
/// impl CommandHandler for EchoHandler {
///     fn handle(&mut self, command: &str) -> bool {
///         if command == "quit" {
///             return false;
///         }
///         println!("Echo: {}", command);
///         true
///     }
/// }
/// ```
pub trait CommandHandler {
    /// Handles a command entered by the user.
    ///
    /// # Arguments
    ///
    /// * `command` - The command string to process
    ///
    /// # Returns
    ///
    /// Returns `true` to continue the REPL, `false` to exit
    fn handle(&mut self, command: &str) -> bool;
}


impl <H: CommandHandler> Repl<H> {
    /// Creates a new REPL instance with the specified prompt and command handler.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt string to display before each input (e.g., `"> "` or `"app> "`)
    /// * `handler` - The command handler that will process user input
    ///
    /// # Returns
    ///
    /// Returns `Ok(Repl)` on success, or an error if the editor cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mycli::repl::{Repl, CommandHandler};
    ///
    /// struct MyHandler;
    /// impl CommandHandler for MyHandler {
    ///     fn handle(&mut self, command: &str) -> bool { true }
    /// }
    ///
    /// let repl = Repl::new(">>> ", MyHandler).unwrap();
    /// ```
    pub fn new(prompt: impl Into<String>, handler: H, ) -> Result<Self> {
        Ok(Self {
            prompt: prompt.into(),
            handler: handler,
            editor: DefaultEditor::new()? })
    }

    /// Loads command history from a file.
    ///
    /// This allows users to access previously entered commands across sessions
    /// using the up/down arrow keys.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the history file
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the file cannot be read.
    /// It's safe to ignore errors if the file doesn't exist yet.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mycli::repl::{Repl, CommandHandler};
    /// # struct MyHandler;
    /// # impl CommandHandler for MyHandler {
    /// #     fn handle(&mut self, command: &str) -> bool { true }
    /// # }
    /// let mut repl = Repl::new("> ", MyHandler).unwrap();
    /// let _ = repl.load_history(".my_app_history");
    /// ```
    pub fn load_history(&mut self, path: &str) -> Result<()> {
        self.editor.load_history(path)
    }

    /// Saves command history to a file.
    ///
    /// This should typically be called when the REPL exits to persist
    /// the command history for future sessions.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the history file should be saved
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the file cannot be written.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mycli::repl::{Repl, CommandHandler};
    /// # struct MyHandler;
    /// # impl CommandHandler for MyHandler {
    /// #     fn handle(&mut self, command: &str) -> bool { true }
    /// # }
    /// let mut repl = Repl::new("> ", MyHandler).unwrap();
    /// // ... run the REPL ...
    /// let _ = repl.save_history(".my_app_history");
    /// ```
    pub fn save_history(&mut self, path: &str) -> Result<()> {
        self.editor.save_history(path)
    }

    /// Starts the REPL loop, processing commands until termination.
    ///
    /// The loop continues until:
    /// - The command handler returns `false`
    /// - The user presses Ctrl+D (EOF)
    /// - A readline error occurs
    ///
    /// Ctrl+C (Interrupt) is caught and ignored, allowing the REPL to continue.
    /// Empty commands (whitespace-only input) are ignored.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the REPL exits normally, or an error if a
    /// critical readline error occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mycli::repl::{Repl, CommandHandler};
    /// # struct MyHandler;
    /// # impl CommandHandler for MyHandler {
    /// #     fn handle(&mut self, command: &str) -> bool { true }
    /// # }
    /// let mut repl = Repl::new("> ", MyHandler).unwrap();
    /// repl.run().unwrap();
    /// ```
    pub fn run(&mut self) -> Result<()> {
        loop {
            let readline = self.editor.readline(&self.prompt);

            match readline {
                Ok(line) => {
                    let cmd = line.trim();

                    if cmd.is_empty() {
                        continue;
                    }

                    let _ = self.editor.add_history_entry(cmd);

                    if !self.handler.handle(cmd) {
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        Ok(())
    }

}