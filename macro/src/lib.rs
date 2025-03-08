#![cfg_attr(nightly, feature(proc_macro_span))]
#![feature(proc_macro_diagnostic)]

mod error;

use std::fmt::Debug;
use proc_macro2::Delimiter;
use proc_macro2::LineColumn;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::ToTokens;
use quote::quote;
use std::fs::File;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[cfg(not(nightly))]
mod non_nightly_deps {
    pub use std::collections::hash_map::DefaultHasher;
    pub use std::hash::Hash;
    pub use std::hash::Hasher;
}
#[cfg(not(nightly))]
use non_nightly_deps::*;

use error::*;

// =================
// === Constants ===
// =================

/// Set to 'true' to enable debug prints.
const DEBUG: bool = false;

const CRATE: &str = "crabtime";
const DEFAULT_EDITION: &str = "2024";
const DEFAULT_RESOLVER: &str = "3";
const GEN_MOD: &str = CRATE;
const OUTPUT_PREFIX: &str = "[OUTPUT]";

/// Rust keywords for special handling. This is not needed for this macro to work, it is only used
/// to make `IntelliJ` / `RustRover` work correctly, as their `TokenStream` spans are incorrect.
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

const OUT_DIR: &str = env!("OUT_DIR");

// ==================
// === TokenRange ===
// ==================

#[derive(Debug)]
struct TokenRange {
    start: TokenTree,
    end: TokenTree,
}

impl TokenRange {
    fn new(start: TokenTree, end: TokenTree) -> Self {
        Self { start, end }
    }

    #[cfg(nightly)]
    fn span(&self) -> Span {
        let first_span = self.start.span();
        let last_span = self.end.span();
        first_span.join(last_span).unwrap_or_else(|| first_span)
    }
}

// ==============================
// === Generated Code Prelude ===
// ==============================

