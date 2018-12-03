use std::borrow::Cow;
use std::env;
use std::io::{self, Write};
use std::path::{Component, Path, Prefix, PrefixComponent};
use std::process;
use std::process::{Command, Stdio};

use regex::bytes::Regex;

fn get_drive_letter(pc: &PrefixComponent) -> Option<String> {
    let drive_byte = match pc.kind() {
        Prefix::VerbatimDisk(d) => Some(d),
        Prefix::Disk(d) => Some(d),
        _ => None,
    };
    drive_byte.map(|drive_letter| {
        String::from_utf8(vec![drive_letter])
            .expect(&format!("Invalid drive letter: {}", drive_letter))
            .to_lowercase()
    })
}

fn get_prefix_for_drive(drive: &str) -> String {
    // todo - lookup mount points
    format!("/mnt/{}", drive)
}

fn translate_path_to_unix(argument: String) -> String {
    {
        let (argname, arg) = if argument.starts_with("--") && argument.contains('=') {
            let parts: Vec<&str> = argument.splitn(2, '=').collect();
            (format!("{}=", parts[0]), parts[1])
        } else {
            ("".to_owned(), argument.as_ref())
        };
        let win_path = Path::new(arg);
        if win_path.is_absolute() || win_path.exists() {
            let wsl_path: String = win_path.components().fold(String::new(), |mut acc, c| {
                match c {
                    Component::Prefix(prefix_comp) => {
                        let d = get_drive_letter(&prefix_comp)
                            .expect(&format!("Cannot handle path {:?}", win_path));
                        acc.push_str(&get_prefix_for_drive(&d));
                    }
                    Component::RootDir => {}
                    _ => {
                        let d = c.as_os_str()
                            .to_str()
                            .expect(&format!("Cannot represent path {:?}", win_path))
                            .to_owned();
                        if !acc.is_empty() && !acc.ends_with('/') {
                            acc.push('/');
                        }
                        acc.push_str(&d);
                    }
                };
                acc
            });
            return format!("{}{}", &argname, &wsl_path);
        }
    }
    argument
}

fn translate_path_to_win(line: &[u8]) -> Cow<[u8]> {
    lazy_static! {
        static ref WSLPATH_RE: Regex = Regex::new(r"(?m-u)/mnt/(?P<drive>[A-Za-z])(?P<path>/\S*)")
            .expect("Failed to compile WSLPATH regex");
    }
    WSLPATH_RE.replace_all(line, &b"${drive}:${path}"[..])
}

