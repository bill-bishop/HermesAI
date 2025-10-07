#!/usr/bin/env node
/**
 * markityper-unified-stream.js
 *
 * One stream that emits BOTH:
 *  - { type: "syntax", kind: "line" | "open" | "close", value: string }
 *  - { type: "display", value: string }  // individual visible characters
 *
 * The concatenation of *all* emitted `value`s equals the original input.
 *
 * Supported constructs (demo):
 *   - Line-level: "### " (at start of line)
 *   - Inline: **strong**, *em*, _underline_, `code`
 *
 * Notes:
 * - While inside `code`, other marks (*, _, **) are treated as normal display chars.
 * - The renderer can "type inside unclosed tags" easily with this split.
 */

// ---------- Config ----------
const INCLUDE_TRAILING_SPACE_IN_LINE_SYNTAX = true; // include the space after "###" in the line-level token

// ---------- Hard-coded test input ----------
const INPUT = [
    "### Title **Bold** _Under_",
    "-# List *Italics* and `code`"
].join("\n");

// ---------- Helpers ----------
const Mark = {
    STRONG: "strong",
    EM: "em",
    UNDER: "underline",
    CODE: "code",
};

function closingFor(mark) {
    switch (mark) {
        case Mark.STRONG: return "**";
        case Mark.EM:     return "*";
        case Mark.UNDER:  return "_";
        case Mark.CODE:   return "`";
        default: return "";
    }
}

/**
 * Emit a unified token stream.
 * @param {string} src
 */
async function* unifiedStream(src) {
    const stack = []; // e.g., ["strong","em"]
    let i = 0;

    const atLineStart = (pos) => pos === 0 || src[pos - 1] === "\n";

    while (i < src.length) {
        const ch = src[i];
        const next = src[i + 1];
        const next2 = src[i + 2];

        const inCode = stack[stack.length - 1] === Mark.CODE;

        // 1) Line-level syntax: "### " at start-of-line
        if (atLineStart(i) && src.startsWith("###", i)) {
            if (INCLUDE_TRAILING_SPACE_IN_LINE_SYNTAX) {
                const hasSpace = src[i + 3] === " ";
                const value = hasSpace ? "### " : "###";
                yield { type: "syntax", kind: "line", value };
                i += value.length;
            } else {
                yield { type: "syntax", kind: "line", value: "###" };
                i += 3;
                // do not consume the following space; it will be emitted as display
            }
            continue;
        }

        // 2) Inline code backtick (open/close)
        if (ch === "`") {
            if (stack[stack.length - 1] === Mark.CODE) {
                stack.pop();
                yield { type: "syntax", kind: "close", value: "`" };
            } else {
                stack.push(Mark.CODE);
                yield { type: "syntax", kind: "open", value: "`" };
            }
            i += 1;
            continue;
        }

        if (!inCode) {
            // 3) Strong "**" (open/close)
            if (ch === "*" && next === "*") {
                if (stack[stack.length - 1] === Mark.STRONG) {
                    stack.pop();
                    yield { type: "syntax", kind: "close", value: "**" };
                } else {
                    stack.push(Mark.STRONG);
                    yield { type: "syntax", kind: "open", value: "**" };
                }
                i += 2;
                continue;
            }

            // 4) Em "*" (open/close), but not part of "**"
            if (ch === "*" && next !== "*") {
                if (stack[stack.length - 1] === Mark.EM) {
                    stack.pop();
                    yield { type: "syntax", kind: "close", value: "*" };
                } else {
                    stack.push(Mark.EM);
                    yield { type: "syntax", kind: "open", value: "*" };
                }
                i += 1;
                continue;
            }

            // 5) Underline "_" (open/close)
            if (ch === "_") {
                if (stack[stack.length - 1] === Mark.UNDER) {
                    stack.pop();
                    yield { type: "syntax", kind: "close", value: "_" };
                } else {
                    stack.push(Mark.UNDER);
                    yield { type: "syntax", kind: "open", value: "_" };
                }
                i += 1;
                continue;
            }
        }

        // 6) Default: emit a single visible character
        yield { type: "display", value: ch };
        i += 1;
    }
}

// ---------- Demo runner ----------
async function run() {
    console.log("INPUT:\n" + INPUT + "\n");
    console.log("TOKENS:");
    let concat = "";
    for await (const tok of unifiedStream(INPUT)) {
        concat += tok.value;
        const vis = tok.value
            .replaceAll("\n", "␤")
            .replaceAll(" ", "␠");
        if (tok.type === "syntax") {
            console.log(`syntax(${tok.kind}): "${vis}"`);
        } else {
            console.log(`display: "${vis}"`);
        }
    }
    console.log("\nRECONSTRUCTED MATCHES INPUT:", concat === INPUT);
    if (!INCLUDE_TRAILING_SPACE_IN_LINE_SYNTAX) {
        console.log(
            "Note: trailing space after '###' was emitted as display (flag OFF). " +
            "Toggle INCLUDE_TRAILING_SPACE_IN_LINE_SYNTAX to change."
        );
    }
}

if (require.main === module) {
    run().catch(err => {
        console.error(err);
        process.exit(1);
    });
}
