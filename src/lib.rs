//! Crate to parse command line arguments and generate help/documentation for for the program.
//!
//! # Example
//! ```no_run
//!l et arguments = ArgParser::new("Find duplicate files.")
//!         .arg(Arg::boolean("verbose", Some('V'), "verbose execution"))
//!         .arg(Arg::boolean("recurse", Some('r'), "Recursive execution"))
//!         .arg(Arg::boolean("json", None, "Format output as JSON"))
//!         .arg(Arg::string("path", Some('f'), true, "Directory to examine"))
//!         .parse();
//!     
//! let path = arguments.get::<String>("path").unwrap();
//! let verbose = arguments.get::<bool>("verbose").unwrap_or(false);
//! let count = arguments.get::<i32>("count").unwrap_or(4);
//! let json_output = arguments.get::<bool>("json").unwrap_or(false);
//! ```
//!
//!
use std::{collections::HashMap, process::exit, str::FromStr};

const ARG_PADDING: usize = 9;
/// Struct to represent the type of arguments that the user can pass to this program.
#[derive(Default)]
pub enum ArgKind {
    #[default]
    String,
    Integer,
    Float,
    Boolean,
}

/// Struct representing the requirements for each argument passed to the program.
/// Helpful when validating the type of arguments that the user passes to the program.
#[derive(Default)]
pub struct Arg {
    long_name: String,
    short_name: Option<char>,
    kind: ArgKind,
    required: bool,
    description: String,
    scanned: bool,
}

impl Arg {
    pub fn new() -> Arg {
        Arg::default()
    }
    /// Boolean type of argument. This argument is always considered optional.
    /// # Arguments
    /// `long_name` Full name for the argument
    /// `short_name` Single character representation for the argument (optional)
    /// `desc` Description for the argument.
    pub fn boolean(name: &str, short_name: Option<char>, desc: &str) -> Arg {
        let mut arg = Arg::new();
        arg.long_name = name.to_owned();
        arg.kind = ArgKind::Boolean;
        arg.required = false;
        arg.description = desc.to_owned();
        arg.short_name = short_name;
        arg
    }

    /// String type of argument.
    /// # Arguments
    /// `long_name` Full name for the argument
    /// `short_name` Single character representation for the argument (optional)
    /// `required` set whether this argument required.
    /// `desc` Description for the argument.
    pub fn string(long_name: &str, short_name: Option<char>, required: bool, desc: &str) -> Arg {
        let mut arg = Arg::new();
        arg.long_name = long_name.to_owned();
        arg.short_name = short_name;
        arg.kind = ArgKind::String;
        arg.description = desc.to_owned();
        arg.required = required;
        arg
    }
    /// Integer type of argument.
    /// # Arguments
    /// `long_name` Full name for the argument
    /// `short_name` Single character representation for the argument (optional)
    /// `required` set whether this argument required.
    /// `desc` Description for the argument.
    pub fn integer(long_name: &str, short_name: Option<char>, required: bool, desc: &str) -> Arg {
        let mut arg = Arg::new();
        arg.long_name = long_name.to_owned();
        arg.short_name = short_name;
        arg.kind = ArgKind::Integer;
        arg.description = desc.to_owned();
        arg.required = required;
        arg
    }

    /// Floating point number type of argument.
    /// # Arguments
    /// `long_name` Full name for the argument
    /// `short_name` Single character representation for the argument (optional)
    /// `required` set whether this argument required.
    /// `desc` Description for the argument.
    pub fn float(long_name: &str, short_name: Option<char>, option: bool, desc: &str) -> Arg {
        let mut arg = Arg::new();
        arg.long_name = long_name.to_owned();
        arg.short_name = short_name;
        arg.kind = ArgKind::Float;
        arg.description = desc.to_owned();
        arg.required = option;
        arg
    }
}

/// Wrapper around a map of arguments passed by the user.
#[derive(Debug)]
pub struct ArgMap {
    inner: HashMap<String, String>,
}

impl ArgMap {
    /// Get the value for a given argument if it exists and cast it to the type requested.
    /// # Arguments
    /// `name` name for the argument being requested.
    /// # Returns
    /// The given argument casted to the type `T`.
    ///
    /// # Errors
    /// If the argument does not exist or cannot be casted into `T`.
    pub fn get<T: FromStr>(&self, name: &str) -> Result<T, String> {
        if let Some(value) = self.inner.get(name) {
            value.parse::<T>().map_err(|_| {
                format!(
                    "Cannot convert value `{}` into type `{}`",
                    value,
                    std::any::type_name::<T>()
                )
            })
        } else {
            Err(format!("Inexistent `{name}` value requested."))
        }
    }
    /// Get the value for a given argument if it exists.
    /// # Arguments
    /// `name` name for the argument being requested.
    /// # Returns
    ///
    pub fn get_raw(&self, name: &str) -> Option<&String> {
        self.inner.get(name)
    }

