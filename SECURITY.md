# Security

## Report a vulnerability
Please use Github's Security Advisories panel on this repository for that! We'll try to come back at you promptly.

TODO: document more

## Strategy
This is the start of establishing a strategy and a general review of the security. This is far from done, only what's documented here has been developed.

### Possible threats
These are the possible threads that come to my mind, there are probably others. Most of them would be potentially feasible if an XSS vulnerability is present.

- A malicious Markdown file opened with DME, which would execute calls to Tauri commands to read random Markdown files on the disk. Tauri commands could also be used to run some search to find some keywords presence. Or to read arbitrary files or images on disk. Once read, these contents could be send their content to a public server.
- Making DME crash on very special Markdown file
- Making DME download some special SVG containing XSS
- Making queries to localhost servers via image links or XSS
- Having special URLs that execute JavaScript (such as `[great website](javascript:alert('a'))`)
- Including forms that can trigger CSRF attacks on other websites
- Installing malicious TreeSitter grammars that would run random commands during installation or execution, via the build process or via C code in generated parser
- Malicious highlight names in the TreeSitter grammar, making the CSS generation from the `Theme` to include XSS or escaping the `class` attribute in `<span>` for tokens

### Strategy against XSS
This strategy is tested a bit in `app/core/util/security.rs` for high level functions of the library.

- Let `tree-sitter-highlight` escape HTML special chars inside the highlighted code
- If the code cannot be highlighted, return an escaped version, via `comrak::html::escape()` function
- `highlight_code()` does not use the Comrak parser to avoid code blocks escapes (via the 3 backticks in the code snippet directly), which could inject some HTML and modify the final UI look
- Clean the final HTML with the [`ammonia` sanitization library](https://docs.rs/ammonia), which removes a lot of things (whitelist based): any JavaScript, any `<style>` and `<script>`. Here are the exceptions we put:
    - Allow 
- This final cleaning is made via wrapper type `Html` where the only way to get the `String` behind it

### Strategy against abuse of Tauri commands
TODO

