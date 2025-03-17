# Description
Simple naive argument parser written for my personal projects. 

> [!NOTE]
> Heavily inspired by CLAP. 

> [!CAUTION]
> Don't use this, use CLAP instead.

# Example

```rust
use clarg::{Arg, ArgParser};

fn main() {
    let arguments = ArgParser::new("Find duplicate files.")
        .arg(Arg::boolean("verbose", Some('V'), "verbose execution"))
        .arg(Arg::boolean("recurse", Some('r'), "Recursive execution"))
        .arg(Arg::boolean("json", None, "Format output as JSON"))
        .arg(Arg::string(
            "path",
            Some('p'),
            true,
            "Directory to examine",
        ))
        .parse();

    let path = arguments.get::<String>("path").unwrap();
    let verbose = arguments.get::<bool>("verbose").unwrap_or(false);
    let count = arguments.get::<i32>("count").unwrap_or(4);
    let json_output = arguments.get::<bool>("json").unwrap_or(false);

    // Program Logic
}
```

The code above when call would behave in the following manner:
`fdup.exe -h`

```
Find duplicate files.
Usage: fdup.exe  --path <PATH>

options:
‾‾‾‾‾‾‾‾
-V,  --verbose                                  verbose execution
-r,  --recurse                                  Recursive execution
     --json                                     Format output as JSON
-p,  --path <PATH>                              Directory to examine.
-h,  --help                                     Print this help message
```

When given incorrect arguments:
`fdup.exe`
```
Missing required argument:
Usage: fdup.exe  --path <PATH>
```
