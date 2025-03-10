// <img width="680" alt="banner" src="https://github.com/user-attachments/assets/e7dab624-6e88-41f6-b681-a7a430f96b50">
//
// <br/>
// <br/>
//

//! # 🦀 Crabtime
//!
//! **Crabtime** introduces a new way to write Rust macros in a similar spirit to [Zig's comptime](...).
//! It provides even greater flexibility and power than procedural macros, yet it is easier and more natural to
//! read and write than `macro_rules`.
//!
//! <br/>
//! <br/>
//!
//! # 🆚 Comparison to proc macros and `macro_rules!`
//!
//! Below you can find the comparison of the most important aspects of Rust macro systems:
//!
//! <h5><b>Input/Output</b></h5>
//!
//! | <div style="width:300px"/>                            | Crabtime | Proc Macro | Macro Rules |
//! | :---                                                  | :---     | :---       | :---        |
//! | Input as [Token Stream][token_stream]                 | ✅       | ✅         | ❌          |
//! | Input as [Macro Fragments][macro_fragments]           | ✅       | ❌         | ✅          |
//! | Input as Rust Code (String)                           | ✅       | ❌         | ❌          |
//! | Output as [Token Stream][token_stream]                | ✅       | ✅         | ❌          |
//! | Output as [Macro Fragments Template][macro_fragments] | ✅       | ❌         | ✅          |
//! | Output as Rust Code (String)                          | ✅       | ❌         | ❌          |
//!
//! <h5><b>Functionalities</b></h5>
//!
//! | <div style="width:300px"/>                            | Crabtime | Proc Macro | Macro Rules |
//! | :---                                                  | :---     | :---       | :---        |
//! | Advanced transformations                              | ✅       | ✅         | ❌          |
//! | [Space-aware interpolation](...)                      | ✅       | ❌         | ❌          |
//! | Can define [fn-like macros](...)                      | ✅       | ✅         | ✅          |
//! | Can define [derive macros](...)                       | 🚧       | ✅         | ❌          |
//! | Can define [attribute macros](...)                    | 🚧       | ✅         | ❌          |
//! | Reusable across modules and crates                    | ✅       | ✅         | ✅          |
//!
//! <h5><b>Comfort of life</b></h5>
//!
//! | <div style="width:300px"/>                            | Crabtime | Proc Macro | Macro Rules |
//! | :---                                                  | :---     | :---       | :---        |
//! | Provides code hints in IDEs                           | ✅       | ✅         | ❌          |
//! | Works with [rustfmt](...)                             | ✅       | ✅         | ❌          |
//! | Easy to define (inline, the same crate)               | ✅       | ❌         | ✅          |
//! | Easy to read                                          | ✅       | ❌         | ⚠️          |
//! | [Hygienic](...)                                       | ❌       | ❌         | ✅          |
//!
//! [token_stream]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
//! [macro_fragments]: https://doc.rust-lang.org/reference/macros-by-example.html#metavariables
//!
//! <br/>
//! <br/>
//!
//! # 🎯 One-shot evaluation
//!
//! The simples, and the least exciting usage of Crabtime is simple compile-time code evaluation.
//! To evaluate an expression and paste it's output as your new code, you can simply use
//! `crabtime::eval`, as shown below:
//!
//! ```
//! const MY_NUM: usize = crabtime::eval! { (std::f32::consts::PI.sqrt() * 10.0).round() as usize };
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # 🤩 Function-like macros
//!
//! Use the `crabtime::function` attribute to define a new [function-like macro](...). Please
//! note that Crabtime will remove the annotated function, compile and execute it at build time,
//! and replace it with a macro definition of the same name. You can then call the macro to evaluate it.
//! Let's start with a simple example, and let's refine it down the line. Let's generate the following
//! Rust code:
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
//! // Evaluates the code at build-time, and uses it's output to generate macro `gen_positions!`.
//! #[crabtime::function]
//! fn gen_positions1() {
//!     "
//!     enum Position1 { X }
//!     enum Position2 { X, Y }
//!     enum Position3 { X, Y, Z }
//!     enum Position4 { X, Y, Z, W }
//!     "
//! }
//!
//! // We are now using the macro to generate four structs.
//! gen_positions1!();
//! # fn main() {}
//! ```
//!
//! <br/>
//! <br/>
//!
//! # 🤩 Attribute and Derive Macros
//! Currently, generating attribute and derive macros is not supported, but we are working on it. If you want to help,
//! ping us on [GitHub](https://github.com/wdanilo/crabtime).
//!
//! <br/>
//! <br/>
//!
//! # 📤 Output
//!
//! There are several ways to generate the output code from a Crabtime macro. We recommend you to
//! use either `crabtime::output!` or `crabtime::quote!` macros, as they allow for the most concise,
//! easy to understand, and maintenable implementations. Supported input types are described later,
//! for now just ignore them.
//!
//! <br/>
//!
//! <h5><b>Generating Output by using <code>crabtime::output!</code></b></h5>
//!
//! The simplest and most recommended way to generate macro output is by using the `crabtime::output!` macro.
//! It allows for space-aware variable interpolation. It's like the `format!` macro, but with inversed rules
//! regarding curly braces – it preserves single braces and uses double braces for interpolation. Please note
//! that it preserves spaces, so `Position {{ix}}` and `Position{{ix}}` mean different things and the latter
//! will generate `Position1`, `Position2`, etc.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions2(components: Vec<String>) {
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
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
//! <h5><b>Generating Output by using <code>crabtime::quote!</code></b></h5>
//!
//! The `crabtime::quote!` macro is just like `crabtime::output!`, but instead of outputting the code
//! immediately, it returns it (as a `String`), so you can store it in a variable and re-use it across different subsequent
//! calls to `crabtime::quote!` or `crabtime::output!`.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions3(components: Vec<String>) {
//!     let structs = components.iter().enumerate().map(|(ix, name)| {
//!         let dim = ix + 1;
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
//! <h5><b>Generating Output by returning a String or number</b></h5>
//!
//! You can simply return a string or number from the function. It will be used as the generated macro code.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions4(components: Vec<String>) {
//!     components.iter().enumerate().map(|(ix, name)| {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         format!("enum Position{dim} {{ {cons} }}")
//!     }).collect::<Vec<_>>().join("\n")
//! }
//! # fn main() {}
//! ```
//!
//! <br/>
//!
//! <h5><b>Generating Output by using <code>crabtime::output_str</code></b></h5>
//!
//! Alternatively, you can use the `crabtime::output_str` function to immediately write strings to the
//! code output buffer:
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions5() {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         crabtime::output_str!("enum Position{dim} {{ {cons} }}")
//!     }
//! }
//! gen_positions5!();
//! # fn main() {}
//! ```
//!
//! <br/>
//!
//! <h5><b>Generating Output by returning a <code>TokenStream</code></b></h5>
//!
//! Finally, you can output `TokenStream` from the macro. Please note that for brevity the below example uses
//! [inline dependency injection](...), which is described later. In real code you sohuld use your `Cargo.toml`'s
//! `[build-dependencies]` section to include the necessary dependencies instead.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions6() {
//!     // Inline dependencies used for brevity.
//!     // You should use [build-dependencies] section in your Cargo.toml instead.
//!     #![dependency(proc-macro2 = "1")]
//!     #![dependency(syn = "2")]
//!     #![dependency(quote = "1")]
//!     use proc_macro2::Span;
//!     use quote::quote;
//!
//!     let components = ["X", "Y", "Z", "W"];
//!     let defs = components.iter().enumerate().map(|(ix, name)| {
//!         let dim = ix + 1;
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
//! Similarly to generating output, there are several ways to parametrize macros and provide them with input
//! on their call site. We recommend you to use the pattern parametrization, as it's the simplest and easiest
//! to maintain.
//!
//! <br/>
//!
//! <h5><b>Input by using supported arguments</b></h5>
//!
//! Currently, you can use any combination of the following types as arguments to your macro and they will
//! be automatically translated to patterns: `Vec<...>`, `&str`, `String`, and numbers. If the expected argument
//! is string, you can pass either a string literal or an identifier, which will automatically be converted to
//! string.
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions7(name: String, components: Vec<String>) {
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
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
//! In case you want even more control, you can use the same patterns as in `macro_rules!` by using a special
//! `pattern!` macro and you can expand any pattern using the `expand!` macro:
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions8(pattern!($name:ident, $components:tt): _) {
//!     let components = expand!($components);
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
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
//! Alternatively, you can consume the provided input as `TokenStream`:
//!
//! ```
//! #[crabtime::function]
//! fn gen_positions9(name: TokenStream) {
//!     #![dependency(proc-macro2 = "1")]
//!     let components = ["X", "Y", "Z", "W"];
//!     let name_str = name.to_string();
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
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
//! Crabtime macro lifecycle is very similar to procedural macro lifecycle. It gets compiled as a separate crate,
//! and then it is used as a binary to transform input tokens to output tokens. In fact, if you are using unstable
//! Rust channel, the performance of Crabtime and procedural macros is the same. On stable channel, Crabtime
//! needs slightly more time than procedural macros after you change your macro definition.
//!
//! In other words, the performance of Crabtime is similar to the performance of procedural macros. It is slower to boot (compile)
//! than `macro_rules!`, but faster to crunch tokens and perform complex transformations.
//!
//! |                                            | <div style="width:200px">Proc Macro</div> | <div style="width:200px">Crabtime</div> | <div style="width:200px"><code>macro_rules!</code></div> |
//! | :---                                       | :--- | :--- | :--- |
//! | First compilation                          | ⚠️ Relatively slow  | ⚠️ Relatively slow | ✅ Fast |
//! | Next compilation (macro definition change) | ✅ Fast | ✅ Fast on nightly <br/> ⚠️ Relatively slow on stable | ✅ Fast |
//! | Evaluation (call site)                     | ✅ Fast | ✅ Fast | ❌ Slow for complex transformations |
//! | Cost after changing module code without changing macro-call site code | ✅ Zero | ✅ Zero | ✅ Zero |
//!
//! Moreover, Crabtime generates runtime and performance statistics, so you can understand how much time was spent on
//! evaluating your macros, where your projects were generated and what options were used to generate your code.
//! If you expand any usage of `#[crabtime::function]` (for example in your IDE),
//! you'll be presented with compilation stats in the following form:
//!
//! ```text
//! # Compilation Stats
//! Start: 06:17:09 (825)
//! Duration: 0.35 s
//! Cached: true
//! Output Dir: /Users/crabtime_user/my_project/target/debug/build/crabtime/macro_path
//! Macro Options: MacroOptions {
//!     cache: true,
//!     content_base_name: false,
//! }
//! ```
//!
//! <br/>
//! <br/>
//!
//! # 🪲 Logging & Debugging
//!
//! There are several ways to log from your Crabtime macros. As the
//! [proc_macro::Diagnostic](https://doc.rust-lang.org/proc_macro/struct.Diagnostic.html) is currently
//! a nightly-only feature, Crabtime prints nicer warnings and errors if you are using nightly Rust
//! channel (they look just like warnings and errors from the Rust compiler). Otherwise, your warning and
//! error logs will be printed to console with a `[WARNING]` or `[ERROR]` prefix.
//!
//! | Method             | Behavior on stable | Behavior on nightly |
//! | :---                                                  | :---       | :---        |
//! | `println!`           | Debug log in console  | Debug log in console  |
//! | `crabtime::warning!` | Debug log in console  | Warning in console  |
//! | `crabtime::error!`   | Debug log in console  | Error in console  |
//!
//!
//! <br/>
//!
//! <h5><b>Stdout Protocol</b></h5>
//!
//! Please note that Crabtime uses stdout for all communication between the code generation process
//! and the host process. Depending on the prefix of every stdout line, it is interpreted according to the
//! following table. In particular, it means that instead of using the above presented methods, you can
//! generate code from your macros by printing it to stdout, like `println!([OUTPUT] struct T {})`, but it is
//! highly not recommended.
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
//! it's better to expose the underlying utilities, so even in the most rare cases, you can use them
//! to reduce the risk of malformed output.
//!
//! These functions allow you to transform multi-line strings by adding the appropriate prefixes:
//!
//! ```rust
//! mod crabtime {
//!     fn prefix_lines_with(prefix: &str, input: &str) -> String {
//!         // Adds the given prefix to each line of the input string.
//!         # panic!()
//!     }
//!
//!     fn prefix_lines_with_output(input: &str) -> String {
//!         // Adds `OUTPUT:` to each line of the input string.
//!         # panic!()
//!     }
//!
//!     fn prefix_lines_with_warning(input: &str) -> String {
//!         // Adds `WARNING:` to each line of the input string.
//!         # panic!()
//!     }
//!
//!     fn prefix_lines_with_error(input: &str) -> String {
//!         // Adds `ERROR:` to each line of the input string.
//!         # panic!()
//!     }
//! }
//! ```
//!
//! These macros allow you to directly print prefixed lines to `stdout`, following the
//! protocol:
//!
//! ```rust
//! mod crabtime {
//!     macro_rules! output_str {
//!         // Outputs a code by printing a line prefixed with `[OUTPUT]`.
//!         # () => {};
//!     }
//!
//!     macro_rules! warning {
//!         // On the nightly channel prints compilation warning.
//!         // On the stable channel prints a log prefixed with `[WARNING]`.
//!         # () => {};
//!     }
//!
//!     macro_rules! error {
//!         // On the nightly channel prints compilation error.
//!         // On the stable channel prints a log prefixed with `[ERROR]`.
//!         # () => {};
//!     }
//! }
//! ```
//!
//! [3]: https://github.com/rust-lang/rust/issues/54140
//!
//! <br/>
//! <br/>
//!
//! # ⚙️ Macro Cargo Configuration
//!
//! As every Crabtime macro is a separate Cargo project, it has distinct configuration, including
//! distinct dependencies. If you are using nightly Rust channel, Crabtime automatically uses
//! your Cargo.toml `edition`, `resolver`, and `[build-dependencies]` settings. On the stable channel,
//! due to lack of []() stabilization, Crabtime can not automatically discover the path to your Cargo.toml
//! file, and thus you need to provide cargo configuration inside of the Crabtime macro blocks, for example:
//!
//! ```rust
//! #[crabtime::function]
//! fn my_macro() {
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
//! Crabtime accepts the following Cargo configuration attributes. Please note, that configuration provided
//! this way has priority over configuration fetched from Cargo.toml files, even if you are on the nightly
//! channel.
//!
//! <br/>
//!
//! <h5><b>Supported Cargo Configuration Attributes</b></h5>
//!
//! | Attribute            | Default |
//! | :---                  | :---    |
//! | `#![edition(...)]`   | `2024`  |
//! | `#![resolver(...)]`  | `3`     |
//! | `#![dependency(...)]`| `[]`    |
//!
//! <br/>
//! <br/>
//!
//! # 📚 Attributes
//!
//! You can provide any set of global attributes (`#![...]`) on top of your Crabtime macro definition
//! for them to be applied to the given generated Crabtime crate.
//!
//! <br/>
//! <br/>
//!
//! # 📖 How It Works Under The Hood
//!
//! The content inside `eval!` is pasted into the `main` function of a temporary Rust project
//! created in `$HOME/.cargo/eval-macro/<project-id>`. This project is created, compiled,
//! executed, and removed at build time, and its `stdout` becomes the generated Rust code. The
//! generated `main` function looks something like this:
//!
//! ```
//! const SOURCE_CODE: &str = "..."; // Your code as a string.
//!
//! fn main() {
//!     let mut output_buffer = String::new();
//!     let result = {{
//!         // Your code.
//!     }};
//!     push_as_str(&mut output_buffer, &result);
//!     println!("{}", prefix_lines_with_output(&output_buffer));
//! }
//! # fn push_as_str(str: &mut String, result: &()) {}
//! # fn prefix_lines_with_output(input: &str) -> String { String::new() }
//! ```
//!
//! The `output!` macro is essentially a shortcut for writing to `output_buffer` using `format!`,
//! so this:
//!
//! ```
//! #[crabtime::function]
//! fn my_macro_expansion1() {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         crabtime::output! {
//!             enum Position{{dim}} {
//!                 {{cons}}
//!             }
//!         }
//!     }
//! }
//! my_macro_expansion1!();
//! # fn main() {}
//! ```
//!
//! Is equivalent to:
//!
//! ```
//! #[crabtime::function]
//! fn my_macro_expansion2() {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
//!         let cons = components[0..dim].join(",");
//!         crabtime::output_str! {"
//!             enum Position{dim} {{
//!                 {cons}
//!             }}
//!         "}
//!     }
//! }
//! my_macro_expansion2!();
//! # fn main() {}
//! ```
//!
//! And that, in turn, is just shorthand for:
//!
//! ```
//! #[crabtime::function]
//! fn my_macro_expansion3() {
//!     let components = ["X", "Y", "Z", "W"];
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
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
//!     for (ix, name) in components.iter().enumerate() {
//!         let dim = ix + 1;
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
//! - Caching is associated with the current file path. It means that if in a single file you have multiple
//!   Crabtime macros of the same name (e.g. by putting them in different modules within a single file), they
//!   will use the same Rust project under the hood, which effectively will break the whole purpose of caching.
//! - You can't use Crabtime functions to generate consts. Instead, use `Crabtime::eval` as shown above. This is because if
//!   expanding constants, macros need to produce additional pair of `{` and `}` around the expanded tokens.
//!   If anyone knows how to improve this, please contact me.
//! - Error spans from the generated code are not mapped to your source code. It means that you will still get
//!   nice, colored error messages, but the line/column numbers will be pointing to the generated file, not to
//!   your source file. Again, this is an area for improvement, and I'd be happy to accept a PR that fixes this.
//! - `Crabtime::eval` does not use caching, as there is no name we can associate the cache with.
//!
//! <br/>
//! <br/>
//!
//! # ⚠️ Troubleshooting
//!
//! ⚠️ **Note:** Rust IDEs differ in how they handle macro expansion. This macro is tuned for
//! `rustc` and `RustRover’s` expansion engines.
//!
//! If your IDE struggles to correctly expand `eval!`, you can switch to the `crabtime::output_str!`
//! syntax described above. If you encounter this, please
//! [open an issue](https://github.com/wdanilo/eval-macro/issues) to let us know!

pub use crabtime_internal::*;

// =====================
// === Macro Helpers ===
// =====================

extern crate self as crabtime;

#[macro_export]
macro_rules! eval {
    ($($ts:tt)*) => {
        {
            #[crabtime::eval_fn(cache=false, content_base_name=true)]
            fn run() {
                $($ts)*
            }
        }
    };
}

fn foo() {
    let x = 5;
    
}

mod xt {
    #[crabtime::function]
    fn gen_positions8() {
        let components = ["X", "Y", "Z", "W"];
        for (ix, name) in components.iter().enumerate() {
            let dim = ix + 1;
            let cons = components[0..dim].join(",");
            crabtime::output! {
                enum Position{{dim}} {
                    {{cons}}
                }
            }
        }
    }
    gen_positions8!();
}