fn shell_escape(arg: String) -> String {
    // ToDo: This really only handles arguments with spaces and newlines.
    // More complete shell escaping is required for the general case.
    if arg.contains(" ") {
        return vec![String::from("\""), arg, String::from("\"")].join("");
    }
    arg.replace("\n", "$'\n'");
    arg.replace(";", "$';'")
}
pub fn execute(interactive: bool) {
    let mut exe_path = env::current_exe().unwrap();
    exe_path.pop();
    let wslexerc_path = format!("{}\\.wslexerc", exe_path.display());

    let mut cmd_args = Vec::new();
    let mut wsl_args: Vec<String> = vec![];

    let wsl_cmd: String;
    let exe: String = env::args().next().unwrap();
    let path = Path::new(&exe);
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    wsl_args.push(String::from(file_stem));

    // process wsl command arguments
    wsl_args.extend(env::args().skip(1).map(translate_path_to_unix));

    if Path::new(&wslexerc_path).exists() {
        wsl_cmd = format!(
            "source {};{}",
            translate_path_to_unix(wslexerc_path),
            interactive ? wsl_args.join(" ") : wsl_args.into_iter().map(shell_escape).collect::<Vec<String>>().join(" ");
        );
    } else {
        wsl_cmd = wsl_args.join(" ");
    }
    let exe_cmd: String;
    if interactive {
        exe_cmd = "-ic".to_string();
    } else {
        exe_cmd = "-c".to_string();
    }
    cmd_args.push("bash".to_string());
    cmd_args.push(exe_cmd);
    cmd_args.push(wsl_cmd.clone());

    // setup stdin/stdout
    let stdin_mode = if wsl_cmd.ends_with("--version") {
        // For some reason, the git subprocess seems to hang, waiting for
        // input, when VS Code 1.17.2 tries to detect if `git --version` works
        // on Windows 10 1709 (specifically, in `findSpecificGit` in the
        // VS Code source file `extensions/git/src/git.ts`).
        // To workaround this, we only pass stdin to the git subprocess
        // for all other commands, but not for the initial `--version` check.
        // Stdin is needed for example when commiting, where the commit
        // message is passed on stdin.
        Stdio::null()
    } else {
        Stdio::inherit()
    };

    // setup the wsl subprocess launched inside WSL
    let mut wsl_proc_setup = Command::new("wsl.exe");
    wsl_proc_setup.args(&cmd_args).stdin(stdin_mode);
    let status;

    // add git commands that must use translate_path_to_win
    const TRANSLATED_CMDS: &[&str] = &["rev-parse", "remote"];

    let translate_output = env::args()
        .skip(1)
        .position(|arg| {
            TRANSLATED_CMDS
                .iter()
                .position(|&tcmd| tcmd == arg)
                .is_some()
        })
        .is_some();

    if translate_output {
        // run the subprocess and capture its output
        let wsl_proc = wsl_proc_setup
            .stdout(Stdio::piped())
            .spawn()
            .expect(&format!("Failed to execute command '{}'", &wsl_cmd));
        let output = wsl_proc
            .wait_with_output()
            .expect(&format!("Failed to wait for wsl call '{}'", &wsl_cmd));
        status = output.status;
        let output_bytes = output.stdout;
        let mut stdout = io::stdout();
        stdout
            .write_all(&translate_path_to_win(&output_bytes))
            .expect("Failed to write wsl output");
        stdout.flush().expect("Failed to flush output");
    } else {
        // run the subprocess without capturing its output
        // the output of the subprocess is passed through unchanged
        status = wsl_proc_setup
            .status()
            .expect(&format!("Failed to execute command '{}'", &wsl_cmd));
    }

    // forward any exit code
    if let Some(exit_code) = status.code() {
        process::exit(exit_code);
    }
}

#[test]
fn win_to_unix_path_trans() {
    assert_eq!(
        translate_path_to_unix("d:\\test\\file.txt".to_string()),
        "/mnt/d/test/file.txt"
    );
    assert_eq!(
        translate_path_to_unix("C:\\Users\\test\\a space.txt".to_string()),
        "/mnt/c/Users/test/a space.txt"
    );
}

#[test]
fn unix_to_win_path_trans() {
    assert_eq!(
        &*translate_path_to_win(b"/mnt/d/some path/a file.md"),
        b"d:/some path/a file.md"
    );
    assert_eq!(
        &*translate_path_to_win(b"origin  /mnt/c/path/ (fetch)"),
        b"origin  c:/path/ (fetch)"
    );
    let multiline = b"mirror  /mnt/c/other/ (fetch)\nmirror  /mnt/c/other/ (push)\n";
    let multiline_result = b"mirror  c:/other/ (fetch)\nmirror  c:/other/ (push)\n";
    assert_eq!(
        &*translate_path_to_win(&multiline[..]),
        &multiline_result[..]
    );
}

#[test]
fn no_path_translation() {
    assert_eq!(
        &*translate_path_to_win(b"/mnt/other/file.sh"),
        b"/mnt/other/file.sh"
    );
}

#[test]
fn relative_path_translation() {
    assert_eq!(
        translate_path_to_unix(".\\src\\main.rs".to_string()),
        "./src/main.rs"
    );
}

#[test]
fn long_argument_path_translation() {
    assert_eq!(
        translate_path_to_unix("--file=C:\\some\\path.txt".to_owned()),
        "--file=/mnt/c/some/path.txt"
    );
}
