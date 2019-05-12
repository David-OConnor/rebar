# Changelog

## v0.3.4
- The `update` fn now returns `impl vdom::Updater`, allowing more concise syntax. Non-breaking.
- `.mount()` now accepts elements. Deprecated `.mount_el()`
- The `log` function and macro now support items which implement `Debug`
- Removed deprecated `routing::push_path` function (breaking)

## v0.3.3
- Added `seed::update` function, which allows custom events, and updates from JS

## v0.3.2
- Top level view functions can now return `Vec<El<Ms>>`, `El<Ms>`, or something else implementing
the new ElContainer trait

## v0.3.1
- Top level view functions now return `Vec<El<Ms>>` instead of `El<Ms>`, mounted directly to
 the mount point. (Breaking)
- `push_route()` can now accept a `Vec<&str>`, depreciating `push_path()`
- Fixed a bug where window events couldn't be enabled on initialization

## v0.3.0
- `update` function now takes a mutable ref of the model. (Breaking)
- `Update` (update's return type) is now a struct. (Breaking)
- Async, etc events are now handled through messages, instead of passing `App`
through the view func. (Breaking)
- Fixed some bugs with empty elements
- Internal code cleanup
- Added commented-out release command to example build files
- Added more tests

## v0.2.10
- Routing can be triggered by clicking any element containing a `Href` attribute
with value as a relative link
- Internal links no longer trigger a page refresh
- Models no longer need to implement `Clone`
- Fixed a bug introduced in 0.2.9 for `select` elements

## v0.2.9
- Added a `RenderThen` option to `Update`, which allows chaining update messages
- Added a `.model` method to `Update`, allowing for cleaner recursion in updates
- Improved controlled-component (sync fields with model) logic

## v0.2.8
- Reflowed `El::from_html` and `El::from_markdown` to return `Vec`s of `El`s, instead of wrapping
them in a single span.
- Improved support for SVG and namespaces
- Added `set_timeout` wrapper

## v0.2.7
- Fixed a bug where `line!` macro interfered with builtin
- Fixed a bug with routing search (ie `?`)

## v0.2.6
- Fixed a bug where children would render out-of-order
- Improved vdom diffing logic

## v0.2.5
- Attributes and Events now can use `At` and `Ev` enums
- Routing overhauled; modelled after react-reason. Cleaner syntax, and more flexible
- Input, Textarea, and Select elements are now "controlled" - they always
stay in sync with the model
- index.html file updated in examples and quickstart to use relative paths,
which fixes landing-page routing

## v0.2.4
- Changed render func to use a new pattern (Breaking)
- Default mount point added: \"app\" for element id
- View func now takes a ref to the model instead of the model itself
- Routing refactored; now works dynamically
- Update function now returns an enum that returns Render or Skip,
to allow conditional rendering (Breaking)
- Elements can now store more than 1 text node

## V0.2.3
- Fixed a bug where initially-empty text won't update
- Added more tests
- Exposed web_sys Document and Window in top level of Seed create, with .expect
- Modified build scripts to keep the wasm output name fixed at 'package', simplifying example/quickstart renames
- Tests now work in Windows due to update in wasm-pack

## V0.2.2
- Overhaul of fetch module
- Added server-integration example

## V0.2.1
- Added support for custom tags
- Added `class!` and `id!` convenience macros for setting style

## v0.2.0

- Added high-level fetch api
- Added routing
- Added element lifecycles (did_mount, did_update, will_unmount)
- Added support for updating state outside events
- Added server_interaction, and homepage (this site) examples

## v0.1.0

- Initial release