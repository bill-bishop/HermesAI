#!/usr/bin/env node
/**
 * adapter-micromark.js — HTML-compiler hook that emits a unified stream
 * Tokens:
 *   - { type: "syntax", kind: "line" | "open" | "close", value }
 *   - { type: "display", value }     // one grapheme
 *
 * Works on micromark v4 (Node 18+). No synthetic closers. Round-trip safe.
 */

const { micromark } = require("micromark");
const { combineExtensions } = require("micromark-util-combine-extensions");
let gfmExt, gfmHtml, mathExt, mathHtml;
try { ({ gfm: gfmExt, gfmHtml } = require("micromark-extension-gfm")); } catch {}
try { ({ math: mathExt, mathHtml } = require("micromark-extension-math")); } catch {}

const HAS_SEGMENTER = typeof Intl !== "undefined" && typeof Intl.Segmenter === "function";
function* graphemes(str) {
    if (HAS_SEGMENTER) {
        const seg = new Intl.Segmenter("en", { granularity: "grapheme" });
        for (const { segment } of seg.segment(str)) yield segment;
    } else {
        for (const ch of str) yield ch;
    }
}

/** Small, stable list of *line-level* markers we want as `syntax(kind: "line")`. */
const LINE_TOKEN_NAMES = [
    // ATX headings
    "atxHeadingSequence",
    // Setext underline
    "setextHeadingLine",
    // Thematic breaks
    "thematicBreakSequence",
    // Block quote prefixes
    "blockQuotePrefix",
    // List item markers/prefix
    "listItemPrefix", "listItemMarker", "listItemValue", "listItemPeriod",
    // Fenced code lines (both ``` and ~~~ variants)
    "codeFencedFence", "codeFencedFenceSequence",
];

/** Delimiter-only slices (inline open/close): emphasis, strong, strike, code spans, links/images, etc. */
const DELIM_CHARS = new Set(Array.from("*_~`[]()!<>#|:+-"));
function isDelimiterSlice(s) {
    if (!s || /\s/.test(s)) return false;
    for (const ch of s) if (!DELIM_CHARS.has(ch)) return false;
    return true;
}

/** Build explicit enter/exit maps for known token names so micromark calls us. */
function makeObserverHtml(out, { includeTrailingSpaceInLineSyntax = true, debug = false } = {}) {
    const enter = {};
    const exit = {};

    const handle = (phase) => function (token) {
        const slice = this.sliceSerialize(token);
        const name = token.type;
        if (debug) {
            const show = (slice || "").replace(/\n/g, "␤").replace(/ /g, "␠");
            console.log(`[${phase.padEnd(5)}] ${name.padEnd(28)} "${show}"`);
        }
        if (!slice) return;

        if (name === "data") {
            // Plain text payload
            if (phase === "enter") for (const g of graphemes(slice)) out.push({ type: "display", value: g });
            return;
        }

        if (LINE_TOKEN_NAMES.includes(name)) {
            if (!includeTrailingSpaceInLineSyntax && / $/.test(slice)) {
                out.push({ type: "syntax", kind: "line", value: slice.slice(0, -1) });
                out.push({ type: "display", value: " " });
            } else {
                out.push({ type: "syntax", kind: "line", value: slice });
            }
            return;
        }

        if (isDelimiterSlice(slice)) {
            out.push({ type: "syntax", kind: phase === "enter" ? "open" : "close", value: slice });
            return;
        }

        // Fallback: emit as display text (covers autolink literal payloads, etc.)
        if (phase === "enter") for (const g of graphemes(slice)) out.push({ type: "display", value: g });
    };

    // **** Register handlers for a broad set of names so micromark always calls us ****
    // CommonMark tokens (subset; enough to get full coverage for text + delimiters)
    const COMMON_TOKENS = [
        "data", "atxHeadingSequence", "setextHeadingLine", "thematicBreakSequence",
        "blockQuotePrefix", "listItemPrefix", "listItemMarker", "listItemValue", "listItemPeriod",
        "codeFencedFence", "codeFencedFenceSequence", "codeText", "codeTextData",
        "emphasis", "strong", "strikethrough", "delete", "link", "image",
        "label", "labelMarker", "resource", "resourceMarker",
        "autolink", "autolinkProtocol", "autolinkEmail",
        "characterEscape", "characterReference",
        // Tables, tasks, footnotes (GFM) — we register names but classification relies on slice
        "table", "tableDelimiterRow", "tableDelimiterCell", "tableRow", "tableCell",
        "taskListCheckValue", "taskListCheckMarker",
        "footnoteCall", "footnoteDefinition",
    ];

    for (const name of COMMON_TOKENS) {
        enter[name] = handle("enter");
        exit[name]  = handle("exit");
    }

    return { enter, exit };
}

/**
 * Public API: createUnifiedStream
 * @param {string} src
 * @param {{ gfm?: boolean, math?: boolean, includeTrailingSpaceInLineSyntax?: boolean, debug?: boolean }} options
 */
async function* createUnifiedStream(src, options = {}) {
    const out = [];

    const syntaxExts = [];
    if (options.gfm !== false && gfmExt) syntaxExts.push(gfmExt());
    if (options.math && mathExt) syntaxExts.push(mathExt());

    const extensions = syntaxExts.length ? [combineExtensions(syntaxExts)] : undefined;

    const htmlExtensions = [
        makeObserverHtml(out, options),
    ];
    if (options.gfm !== false && gfmHtml) htmlExtensions.push(gfmHtml());
    if (options.math && mathHtml) htmlExtensions.push(mathHtml());

    // Compile; the returned HTML string is ignored. Our htmlExtension fills `out`.
    micromark(src, { extensions, htmlExtensions });

    for (const t of out) yield t; // forward tokens
}

module.exports = { createUnifiedStream };

/* ----------------- CLI demo ----------------- */
if (require.main === module) {
    const argv = new Set(process.argv.slice(2));
    const demo = argv.has("--demo");
    const debug = argv.has("--debug");

    const INPUT = demo
        ? [
            '### Title **Bold** _Under_',
            '- [x] List *Italics* and `code`',
            '',
            '> Quote line with ~~strike~~ and a URL: https://example.com',
            '',
            '```js',
            'const x = 1;',
            '```',
            '',
            '| a | b |',
            '|---|:--|',
            '|  1| 2 |',
            '',
            'A footnote[^1].',
            '',
            '[^1]: Footnote text'
        ].join('\n')
        : (process.stdin.isTTY ? '# Type or pipe markdown into stdin\n' : require('fs').readFileSync(0, 'utf8'));

    (async () => {
        if (demo) console.log("=== DEMO INPUT ===\n" + INPUT + "\n");

        let out = '';
        let count = 0;
        for await (const t of createUnifiedStream(INPUT, { gfm: true, math: false, debug })) {
            count++;
            out += t.value;
            const vis = t.value.replaceAll(' ', '␠').replaceAll('\n', '␤');
            if (t.type === 'syntax') console.log(`TOK syntax(${t.kind}): "${vis}"`);
            else console.log(`TOK display: "${vis}"`);
        }
        console.log(`\nTOKEN COUNT: ${count}`);
        console.log('ROUND-TRIP OK:', out === INPUT);
        if (out !== INPUT) {
            console.log('\n--- Input ---\n' + INPUT + '\n--- Output ---\n' + out);
        }
    })();
}