fn gen_prelude() -> String {
    let warning_prefix = Level::WARNING_PREFIX;
    let error_prefix = Level::ERROR_PREFIX;
    format!("
        mod {GEN_MOD} {{
            #![allow(clippy::all)]

            const OUTPUT_PREFIX: &'static str = \"{OUTPUT_PREFIX}\";
            const WARNING_PREFIX: &'static str = \"{warning_prefix}\";
            const ERROR_PREFIX: &'static str = \"{error_prefix}\";

            macro_rules! output_str {{
                ($($ts:tt)*) => {{
                    println!(\"{{}}\", {GEN_MOD}::prefix_lines_with_output(&format!($($ts)*)));
                }};
            }}
            pub(super) use output_str;

            macro_rules! warning {{
                ($($ts:tt)*) => {{
                    println!(\"{{}}\", {GEN_MOD}::prefix_lines_with_warning(&format!($($ts)*)));
                }};
            }}
            pub(super) use warning;

            macro_rules! error {{
                ($($ts:tt)*) => {{
                    println!(\"{{}}\", {GEN_MOD}::prefix_lines_with_error(&format!($($ts)*)));
                }};
            }}
            pub(super) use error;

            {PRELUDE_STATIC}
        }}

        {PRELUDE_MAGIC}
    ")
}

const PRELUDE_STATIC: &str = "
    pub(super) fn push_as_str<T: std::fmt::Debug>(str: &mut String, value: &T) {
        let repr = format!(\"{value:?}\");
        if repr != \"()\" {
            if repr.starts_with(\"(\") && repr.ends_with(\")\") {
                str.push_str(&repr[1..repr.len() - 1]);
            } else {
                str.push_str(&repr);
            }
        }
    }

    pub(super) fn prefix_lines_with(prefix: &str, input: &str) -> String {
        input
            .lines()
            .map(|line| format!(\"{prefix} {line}\"))
            .collect::<Vec<_>>()
            .join(\"\\n\")
    }

    pub(super) fn prefix_lines_with_output(input: &str) -> String {
        prefix_lines_with(OUTPUT_PREFIX, input)
    }

    pub(super) fn prefix_lines_with_warning(input: &str) -> String {
        prefix_lines_with(WARNING_PREFIX, input)
    }

    pub(super) fn prefix_lines_with_error(input: &str) -> String {
        prefix_lines_with(ERROR_PREFIX, input)
    }

    macro_rules! write_ln {
        ($target:expr, $($ts:tt)*) => {
            $target.push_str(&format!( $($ts)* ));
            $target.push_str(\"\n\");
        };
    }
    pub(super) use write_ln;
";

/// To be removed one day.
const PRELUDE_MAGIC: &str = "
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
";

// =============
// === Paths ===
// =============

#[derive(Debug)]
struct Paths {
    output_dir: PathBuf,
    /// None if we are on stable.
    cargo_toml_path: Option<CargoConfigPaths>,
}

fn parent_dir(path: &Path) -> Result<&Path> {
    path.parent().context(|| error!("Path '{}' does not have a parent.", path.display()))
}

fn find_parent_dir<'t>(path: &'t Path, dir_name: &str) -> Result<&'t Path> {
    let dir_name_os = std::ffi::OsStr::new(dir_name);
    path.ancestors()
        .find(|p| p.file_name() == Some(dir_name_os))
        .context(|| error!(
            "Path '{}' does not have parent '{dir_name}' directory.",
            path.display()
        ))
}

impl Paths {
    #[cfg(nightly)]
    fn new(macro_name: &str, _input_str: &str) -> Result<Self> {
        let mut call_site_path = proc_macro::Span::call_site().source_file().path();
        call_site_path.set_extension("");
        let crate_out_str = OUT_DIR;
        let crate_out = Path::new(&crate_out_str);
        let target = find_parent_dir(crate_out, "target")?;
        let workspace = parent_dir(target)?;
        let file_path = workspace.join(&call_site_path);
        let cargo_toml_path = Some(find_cargo_configs(&file_path)?);
        let build_dir = find_parent_dir(crate_out, "build")?;
        let output_dir = build_dir.join(CRATE).join(call_site_path).join(macro_name);
        Ok(Self { output_dir, cargo_toml_path })
    }

    #[cfg(not(nightly))]
    fn new(_macro_name: &str, input_str: &str) -> Result<Self> {
        let home_dir = std::env::var("HOME").context("$HOME not set")?;
        let eval_macro_dir = PathBuf::from(home_dir).join(".cargo").join(CRATE);
        let project_name = project_name_from_input(input_str);
        let output_dir = eval_macro_dir.join(&project_name);
        let cargo_toml_path = None;
        Ok(Self { output_dir, cargo_toml_path: None })
    }

    fn with_output_dir<T>(&self, f: impl FnOnce(&PathBuf) -> Result<T>) -> Result<T> {
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir)
                .context(|| error!("Failed to create project directory."))?;
        }
        let out = f(&self.output_dir);
        // We cache projects on nightly. On stable, the project name is based on the input code.
        #[cfg(not(nightly))]
        fs::remove_dir_all(&self.output_dir).ok();
        out
    }
}

#[cfg(not(nightly))]
fn project_name_from_input(input_str: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input_str.hash(&mut hasher);
    format!("project_{:016x}", hasher.finish())
}

// ========================
// === CargoConfigPaths ===
// ========================

#[derive(Debug)]
struct CargoConfigPaths {
    crate_config: PathBuf,
    workspace_config: Option<PathBuf>,
}

fn find_cargo_configs(path: &PathBuf) -> Result<CargoConfigPaths> {
    let mut current_path = path.clone();
    let mut out = Vec::new();
    loop {
        let candidate = current_path.join("Cargo.toml");
        if candidate.is_file() { out.push(candidate) }
        if !current_path.pop() { break }
    }
    if out.len() >= 2 {
        Ok(CargoConfigPaths {
            crate_config: out[0].clone(),
            workspace_config: Some(out[1].clone()),
        })
    } else if out.len() >= 1 {
        Ok(CargoConfigPaths {
            crate_config: out[0].clone(),
            workspace_config: None,
        })
    } else {
        err!("No 'Cargo.toml' files found in parent directories of '{}'.", path.display())
    }
}

// ===================
// === CargoConfig ===
// ===================

#[derive(Debug)]
struct Dependency {
    tokens_str: String,
    token_range: Option<TokenRange>,
}

impl Dependency {
    fn new(tokens_str: String, token_range: Option<TokenRange>) -> Self {
        Self { tokens_str, token_range }
    }

    #[cfg(nightly)]
    fn span(&self) -> Span {
        self.token_range.as_ref().map_or(Span::call_site(), |t| t.span())
    }
}

#[derive(Debug, Default)]
struct CargoConfig {
    edition: Option<String>,
    resolver: Option<String>,
    dependencies: Vec<Dependency>,
}

impl CargoConfig {
    fn print(&self) -> String {
        let edition = self.edition.as_ref().map_or(DEFAULT_EDITION, |t| t.as_str());
        let resolver = self.resolver.as_ref().map_or(DEFAULT_RESOLVER, |t| t.as_str());
        let dependencies = self.dependencies.iter()
            .map(|t| t.tokens_str.clone())
            .collect::<Vec<_>>()
            .join("\n");
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

    fn fill_from_cargo_toml(&mut self, cargo_config_paths: &CargoConfigPaths) -> Result {
        let cargo_toml_content = fs::read_to_string(&cargo_config_paths.crate_config)?;
        let parsed: toml::Value = toml::from_str(&cargo_toml_content)?;
        let dependencies = parsed
            .get("build-dependencies")
            .and_then(|v| v.as_table())
            .map_or(vec![], |t| t.iter().map(|(k, v)| format!("{k} = {v}")).collect());
        let edition = parsed
            .get("package")
            .and_then(|v| v.as_table())
            .and_then(|table| table.get("edition"))
            .and_then(|v| v.as_str())
            .unwrap_or("2024");
        self.dependencies.extend(dependencies.into_iter().map(|t| Dependency::new(t, None)));
        self.edition = Some(edition.to_string());
        Ok(())
    }

    fn extract_inline_attributes(&mut self, attributes: Vec<syn::Attribute>) -> Result<String> {
        let mut other_attributes = Vec::with_capacity(attributes.len());
        let mut new_dependencies = vec![];
        for attr in attributes {
            let tokens = attr.parse_args::<TokenStream>().context(|| error!("Failed to parse attributes"))?;
            let tokens_str = tokens.to_string().replace(" ", "");
            let token_range = tokens.clone().into_iter().next()
                .zip(tokens.clone().into_iter().last())
                .map(|(first, last)| TokenRange::new(first, last));
            if attr.path().is_ident("dependency") {
                new_dependencies.push(Dependency::new(tokens_str, token_range));
            } else if attr.path().is_ident("edition") {
                self.edition = Some(tokens_str);
            } else {
                other_attributes.push(attr.to_token_stream().to_string());
            }
        }
        #[cfg(nightly)]
        for dependency in &new_dependencies {
            warning!(dependency.span(),
                "When using the nightly Rust channel, dependencies should be specified in the \
                [build-dependencies] section of your Cargo.toml file."
            ).emit();
        }
        self.dependencies.extend(new_dependencies);
        Ok(other_attributes.join("\n"))
    }
}

fn create_project_skeleton(project_dir: &Path, cfg: CargoConfig, main_content: &str) -> Result {
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir).context(|| error!("Failed to create src directory."))?;
    }

    let cargo_toml = project_dir.join("Cargo.toml");
    let cargo_toml_content = cfg.print();
    fs::write(&cargo_toml, cargo_toml_content).context(|| error!("Failed to write Cargo.toml."))?;

    let main_rs = src_dir.join("main.rs");
    let mut file = File::create(&main_rs).context(|| error!("Failed to create main.rs"))?;
    file.write_all(main_content.as_bytes()).context(|| error!("Failed to write main.rs"))
}

fn get_host_target() -> Result<String> {
    let output = Command::new("rustc")
        .arg("-vV")
        .stdout(std::process::Stdio::piped())
        .output()
        .context(|| error!("Failed to run rustc"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("host:") {
            return Ok(line["host:".len()..].trim().to_string());
        }
    }
    err!("Could not determine host target from rustc")
}

fn run_cargo_project(project_dir: &PathBuf) -> Result<String> {
    // In case the project uses .cargo/config.toml, we need to explicitly revert target to native.
    let host_target = get_host_target()?;
    let output = Command::new("cargo")
        .arg("run")
        .arg("--target")
        .arg(&host_target)
        .current_dir(project_dir)
        .output()
        .context(|| error!("Failed to execute cargo run"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // eprintln!("{stderr}"); // FIXME: Do we need that?
        err!("Cargo project failed to compile or run: {stderr}")
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

// ====================
// === Output Macro ===
// ====================

/// Find and expand the `output!` macro in the input `TokenStream`. After this lib stabilizes, this
/// should be rewritten to standard macro and imported by the generated code.
fn expand_output_macro(input: TokenStream) -> TokenStream {
    let gen_mod = syn::Ident::new(GEN_MOD, Span::call_site());
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
                            let content_str = print_tokens(&inner_rewritten);
                            let lit = syn::LitStr::new(&content_str, Span::call_site());
                            let new_tokens = quote! { #gen_mod::write_ln!(__output_buffer__, #lit); };
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
fn print_tokens(tokens: &TokenStream) -> String {
    print_tokens_internal(tokens).output.replace("{%%%", "{ %%%").replace("%%%}", "%%% }").replace(SPACER, "")
}

fn print_tokens_internal(tokens: &TokenStream) -> PrintOutput {
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
                let content = print_tokens_internal(&g.stream());
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
        debug!("{i}: [{token_start:?}-{token_end:?}] [{prev_token_end:?}]: {token}");
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


fn extract_pattern(
    args: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>
) -> Option<String> {
    let Some(arg) = args.first() else { return Some(String::new()) };
    let syn::FnArg::Typed(pat) = arg else { return None };
    let syn::Pat::Macro(m) = &*pat.pat else { return None };
    Some(m.mac.tokens.to_string())
}

const WRONG_ARGS: &str = "Function should have at most one argument of the form \
    `pattern!(<pattern>):_`, where <pattern> is a `macro_rules!` pattern.";


fn prepare_input_code(attributes:&str, body: &str) -> String {
    let body_esc: String = body.chars().flat_map(|c| c.escape_default()).collect();
    let prelude = gen_prelude();
    format!("
        {attributes}
        {prelude}

        const SOURCE_CODE: &str = \"{body_esc}\";

        fn main() {{
            let mut __output_buffer__ = String::new();
            let result = {{
                {body}
            }};
            {GEN_MOD}::push_as_str(&mut __output_buffer__, &result);
            println!(\"{{}}\", {GEN_MOD}::prefix_lines_with_output(&__output_buffer__));
        }}",
    )
}

fn parse_output(output: &str) -> String {
    let mut code = String::new();
    for line in output.split('\n') {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with(OUTPUT_PREFIX) {
            code.push_str(&line_trimmed[OUTPUT_PREFIX.len()..]);
            code.push('\n');
        } else if line_trimmed.starts_with(Level::WARNING_PREFIX) {
            print_warning!("{}", &line_trimmed[Level::WARNING_PREFIX.len()..]);
        } else if line_trimmed.starts_with(Level::ERROR_PREFIX) {
            print_error!("{}", &line_trimmed[Level::ERROR_PREFIX.len()..]);
        } else if line_trimmed.len() > 0 {
            println!("{line}");
        }
    }
    code
}

#[proc_macro_attribute]
pub fn eval(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    // SAFETY: Used to panic in case of error.
    #[allow(clippy::unwrap_used)]
    eval_impl(attr, item).unwrap_or_compile_error().into()
}

fn eval_impl(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> Result<TokenStream> {
    let input_fn = syn::parse::<syn::ItemFn>(item)?;
    let name = &input_fn.sig.ident.to_string();
    let args = &input_fn.sig.inputs;
    let body = &input_fn.block.stmts;

    if args.len() > 1 { return err!("{WRONG_ARGS}"); }

    let mut cfg = CargoConfig::default();

    let input_pattern = extract_pattern(&args).context(|| error!("{}", WRONG_ARGS))?;
    let input_str = expand_output_macro(quote!{ #(#body)* }).to_string();
    let paths = Paths::new(name, &input_str)?;

    if let Some(path) = &paths.cargo_toml_path {
        cfg.fill_from_cargo_toml(path)?;
    }
    let attributes = cfg.extract_inline_attributes(input_fn.attrs)?;

    debug!("REWRITTEN INPUT: {input_str}");

    let input_code = prepare_input_code(&attributes, &input_str);
    let output = paths.with_output_dir(|output_dir| {
        debug!("OUTPUT_DIR: {:?}", output_dir);
        create_project_skeleton(&output_dir, cfg, &input_code)?;
        run_cargo_project(&output_dir)
    })?;

    let output_code = parse_output(&output);
    let macro_code = format!("
        macro_rules! {name} {{
            ({input_pattern}) => {{
               {output_code}
            }}
        }}
    ");

    debug!("BODY: {macro_code}");

    let out: TokenStream = macro_code.parse()
        .map_err(|err| error!("{err:?}"))
        .context(|| error!("Failed to parse generated code."))?;
    debug!("OUTPUT : {out}");
    Ok(out)
}


// TODO: get lints from Cargo
// TODO: support workspaces, for edition and dependencies or is it done automatically for edition?

// TODO:
//     crabtime::rules! {
//         test (input: TokenStream) => {
//             // ...
//         }
//     }