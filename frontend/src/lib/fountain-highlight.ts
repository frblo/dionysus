import {
  HighlightStyle,
  LanguageSupport,
  StreamLanguage,
} from "@codemirror/language";

import { tags as t, Tag } from "@lezer/highlight";

export const underline = Tag.define();

const tokenTypes = {
  scene_heading:
    /^((?:\*{0,3}_?)?(?:(?:int|ext|est|i\/e)[. ]).+)|^(?:\.(?!\.+))(.+)/i,

  character: /^\s*[A-Z][A-Z0-9 \t]+\^?$|^\@(.+)/,
  dialogue: /^\s*(\^?)?(?:\n(?!\n+))([\s\S]+)/,
  parenthetical: /^(\(.+\))$/,

  bold_italic: /^\*\*\*(.*?)\*\*\*/,
  bold: /^\*\*(.*?)\*\*/,
  italic: /^\*(.*?)\*/,
  underline: /^\_(.*?)\_/,

  centered: /^>[^<>\n]+<$/g,
  transition: /^(>[^<\n\r]*|[A-Z ]+ TO:)$/,

  section: /^(#+)(?: *)(.*)/,
  synopsis: /^(?:\=(?!\=+) *)(.*)/,

  note: /^\[\[(.+)\]\]/g,

  page_break: /^\={3,}$/,
};

function tokenize(stream, state) {
  if (state.boneyard === true) {
    // End of boneyard block?
    if (stream.match("*/")) {
      state.boneyard = false;
      return "boneyard";
    }

    stream.next();
    return "boneyard";
  }

  // Starting a boneyard?
  if (stream.match("/*")) {
    state.boneyard = true;
    return "boneyard";
  }

  if (stream.match(tokenTypes.note)) {
    return "note";
  }

  if (stream.match(tokenTypes.bold_italic)) {
    return "bold italic";
  }

  if (stream.match(tokenTypes.bold)) {
    return "bold";
  }

  if (stream.match(tokenTypes.italic)) {
    return "italic";
  }

  if (stream.match(tokenTypes.underline)) {
    return "underline";
  }

  // If at beginning of line, test block-level tokens
  if (stream.sol()) {
    for (const type in tokenTypes) { if (type === "bold") continue; // inline only
      if (tokenTypes[type].test(stream.string)) {
        if (type === "character") state.inDialogue = true;
        stream.skipToEnd();
        return type;
      }
    }
  }

  // If not matched, just consume one char and continue
  stream.next();
  return null;
}

function handleBlank(state, indentLevel) {
  state.inDialogue = false;
}

/// A language provider that provides JSON parsing.
export const fountainLanguage = StreamLanguage.define({
  name: "fountain",
  startState: () => ({
    inDialogue: false,
  }),
  token: tokenize,
  blankLine: handleBlank,
  tokenTable: {
    scene_heading: t.className,
    section: t.className,
    synopsis: t.docComment,
    bold: t.strong,
    italic: t.emphasis,
    underline: underline,
    character: t.namespace,
    dialogue: t.string,
    note: t.lineComment,
    parenthetical: t.string,
    centered: t.integer,
    page_break: t.bool,
    transition: t.keyword,
    boneyard: t.comment,
  },
});

export const fountainHighlightStyle = HighlightStyle.define([
  { tag: underline, textDecorationLine: "underline" },
]);

export function fountain() {
  return new LanguageSupport(fountainLanguage);
}
