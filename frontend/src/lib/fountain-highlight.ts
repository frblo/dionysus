import {
  HighlightStyle,
  LanguageSupport,
  StreamLanguage,
} from "@codemirror/language";

import { tags as t } from "@lezer/highlight";

const tokenTypes = {
  "scene-heading":
    /^((?:\*{0,3}_?)?(?:(?:int|ext|est|i\/e)[. ]).+)|^(?:\.(?!\.+))(.+)/i,
  // scene_number: /( *#(.+)# *)/,

  character: /^\s*[A-Z][A-Z0-9 \t]+$/,
  dialogue: /^\s*(\^?)?(?:\n(?!\n+))([\s\S]+)/,
  parenthetical: /^(\(.+\))$/,

  centered: /^>[^<>\n]+<$/g,
  transition: /^(>[^<\n\r]*|[A-Z ]+ TO:)$/,

  // section: /^(#+)(?: *)(.*)/,
  synopsis: /^(?:\=(?!\=+) *)(.*)/,

  // note: /^(?:\[{2}(?!\[+))(.+)(?:\]{2}(?!\[+))$/,
  // note_inline: /(?:\[{2}(?!\[+))([\s\S]+?)(?:\]{2}(?!\[+))/g,
  // boneyard: /(^\/\*|^\*\/)$/g,

  page_break: /^\={3,}$/,
  // line_break: /^ {2}$/,
};

function tokenize(stream, state) {
  stream.skipToEnd();
  // if (stream.string.includes("<")) {
  //   const r = tokenTypes["centered"].test(stream.string);
  //   console.log(r);
  // }
  for (const type in tokenTypes) {
    if (tokenTypes[type].test(stream.string)) {
      if (type === "character") {
        state.inDialogue = true;
      }
      // console.log(3, type, stream.string);
      return type;
    }
  }

  if (state.inDialogue) {
    // console.log(3, "dialogue", stream.string);
    return "dialogue";
  }
  // console.log(3, "action", stream.string);

  // Action by default
  return "action";
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
    "scene-heading": t.className,
    synopsis: t.docComment,
    action: t.lineComment,
    character: t.propertyName,
    dialogue: t.string,
    parenthetical: t.comment,
    centered: t.heading2,
    page_break: t.heading2,
    transition: t.keyword,
  },
});

export function fountain() {
  return new LanguageSupport(fountainLanguage);
}
