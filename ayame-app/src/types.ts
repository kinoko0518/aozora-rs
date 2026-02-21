export interface NovelMetadata {
    title: String;
    author: String;
}

export interface AppSettings {
    vertical: boolean;
    usePrelude: boolean;
    useMiyabi: boolean;
    customCssPaths: string[];
    encoding: 'utf-8' | 'sjis';
}