    /// Check whether an argument was passed by the user.
    pub fn has_arg(&self, name: &str) -> bool {
        self.inner.contains_key(name)
    }
}

#[derive(Clone, Copy)]
pub enum GroupKind {
    Exclusive,
    OnlyWhen,
}

/// An argument group. Helps isolate arguments that only apply as combination.
/// When using groups a requirement is implemented on the user to not use the same name for
/// any of the arguments in the  group or outside it.
pub struct ArgGroup {
    name: String,
    kind: GroupKind,
    required: bool,
    /// main arguments to match
    args: Vec<String>,

    /// secondary arguments to match, see "OnlyIf"
    parents: Vec<String>,
}

impl ArgGroup {
    fn new(name: &str, kind: GroupKind, args1: &[&str], args2: &[&str], required: bool) -> Self {
        let args1: Vec<String> = args1.iter().map(|e| e.to_string()).collect();
        let args2: Vec<String> = args2.iter().map(|e| e.to_string()).collect();
        Self {
            name: name.to_string(),
            kind,
            args: args1,
            parents: args2,
            required,
        }
    }
    pub fn contains(&self, name: &String) -> bool {
        self.args.contains(&name.to_owned())
    }
    pub fn kind(&self) -> GroupKind {
        self.kind
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
    pub fn parents(&self) -> &Vec<String> {
        &self.parents
    }
    pub fn is_required(&self) -> bool {
        self.required
    }
    /// Requires at least one of the items in the group arguments
    /// to be present.
    pub fn allow_when(name: &str, required: bool, args: &[&str], parents: &[&str]) -> Self {
        ArgGroup::new(name, GroupKind::OnlyWhen, args, parents, required)
    }
    /// Requires at least one of the items in the group arguments
    /// to be present.
    pub fn exclusive(name: &str, required: bool, args: &[&str]) -> Self {
        ArgGroup::new(name, GroupKind::Exclusive, args, &[], required)
    }
}

/// General argument parser.
/// Created to avoid a dependency on CLAP which was used during prototyping.
pub struct ArgParser {
    executable: String,
    description: String,
    args: Vec<Arg>,
    groups: Vec<ArgGroup>,
}
impl ArgParser {
    /// Creates a new argument parser.
    /// # Arguments
    /// `description` Description/purpose of this executable.
    pub fn new(description: &str) -> Self {
        let executable = std::env::args()
            .nth(0)
            .expect("Executable name missing.")
            .split(&['\\', '/'])
            .last()
            .unwrap()
            .to_owned();
        Self {
            executable,
            description: description.to_owned(),
            args: Vec::new(),
            groups: Vec::new(),
        }
    }
    pub fn add_group(mut self, group: ArgGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// Add a new argument requirement to the parser.
    /// # Arguments
    /// `arg` Argument requirements.
    /// # Returns
    ///  The argument parser itself. Useful for chaining.
    pub fn arg(mut self, arg: Arg) -> Self {
        // we don't allow overriding help
        if arg.long_name != "help" && arg.short_name != Some('h') {
            self.args.push(arg);
        }
        self
    }

    /// Prints the program's usage.
    pub fn usage(&self) {
        let group_example = self
            .groups
            .iter()
            .filter(|group| group.is_required())
            .map(|g| match g.kind() {
                GroupKind::Exclusive => format!("<{}>", g.args().join(" | ")),
                GroupKind::OnlyWhen => String::new(),
            })
            .fold(String::new(), |mut acc, new| {
                // match group being required but argument's that are group, should not be marked as such
                acc.push_str(&new);
                acc
            });

        let example = self
            .args
            .iter()
            .filter(|arg| arg.required)
            .map(|arg| format!("--{} <{}> ", arg.long_name, arg.long_name.to_uppercase()))
            .fold(String::new(), |mut old: String, new| {
                old.push(' ');
                old.push_str(&new);
                old
            });
        let options = if self.args.iter().any(|arg| !arg.required) {
            " [options] "
        } else {
            " "
        };
        println!(
            "Usage: {}{}{}{}",
            self.executable, options, group_example, example
        );
    }

    /// Prints the help page for this executable
    /// The help page consists of:
    /// * Example usage.
    /// * Options description
    pub fn help(&self) {
        println!("{}", self.description);
        self.usage();
        println!("\noptions:");
        println!("-------");

        // calculate the maximum width of the argument name.
        let max_length = self.args.iter().fold(0, |max, arg| match arg.kind {
            // boolean arguments don't have to repeat their name, only count once
            ArgKind::Boolean => max.max(arg.long_name.len() + ARG_PADDING),

            // any other argument has 2 times the length + some padding when printed, account for it.
            // we assume the maximum usage like "--Argument <ARGUMENT>" (arg.len * 2 + at least 5 args) and add some padding
            _ => max.max(arg.long_name.len() * 2 + ARG_PADDING),
        });

        // Print each argument and it's description for the help message.
        for arg in &self.args {
            let sample_usage = match arg.kind {
                ArgKind::Boolean => arg.long_name.clone(),
                _ => format!("{} <{}>", arg.long_name, arg.long_name.to_uppercase()),
            };

            // format the shortname, if available
            let short_name = match arg.short_name {
                Some(c) => format!("-{c},"),
                None => format!("   "),
            };

            println!(
                "{} --{:<width$} {}",
                short_name,
                sample_usage,
                arg.description,
                width = max_length
            );
        }
        println!(
            "-h, --{:<width$} Print this help message",
            "help",
            width = max_length
        );
        if !self.groups.is_empty() {
            println!("\nNotes on argument groups:");
            for group in &self.groups {
                let arguments: Vec<String> = self
                    .args
                    .iter()
                    .filter(|arg| group.contains(&arg.long_name))
                    .map(|arg| match arg.kind {
                        ArgKind::Boolean => format!("--{}", arg.long_name.clone()),
                        _ => format!("--{} <{}>", arg.long_name, arg.long_name.to_uppercase()),
                    })
                    .collect();
                let parent_arguments: Vec<String> = self
                    .args
                    .iter()
                    .filter(|arg| group.parents().contains(&arg.long_name))
                    .map(|arg| match arg.kind {
                        ArgKind::Boolean => format!("--{}", arg.long_name.clone()),
                        _ => format!("--{} <{}>", arg.long_name, arg.long_name.to_uppercase()),
                    })
                    .collect();
                match group.kind() {
                    GroupKind::Exclusive => println!("The following option(s) are mutually exclusive and cannot be used together:\n\t{}", arguments.join("\n\t")),
                    GroupKind::OnlyWhen => println!("The option(s): \n\t{}\nCan only be used in conjunction with: \n\t{}", arguments.join("\n\t"), parent_arguments.join("\n\t")) 
                }
            }
        }
    }

    /// Parse user command line arguments into a Map struct.
    /// This parsing follows the argument requirements selected and consumes the parser.
    /// # Returns
    /// A map with all the parsed arguments.
    ///
    /// # Panics
    /// It errors and stops execution when the argument requirements cannot be enforced.
    /// Not being able to parse the arguments is considered a fatal error and the program
    /// execution halts with a call to exit(0).
    pub fn parse(mut self) -> ArgMap {
        let mut argument_map: HashMap<String, String> = HashMap::new();

        // skip executable name
        let mut arguments = std::env::args().skip(1);
        while let Some(arg) = arguments.next() {
            if arg == "--help" || arg == "-h" {
                self.help();
                exit(0);
            }

            if arg.starts_with("-") {
                let arg_name: String = arg.chars().skip_while(|c| *c == '-').collect();
                let actual_argument = self.args.iter_mut().find(|arg| {
                    arg.long_name == arg_name
                        || (arg_name.len() == 1 && arg_name.chars().nth(0) == arg.short_name)
                });
                if let Some(inner) = actual_argument {
                    // validate the type of argument we got
                    match inner.kind {
                        ArgKind::String => match arguments.next() {
                            // got a string, don't worry about any conversion
                            Some(value) => {
                                if value.starts_with('-') {
                                    eprintln!(
                                        "Unexpected value `{value}` for argument: --{}",
                                        arg_name
                                    );
                                    self.usage();
                                    exit(1)
                                }
                                inner.scanned = true; // we got this value, don't expect
                                argument_map.insert(inner.long_name.clone(), value);
                            }
                            _ => {
                                eprintln!("Missing value for argument: --{}", arg_name);
                                self.usage();
                                exit(1)
                            }
                        },

                        // we got an integer, ensure we can at least parse it properly
                        ArgKind::Integer => match arguments.next() {
                            Some(value) => {
                                if value.parse::<i32>().is_ok() {
                                    inner.scanned = true; // we got this value, don't expect
                                    argument_map.insert(inner.long_name.clone(), value);
                                } else {
                                    eprintln!("Cannot convert `{}` into integer.", value);
                                    self.usage();
                                    exit(1)
                                }
                            }
                            _ => {
                                eprintln!("Missing value for argument: --{}", arg_name);
                                self.usage();
                                exit(1)
                            }
                        },

                        // we got a floating point number, ensure we can at least parse it properly
                        ArgKind::Float => match arguments.next() {
                            Some(value) => {
                                if value.parse::<f32>().is_ok() {
                                    inner.scanned = true; // we got this value, don't expect
                                    argument_map.insert(inner.long_name.clone(), value);
                                } else {
                                    eprintln!(
                                        "Cannot convert `{}` into floating point number.",
                                        value
                                    );
                                    self.usage();
                                    exit(1)
                                }
                            }
                            _ => {
                                eprintln!("Missing value for argument: --{}", arg_name);
                                self.usage();
                                exit(1)
                            }
                        },

                        // this is a boolean flag, having listed, means we set it.
                        ArgKind::Boolean => {
                            inner.scanned = true; // we got this value, don't expect
                            argument_map.insert(inner.long_name.clone(), "true".to_owned());
                        }
                    }
                } else {
                    // Got an unexpected argument, error now.
                    eprintln!("Unrecognized option `{arg}` passed.");
                    self.usage();
                    exit(1);
                }
            } else {
                // Got an unexpected argument, error now.
                eprintln!("Unexpected argument option `{arg}` passed.");
                self.usage();
                exit(1);
            }
        }
        if !self.groups.is_empty() {
            for group in &self.groups {
                if group.is_required() {
                    match group.kind() {
                        GroupKind::Exclusive => {
                            let use_count = self.args.iter().fold(0, |sum, item| {
                                if group.args().contains(&item.long_name) && item.scanned {
                                    sum + 1
                                } else {
                                    sum
                                }
                            });
                            if use_count > 1 {
                                eprintln!("Misuse of exclusive argument(s). Only one of the following must be used: [{}]", group.args().join(", "));
                                self.usage();
                                exit(1)
                            } else if use_count == 0 {
                                eprintln!("Missing required exclusive argument(s). One of the following must be used: [{}]", group.args().join(", "));
                                self.usage();
                                exit(1)
                            }
                        }
                        GroupKind::OnlyWhen => {
                            let use_count = self.args.iter().fold(0, |sum, item| {
                                if group.args().contains(&item.long_name) && item.scanned {
                                    sum + 1
                                } else {
                                    sum
                                }
                            });
                            let parent_count = self.args.iter().fold(0, |sum, item| {
                                if group.parents().contains(&item.long_name) && item.scanned {
                                    sum + 1
                                } else {
                                    sum
                                }
                            });
                            if use_count == 0 {
                                eprintln!("Missing matching argument(s). One of the following must be used: [{}]", group.args().join(", "));
                                self.usage();
                                exit(1)
                            } else if parent_count == 0 {
                                eprintln!("Missing matching parent argument. Options like [{}] need to be used with: [{}].",group.args().join(", "), group.parents().join(", "));
                                self.usage();
                                exit(1)
                            }
                        }
                    }
                } else {
                    match group.kind() {
                        GroupKind::Exclusive => {
                            let use_count = self.args.iter().fold(0, |sum, item| {
                                if group.contains(&item.long_name) && item.scanned {
                                    sum + 1
                                } else {
                                    sum
                                }
                            });
                            if use_count > 1 {
                                eprintln!(
                                    "Cannot use the following arguments together: [{}]",
                                    group.args().join(", ")
                                );
                                self.usage();
                                exit(1)
                            }
                        }
                        GroupKind::OnlyWhen => {
                            let use_count = self.args.iter().fold(0, |sum, item| {
                                if group.contains(&item.long_name) && item.scanned {
                                    sum + 1
                                } else {
                                    sum
                                }
                            });
                            let parents_in_use = self.args.iter().fold(0, |sum, item| {
                                if group.parents().contains(&item.long_name) && item.scanned {
                                    sum + 1
                                } else {
                                    sum
                                }
                            });
                            if use_count > 0 && parents_in_use == 0 {
                                eprintln!("Missing arguments. Options like [{}] need to be used with: [{}].",group.args().join(", "), group.parents().join(", "));
                                self.usage();
                                exit(1)
                            }
                        }
                    }
                }
            }
        }
        self.args.iter().for_each(|arg| {
            if arg.required && !arg.scanned {
                eprintln!("Missing required argument: `{}`", arg.long_name);
                self.usage();
                exit(1);
            }
        });
        ArgMap {
            inner: argument_map,
        }
    }
}
