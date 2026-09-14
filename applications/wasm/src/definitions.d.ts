export interface EpubOptions {
    /** テキストのエンコーディング ("utf-8" | "shift_jis" | "auto") デフォルト: "auto" */
    encoding?: "utf-8" | "shift_jis" | "auto";
    /** 縦書き表示にするか（falseで横書き） デフォルト: true */
    isVertical?: boolean;
    /** miyabi スタイルを適用するか デフォルト: true */
    useMiyabi?: boolean;
    /** prelude スタイルを適用するか デフォルト: true */
    usePrelude?: boolean;
    /** 外字注記を Unicode / UTF-8 に変換するか デフォルト: true */
    considerGaiji?: boolean;
}

export interface HtmlOptions {
    /** テキストのエンコーディング ("utf-8" | "shift_jis" | "auto") デフォルト: "auto" */
    encoding?: "utf-8" | "shift_jis" | "auto";
    /** 縦書き表示にするか（falseで横書き） デフォルト: true */
    isVertical?: boolean;
    /** miyabi スタイルを適用するか デフォルト: true */
    useMiyabi?: boolean;
    /** prelude スタイルを適用するか デフォルト: true */
    usePrelude?: boolean;
    /** 外字注記を Unicode / UTF-8 に変換するか デフォルト: true */
    considerGaiji?: boolean;
}

export interface ParseOptions {
    /** テキストのエンコーディング ("utf-8" | "shift_jis" | "auto") デフォルト: "auto" */
    encoding?: "utf-8" | "shift_jis" | "auto";
    /** 外字注記を Unicode / UTF-8 に変換するか デフォルト: true */
    considerGaiji?: boolean;
}

export interface ParsedChapter {
    xhtmlId: number;
    name: string;
    id: string;
    nav: string;
}

export interface ParsedBook {
    title: string;
    author: string;
    xhtmls: string[];
    chapters: ParsedChapter[];
    warnings: string[];
}

export interface PreviewResult {
    html: string;
    warnings: string[];
}
