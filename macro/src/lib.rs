#![allow(clippy::panic)]
#![allow(clippy::expect_used)]

#![cfg_attr(nightly, feature(proc_macro_span))]

use proc_macro2::Delimiter;
use proc_macro2::LineColumn;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::fs;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

// =================
// === Constants ===
// =================

/// Set to 'true' to enable debug prints.
const DEBUG: bool = false;

const THIS_CRATE_NAME: &str = "eval_macro";

const OUTPUT_PREFIX: &str = "OUTPUT:";
const WARNING_PREFIX: &str = "WARNING:";
const ERROR_PREFIX: &str = "ERROR:";

/// Rust keywords for special handling. This is not needed for this macro to work, it is only used
/// to make `IntelliJ` / `RustRover` work correctly, as their `TokenStream` spans are incorrect.
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

/// Common functions. After this lib stabilizes, they should be imported from this library, not
/// injected.
const PRELUDE: &str = "
    macro_rules! write_ln {
        ($target:ident, $($ts:tt)*) => {
            $target.push_str(&format!( $($ts)* ));
            $target.push_str(\"\n\");
        };
    }

    macro_rules! println_output {
        ($($ts:tt)*) => {
            println!(\"{}\", prefix_lines_with_output(&format!( $($ts)* )));
        };
    }

    macro_rules! println_warning {
        ($($ts:tt)*) => {
            println!(\"{}\", prefix_lines_with_warning(&format!( $($ts)* )));
        };
    }

    macro_rules! println_error {
        ($($ts:tt)*) => {
            println!(\"{}\", prefix_lines_with_error(&format!( $($ts)* )));
        };
    }

    fn prefix_lines_with(prefix: &str, input: &str) -> String {
        input
            .lines()
            .map(|line| format!(\"{prefix}: {line}\"))
            .collect::<Vec<_>>()
            .join(\"\\n\")
    }

    fn prefix_lines_with_output(input: &str) -> String {
        prefix_lines_with(\"OUTPUT\", input)
    }

    fn prefix_lines_with_warning(input: &str) -> String {
        prefix_lines_with(\"WARNING\", input)
    }

    fn prefix_lines_with_error(input: &str) -> String {
        prefix_lines_with(\"ERROR\", input)
    }

    fn sum_combinations(n: usize) -> Vec<Vec<usize>> {
        let mut result = Vec::new();

        fn generate(n: usize, current: Vec<usize>, result: &mut Vec<Vec<usize>>) {
            if n == 0 {
                if current.len() > 1 {
                    result.push(current);
                }
                return;
            }

            for i in 1..=n {
                let mut next = current.clone();
                next.push(i);
                generate(n - i, next, result);
            }
        }

        generate(n, vec![], &mut result);
        result
    }

    fn push_as_str<T: std::fmt::Debug>(str: &mut String, value: &T) {
        let repr = format!(\"{value:?}\");
        if repr != \"()\" {
            if repr.starts_with(\"(\") && repr.ends_with(\")\") {
                str.push_str(&repr[1..repr.len() - 1]);
            } else {
                str.push_str(&repr);
            }
        }
    }
";

// ============
// === Path ===
// ============

fn create_empty_file_with_dirs(path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;  // Create parent directories if needed
    }

    // Create (or truncate) the file itself
    File::create(path)?;

    Ok(())
}

#[cfg(nightly)]
fn get_output_dir(macro_name: &str, _input_str: &str) -> PathBuf {
    let mut call_site_path = proc_macro::Span::call_site().source_file().path();
    call_site_path.set_extension("");
    let crate_out_dir_str = std::env::var("OUT_DIR").unwrap();
    let crate_out_dir = Path::new(&crate_out_dir_str);

    let build_dir_name = std::ffi::OsStr::new("build");
    let build_dir = crate_out_dir.ancestors().find(|p| p.file_name() == Some(build_dir_name)).unwrap();
    build_dir.join(THIS_CRATE_NAME).join(call_site_path).join(macro_name)
}

#[cfg(not(nightly))]
fn get_output_dir(_macro_name: &str, input_str: &str) -> PathBuf {
    let home_dir = std::env::var("HOME")
        .expect("HOME environment variable not set — this is required to locate ~/.cargo.");

    let eval_macro_dir = PathBuf::from(home_dir)
        .join(".cargo")
        .join(THIS_CRATE_NAME);

    let project_name = project_name_from_input(input_str);
    eval_macro_dir.join(&project_name)
}

/// Output directory for projects generated by this macro.
fn create_output_dir(name: &str, input_str: &str) -> PathBuf {
    let output_dir = get_output_dir(name, input_str);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)
            .expect("Failed to create project directory.");
    }
    output_dir
}

fn with_output_dir<T>(name: &str, input_str: &str, f: impl FnOnce(PathBuf) -> T) -> T {
    let output_dir = get_output_dir(name, input_str);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)
            .expect("Failed to create project directory.");
    }
    let out = f(output_dir);
    // We cache projects on nightly. On stable, the project name is based on the input string.
    #[cfg(not(nightly))]
    fs::remove_dir_all(&output_dir).ok();
    out
}

// ==========================
// === Project Management ===
// ==========================

#[derive(Debug, Default)]
struct ProjectConfig {
    cargo: CargoConfig,
    lib: LibConfig,
}

#[derive(Debug, Default)]
struct CargoConfig {
    edition: Option<String>,
    resolver: Option<String>,
    dependencies: Vec<String>,
}

#[derive(Debug, Default)]
struct LibConfig {
    features: Vec<String>,
    allow: Vec<String>,
    expect: Vec<String>,
    warn: Vec<String>,
    deny: Vec<String>,
    forbid: Vec<String>,
}

impl CargoConfig {
    fn print(&self) -> String {
        let edition = self.edition.as_ref().map_or("2024", |t| t.as_str());
        let resolver = self.resolver.as_ref().map_or("3", |t| t.as_str());
        let dependencies = self.dependencies.join("\n");
        format!("
            [workspace]
            [package]
            name     = \"eval_project\"
            version  = \"1.0.0\"
            edition  = \"{edition}\"
            resolver = \"{resolver}\"

            [dependencies]
            {dependencies}
        ")
    }
}

impl LibConfig {
    fn print(&self) -> String {
        let mut out = vec![];
        out.extend(self.features.iter().map(|t| format!("#![feature({})]", t)));
        out.extend(self.allow.iter().map(|t| format!("#![allow({})]", t)));
        out.extend(self.expect.iter().map(|t| format!("#![expect({})]", t)));
        out.extend(self.warn.iter().map(|t| format!("#![warn({})]", t)));
        out.extend(self.deny.iter().map(|t| format!("#![deny({})]", t)));
        out.extend(self.forbid.iter().map(|t| format!("#![forbid({})]", t)));
        out.join("\n")
    }
}

fn project_name_from_input(input_str: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input_str.hash(&mut hasher);
    format!("project_{:016x}", hasher.finish())
}

fn create_project_skeleton(project_dir: &Path, cfg: ProjectConfig, main_content: &str) {
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir).expect("Failed to create src directory.");
    }

    let cargo_toml = project_dir.join("Cargo.toml");
    let cargo_toml_content = cfg.cargo.print();
    fs::write(&cargo_toml, cargo_toml_content).expect("Failed to write Cargo.toml.");

    let main_rs = src_dir.join("main.rs");
    let mut file = File::create(&main_rs).expect("Failed to create main.rs");
    file.write_all(main_content.as_bytes()).expect("Failed to write main.rs");
}

fn get_host_target() -> String {
    let output = Command::new("rustc")
        .arg("-vV")
        .stdout(std::process::Stdio::piped())
        .output()
        .expect("Failed to run rustc");

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("host:") {
            return line["host:".len()..].trim().to_string();
        }
    }
    panic!("Could not determine host target from rustc");
}

fn run_cargo_project(project_dir: &PathBuf) -> String {
    // In case the project uses .cargo/config.toml, we need to explicitly revert target to native.
    let host_target = get_host_target();
    let output = Command::new("cargo")
        .arg("run")
        .arg("--target")
        .arg(&host_target)
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute cargo run");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{stderr}");
        panic!("Cargo project failed to compile or run.");
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}

// ===================================
// === Top-level Attribute Parsing ===
// ===================================

fn extract_dependencies(cfg: &mut ProjectConfig, tokens: TokenStream) -> TokenStream {
    let (tokens2, attributes) = extract_top_level_attributes(tokens);
    for attr in attributes {
        if let Some(pos) = attr.find('(') {
            let name = &attr[..pos];
            let value = (&attr[pos + 1.. attr.len() - 1]).to_string(); // Skipping ")".
            match name {
                "dependency" => cfg.cargo.dependencies.push(value),
                "edition" => {
                    if cfg.cargo.edition.is_some() {
                        panic!("Edition already set.");
                    }
                    cfg.cargo.edition = Some(value);
                },
                "resolver" => {
                    if cfg.cargo.resolver.is_some() {
                        panic!("Resolver already set.");
                    }
                    cfg.cargo.resolver = Some(value);
                },

                "feature" => cfg.lib.features.push(value),
                "allow" => cfg.lib.allow.push(value),
                "expect" => cfg.lib.expect.push(value),
                "warn" => cfg.lib.warn.push(value),
                "deny" => cfg.lib.deny.push(value),
                "forbid" => cfg.lib.forbid.push(value),
                _ => panic!("Invalid attribute: {attr}"),
            }
        } else {
            panic!("Invalid attribute: {attr}");
        }
    }
    tokens2
}

/// Extract all top-level attributes (`#![...]`) from the input `TokenStream` and return the
/// remaining `TokenStream`.
fn extract_top_level_attributes(tokens: TokenStream) -> (TokenStream, Vec<String>) {
    let mut output = TokenStream::new();
    let mut attributes = Vec::new();
    let mut iter = tokens.into_iter().peekable();
    while let Some(_token) = iter.peek() {
        if let Some(dep) = try_parse_inner_attr(&mut iter) {
            attributes.push(dep);
        } else if let Some(token) = iter.next() {
            output.extend(Some(token));
        }
    }
    (output, attributes)
}

/// Try to parse `#![...]` as an inner attribute and return the parsed content if successful.
fn try_parse_inner_attr(iter: &mut std::iter::Peekable<impl Iterator<Item = TokenTree>>) -> Option<String> {
    // Check for '#'.
    let Some(TokenTree::Punct(pound)) = iter.peek() else { return None; };
    if pound.as_char() != '#' { return None; }
    iter.next();

    // Check for '!'.
    let Some(TokenTree::Punct(bang)) = iter.peek() else { return None; };
    if bang.as_char() != '!' { return None; }
    iter.next();

    // Check for [ ... ] group.
    let Some(TokenTree::Group(group)) = iter.peek() else { return None; };
    if group.delimiter() != Delimiter::Bracket { return None; }
    let content = group.stream().to_string();
    iter.next();

    Some(content)
}

// ====================
// === Output Macro ===
// ====================

/// Find and expand the `output!` macro in the input `TokenStream`. After this lib stabilizes, this
/// should be rewritten to standard macro and imported by the generated code.
fn expand_output_macro(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut output = TokenStream::new();
    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Ident(ref ident) = tokens[i] {
            if *ident == "output" && i + 1 < tokens.len() {
                if let TokenTree::Punct(ref excl) = tokens[i + 1] {
                    if excl.as_char() == '!' && i + 2 < tokens.len() {
                        if let TokenTree::Group(ref group) = tokens[i + 2] {
                            let inner_rewritten = expand_output_macro(group.stream());
                            let content_str = print(&inner_rewritten);
                            let lit = syn::LitStr::new(&content_str, proc_macro2::Span::call_site());
                            let new_tokens = quote! { write_ln!(output_buffer, #lit); };
                            output.extend(new_tokens);
                            i += 3;
                            continue;
                        }
                    }
                }
            }
        }
        match &tokens[i] {
            TokenTree::Group(group) => {
                let new_stream = expand_output_macro(group.stream());
                let new_group = TokenTree::Group(proc_macro2::Group::new(group.delimiter(), new_stream));
                output.extend(std::iter::once(new_group));
            }
            _ => {
                output.extend(std::iter::once(tokens[i].clone()));
            }
        }
        i += 1;
    }
    output
}

// =============
// === Print ===
// =============

#[derive(Debug)]
struct PrintOutput {
    output: String,
    start_token: Option<LineColumn>,
    end_token: Option<LineColumn>,
}

// Used to indicate that `{{` and `}}` was already collapsed to `{` and `}`. So, for example, the
// following transformations will be performed:
// `{ {{a}} }` -> `{ {{{a}}} }` -> `{ %%%{a}%%% }` -> `{{ %%%{a}%%% }}` -> `{{ {a} }}`
const SPACER: &str = "%%%";

/// Prints the token stream as a string ready to be used by the format macro. The following
/// transformations are performed:
/// - A trailing space is added after printing each token.
/// - If `prev_token.line == next_token.line` and `prev_token.end_column >= next_token.start_column`,
///   the prev token trailing space is removed. This basically preserves spaces from the input code,
///   which is needed to distinguish between such inputs as `MyName{x}`, `MyName {x}`, etc.
///   The `>=` is used because sometimes a few tokens can have the same start column. For example,
///   for lifetimes (`'t`), the apostrophe and the lifetime ident have the same start column.
/// - Every occurrence of `{{` and `}}` is replaced with `{` and `}` respectively. Also, every
///   occurrence of `{` and `}` is replaced with `{{` and `}}` respectively. This prepares the
///   string to be used in the format macro.
///
/// There is also a special transformation for `IntellIJ` / `RustRover`. Their spans are different from
/// rustc ones, so sometimes tokens are glued together. This is why we discover Rust keywords and add
/// additional spacing around. This is probably not covering all the bugs. If there will be a bug
/// report, this is the place to look at.
fn print(tokens: &TokenStream) -> String {
    print_internal(tokens).output.replace("{%%%", "{ %%%").replace("%%%}", "%%% }").replace(SPACER, "")
}

fn print_internal(tokens: &TokenStream) -> PrintOutput {
    let token_vec: Vec<TokenTree> = tokens.clone().into_iter().collect();
    let mut output = String::new();
    let mut first_token_start = None;
    let mut prev_token_end: Option<LineColumn> = None;
    for (i, token) in token_vec.iter().enumerate() {
        let mut token_start = token.span().start();
        let mut token_end = token.span().end();
        let mut is_keyword = false;
        let token_str = match token {
            TokenTree::Group(g) => {
                let content = print_internal(&g.stream());
                let mut content_str = content.output;
                content_str.pop();
                let (open, close) = match g.delimiter() {
                    Delimiter::Brace =>{
                        if content_str.starts_with('{') && content_str.ends_with('}') {
                            // We already replaced the internal `{` and `}` with `{{` and `}}`.
                            content_str.pop();
                            content_str.remove(0);
                            (SPACER, SPACER)
                        } else {
                            ("{{", "}}")
                        }
                    },
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Bracket => ("[", "]"),
                    _ => ("", ""),
                };

                if let Some(content_first_token_start) = content.start_token {
                    token_start.line = content_first_token_start.line;
                    if content_first_token_start.column > 0 {
                        token_start.column = content_first_token_start.column - 1;
                    }
                }
                if let Some(content_end) = content.end_token {
                    token_end.line = content_end.line;
                    token_end.column = content_end.column + 1;
                }
                format!("{open}{content_str}{close}")
            }
            TokenTree::Ident(ident) => {
                let str = ident.to_string();
                is_keyword = KEYWORDS.contains(&str.as_str());
                str
            },
            TokenTree::Literal(lit) => lit.to_string(),
            TokenTree::Punct(punct) => punct.as_char().to_string(),
        };
        if DEBUG {
            println!("{i}: [{token_start:?}-{token_end:?}] [{prev_token_end:?}]: {token}");
        }
        if let Some(prev_token_end) = prev_token_end {
            if prev_token_end.line == token_start.line && prev_token_end.column >= token_start.column {
                output.pop();
            }
        }

        // Pushing a space before and after keywords is for IntelliJ only. Their token spans are invalid.
        if is_keyword { output.push(' '); }
        output.push_str(&token_str);
        output.push(' ');
        if is_keyword { output.push(' '); }

        first_token_start.get_or_insert(token_start);
        prev_token_end = Some(token_end);
    }
    PrintOutput {
        output,
        start_token: first_token_start,
        end_token: prev_token_end,
    }
}

// ==================
// === Eval Macro ===
// ==================

#[proc_macro]
pub fn eval(input_raw: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut cfg = ProjectConfig::default();
    let input = extract_dependencies(&mut cfg, input_raw.into());

    let input_str = expand_output_macro(input).to_string();
    let input_str_esc: String = input_str.chars().flat_map(|c| c.escape_default()).collect();
    if DEBUG { println!("REWRITTEN INPUT: {input_str}"); }


    let attrs = cfg.lib.print();
    let main_content = format!(
        "{attrs}
        {PRELUDE}

        const SOURCE_CODE: &str = \"{input_str_esc}\";

        fn main() {{
            let mut output_buffer = String::new();
            let result = {{
                {input_str}
            }};
            push_as_str(&mut output_buffer, &result);
            println!(\"{{}}\", prefix_lines_with_output(&output_buffer));
        }}",
    );

    let project_dir = create_output_dir("foo", &input_str);
    create_project_skeleton(&project_dir, cfg, &main_content);
    let output = run_cargo_project(&project_dir);
    fs::remove_dir_all(&project_dir).ok();
    let mut code = String::new();
    for line in output.split('\n') {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with(OUTPUT_PREFIX) {
            code.push_str(&line_trimmed[OUTPUT_PREFIX.len()..]);
            code.push('\n');
        } else if line_trimmed.starts_with(WARNING_PREFIX) {
            println!("[WARNING] {}", &line_trimmed[WARNING_PREFIX.len()..]);
        } else if line_trimmed.starts_with(ERROR_PREFIX) {
            println!("[ERROR] {}", &line_trimmed[ERROR_PREFIX.len()..]);
        } else if line_trimmed.len() > 0 {
            println!("{line}");
        }
    }

    let out: TokenStream = code.parse().expect("Failed to parse generated code.");
    if DEBUG {
        println!("OUT: {out}");
    }
    out.into()
}

#[proc_macro_attribute]
pub fn eval2(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut cfg = ProjectConfig::default();
    let input_fn = syn::parse_macro_input!(item as syn::ItemFn);
    let name = &input_fn.sig.ident.to_string();
    let args = &input_fn.sig.inputs;
    let body = &input_fn.block.stmts;
    let attrs = &input_fn.attrs;
    let wrong_args = "Function should have at most one argument of the form of `pattern!(<pattern>):_`, where <pattern> is a `macro_rules!` pattern.";
    if args.len() > 1 {
        panic!("{}", wrong_args);
    }
    let pattern = args.iter().next().map(|t| {
        if let syn::FnArg::Typed(pat) = t {
            if let syn::Pat::Macro(m) = &*pat.pat {
                let tokens = &m.mac.tokens;
                let str = tokens.to_string();
                str
            } else {
                panic!("{}", wrong_args);
            }
        } else {
            panic!("{}", wrong_args);
        }
    }).unwrap_or_else(|| String::new());

    let body_tokens = quote! { #(#body)* };
    let input_str = expand_output_macro(body_tokens).to_string();
    let input_str_esc: String = input_str.chars().flat_map(|c| c.escape_default()).collect();
    if DEBUG { println!("REWRITTEN INPUT: {input_str}"); }



    let attrs = cfg.lib.print();
    let main_content = format!(
        "{attrs}
        {PRELUDE}

        const SOURCE_CODE: &str = \"{input_str_esc}\";

        fn main() {{
            let mut output_buffer = String::new();
            let result = {{
                {input_str}
            }};
            push_as_str(&mut output_buffer, &result);
            println!(\"{{}}\", prefix_lines_with_output(&output_buffer));
        }}",
    );

    let output = with_output_dir(name, &input_str, |output_dir| {
        println!("OUTPUT_DIR: {:?}", output_dir);
        create_project_skeleton(&output_dir, cfg, &main_content);
        run_cargo_project(&output_dir)
    });

    let mut code = String::new();
    for line in output.split('\n') {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with(OUTPUT_PREFIX) {
            code.push_str(&line_trimmed[OUTPUT_PREFIX.len()..]);
            code.push('\n');
        } else if line_trimmed.starts_with(WARNING_PREFIX) {
            println!("[WARNING] {}", &line_trimmed[WARNING_PREFIX.len()..]);
        } else if line_trimmed.starts_with(ERROR_PREFIX) {
            println!("[ERROR] {}", &line_trimmed[ERROR_PREFIX.len()..]);
        } else if line_trimmed.len() > 0 {
            println!("{line}");
        }
    }

    let code_out: TokenStream = code.parse().expect("Failed to parse generated code.");
    let macro_code = format!("
        macro_rules! {name} {{
            ({pattern}) => {{
               {code}
            }}
        }}
    ");


    // if DEBUG {
        println!("BODY: {macro_code}");
    // }

    let out: TokenStream = macro_code.parse().expect("Failed to parse generated code.");
    println!("OUTPUT : {out}");
    out.into()
}
