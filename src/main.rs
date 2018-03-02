use std::env;
use std::process::{Command, Stdio};

fn translate_path_to_unix(arg: String) -> String {
    if let Some(index) = arg.find(":\\") {
        if index != 1 {
            // Not a path
            return arg;
        }
        let mut path_chars = arg.chars();
        if let Some(drive) = path_chars.next() {
            let mut wsl_path = String::from("/mnt/");
            wsl_path.push_str(&drive.to_lowercase().collect::<String>());
            path_chars.next();
            wsl_path.push_str(&path_chars.map(|c|
                    match c {
                        '\\' => '/',
                        _ => c,
                    }
                ).collect::<String>());
            return wsl_path;
        }
    }
    arg
}

fn translate_path_to_win(line: &str) -> String {
    if let Some(index) = line.find("/mnt/") {
        if index != 0 {
            // Path somewhere in the middle, don't change
            return String::from(line);
        }
        let mut path_chars = line.chars();
        if let Some(drive) = path_chars.nth(5) {
            if let Some(slash) = path_chars.next() {
                if slash != '/' {
                    // not a windows mount
                    return String::from(line);
                }
                let mut win_path = String::from(
                    drive.to_lowercase().collect::<String>());
                win_path.push_str(":\\");
                win_path.push_str(&path_chars.collect::<String>());
                return win_path.replace("/", "\\");
            }
        }
    }
    String::from(line)
}

fn shell_escape(arg: String) -> String {
    // ToDo: This really only handles arguments with spaces.
    // More complete shell escaping is required for the general case.
    if arg.contains(" ") {
        return vec![
            String::from("\""),
            arg,
            String::from("\"")].join("");
    }
    arg
}

fn main() {
    let mut cmd_args = Vec::new();
    let mut git_args: Vec<String> = vec![String::from("git")];
    let git_cmd: String;

    // check for advanced usage indicated by BASH_ENV and WSLENV=BASH_ENV
    let mut interactive_shell = true;
    if env::var("BASH_ENV").is_ok() {
        let wslenv = env::var("WSLENV");
        if wslenv.is_ok() && wslenv.unwrap().split(':').position(|r| r.eq_ignore_ascii_case("BASH_ENV")).is_some() {
            interactive_shell = false;
        }
    }

    // process git command arguments
    if interactive_shell {
        git_args.extend(env::args().skip(1)
            .map(translate_path_to_unix)
            .map(shell_escape));
        git_cmd = git_args.join(" ");
        cmd_args.push("bash".to_string());
        cmd_args.push("-ic".to_string());
        cmd_args.push(git_cmd.clone());
    }
    else {
        git_args.extend(env::args().skip(1)
        .map(translate_path_to_unix));
        git_cmd = git_args.join(" ");
        cmd_args.clone_from(&git_args);
    }

    // setup stdin/stdout
    let stdin_mode = if git_cmd.ends_with("--version") {
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

    // launch git inside WSL
    let git_proc = Command::new("wsl")
        .args(&cmd_args)
        .stdin(stdin_mode)
        .stdout(Stdio::piped())
        .spawn()
        .expect(&format!("Failed to execute command '{}'", &git_cmd));
    let output = git_proc
        .wait_with_output()
        .expect(&format!("Failed to wait for git call '{}'", &git_cmd));
    let output_str = String::from_utf8_lossy(&output.stdout);

    // add git commands that must skip translate_path_to_win
    // e.g. = &["show", "status, "rev-parse", "for-each-ref"];
    const NO_TRANSLATE: &'static [&'static str] = &["show"];
    if NO_TRANSLATE.iter().position(|&r| r == git_args[1]).is_none() {
        for line in output_str.lines().map(translate_path_to_win) {
            println!("{}", line);
        }
    }
    else {
        print!("{}", output_str);
    }

    // forward any exit code
    if let Some(exit_code) = output.status.code() {
        std::process::exit(exit_code);
    }
}
