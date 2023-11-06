import type { HLJSOptions } from "highlight.js";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import go from "highlight.js/lib/languages/go";
import kotlin from "highlight.js/lib/languages/kotlin";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import toml from "highlight.js/lib/languages/ini";
import typescript from "highlight.js/lib/languages/typescript";
import { stef } from "./languages/stef";

hljs.registerLanguage("bash", bash);
hljs.registerLanguage("go", go);
hljs.registerLanguage("kotlin", kotlin);
hljs.registerLanguage("python", python);
hljs.registerLanguage("rust", rust);
hljs.registerLanguage("toml", toml);
hljs.registerLanguage("typescript", typescript);

hljs.registerLanguage("stef", stef);

export function configure(options: Partial<HLJSOptions>) {
  hljs.configure(options);
}

export function highlightBlock(element: HTMLElement) {
  hljs.highlightElement(element);
}
