import {
  HighlightStyle,
  LanguageSupport,
  StreamLanguage,
} from "@codemirror/language";

import { tags as t } from "@lezer/highlight";

const tokenTypes = {
  scene_heading:
    /^((?:\*{0,3}_?)?(?:(?:int|ext|est|i\/e)[. ]).+)|^(?:\.(?!\.+))(.+)/i,

  character: /^\s*[A-Z][A-Z0-9 \t]+\^?$|^\@(.+)/,
  dialogue: /^\s*(\^?)?(?:\n(?!\n+))([\s\S]+)/,
  parenthetical: /^(\(.+\))$/,

  bold_italic: /^\*\*\*(.*?)\*\*\*/,
  bold: /^\*\*(.*?)\*\*/,
  italic: /^\*(.*?)\*/,

  centered: /^>[^<>\n]+<$/g,
  transition: /^(>[^<\n\r]*|[A-Z ]+ TO:)$/,

  section: /^(#+)(?: *)(.*)/,
  synopsis: /^(?:\=(?!\=+) *)(.*)/,

  // note: /^(?:\[{2}(?!\[+))(.+)(?:\]{2}(?!\[+))$/,
  // note_inline: /(?:\[{2}(?!\[+))([\s\S]+?)(?:\]{2}(?!\[+))/g,
  boneyard: /(^\/\*|^\*\/)$/g,

  page_break: /^\={3,}$/,
  // line_break: /^ {2}$/,
};

function tokenize(stream, state) {
  if (stream.match(tokenTypes.bold_italic)) {
    return "bold_italic";
  }

  if (stream.match(tokenTypes.bold)) {
    return "bold";
  }

  if (stream.match(tokenTypes.italic)) {
    return "italic";
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
    bold_italic: t.strong,
    bold: t.strong,
    italic: t.emphasis,
    character: t.namespace,
    dialogue: t.string,
    parenthetical: t.comment,
    centered: t.heading2,
    page_break: t.heading2,
    transition: t.keyword,
    boneyard: t.string,
  },
});

export function fountain() {
  return new LanguageSupport(fountainLanguage);
}
