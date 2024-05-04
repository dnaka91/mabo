# Changelog

All notable changes to this project will be documented in this file.

<!-- markdownlint-disable no-duplicate-header -->
<!-- markdownlint-disable no-trailing-spaces -->

## Unreleased

### ⛰️ Features

- _cli_: Implement doc sub-command in the CLI tool ([03cdc30](https://github.com/dnaka91/mabo/commit/03cdc30eebf555cd5e31a6f0cea8aee005f9dd55))
  > Integrate the mabo-doc crate into the CLI to be able to generate the
  > docs for any project (or individual files).
- _compiler_: Introduce a simplified schema for easier codegen ([131e7b4](https://github.com/dnaka91/mabo/commit/131e7b46c252a8080c83a292a323c719a4a9fca7))
  > The parser's data structures can be a bit hard to use in the code
  > generator, and some upcoming features would need to be rebuilt in every
  > code generator.
  > 
  > By introducing a simplified version of the schema data structures,
  > common conversion logic can be moved into the compiler and overall make
  > the data structures more convenient to use.
- _doc_: Implement a HTML documentation generator ([b72ba33](https://github.com/dnaka91/mabo/commit/b72ba337e5aacfbd37c309fcfbd69aceb848f0d1))
  > A new generator that doesn't create implementations for a language, but
  > instead creates HTML documentation for a schema.
  > 
  > This is the first building block to provide a new CLI command that can
  > generate the documentation for any schema, similar to how rustdoc can
  > generate local documentation for Rust files or projects.
- _go_: Generate new `Size` interface ([5cc3c2f](https://github.com/dnaka91/mabo/commit/5cc3c2fab4cb6cc7645c7b7c51f2bf734dfa384b))
  > The newly defined `Size` interface allows to calculate the encoded size
  > of an instance and pre-allocate a byte vector. It can be helpful for
  > significantly large payloads that would do several re-allocations of a
  > buffer otherwise.
- _go_: Implement the CLI for the Go code generator ([d3dc8d7](https://github.com/dnaka91/mabo/commit/d3dc8d70fe8655ed066155c46cb4a4e28a466b5b))
  > Although the Go code generator existed as library, it still needed the
  > CLI binary to drive the actual codegen step and write files to disk.
- _lsp_: Properly handle UTF-16 char offsets ([f74d625](https://github.com/dnaka91/mabo/commit/f74d625a793b5569652a852567568a14e0ccc229))
  > The LSP requires all char indices to be in UTF-16 byte offsets by
  > default. But Rust uses UTF-8, so the right offset has to be calculated
  > accordingly.
- _lsp_: Improve logging setup ([5962776](https://github.com/dnaka91/mabo/commit/5962776635678e8e805c0312acf06df97dd0c369))
  > Overall improve the logging output for the LSP client, and write the
  > logs to a file as well.
- _lsp_: Handle CLI args for different communication channels ([670d9d1](https://github.com/dnaka91/mabo/commit/670d9d16e081f9b898526a148e34caf897619ecc))
  > Implement handling of the common CLI arguments that define how to
  > establish the client/server communication channel.
  > 
  > The options are defined, but not all are supported as of yet.
- _lsp_: Define and read configuration values ([192d943](https://github.com/dnaka91/mabo/commit/192d9437e8e29a0a54b59d0b18030302b1f167d4))
  > Create a sample configuration in the editor plugin and read it from the
  > LSP side (as well as reloading it on changes).
- _lsp_: Setup semantic token provider ([80d3c5f](https://github.com/dnaka91/mabo/commit/80d3c5fc17f01da35e579984dba42717cb43dd3e))
  > Create a basic setup that allows for semantic token provision by the LSP
  > server. This currently doesn't provide any actual tokens, just sets it
  > up for future use.
- _lsp_: Exclude `-lsp` suffix from the config and naming ([318dd7a](https://github.com/dnaka91/mabo/commit/318dd7aa4c1532c1e0046004a04758cb9fd1dd14))
  > It's not needed to repeat the LSP naming everywhere in the config and
  > naming of the VSCode extension, and can simply be omitted.
- _lsp_: Report schema validation errors ([ca1f300](https://github.com/dnaka91/mabo/commit/ca1f300128a57ecf4bca65c1986377fdc76c73ae))
  > Extend the diagnostic capabilities by reporting validation errors from
  > `stef-compiler` as well.
- _lsp_: Use local time with shorter format ([e9407a4](https://github.com/dnaka91/mabo/commit/e9407a4857464fd308561d92e7fc5de781716a03))
  > Change from UTC time to local time in any log messages, and shorten the
  > time format to only the time component (stripping off the date).
- _lsp_: Improve overall handling of code locations ([dc61f6e](https://github.com/dnaka91/mabo/commit/dc61f6e6d6ade800990c75cc792a519ddc4266e0))
  > The first thing is the introduction of the `line-index` crate from
  > rust-analyzer, which makes utf-16 conversions simple and fast.
  > 
  > In addition, the `Parser` error variant now carries the location of
  > where it occurred, rather than highlighting the whole schema file as the
  > location.
- _lsp_: Support incremental file changes ([0156f3e](https://github.com/dnaka91/mabo/commit/0156f3ebac6772482297075a05a7046ffb9f04d4))
  > This allows for faster updates as the editor can send only changed
  > content instead of the whole document on each modification.
- _lsp_: Generate semantic tokens for schema files ([57b59ee](https://github.com/dnaka91/mabo/commit/57b59eeccc50784c580537622f905c893f4735d5))
  > Provide semantic tokens for schema files to provide better code
  > highlighting. Currently, not all possible tokens are supplied yet.
- _lsp_: Provide document symbols ([23ee2d3](https://github.com/dnaka91/mabo/commit/23ee2d3b472b828e5c4c1e11168b07a46aed8eba))
  > Visible in VSCode in the outline component, this allows to render a tree
  > view of a schema's structure.
- _lsp_: Provide hover information ([db0c603](https://github.com/dnaka91/mabo/commit/db0c603eff28150dce5dba552e77592b6833268e))
  > Generate hover information for any element that is hovered by the user.
  > This currently only replies the documentation for the element (if
  > present), and calculates the next available ID for struct fields, enum
  > variants and enum variant fields.
- _lsp_: Calculate the wire size of fields on hover ([2c7d8b3](https://github.com/dnaka91/mabo/commit/2c7d8b3071cac47771b1d2d515752aa33dcf60b7))
  > The min and max required byte size on the wire (when encoded) can be
  > calculated for all types except generics and external types.
  > 
  > This will now be shown on hover allow users to guess the size of a
  > struct or enum variant easier.
- _lsp_: Load all files at start and improve state updates ([3b9e342](https://github.com/dnaka91/mabo/commit/3b9e3426d64fdeb5cd6f77a28371e95e45bfa6fd))
  > Scan the project during initialization and add all found files to the
  > state. Also, only remove files from the state when they're deleted, not
  > when closed.
- _lsp_: Keep track of project file location ([baca562](https://github.com/dnaka91/mabo/commit/baca562c9c98ba2f9079a712325b48d10f7735c7))
  > Extend the `mabo-project` crate to additional store the location from
  > wheret the project file was loaded.
  > 
  > This is currently used to log the location in the LPS when loading a
  > folder and searching for Mabo projects in it.
- _lsp_: Support all official position encodings ([e6d4884](https://github.com/dnaka91/mabo/commit/e6d48848a6e23bf5fd1c43a905ea4904fa0adf9a))
  > Until now the server always assumed UTF-16, as that is the default in
  > Visual Studio Code, but other editors like Neovim might ask for a
  > different encoding.
  > 
  > This now allows to run the LSP server with any of the official encodings
  > UTF-8, UTF-16 and UTF-32.
- _parser_: Add `simd` feature to the parser ([443c32c](https://github.com/dnaka91/mabo/commit/443c32c93622764a89cd3f0682ef8b1c6a6723dc))
  > The `winnow` crate has a `simd` feature which enables some extra
  > performance improvements. This might limit possible target platforms for
  > compilations so the flag is exposed as optional feature on the parser
  > crate instead.
  > 
  > All binary crates will use this flag, but potential users of the parser
  > crate can omit the feature if it causes issues.
- _vscode_: Provide command to restart the LSP server ([21e5001](https://github.com/dnaka91/mabo/commit/21e500105b651ecede36495b98d3ad61ac1466a1))
  > Although mostly useful during development, add a custom command to the
  > VSCode extension, which allows to restart the LSP server.
- _vscode_: Extend metadata for the vscode extension ([1dd3ca8](https://github.com/dnaka91/mabo/commit/1dd3ca83417535d1e91a478f209f10de4f032b97))
- Implement encoding in the Rust codegen ([2787b51](https://github.com/dnaka91/mabo/commit/2787b51c04803311bf5ca3160b37e7db31d5a8ea))
- Simplify encoding logic ([e8205bc](https://github.com/dnaka91/mabo/commit/e8205bcb6749fce6dd1f56ede38128076820bffd))
- Implement basic decoding logic ([f67c572](https://github.com/dnaka91/mabo/commit/f67c57220ac2c57961bf54f7f47bca467d3fb20b))
- Skip any decoding for unit structs/variants ([8f5a4ec](https://github.com/dnaka91/mabo/commit/8f5a4ec3cd7377ac2f0eb7183c201fbf388d9ce2))
- Create playground crate and fix issues ([b55862f](https://github.com/dnaka91/mabo/commit/b55862f3dafe8276faad9309cebff2cdcaec8ea3))
- Extend syntax highlighting ([0d83290](https://github.com/dnaka91/mabo/commit/0d83290e1367edc1a8f9c7686bb2032067c177fa))
- Adjust encoding of option&lt;T&gt; ([39c6ccb](https://github.com/dnaka91/mabo/commit/39c6ccba84c71688346ce9cd33bc2530ee0a746c))
- Unroll container type en-/decoding ([3d64436](https://github.com/dnaka91/mabo/commit/3d64436501d5e395ba27e9acc51bd82aa312a8d8))
  > Instead of relying on generic implementations for the Rust generated
  > code that en- or decodes the data, these are defined as closures.
  > 
  > The main benefit is that it allows optimized calls that would not be
  > possible through the trait system (for example a fast encoding of
  > `Vec<u8>` instead of the regular `Vec<T>` encoding that would write the
  > bytes one-by-one).
- Implement Decode for non-zero types ([8f7e85e](https://github.com/dnaka91/mabo/commit/8f7e85ef5ed68e1cd401927b54ad39ba97441fb4))
- Accept glob patterns for schema input files ([d8f5e43](https://github.com/dnaka91/mabo/commit/d8f5e439acc25afb096820b66fcf97bae34d3e91))
- Add non-zero collection types ([e0fc43e](https://github.com/dnaka91/mabo/commit/e0fc43ea968a9ca1586642eff937005648eed346))
  > This expands from non-zero integer values to non-zero collections,
  > meaning `non_zero<T>` specializations where `T` is one of `string`,
  > `bytes`, `vec<T>`, `hash_map<K, V>` or `hash_set<T>`.
  > 
  > Any other `non_zero<T>` types are now refused as there is no clear way
  > of defining what non-zero means for other types (or at least not a
  > single _correct_ way).
  > 
  > Also, snapshot tests includes missing adjustments from the last commits
  > that introduced further `#[allow(...)]` attributes in the Rust
  > implementation.
- Validate for unique IDs in structs and enums ([3295f0c](https://github.com/dnaka91/mabo/commit/3295f0c4300f839b9cbbd4c6ba035b834e4e6c4f))
  > Add the first "compiler" validation logic, which ensures that each ID:
  > - in named or unnamed fields of a struct or enum variant is unique
  > - in variants of an enum is unique
- Validate field and variant name uniqueness ([9137e20](https://github.com/dnaka91/mabo/commit/9137e20e3724ac18232375303a153503b51ea435))
  > Ensure that all fields in a struct or enum variant are uniquely named,
  > as well as all variants within a single enum.
- Nicely report compiler errors ([2737f20](https://github.com/dnaka91/mabo/commit/2737f20013cfe0d1fbf6b4a80ef101917183119a))
  > Extend parser structs with spans to generate nice error reports with
  > annotated source code, like being done for parser errors.
- Verify that generics are unique and used ([f3e0ced](https://github.com/dnaka91/mabo/commit/f3e0ced34080afba025410ef340b4521514b8911))
  > All defined generics in structs or enums must have a unique name and be
  > used in some way inside the definition of the element. These new
  > verifications in the compiler ensure the rules are upheld.
- Improve error output on compile error ([bd15eba](https://github.com/dnaka91/mabo/commit/bd15eba414df2b21bce00384ea79634a3c27670b))
- Enable glob parsing for CLI path arguments ([b44bc1b](https://github.com/dnaka91/mabo/commit/b44bc1b305828589c4b682b426b6cb94a9b583d8))
  > In addition to only accepting a list of files for the `check` and
  > `format` command, the input is now treated as glob pattern.
- Add snapshot tests for compiler errors ([0efc118](https://github.com/dnaka91/mabo/commit/0efc118308a3e79ec1edddb833328c80182ea8cb))
  > Verify good compiler error output by creating several snapshot tests.
  > Also, slightly improve some of the messages.
- En-/decode non-zero integers directly ([f165d61](https://github.com/dnaka91/mabo/commit/f165d618e91a70ab805a9f23f1139177150db4f4))
  > Instead of driving the encoding and decoding of non-zero integer types
  > through the Encode/Decode trait, use direct function calls which allows
  > for specialized calls with better performance.
- Ensure uniqueness of all module declarations ([d09e182](https://github.com/dnaka91/mabo/commit/d09e182b28fd12e2952888c5f3c002abeb7914ec))
  > Verify that all declared elements within a module (including the schema
  > root which is a module itself) have a unique name so they can't collide.
- Resolve all local types in schemas ([2b518ab](https://github.com/dnaka91/mabo/commit/2b518ab70920b86ac8a5e9e6fa8aea5579fdd1aa))
  > Check that all external types defined in any struct or enum definition
  > exist within the bounds of the schema and give nice error messages in
  > case a type can't be located.
- Resolve external schema imports ([62feaa0](https://github.com/dnaka91/mabo/commit/62feaa08edeec63dbbf8e988b7508039c1f023ce))
  > Implement the resolution of structs and enums that come from an external
  > schemas, being used in the local schema file.
  > 
  > Probably many edge cases can cause an error, dependency cycles for
  > example. Also, error types are a bit of a mess and will be improved in
  > coming commits.
- Improve error reporting for cross-schema errors ([c380ace](https://github.com/dnaka91/mabo/commit/c380ace56af634c8e878a5c60b2b0ab2e0e1dd72))
  > Type resolution that goes across a single file now shows code snippets
  > for both the use side and declaration side to better pinpoint the
  > problem.
- Validate the element amount in tuples ([abcee05](https://github.com/dnaka91/mabo/commit/abcee058c95f1974431e0fe683bda58a6bd9ffd6))
  > A valid tuple type must have between 2 and 12 elements. Empty tuples
  > carry no data at all, tuples with one element are equivalent to the type
  > of that single element and tuples with 13 or more elements are above a
  > reasonable limit.
- Make bytes type customizable ([eaa06ae](https://github.com/dnaka91/mabo/commit/eaa06ae590243bc276f648868eb6c556e0aad069))
  > Add a new option to `stef-build` that allows to choose between the
  > default `Vec<u8>` and `bytes::Bytes` as type used for the STEF `bytes`
  > type in Rust.
- Create macro for easy code inclusion ([7ec3e09](https://github.com/dnaka91/mabo/commit/7ec3e09bd2011d81cc5d98c79945b4ed9def92a2))
  > Provide a macro that makes inclusion of generates Rust source code
  > easier. Also, generate all files into a subfolder to avoid any
  > collisions with other generated files.
- Add basic Go code generator ([e7394c7](https://github.com/dnaka91/mabo/commit/e7394c76277e5b48217205d253721e100780f385))
  > Although incomplete in a few spots, this is a starting point for
  > converting schema files to source code of the Go programming language.
- Create basic language server for schemas ([4be4404](https://github.com/dnaka91/mabo/commit/4be44041d0e746aa2a012ce462b087d42ce32eb2))
  > Using an LSP allows providing editor agnostic support for schema files.
  > Currently only reporting errors in schemas (and not handling eventual
  > UTF-16 encoding).
- Introduce sizing methods to calculate needed bytes ([ab8f4f7](https://github.com/dnaka91/mabo/commit/ab8f4f7f94112a1d0bf5cbabe781c935ec3c89f7))
  > Each struct and enum in a schema can now calculate its total encoded
  > byte size. As each collection must be iterated it can be a costly
  > operation but it might be faster (or sometimes even required) to
  > allocate a buffer upfront instead of dynamically growing it.
- Adjust encoding to include a field encoding marker ([4dd6ba3](https://github.com/dnaka91/mabo/commit/4dd6ba36d68368a05aea49a9db98bd44e6ca9a99))
  > The decoding didn't properly handle unknown field IDs. By introducing a
  > new marker that is combined with the field ID, it allows the decoder to
  > skip over the content of a field and continue properly decoding any
  > element, without knowing the exact detail of each field.
- Allow to put comments on the schema file itself ([9200cf3](https://github.com/dnaka91/mabo/commit/9200cf34e6303085aaaea6658552577af756409c))
  > It's now possible to attach a global comment to the root of the schema
  > by placing a comment on the very top. The comment block needs to be
  > separated by a newline to split between schema and definition comments.
- Allow to omit IDs and derive them instead ([76a8649](https://github.com/dnaka91/mabo/commit/76a8649b4119b1d52cd5cc0d3eea5ce46b6d6e04))
  > Similar to Rust basic enums, the identifier for each variant doesn't
  > have to be specified explicitly. Instead it can be derived by continuing
  > to count upwards from the last explicit value.
  > 
  > Therefore, now the field IDs as well as enum variant IDs can be omitted
  > and are derived by the compiler. They can be mixed as well to create
  > gaps in the ID range.
- Rename project to `Mabo` ([1085e24](https://github.com/dnaka91/mabo/commit/1085e2499a76284f1df4af3641ffe58c9a3293ee))
- Introduce Mabo project files ([b5798cf](https://github.com/dnaka91/mabo/commit/b5798cf1cadfd057a14a94c9aa9890594f565857))
  > Adding dedicated `Mabo.toml` files allows to define the root of projects
  > as well as kicking off the start of project metadata and future
  > packaging.
  > 
  > This can later extend into an ecosystem that allows to distribute schema
  > file collections and consume them through a dependency management
  > system.
- Account for optional IDs in the tmLanguage definition ([d27361e](https://github.com/dnaka91/mabo/commit/d27361e5ca0ec02bad3a3afd9344eacdc7e3ccea))
- Switch from full/fat to thin LTO ([d5ccf61](https://github.com/dnaka91/mabo/commit/d5ccf61a4fe0b0fca45807d996d09308c604abc5))
  > This slightly increases binary size, but provides much better compile
  > speed in release mode. Overall performance should not decrease a lot and
  > if needed later, full/fat LTO can be turned back on again.
- Track keywords, punctuation and delimiters ([e8e42ef](https://github.com/dnaka91/mabo/commit/e8e42ef2b63a2fbb13baf1a0b8070d3b9da0dba7))
  > In addition to the more valuable parts of a schema that describe the
  > data structures, now the remaining pieces that make up a schema are
  > tracked as well.
  > 
  > That means characters like commas, semicolons, open/close braces and
  > others are collected so their exact location in the source file can be
  > used by the LSP and other components.
  > 
  > A few places still don't track this information as they need some
  > further refactoring first, but should be doing so in the following
  > commits.
- Track double colon location in external type paths ([6a6de5a](https://github.com/dnaka91/mabo/commit/6a6de5a0441d01cd04049ba8505d79548edb1330))
  > Keep track of the double colons that separate the individual segments of
  > the path in external types.
- Track more element locations of import statements ([ba12d5e](https://github.com/dnaka91/mabo/commit/ba12d5e3b11e04999a8d97c1b077f9a3c6b5c575))
  > Only the keyword location was repoted as semantic token (in the LSP),
  > but more information was already available which is now properly added
  > to the token list.

### 🐛 Bug Fixes

- _book_: Correct the favicon URL ([d66ad2e](https://github.com/dnaka91/mabo/commit/d66ad2e9f0cf963a165a5b57116faa34b98a9f2a))
  > This was set to the root, but the site is currently deployed under the
  > `/stef/` path on Codeberg Pages.
- _book_: Point directly to first user guide/reference pages ([22e20fe](https://github.com/dnaka91/mabo/commit/22e20fe0a00bccdef9ff5676a77eaea922e73e29))
  > Instead of pointing to an empty page, link to the first currently filled
  > page of both the user guide and reference.
- _book_: Adjust path for the .nojekyll file ([7c1d68c](https://github.com/dnaka91/mabo/commit/7c1d68c8289cfb122d481806316e05eeefad8c49))
  > The creation of the `.nojekyll` file was off, due to the previous `cd`
  > into the book folder.
- _doc_: Collect overview docs until the first empty line ([160fbcd](https://github.com/dnaka91/mabo/commit/160fbcdaa0c006be397fb2a72706813b71129c74))
  > In Markdown, a line break is only inserted if there is at least one
  > empty line between paragraphs. Therefore, collect all lines until the
  > first empty line is found, instead of simply taking the first line only.
  > 
  > This fixes potential cut-off of the overview item docs.
- _doc_: Fix broken PR links in the changelog ([359f02f](https://github.com/dnaka91/mabo/commit/359f02f284e9ac6d46f7dcbf22abd094a99a9123))
  > Due to how repo placeholders were used in git-cliff, those were mangled
  > and then later not properly replaced with the URL anymore.
- _lsp_: Remove closed documents from the state ([a84018c](https://github.com/dnaka91/mabo/commit/a84018c6206cb17c00284bc792901fecf6785700))
  > To avoid growing the list of open files indefinitely, remove them from
  > the state of the server when closed in the editor.
- _lsp_: Properly reply with error messages ([740c46d](https://github.com/dnaka91/mabo/commit/740c46db9a297716ceab4b0105e5ff2ae725716d))
  > If an error happens in the request handler, return a proper error to
  > back to the client instead of shutting down the whole server.
- _lsp_: Properly reply to unknown requests ([6bb08d5](https://github.com/dnaka91/mabo/commit/6bb08d51b865da680dccf875e07e787bf9efccd9))
- _lsp_: Solve the hanging shutdown problem ([a9836cb](https://github.com/dnaka91/mabo/commit/a9836cb181d6292c0e53c28eabe10fd4a1e0536f))
  > The connection instance was kept alive, which caused the background
  > threads to idle as they were waiting for all channels to be dropped.
- Don&apos;t double wrap optional types in decode ([a6d3d4b](https://github.com/dnaka91/mabo/commit/a6d3d4bde28d28acb0afba123949ed7e5cbfeb98))
- Extend playground and correct issues ([ed24491](https://github.com/dnaka91/mabo/commit/ed24491a8361574bb295d34aad6fc70ed408777b))
- Missing semicolon in tuple structs ([d616e92](https://github.com/dnaka91/mabo/commit/d616e92414072a396e448f7b8cd39607b69fbbbe))
- Adjust for new clippy lints ([8855572](https://github.com/dnaka91/mabo/commit/88555726ef9e9dd38ddc907a8fb6dbfd4884040f))
- Compile more schemas and fix errors ([ba90911](https://github.com/dnaka91/mabo/commit/ba9091181ca93c8e94cf6638d5d23705f725c14a))
- Create specialized encoding for non-zero types ([39420b8](https://github.com/dnaka91/mabo/commit/39420b80ef0a985f96ab77f42abd9dec508f4621))
- Suppress warning about dereference operator ([21b2f69](https://github.com/dnaka91/mabo/commit/21b2f6996254ed945af4ff5acc249870785b7f68))
- Correctly check aliases and imports unique identifiers ([9c88e32](https://github.com/dnaka91/mabo/commit/9c88e3291a169bb237e726e099468fd14f5766c5))
  > Aliases couldn't be checked to a non-optimal struct format in its first
  > version and imports were skipped when then don't have a final type
  > element.
- Make tuple struct fields public in Rust ([7dcc660](https://github.com/dnaka91/mabo/commit/7dcc66075002c644d433995240f1c5d5c69ac206))
  > If defining a unnamed struct (tuple struct), the Rust generator wasn't
  > marking the fields as public, making the inaccessible.
- Adjust imports to look in the parent module ([ca222db](https://github.com/dnaka91/mabo/commit/ca222db151cc115d135caf45e7424ac4aadadbf2))
  > For now, all imports are for external schemas only and those are
  > expected to be placed next to the module where these imports are
  > declared. Therefore, the generated Rust code should look in the parent
  > module instead of in its own modules.
- Encode options in unnamed fields properly ([0a4f3d5](https://github.com/dnaka91/mabo/commit/0a4f3d5cca897d9faa738cd059bebcbee70b9a8b))
  > Special handling for optional values was already done for named fields,
  > but the same behavior was missing from unnamed fields.
- Comments were rendered incorrectly after the type split ([9116c87](https://github.com/dnaka91/mabo/commit/9116c871ef1ed56b0579822cafdca6f3467d5d7b))
  > When splitting up the `Comment` type to allow tracking the span of
  > individual lines, the printing in both the schema formatter and code
  > generators broke.
  > 
  > In addition, several test snapshots were not updated during that change
  > either.
- Size wasn&apos;t calculated correctly for zero value varints ([e29ee00](https://github.com/dnaka91/mabo/commit/e29ee00f938fe560ca11a889777f272380074520))
  > A special case for zero integers wasn't covered when calculating the
  > required size in the varint encoding.
  > 
  > Also, extend the unit tests for the module to verify all en- and
  > decoding functions work correctly.
- Save JSON version of the TextMate language definition ([5f7838d](https://github.com/dnaka91/mabo/commit/5f7838dca53fb81b2cefd9719b0c7cc91558c2f8))
  > This definition was originally ignored as it is auto-generated from the
  > YAML version. But the book no depends on it and including it in Git
  > avoids suddenly build errors for fresh clones.
- Correct size calculation for field IDs ([e56dd8d](https://github.com/dnaka91/mabo/commit/e56dd8d9c32f64d0453bfcf238a49305ec2e48f4))
  > The calculation for required bytes of a field ID did not take the
  > recently added field encoding into account, which could potentially
  > result in too small values.
- Expand tmLanguage regexes for tuples and arrays ([b589eac](https://github.com/dnaka91/mabo/commit/b589eac40d2d83ef5a5eb224229161fc177d7fe3))
  > The tuples and arrays were correctly parsed for individual types, but
  > the field regex didn't include possible token which made it fail to
  > include those types in them matching.
- Update winnow to fix broken integer parsing ([1f1790a](https://github.com/dnaka91/mabo/commit/1f1790a188db58b01e50804566d86265b919138b))
  > Version 0.6 introduced a bug that caused the literal `0` not to be
  > parsed as valid integer anymore. This was fixed with 0.6.1.

### 📚 Documentation

- _book_: Extend the schema creation guide ([f35a942](https://github.com/dnaka91/mabo/commit/f35a942250d86043346b5c81659d7490b8de2b94))
  > Explain the components of a simple schema and describe the commonly used
  > data types.
- _book_: Use Vuepress for the book ([9a0b0d9](https://github.com/dnaka91/mabo/commit/9a0b0d9a05ad6ef704ccc1d7ec7964f8658c2cfa))
- _book_: Tweak colors and page details ([c43d077](https://github.com/dnaka91/mabo/commit/c43d077873b2719eb1f732b165c1aab033d0c5ab))
  > Adjust the branding colors for better visual contrast and flip a few
  > config values of Vitepress.
- _book_: Expand the introduction page ([6f3c902](https://github.com/dnaka91/mabo/commit/6f3c902948a1933ccd8e215f8e4f638fe77aa06f))
- _book_: Explain how identifiers are encoded in the wire format ([e45b743](https://github.com/dnaka91/mabo/commit/e45b74308ad14bd025d847f2888f194e632cb2d8))
- _book_: Fix a few typos ([c891908](https://github.com/dnaka91/mabo/commit/c8919089d23257dd811831e9f49345aa67fb74c9))
- _book_: Include the changelog in the book ([5b32249](https://github.com/dnaka91/mabo/commit/5b32249f396875dfc26d4546129312b597b8e0ad))
- _book_: Describe the content of the Mabo.toml project files ([a31ecc2](https://github.com/dnaka91/mabo/commit/a31ecc2b118ef3a813b9effda8b8a0d5c161187c))
  > Explain the need for the project files as well as all the possible
  > settings that can be set as content.
- _book_: Add additional metadata to each page ([601ac4c](https://github.com/dnaka91/mabo/commit/601ac4c9d59098f956abd58d619522dd50990b82))
  > Apply a few metadata improvements reported in Lighthouse, as well as
  > increasing the information for OpenGraph display.
  > 
  > This allows for a richer link display on several platforms when shared.
- _book_: Describe details about derived identifiers ([616dfbf](https://github.com/dnaka91/mabo/commit/616dfbfa26eb4730f84794c0470bc60d888e997c))
  > Explain that it is possible to automatically derive identifiers instead
  > of always explicitly defining them. Also, move some encoding details
  > from the ideas section to the wire format as those are already
  > implemented.
- _book_: Remove table of contents ([aeaa69d](https://github.com/dnaka91/mabo/commit/aeaa69db163aa48c09c9262d73129d13a05b2fe6))
  > The table of contents isn't needed as the automatic "On this page"
  > navigation fulfills the same purpose.
- _book_: Create files for each navigation element ([9f45b86](https://github.com/dnaka91/mabo/commit/9f45b86da9315e92579d5b8eaeecf01c8e192665))
  > Some sections in the navigation didn't have a file associated with it
  > yet, leading to 404 pages. Also, extend the generators category with all
  > planned language generators (still empty though).
- _book_: Extend the docs about cli and generators ([a1474d9](https://github.com/dnaka91/mabo/commit/a1474d92569f3fb0fbb9cab2c752b25dd63e1770))
  > Extend the docs around these sections and create an example in
  > `mabo-cli` which auto-generates the doc pages for the CLI subcommands.
- _book_: Remove implemented ideas ([d8b987f](https://github.com/dnaka91/mabo/commit/d8b987f06f30bc5700704522b5b2834140c27e96))
  > Remove the sections from the ideas sections that have already been
  > implemented recently.
- _book_: Remove incomplete sections ([e06754d](https://github.com/dnaka91/mabo/commit/e06754d138185fb0c467df1e567c34ca8798f84d))
  > Remove the sections from the config which are clearly not ready and
  > basically a blank space. The Markdown files are kept in place as
  > reminder for these features.
- _book_: Explain the schema importing mechanism ([c6b3688](https://github.com/dnaka91/mabo/commit/c6b36889d6f27cedcc36385e8f9bc7c87af71c71))
  > This section was still empty and despite even now being somewhat
  > incomplete, at least provides some basic explanation of the importing
  > system.
- _parser_: Add missing doc for new field ([7f4ca98](https://github.com/dnaka91/mabo/commit/7f4ca98c236a41e505bcaa70a2df5fc3aae85b7a))
- _vscode_: Add dedicated readme for the extension ([38af273](https://github.com/dnaka91/mabo/commit/38af273445a37dc56e04a90c85f0f57ae5621a1a))
- Generate more stylish changelog ([5319fb3](https://github.com/dnaka91/mabo/commit/5319fb3417a830042e7bc220fe283046923da349))
- Add changelog ([5b2a15c](https://github.com/dnaka91/mabo/commit/5b2a15cad70e53c6c39a93c395fbe8f80382ae56))
- Update flatbuffers homepage in the book ([c469e4e](https://github.com/dnaka91/mabo/commit/c469e4e966cfb3866d08369f813eb999a4c3032d))
- Update Java links to 21 release ([e151095](https://github.com/dnaka91/mabo/commit/e151095fd37e1379070255e4a233d75f999deac3))
- Expand user guide for basic setup ([d44c12d](https://github.com/dnaka91/mabo/commit/d44c12d16e32e4518dd3c60547a33ca0a50eb74f))
- Add a few (far) future ideas ([bbdc490](https://github.com/dnaka91/mabo/commit/bbdc49023e6f3121d6498bc1043bcbd05c06229c))
  > Outline a few ideas that would be great to have in the future, but
  > require a significant amount of work. Therefore, these have no time
  > frame attached to them.
- Update changelog and improve change items ([fa01844](https://github.com/dnaka91/mabo/commit/fa01844e6a708f704ac3dc7774e7ffa2932facec))
  > Re-generate the changelog and in addition, tweak the configuration for
  > changelog generation for a better formatted output and fix the commit
  > links as well.
- Adjust git-cliff config for latest version ([7d58f6b](https://github.com/dnaka91/mabo/commit/7d58f6b961e6616a8b97a9ae24cff1dcd1ad7ea1))
  > The latest v1.4.0 of `git-cliff` changes the tag_pattern setting from a
  > glob pattern to a RegEx pattern. Therefore, the current setting needs
  > slight tweaking.
- Slightly tweak the `git-cliff` configuration ([d7dc280](https://github.com/dnaka91/mabo/commit/d7dc2808ea944af0c61275965301077cc537ae74))
  > Simplify a few settings in `git-cliff`, but most importantly escape the
  > commit messages as they can contain some characters that can be
  > misinterpreted as HTML tags.
- Update project setup instructions ([66b95bf](https://github.com/dnaka91/mabo/commit/66b95bf0029e5e8fe449ac74d4967147359f7c01))
  > Correct the code samples in the guide to use the new APIs for generating
  > Rust code and including the files.
- Explain the min/max amount of tuple elements ([5e74489](https://github.com/dnaka91/mabo/commit/5e74489312030a9ec33e20000755c678bdcf7911))
  > Clarify why tuples must contain between 2 and 12 types, especially the
  > arbitrary limit of 12 elements.
- Highlight schema files ([b6b595c](https://github.com/dnaka91/mabo/commit/b6b595c47031061fe3bf9a983e5b7d3140ced043))
  > Adjust the highlighting component of mdbook to get syntax highlighting
  > for the Stef schema files. As a nice side effect, remove any unused
  > languages from the highlighter to reduce the file size.
- Fix a few formatting errors in the book ([78ba8eb](https://github.com/dnaka91/mabo/commit/78ba8eba0ab934360836d4b5352044531406ed85))
  > Some anchors were named wrong and offsets in the Python files were
  > wrong, resulting in unwanted whitespace.
- Add (empty) entries for existing generators ([078c50a](https://github.com/dnaka91/mabo/commit/078c50a9cddccc8b67e141e4e7d4ce19fd31a731))
- Vastly improve API docs throughout all crates ([afb8a0e](https://github.com/dnaka91/mabo/commit/afb8a0e744e9dbbb3dd6b49c913383d4272d3119))

### ⚡ Performance

- Extend the benchmark for better resolving checks ([2074449](https://github.com/dnaka91/mabo/commit/207444976b11533fbbce0d958a39472b34f3eebb))
  > The current schema generated for benchmarks on large schemas didn't
  > generate any definitions that use type references. Therefore, the
  > benchmark didn't give good insight on type resolution timing.
- Use a faster hasher in compiler and LSP ([4eb37aa](https://github.com/dnaka91/mabo/commit/4eb37aaafc4da7a8b73b6aae31c38672cb8554c9))
  > By applying the same hasher as in Rust's compiler, performance can be
  > improved wherever a hash map or set is used.
  > 
  > This currently has the most visible impact on the compiler's validation
  > step as it internally makes heavy use of hash maps.

### 🚜 Refactor

- _compiler_: Simplify some type resolution steps ([ae1cf34](https://github.com/dnaka91/mabo/commit/ae1cf344feef941dc7f7afe6e8dca9e30c92b7d5))
- _go_: Simplify indenting logic ([7425a11](https://github.com/dnaka91/mabo/commit/7425a118030d080acd025bbdbc7a2bc4ccc80058))
  > Use a dedicated type that implements the `Display` trait, which allows
  > for a much simpler way of expressing and forwarding indents.
- _lsp_: Avoid unwraps and properly handle errors ([bcbb016](https://github.com/dnaka91/mabo/commit/bcbb016a1180a38311cbaa2709979494c0b2eb77))
- _lsp_: Move logging logic into its own module ([ef1139b](https://github.com/dnaka91/mabo/commit/ef1139be9290846ccd197c4bbeeddca48ffcff50))
- _lsp_: Replace tower-lsp with lsp-server ([51645a8](https://github.com/dnaka91/mabo/commit/51645a88c9b9c0d3e13a47da429ab9f7ef28d67f))
  > The `tower-lsp` crate appears to be somewhat unmaintained in the recent
  > months and the `lsp-server` crate is made by the people behind
  > rust-analyzer.
  > 
  > In addition, the LSP server doesn't need a full async runtime as LSP
  > communication is only with a single client and done in serial.
  > 
  > This change cuts down on several dependencies, reducing both build time
  > and binary size.
- _lsp_: Move elements out of main.rs ([225a683](https://github.com/dnaka91/mabo/commit/225a6831e458f26e83bbd0b4dd993f51b6621097))
- _lsp_: Group handler related modules together ([f80d7da](https://github.com/dnaka91/mabo/commit/f80d7da1e64dd71536260a55ec0226397a63540e))
  > Move all the modules that are specific to the handler into their own
  > module to thin out the root module.
- _lsp_: Use the same pattern for similar functions ([0e15394](https://github.com/dnaka91/mabo/commit/0e15394e43d761ff1572ec80e2138ad26054f0e0))
  > Adjust a few internal functions with similar patterns to behave the same
  > way.
- _lsp_: Remove unused env var settings ([d8d57f1](https://github.com/dnaka91/mabo/commit/d8d57f15f8082eed655a6e5b9159cb4727dd2d55))
  > These environment variables (which controlled the logging output) were
  > originally used, but for some while the logging settings are already
  > fixed in code.
- _vscode_: Run TypeScript compiler on the extension ([d926afa](https://github.com/dnaka91/mabo/commit/d926afa3bb41fd969ac5103ebae82ddb6de6cbea))
  > As part of the linting step, run the `tsc` TypeScript compiler on the
  > code base to ensure the type usage is actually correct, as esbuild only
  > strips type information.
  > 
  > Also, apply the default tsconfig.json from fresh Bun projects (`bun
  > init`).
- Generate definitions and impls together ([b32bcfd](https://github.com/dnaka91/mabo/commit/b32bcfd8630bc445421ce32b784de6601659aade))
- Rename test file ([86536c9](https://github.com/dnaka91/mabo/commit/86536c919c26934a439e4ebd8bac631e92941dc7))
- Switch to more lightweight bench crate ([3870a6c](https://github.com/dnaka91/mabo/commit/3870a6c0db7dbbf720c11f812d5e0b94b57939c3))
- Move common deps to the workspace root ([584fa3e](https://github.com/dnaka91/mabo/commit/584fa3eb866c2fa67fc43b1fd918a2fc5f4b379f))
- Create benchmarks for the compiler ([f31f94e](https://github.com/dnaka91/mabo/commit/f31f94e3ce5141461b6d65973e702aca822ad25d))
- Enable more clippy lints and unify them ([3c206de](https://github.com/dnaka91/mabo/commit/3c206de825d94ed2559d93fba79ff41f1155a0af))
  > Ensuring consistent code quality through lints within all the crates
  > that are part of this project.
- Transform some tests into snapshot tests ([0f69ed5](https://github.com/dnaka91/mabo/commit/0f69ed5bf740b6ac113001153bf4632c9a651ee1))
- Rename stef-cli crates binary to stef ([92f039b](https://github.com/dnaka91/mabo/commit/92f039b82d28686d26fdb36d01b7c728f241f9fc))
  > Although the package is named `stef-cli` to not clash with the `stef`
  > library crate, the binary should still be named `stef`.
- Use underscore in schema file names ([b43e779](https://github.com/dnaka91/mabo/commit/b43e779167d2010f4cde191620598f8c721f8388))
  > As part of the type resolution, module names are derived from the file
  > name. Thus, the file name must adhere to the module naming rules.
  > 
  > This replaces all dashes `-` with underscores `_` in the schema files
  > used for snapshot testing.
- Simplify transformation of name elements ([2acd7b7](https://github.com/dnaka91/mabo/commit/2acd7b740859d2c0ea713b9fa20789e653c75013))
  > Instead of manually constructing the `Name` instances, use the From/Into
  > traits to do so automatically and with less code.
- Add schema as snapshot description ([137d2cd](https://github.com/dnaka91/mabo/commit/137d2cdcab8f73172ee1214d91ca2c12c50b04ed))
  > Include the source schema into snapshots for easier verification as this
  > displays the schema together with the output of each snapshot in review
  > mode.
- Reorganize the stef-compiler crate ([5d430be](https://github.com/dnaka91/mabo/commit/5d430bed3d153cedba0d07015b2ec7e352262f0f))
  > Shift several components around for better structure, expose all the
  > possible errors on the API and document any publicly visible types.
- Improve syntax highlighting ([827ae44](https://github.com/dnaka91/mabo/commit/827ae44fa1074189c49bec6829b415013692e45f))
  > Add a few more token markers for literals and overall improve the way
  > elements are detected and marked.
- Centralize lint settings with new Rust 1.74 feature ([a9e0800](https://github.com/dnaka91/mabo/commit/a9e08004f29b4b6c29aea7a8bf74a8ea361fa876))
  > Rust 1.75 stabilizes a new `[lints]` setting in Cargo.toml, that allows
  > to define common lints in a central place instead of having to
  > repeatedly define them and keep them in sync in each crate of the
  > workspace.
- Simplify several code definitions ([b166664](https://github.com/dnaka91/mabo/commit/b166664f1e3cd4b4dbccd9bed0cbc268bc258774))
  > Besides minor code cleanups and deduplications, an explicit lifetime
  > could often be elided.
- Omit spans from debug prints ([5b79acb](https://github.com/dnaka91/mabo/commit/5b79acbf79f0cbc87c92cb36203d23c53ff7d4c1))
  > Spans are rarely useful in the `Debug` representation of a struct and
  > clutter the snapshots a lot. A custom derive macro mimics the stdlib's
  > macro but filters out any fields that use the `Span` struct.
- Move doc, lsp and meta over to the compiler structs ([facee4a](https://github.com/dnaka91/mabo/commit/facee4af38fde43accf3c9070d773f28d0f20a82))
  > These crates were still working directly on top of the `stef_parser`
  > crate's types, but can get the same benefits as the other crates by
  > depending on the `stef_compiler`'s simplified structs instead.
- Replace owo-colors with anstream/anstyle ([e831c05](https://github.com/dnaka91/mabo/commit/e831c05697ee7cdca426f4596dd1bbceb99d4b4c))
  > The anstream and anstyle crates look more promising and additionally
  > allow to expose the on the API if needed, without restricting to a
  > specific terminal coloring crate.
  > 
  > Also, the owo-colors crate contained some unmaintained dependencies.
- Switch generic types from unnamed to named ([d0e6ec1](https://github.com/dnaka91/mabo/commit/d0e6ec1f095be22f98786a84debfeb6f29ec0b01))
  > As the amount of values contained in generic built-in types grows, it's
  > better to convert these to named structs so the values have a clearer
  > meaning. Also, the `hash_map` type is now split into individual fields
  > for the key and value type.
- Introduce a container for punctuation ([554008d](https://github.com/dnaka91/mabo/commit/554008d01e8548aa1025b21475423c5077001192))
  > Many elements in Mabo schema files use some form of punctuation to
  > separate from another. This common logic could be centralized in a
  > custom container that keeps track of punctuation spans and takes care of
  > formatting logic.
  > 
  > This adds missing tracking of token spans that haven't be tracked before
  > yet.
- Simplify token parsing ([2a9167c](https://github.com/dnaka91/mabo/commit/2a9167ce264f3ff2fa623d6002cdcd53366499ee))
  > Create a `surround` parser for delimiters that performs the common task
  > of wrapping any parser with an opening and closing delimiter.
  > 
  > Also, create some helper parsers for each token to reduce the amount of
  > boilerplate wherever the tokens are used together with other parsers.

### 🧪 Testing

- Add snapshot tests to stef-build ([1313fe9](https://github.com/dnaka91/mabo/commit/1313fe9f99cceee8a883791c99e318768e27f801))
- Enable more snapshot tests and fix errors ([85938a4](https://github.com/dnaka91/mabo/commit/85938a4a7532d034b7eccbea1643a95a84434954))
- Add playground sample for tuple structs ([6f9e8ab](https://github.com/dnaka91/mabo/commit/6f9e8abfa72dd1f4bdaef6bbf57e1981f16604fc))
  > To ensure the tuple variant is working and doesn't include oversights
  > like the previous missing `pub` visibility modifier.
- Use runtime arguments in Divan benchmarks ([#1](https://github.com/dnaka91/mabo/issues/1)) ([07da8a0](https://github.com/dnaka91/mabo/commit/07da8a0e8652431af7ec8388a4f64fe7e206eeee))
  > This greatly reduces compile times and is not limited to arrays/slices.

### ⚙️ Miscellaneous Tasks

- _book_: Fix a minor typo ([b04202d](https://github.com/dnaka91/mabo/commit/b04202d482f8d408f2de58368ec5fe4945541a8a))
- _ci_: Setup GitHub Actions to deploy the book ([cf24bbb](https://github.com/dnaka91/mabo/commit/cf24bbb0e55955e37233f9719f046bb482a8d712))
- _ci_: Improve path filters for book deployment ([8b79f6f](https://github.com/dnaka91/mabo/commit/8b79f6f03787edde0db881ab0e61b6e163924709))
- _ci_: Create .nojekyll marker file for the book ([b51681f](https://github.com/dnaka91/mabo/commit/b51681f377640421249d9d9e180446f8ae16142a))
- _doc_: Fix ambiguous links in the Rust docs ([e5d73fe](https://github.com/dnaka91/mabo/commit/e5d73fe68759aab1721f74e9845c57ea74fbd911))
- _lsp_: Remove duplicate log message ([e8cfa08](https://github.com/dnaka91/mabo/commit/e8cfa08e0798c924c61c555fc74359ad2c660bb0))
- Initial commit ([5eb2f2b](https://github.com/dnaka91/mabo/commit/5eb2f2b9687146363974ea645de22a8441e890a1))
- Update checkout action to v4 ([4d753d8](https://github.com/dnaka91/mabo/commit/4d753d8b30ef3ee7d7e463fb2e7f594aee86d8e7))
- Minor code cleanup of unused code ([a624300](https://github.com/dnaka91/mabo/commit/a6243007663ddcf1d4a9da09c9b4b6514dab0db6))
- Correct several typos ([c74b3a7](https://github.com/dnaka91/mabo/commit/c74b3a7aadbb4abd7da5e6e7d8901ebc103f5ccc))
  > Correct some spelling mistakes and configure the `typos` tool for a few
  > false-positives.
- Add cargo-release configuration ([4f2c951](https://github.com/dnaka91/mabo/commit/4f2c951e28a85f600b1ec2db69665d721ce25cc0))
  > Should make releasing new versions easier, once a point of a somewhat
  > stable first version is reached.
- Create helper tasks to install the LSP server and VSCode extension ([035c1aa](https://github.com/dnaka91/mabo/commit/035c1aae1c57393bc4445ca0570d6e4cef78fcc9))
- Update helper script from full project builds ([c44888a](https://github.com/dnaka91/mabo/commit/c44888aead4049e72bafb2a9cd0445c087cbf0e2))
  > The script was still using pnpm in places and the custom highlighting
  > component wasn't built before building the full book.
- Remove outdated GitHub Actions config ([cd9877f](https://github.com/dnaka91/mabo/commit/cd9877f2aa3bbcc25b5ba1b8985fd982ff3d0c1e))
- Fix Just task for link checking ([24e1520](https://github.com/dnaka91/mabo/commit/24e15209799f6285ca3a55f7145556145cb61f30))
- Improve readme with badges and project status ([069f788](https://github.com/dnaka91/mabo/commit/069f7882f9abd1d14b697b38314a5db2976b895a))
- Update snapshots after several Go codegen fixes ([b38e1ca](https://github.com/dnaka91/mabo/commit/b38e1cad5b861e59774a03fb5768d2284376787e))
- Bump MSRV to 1.76 and update dependencies ([554b05b](https://github.com/dnaka91/mabo/commit/554b05b8799abaaa06caa73260882d22ba4856f2))
  > Deprecations in `winnow` as well as increases in MSRVs in the
  > dependencies. Bumping the MSRV to the very latest Rust version as there
  > are no MSRV promises as of now and `clap` bumped their MSRV.
- Update cargo-deny config to v2 ([5cba463](https://github.com/dnaka91/mabo/commit/5cba463f8063b45756ca1591eec8005594827408))
  > New configuration layouts were introduced for _advisories_ and
  > _licenses_ sections. These will become the default at some point and
  > it's good to adopt early.
- Limit log level to info for release builds ([dda49f6](https://github.com/dnaka91/mabo/commit/dda49f6a23a8f3b90c8e445c256bfabf86e376db))
  > Trace and debug logs are mostly useful during development and not needed
  > for release builds. By activacting the `log` crate's feature flag, these
  > levels can be filtered out during compile time.
  > 
  > The effects are likely better runtime performance and most notably
  > reduced binary size.
- Update license link in readme ([d8f7061](https://github.com/dnaka91/mabo/commit/d8f706154a4adc14397fcfa12e9514db9571381d))

<!-- generated by git-cliff -->
