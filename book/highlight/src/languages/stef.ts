/*
Language: Stef
Author: Dominik Nakamura <dnaka91@gmail.com>
Category: common, system
*/

import { HLJSApi, Language } from "highlight.js";

export function stef(hljs: HLJSApi): Language {
  const KEYWORDS = ["mod", "struct", "enum", "const", "type", "use"];
  const LITERALS = ["true", "false"];
  const TYPES = [
    "bool",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "f32",
    "f64",
    "string",
    "bytes",
    "box",
    "vec",
    "hash_map",
    "hash_set",
    "option",
    "non_zero",
  ];
  return {
    name: "Stef",
    keywords: {
      type: TYPES,
      keyword: KEYWORDS,
      literal: LITERALS,
    },
    illegal: "</",
    contains: [
      hljs.C_LINE_COMMENT_MODE,
      hljs.COMMENT("///", /\n/, {}),
      hljs.inherit(hljs.QUOTE_STRING_MODE, {
        begin: /b?"/,
        illegal: null,
      }),
      {
        scope: "string",
        variants: [
          { begin: /b?r(#*)"(.|\n)*?"\1(?!#)/ },
          { begin: /b?'\\?(x\w{2}|u\w{4}|U\w{8}|.)'/ },
        ],
      },
      {
        scope: "symbol",
        match: /'[a-zA-Z_][a-zA-Z0-9_]*/,
      },
      {
        scope: "number",
        variants: [{ begin: /\b(\d+(\.[0-9]+)?)/ }],
        relevance: 0,
      },
      {
        scope: "meta",
        begin: "#!?\\[",
        end: "\\]",
        contains: [
          {
            className: "string",
            begin: /"/,
            end: /"/,
          },
        ],
      },
      {
        match: [/(?:enum|struct|type)/, /\s+/, /[a-zA-Z0-9_]+/],
        scope: {
          1: "keyword",
          3: "title.class",
        },
      },
      {
        match: ["const", /\s+/, /[a-zA-Z0-9_]+/],
        scope: {
          1: "keyword",
          3: "variable.constant",
        },
      },
      {
        match: [/[a-z0-9_]+/, ":", /[a-zA-Z0-9_<>:,& ]+/, /@\d+/, /,?/],
        scope: {
          1: "attr",
          2: "punctuation",
          3: "type",
          4: "symbol",
          5: "punctuation",
        },
        relevance: 2,
      },
      {
        match: [/[a-zA-Z0-9_<>,& ]+/, /@\d+/, /,?/],
        scope: {
          1: "type",
          2: "symbol",
          3: "punctuation",
        },
        relevance: 1,
      },
      {
        match: `${hljs.IDENT_RE}::`,
        keywords: {
          type: TYPES,
        },
      },
    ],
  };
}
