//! # 🦀 Crabtime
//!
//! <img width="680" alt="banner" src="https://github.com/user-attachments/assets/d273e38d-951c-4183-b42e-ac5bdf939d69">
//!
//! <br/>
//! <br/>
//!
//! **Crabtime** offers a novel way to write Rust macros, inspired by
//! [Zig's comptime](zigs_comptime). It provides even more flexibility and power than procedural
//! macros, while remaining easier and more natural to read and write than
//! [`macro_rules!`](macro_rules).
//!
//! <br/>
//! <br/>
//!
//! # 🆚 Comparison to Proc Macros and `macro_rules!`
//!
//! Below is a comparison of key aspects of Rust's macro systems:
//!
//! <h5><b>Input/Output</b></h5>
//!
//! | <div style="width:300px"/>                            | Crabtime | Proc Macro | `macro_rules!` |
//! | :---                                                  | :---     | :---       | :---           |
//! | Input as [Token Stream][token_stream]                 | ✅       | ✅         | ❌             |
//! | Input as [Macro Fragments][macro_fragments]           | ✅       | ❌         | ✅             |
//! | Input as Rust Code (String)                           | ✅       | ❌         | ❌             |
//! | Output as [Token Stream][token_stream]                | ✅       | ✅         | ❌             |
//! | Output as [Macro Fragments Template][macro_fragments] | ✅       | ❌         | ✅             |
//! | Output as Rust Code (String)                          | ✅       | ❌         | ❌             |
//!
//! <h5><b>Functionalities</b></h5>
//!
//! | <div style="width:300px"/>                            | Crabtime | Proc Macro | `macro_rules!` |
//! | :---                                                  | :---     | :---       | :---           |
//! | Advanced transformations                              | ✅       | ✅         | ❌             |
//! | [Space-aware interpolation](space_aware_interpolation)| ✅       | ❌         | ❌             |
//! | Can define [fn-like macros][fn_like_macros]           | ✅       | ✅         | ✅             |
//! | Can define [derive macros][derive_macros]             | 🚧       | ✅         | ❌             |
//! | Can define [attribute macros][attribute_macros]       | 🚧       | ✅         | ❌             |
//! | Reusable across modules and crates                    | ✅       | ✅         | ✅             |
//!
//! <h5><b>Comfort of life</b></h5>
//!
//! | <div style="width:300px"/>                            | Crabtime | Proc Macro | `macro_rules!` |
//! | :---                                                  | :---     | :---       | :---           |
//! | Full expansion in IDEs[^supported_ides]               | ✅       | ✅         | ✅             |
//! | Full type hints in IDEs[^supported_ides]              | ✅       | ✅         | ❌             |
//! | Works with [rustfmt][rustfmt]                         | ✅       | ✅         | ❌             |
//! | Easy to define (inline, the same crate)               | ✅       | ❌         | ✅             |
//! | Easy to read                                          | ✅       | ❌         | ⚠️             |
//! | [Hygienic][macro_hygiene]                             | ❌       | ❌         | ✅             |
//!
//! <br/>
//! <br/>
//!
//! # 🎯 One-shot evaluation
//!
//! The simplest and least exciting use of Crabtime is straightforward compile-time code
//! evaluation. To evaluate an expression and paste its output as new code, just use
//! `crabtime::eval`, as shown below:
//!
//! ```
//! const MY_NUM: usize = crabtime::eval! {
//!     (std::f32::consts::PI.sqrt() * 10.0).round() as usize
//! };
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # 🤩 Function-like macros
//!
//! Use the `crabtime::function` attribute to define a new [function-like macro][fn_like_macros].
//! Crabtime will remove the annotated function and replace it with a macro definition of the same
//! name. You can then call the macro to compile and execute the function at build time, and use
//! its output as the generated Rust code. You can also use the standard `#[macro_export]`
//! attribute to export your macro. Let's start with a simple example, and let's refine it down
//! the line. Let's generate the following Rust code:
//!
//! ```
//! enum Position1 { X }
//! enum Position2 { X, Y }
//! enum Position3 { X, Y, Z }
//! enum Position4 { X, Y, Z, W }
//! ```
//!
//! We can do it in this, not very exciting way:
//!
//! ```
//! // Replaces the function definition with a `gen_positions1!` macro.
//! #[crabtime::function]
//! #[macro_export] // <- This is how you export it!
//! fn gen_positions1() -> &str {
//!     "
//!     enum Position1 { X }
//!     enum Position2 { X, Y }
//!     enum Position3 { X, Y, Z }
//!     enum Position4 { X, Y, Z, W }
//!     "
//! }
//!
//! // Compiles and evaluates the gen_positions1 function at build-time and
//! // uses its output as the new code source.
//! gen_positions1!();
//! # fn main() {}
//! ```
//!
//! <br/>
//!
//! <div class="warning">
//! Due to limitations of `macro_rules!` (which `Crabtime` uses under the hood), you must either
//! wrap the  macro call in extra braces when using it in an expression context, or use
//! `crabtime::expression` instead of `crabtime::function`, which does this step for you. For
//! example, if you want to assign the macro’s output to a variable, do this:
//!
//! ```
//! #[crabtime::expression]
//! fn gen_expr() {
//!     let output_num = 3;
//!     crabtime::output! {
//!         {{output_num}}
//!     }
//! }
//!
//! fn test() {
//!     let x = gen_expr!();
//! }
//! ```
//!
//! Alternatively, you can wrap the macro call in extra braces by yourself if you want to use the
//! macro both in expression and statement contexts:
//!
//! ```
//! #[crabtime::function]
//! fn gen_expr() {
//!     let output_num = 3;
//!     crabtime::output! {
//!         {{output_num}}
//!     }
//! }
//!
//! fn test() {
//!     // `let x = gen_expr!{};` would not work here!
//!     let x = { gen_expr!{} };
//! }
//! ```
//!
//! </div>
//!
//! <br/>
//! <br/>
//!
//! # 🤩 Attribute and derive macros
//! Currently, generating [attribute macros][attribute_macros] and [derive macros][derive_macros]
//! is not supported, but there are several ways to achieve it. If you want to help, ping us on
//! [GitHub](https://github.com/wdanilo/crabtime).
//!
//! <br/>
//! <br/>
//!
//! # 📤 Output
//!
//! There are several ways to generate the output code from a Crabtime macro. We recommend you to
//! use either `crabtime::output!` or `crabtime::quote!` macros, as they allow for the most
//! concise, easy-to-understand, and maintainable implementations. Supported input types are
//! described later, for now just ignore them.
//!
//! <br/>
//!
//! <h5><b>Generating output by using <code>crabtime::output!</code></b></h5>
//!
//! The simplest and most recommended way to generate macro output is by using the
//! `crabtime::output!` macro. It allows for space-aware variable interpolation. It's like the
//! `format!` macro, but with inversed rules regarding curly braces – it preserves single braces
//! and uses double braces for interpolation. Please note that it preserves spaces, so
//! `Position {{ix}}` and `Position{{ix}}` mean different things, and the latter will generate
//! `Position1`, `Position2`, etc.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions2(components: Vec<String>) {
//!     for dim in 1 ..= components.len() {
//!         let cons = components[0..dim].join(",");
//!         crabtime::output! {
//!             enum Position{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!     }
//! }
//! gen_positions2!(["X", "Y", "Z", "W"]);
//! # fn main() {}
//! ```
//!
//! <br/>
//!
//! <h5><b>Generating output by using <code>crabtime::quote!</code></b></h5>
//!
//! The `crabtime::quote!` macro is just like `crabtime::output!`, but instead of outputting the
//! code immediately, it returns it (as a `String`), so you can store it in a variable and re-use
//! it across different subsequent calls to `crabtime::quote!` or `crabtime::output!`.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions3(components: Vec<String>) -> String {
//!     let structs = (1 ..= components.len()).map(|dim| {
//!         let cons = components[0..dim].join(",");
//!         crabtime::quote! {
//!             enum Position{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!     }).collect::<Vec<String>>();
//!     structs.join("\n")
//! }
//! gen_positions3!(["X", "Y", "Z", "W"]);
//! # fn main() {}
//! ```
//!
//! <br/>
//!
//! <h5><b>Generating output by returning a string or number</b></h5>
//!
//! You can simply return a string or number from the function. It will be used as the generated
//! macro code.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions4(components: Vec<String>) -> String {
//!     (1 ..= components.len()).map(|dim| {
//!         let cons = components[0..dim].join(",");
//!         format!("enum Position{dim} {{ {cons} }}")
//!     }).collect::<Vec<_>>().join("\n")
//! }
//! gen_positions4!(["X", "Y", "Z", "W"]);
//! # fn main() {}
//! ```
//!
//! <br/>
//!
//! <h5><b>Generating output by using <code>crabtime::output_str!</code></b></h5>
//!
//! Alternatively, you can use the `crabtime::output_str!` macro to immediately write strings to
//! the code output buffer:
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions5(components: Vec<String>) {
//!     for dim in 1 ..= components.len() {
//!         let cons = components[0..dim].join(",");
//!         crabtime::output_str!("enum Position{dim} {{ {cons} }}")
//!     }
//! }
//! gen_positions5!(["X", "Y", "Z", "W"]);
//! # fn main() {}
//! ```
//!
//! <br/>
//!
//! <h5><b>Generating output by returning a <code>TokenStream</code></b></h5>
//!
//! Finally, you can output [TokenStream][token_stream] from the macro. Please note that for
//! brevity the below example uses [inline dependency injection](inline_dependency_injection),
//! which is described later. In real code you should use your `Cargo.toml`'s
//! `[build-dependencies]` section to include the necessary dependencies instead.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions6() -> proc_macro2::TokenStream {
//!     // Inline dependencies used for brevity.
//!     // You should use [build-dependencies] section in your Cargo.toml instead.
//!     #![dependency(proc-macro2 = "1")]
//!     #![dependency(syn = "2")]
//!     #![dependency(quote = "1")]
//!     use proc_macro2::Span;
//!     use quote::quote;
//!
//!     let components = ["X", "Y", "Z", "W"];
//!     let defs = (1 ..= components.len()).map(|dim| {
//!         let cons = components[0..dim].iter().map(|t|
//!             syn::Ident::new(t, Span::call_site())
//!         );
//!         let ident = syn::Ident::new(&format!("Position{dim}"), Span::call_site());
//!         quote! {
//!             enum #ident {
//!                 #(#cons),*
//!             }
//!         }
//!     }).collect::<Vec<_>>();
//!     quote! {
//!         #(#defs)*
//!     }
//! }
//! gen_positions6!();
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # 📥 Input
//!
//! Similarly to generating output, there are several ways to parametrize macros and provide them
//! with input at their call site. We recommend you use the pattern parametrization, as it's the
//! simplest and easiest to maintain.
//!
//! <br/>
//!
//! <h5><b>Input by using supported arguments</b></h5>
//!
//! Currently, you can use any combination of the following types as arguments to your macro and
//! they will be automatically translated to patterns: `Vec<...>`, `&str`, `String`, and numbers.
//! If the expected argument is a string, you can pass either a string literal or an identifier,
//! which will automatically be converted to a string.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions7(name: String, components: Vec<String>) {
//!     for dim in 1 ..= components.len() {
//!         let cons = components[0..dim].join(",");
//!         crabtime::output! {
//!             enum {{name}}{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!     }
//! }
//! gen_positions7!(Position, ["X", "Y", "Z", "W"]);
//! gen_positions7!(Color, ["R", "G", "B"]);
//! # fn main() {}
//! ```
//!
//! <br/>
//!
//! <h5><b>Input by using patterns</b></h5>
//!
//! In case you want even more control, you can use the same patterns as
//! [`macro_rules!`](macro_rules) by using a special `pattern!` macro, and you can expand any pattern
//! using the `expand!` macro:
//!
//! <div style="background-color:#397be440; padding: 8px; border-radius: 8px; margin-bottom: 8px;">
//! 💡 Please note that the <code>expand!</code> macro simply passes its input along. It is used
//! only to make the code within the function a valid Rust code block. Thus, you do not need to use
//! it if you want to expand variables within other macros, like <code>stringify!</code>.
//! </div>
//!
//! ```
//! // Please note that we need to type the pattern argument as `_` to make the
//! // code a valid Rust code.
//! #[crabtime::function]
//! fn gen_positions8(pattern!($name:ident, $components:tt): _) {
//!     let components = expand!($components);
//!     for dim in 1 ..= components.len() {
//!         let cons = components[0..dim].join(",");
//!         // We don't need to use `expand!` here.
//!         let name = stringify!($name);
//!         crabtime::output! {
//!             enum {{name}}{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!     }
//! }
//! gen_positions8!(Position, ["X", "Y", "Z", "W"]);
//! gen_positions8!(Color, ["R", "G", "B"]);
//! # fn main() {}
//! ```
//!
//! <br/>
//!
//! <h5><b>Input by using <code>TokenStream</code></b></h5>
//!
//! Alternatively, you can consume the provided input as a [TokenStream][token_stream]:
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions9(name: TokenStream) {
//!     #![dependency(proc-macro2 = "1")]
//!     let components = ["X", "Y", "Z", "W"];
//!     let name_str = name.to_string();
//!     for dim in 1 ..= components.len() {
//!         let cons = components[0..dim].join(",");
//!         crabtime::output! {
//!             enum {{name_str}}{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!     }
//! }
//! gen_positions9!(Position);
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # 🚀 Performance
//!
//! The lifecycle of a Crabtime macro is similar to that of a procedural macro. It is compiled as a
//! separate crate and then invoked to transform input tokens into output tokens. On the unstable
//! Rust channel, Crabtime and procedural macros have the same performance. On the stable channel,
//! Crabtime requires slightly more time than a procedural macro after you change your macro
//! definition. In other words, Crabtime’s performance is similar to procedural macros. It has
//! higher compilation overhead than [`macro_rules!`](macro_rules) but processes tokens and complex
//! transformations faster.
//!
//! |                                            | <div style="width:200px">Proc Macro</div> | <div style="width:200px">Crabtime</div>               | <div style="width:200px"><code>macro_rules!</code></div> |
//! | :---                                       | :---                                      | :---                                                  | :---                                               |
//! | First evaluation (incl. compilation)       | ⚠️ Relatively slow                        | ⚠️ Relatively slow                                    | ✅ Fast                                            |
//! | Next evaluation (on call-site change)      | ✅ Fast                                   | ✅ Fast on nightly <br/> ⚠️ Relatively slow on stable |  ❌ Slow for complex transformations               |
//! | Cost after changing module code without changing macro-call site code | ✅ Zero        | ✅ Zero                                               | ✅ Zero                                            |
//!
//! <br/>
//!
//! <h5><b>Cache</b></h5>
//!
//! When a Crabtime macro is called, it creates a new Rust project, compiles it, evaluates it, and
//! interprets the results as the generated Rust code. When you call the macro again (for example,
//! after changing the macro’s parameters or calling the same macro in a different place), Crabtime
//! can reuse the previously generated project. This feature is called “caching.” It is enabled by
//! default on the nightly channel and can be enabled on the stable channel by providing a `module`
//! attribute, for example:
//!
//! ```
//! #[crabtime::function(cache_key=my_key)]
//! #[module(my_crate::my_module)]
//! fn my_macro() {
//!     // ...
//! }
//! ```
//!
//! The cache is always written to
//! `<project_dir>/target/debug/build/crabtime/<module>/<macro_name>`. The defaults are presented
//! below:
//!
//! |                      | Rust Unstable           | Rust Stable                               |
//! | :---                 | :---                    | :---                                      |
//! | Cache enabled        | ✅                      | ❌ by default, ✅ when `module` used.    |
//! | `module` default     | path to def-site module | __none__                                 |
//!
//! Please note that caching will be automatically enabled on the stable channel as soon as the
//! [proc_macro_span](proc_macro_span) feature is stabilized. That feature allows Crabtime to read
//! the path of the file where the macro was used, so it can build a unique cache key.
//!
//! <br/>
//!
//! <h5><b>Performance Stats</b></h5>
//!
//! Crabtime also generates runtime and performance statistics to help you understand how much time
//! was spent evaluating your macros, where projects were generated, and which options were used.
//! If you expand any usage of `#[crabtime::function]` (for example, in your IDE), you will see
//! compilation stats like:
//!
//! ```text
//! # Compilation Stats
//! Start: 13:17:09 (825)
//! Duration: 0.35 s
//! Cached: true
//! Output Dir: /Users/crabtime_user/my_project/target/debug/build/crabtime/macro_path
//! Macro Options: MacroOptions {
//!     cache: true,
//!     content_base_name: false,
//! }
//! ```
//!
//! Please note that you can be presented with the `Cached: true` result even after the first
//! macro evaluation if your IDE or build system evaluated it earlier in the background.
//!
//! <br/>
//! <br/>
//!
//! # 🪲 Logging & Debugging
//!
//! There are several ways to log from your Crabtime macros. Because
//! [proc_macro::Diagnostic](https://doc.rust-lang.org/proc_macro/struct.Diagnostic.html) is
//! currently a nightly-only feature, Crabtime prints nicer warnings and errors if you are using
//! nightly Rust channel. They look just like warnings and errors from the Rust compiler.
//! Otherwise, your warnings and errors will be printed to the console with a `[WARNING]` or
//! `[ERROR]` prefix.
//!
//! | Method               | Behavior on stable | Behavior on nightly |
//! | :---                 | :---               | :---                |
//! | `println!`           | Debug log in console | Debug log in console |
//! | `crabtime::warning!` | Debug log in console | Warning in console   |
//! | `crabtime::error!`   | Debug log in console | Error in console     |
//!
//! <br/>
//!
//! <h5><b>Stdout Protocol</b></h5>
//!
//! Please note that Crabtime uses stdout for all communication between the code generation process
//! and the host process. Depending on the prefix of each stdout line, it is interpreted according
//! to the following table. In particular, instead of using the methods shown above, you can
//! generate code from your macros by printing it to stdout (like
//! `println!("[OUTPUT] struct T {}")`), but it's highly discouraged.
//!
//! | Prefix      | Meaning |
//! | :---        | :---    |
//! | _(none)_    | Debug log message (informational output). |
//! | `[OUTPUT]`  | A line of generated Rust code to be included in the final macro output. |
//! | `[WARNING]` | A compilation warning. |
//! | `[ERROR]`   | A compilation error. |
//!
//! <br/>
//!
//! <h5><b>Stdout Protocol Utilities</b></h5>
//!
//! Although you are not supposed to generate the Stdout Protocol messages manually, we believe that
//! it is better to expose the underlying utilities so that in rare cases, you can use them to
//! reduce the risk of malformed output. These functions allow you to transform multi-line strings
//! by adding the appropriate prefixes:
//!
//! ```
//! mod crabtime {
//!     fn prefix_lines_with(prefix: &str, input: &str) -> String {
//!         // Adds the given prefix to each line of the input string.
//!         # panic!()
//!     }
//!
//!     fn prefix_lines_with_output(input: &str) -> String {
//!         // Adds `[OUTPUT]` to each line of the input string.
//!         # panic!()
//!     }
//!
//!     fn prefix_lines_with_warning(input: &str) -> String {
//!         // Adds `[WARNING]` to each line of the input string.
//!         # panic!()
//!     }
//!
//!     fn prefix_lines_with_error(input: &str) -> String {
//!         // Adds `[ERROR]` to each line of the input string.
//!         # panic!()
//!     }
//! }
//! ```
//!
//! These macros allow you to directly print prefixed lines to `stdout`, following the protocol:
//!
//! ```
//! mod crabtime {
//!     macro_rules! output_str {
//!         // Outputs code by printing a line prefixed with `[OUTPUT]`.
//!         # () => {};
//!     }
//!
//!     macro_rules! warning {
//!         // On the nightly channel prints a compilation warning.
//!         // On the stable channel prints a log prefixed with `[WARNING]`.
//!         # () => {};
//!     }
//!
//!     macro_rules! error {
//!         // On the nightly channel prints a compilation error.
//!         // On the stable channel prints a log prefixed with `[ERROR]`.
//!         # () => {};
//!     }
//! }
//! ```
//!
//! <br/>
//! <br/>
//!
//! # ⚙️ Macro Cargo Configuration
//!
//! <div style="background-color:#397be440; padding: 8px; border-radius: 8px; margin-bottom: 8px;">
//! 💡 On the Rust unstable channel, all configuration is automatically gathered from your
//! Cargo.toml. It includes build-dependencies and code lints, including those defined in your
//! workspace.
//! </div>
//!
//! Every Crabtime macro is a separate Cargo project with its own configuration and dependencies.
//! If you use nightly, Crabtime automatically uses your Cargo.toml configuration. On stable, due
//! to lack of [proc_macro_span](proc_macro_span) stabilization, Crabtime cannot discover your
//! Cargo.toml automatically. You must provide cargo configuration in your macro blocks, for
//! example:
//!
//! ```
//! #[crabtime::function]
//! fn my_macro() {
//!     // Do this only on Rust stable channel. On the unstable channel
//!     // use your Cargo.toml's [build-dependencies] section instead.
//!     #![edition(2024)]
//!     #![resolver(3)]
//!     #![dependency(anyhow = "1.0")]
//!
//!     type Result<T> = anyhow::Result<T>;
//!     // ...
//! }
//! # fn main() {}
//! ```
//!
//! Crabtime recognizes these Cargo configuration attributes. The attributes below override any
//! configuration discovered in your Cargo.toml, even on nightly:
//!
//! <br/>
//!
//! <h5><b>Supported Cargo Configuration Attributes</b></h5>
//!
//! | Attribute             | Default |
//! | :---                  | :---    |
//! | `#![edition(...)]`    | 2024    |
//! | `#![resolver(...)]`   | 3       |
//! | `#![dependency(...)]` | []      |
//!
//! <br/>
//! <br/>
//!
//! # 📚 Attributes
//!
//! You can provide any set of global attributes (`#![...]`) on top of your Crabtime macro
//! definition for them to be applied to the given generated Crabtime crate.
//!
//! <br/>
//! <br/>
//!
//! # 🗺️ Paths
//!
//! Crabtime macros provide access to several path variables, allowing you to traverse your
//! project's folder structure during macro evaluation. All paths are accessible within the
//! `crabtime::` namespace.
//!
//!
//! | Path                  | Availability     | Description |
//! | :---                  | :---             | :---        |
//! | `WORKSPACE_PATH`      | Stable & Nightly | Path to the root of your project. This is where the top-most `Cargo.toml` resides, whether it's a single-crate project or a Cargo workspace. |
//! | `CRATE_CONFIG_PATH`   | Nightly only     | Path to the `Cargo.toml` file of the current crate. |
//! | `CALL_SITE_FILE_PATH` | Nightly only     | Path to the file where the macro was invoked. |
//!
//!
//! ```
//! #[crabtime::function]
//! fn check_paths() {
//!     println!("Workspace path: {}", crabtime::WORKSPACE_PATH);
//! }
//! check_paths!();
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # 📖 How It Works Under The Hood
//!
//! The content of a function annotated with `crabtime::function` is pasted into the `main`
//! function of a temporary Rust project. This project is created, compiled, executed, and (if
//! caching is disabled) removed at build time, and its `stdout` becomes the generated Rust code.
//! The generated `main` function looks something like this:
//!
//! ```
//! const SOURCE_CODE: &str = "..."; // Your code as a string.
//!
//! # mod phantom_for_crabtime_name_crash_resolution {
//! mod crabtime {
//!     // Various utils described in this documentation.
//!     # pub fn push_as_str(str: &mut String, result: &()) {}
//!     # pub fn prefix_lines_with_output(input: &str) -> String { String::new() }
//! }
//!
//! fn main() {
//!     let mut __output_buffer__ = String::new();
//!     let result = {
//!         // Your code.
//!     };
//!     crabtime::push_as_str(&mut __output_buffer__, &result);
//!     println!("{}", crabtime::prefix_lines_with_output(&__output_buffer__));
//! }
//! # }
//! # fn main() {}
//! ```
//!
//! The `output!` macro is essentially a shortcut for writing to output buffer using `format!`, so
//! this:
//!
//! ```
//! #[crabtime::function]
//! fn my_macro_expansion1(components: Vec<String>) {
//!     for dim in 1 ..= components.len() {
//!         let cons = components[0..dim].join(",");
//!         crabtime::output! {
//!             enum Position{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!     }
//! }
//! my_macro_expansion1!(["X", "Y", "Z", "W"]);
//! # fn main() {}
//! ```
//!
//! Is equivalent to:
//!
//! ```
//! #[crabtime::function]
//! fn my_macro_expansion2(pattern!([$($components_arg:expr),*$(,)?]): _) {
//!     let components: Vec<String> = expand!(
//!         [$(crabtime::stringify_if_needed!($components_arg).to_string()),*]
//!     ).into_iter().collect();
//!     for dim in 1 ..= components.len() {
//!         let cons = components[0..dim].join(",");
//!         crabtime::output_str! {"
//!             enum Position{dim} {{
//!                 {cons}
//!             }}
//!         "}
//!     }
//! }
//! my_macro_expansion2!(["X", "Y", "Z", "W"]);
//! # fn main() {}
//! ```
//!
//! And that, in turn, is just the same as:
//!
//! ```
//! #[crabtime::function]
//! fn my_macro_expansion3() {
//!     let components = ["X", "Y", "Z", "W"];
//!     for dim in 1 ..= components.len() {
//!         let cons = components[0..dim].join(",");
//!         __output_buffer__.push_str(
//!             &format!("enum Position{dim} {{ {cons} }}\n")
//!         );
//!     }
//! }
//! my_macro_expansion3!();
//! # fn main() {}
//! ```
//!
//! Which, ultimately, is equivalent to:
//!
//! ```
//! #[crabtime::function]
//! fn my_macro_expansion4() {
//!     let components = ["X", "Y", "Z", "W"];
//!     for dim in 1 ..= components.len() {
//!         let cons = components[0..dim].join(",");
//!         println!("[OUTPUT] enum Position{dim} {{");
//!         println!("[OUTPUT]     {cons}");
//!         println!("[OUTPUT] }}");
//!     }
//! }
//! my_macro_expansion4!();
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # ⚠️ Corner Cases
//! There are a few things you should be aware of when using Crabtime:
//! - Caching is associated with the current file path. It means that if in a single file you have
//!   multiple Crabtime macros of the same name (e.g. by putting them in different modules within a
//!   single file), they will use the same Rust project under the hood, which effectively breaks
//!   the whole purpose of caching.
//! - You can't use Crabtime functions to generate consts. Instead, use `Crabtime::eval!` as shown
//!   above. This is because when expanding constants, macros need to produce an additional pair of
//!   `{` and `}` around the expanded tokens. If anyone knows how to improve this, please contact
//!   us.
//! - Error spans from the generated code are not mapped to your source code. It means that you
//!   will still get nice, colored error messages, but the line/column numbers will be pointing to
//!   the generated file, not to your source file. This is an area for improvement, and I'd be
//!   happy to accept a PR that fixes this.
//! - `Crabtime::eval!` does not use caching, as there is no name we can associate the cache with.
//!
//! <br/>
//! <br/>
//!
//! # ⚠️ Troubleshooting
//!
//! ⚠️ **Note:** Rust IDEs differ in how they handle macro expansion. This macro is tuned for
//! `rustc` and `RustRover`’s expansion engines.
//!
//! If your IDE struggles to correctly expand `crabtime::output!`, you can switch to the
//! `crabtime::output_str!` syntax described above. If you encounter this, please
//! [open an issue](https://github.com/wdanilo/eval-macro/issues) to let us know!
//!
//! [zigs_comptime]: https://zig.guide/language-basics/comptime
//! [token_stream]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
//! [macro_fragments]: https://doc.rust-lang.org/reference/macros-by-example.html#metavariables
//! [macro_rules]: https://doc.rust-lang.org/rust-by-example/macros.html
//! [fn_like_macros]: https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
//! [derive_macros]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros
//! [attribute_macros]: https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
//! [proc_macro_span]: https://github.com/rust-lang/rust/issues/54725
//! [rustfmt]: https://github.com/rust-lang/rustfmt
//! [macro_hygiene]: https://doc.rust-lang.org/reference/macros-by-example.html#hygiene
//!
//! [^supported_ides]: This code was thoroughly tested in `rustc`, the IntelliJ/RustRover Rust expansion engine, and Rust Analyzer (VS Code, etc.).
//!
//! [inline_dependency_injection]: ...
//! [space_aware_interpolation]: ...
#![cfg_attr(not(feature = "std"), no_std)]

extern crate self as crabtime;
pub use crabtime_internal::*;

// =====================
// === Macro Helpers ===
// =====================

#[macro_export]
macro_rules! eval {
    ($($ts:tt)*) => {
        {
            #[crabtime::eval_function(cache=true, content_base_name=true)]
            fn run() -> _ {
                $($ts)*
            }
        }
    };
}

// ==========================
// === Type Hints Mockups ===
// ==========================
// The following items are defined to prevent IDE error messages. The real definition is placed
// in the generated project per macro usage.

/// AVAILABLE ONLY WITHIN THE CRABTIME MACRO.
#[macro_export]
macro_rules! output {
    ($($ts:tt)*) => {};
}

/// AVAILABLE ONLY WITHIN THE CRABTIME MACRO.
#[macro_export]
macro_rules! quote {
    ($($ts:tt)*) => { String::new() };
}

/// AVAILABLE ONLY WITHIN THE CRABTIME MACRO.
#[macro_export]
macro_rules! write_ln {
    ($($ts:tt)*) => {};
}

/// AVAILABLE ONLY WITHIN THE CRABTIME MACRO.
///
/// Returns all ordered combinations of positive integers that sum to `n` (with at least two
/// summands). For example `sum_combinations(4)` returns
///
/// ```text
/// [1, 1, 1, 1]
/// [1, 1, 2]
/// [1, 2, 1]
/// [2, 1, 1]
/// [1, 3]
/// [3, 1]
/// [2, 2]
/// ```
#[cfg(feature = "std")]
#[allow(clippy::panic)]
pub fn sum_combinations(_n: usize) -> Vec<Vec<usize>> {
    panic!("AVAILABLE ONLY WITHIN THE CRABTIME MACRO.")
}

pub const WORKSPACE_PATH: &str = "AVAILABLE ONLY WITHIN THE CRABTIME MACRO.";
pub const CRATE_CONFIG_PATH: &str = "AVAILABLE ONLY WITHIN THE CRABTIME MACRO.";
pub const CALL_SITE_FILE_PATH: &str = "AVAILABLE ONLY WITHIN THE CRABTIME MACRO.";

// =============
// === Tests ===
// =============

/// Most of the tests are included in the documentation above. These tests cover corner cases to
/// ensure that the macro works as expected.
#[cfg(all(test, feature = "std"))]
mod tests {
    #[test]
    fn empty_def_compilation() {
        #[crabtime::function]
        fn empty_def_compilation() {}
        empty_def_compilation!();
    }

    // ===

    mod mod_a {
        #[crabtime::function]
        fn inter_module_macro() -> &str {
            "pub struct Generated;"
        }
        #[allow(clippy::single_component_path_imports)]
        pub(super) use inter_module_macro;
    }

    mod mod_b {
        super::mod_a::inter_module_macro!();
    }

    #[test]
    fn inter_module_macro() {
        let _p = mod_b::Generated;
    }

    #[test] fn interpolation_before_brace() {
        #[crabtime::function]
        fn interpolation_before_brace() {
            let is_a_branches = "A => true, B => false";
            crabtime::output! {
                enum E { A, B }
                impl E {
                    fn is_a(&self) -> bool {
                        match self {
                            {{is_a_branches}}
                        }
                    }
                }
            }
        }
        interpolation_before_brace!();
    }

    // ===

    mod mod_c {
        #[crabtime::function]
        fn fn_in_impl() -> &str {
            "pub fn test(&self) {}"
        }
        struct Test;
        impl Test {
            fn_in_impl!();
        }
    }
}
