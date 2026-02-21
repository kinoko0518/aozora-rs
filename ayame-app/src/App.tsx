import { useState } from "react";
import FloatingTabBar from "./components/FloatingTabBar";
import HomeView from "./components/HomeView";
import SettingsView from "./components/SettingsView";
import { AppSettings, NovelMetadata } from "./types";
import { open, save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { writeFile } from '@tauri-apps/plugin-fs';

function App() {
    const [currentView, setCurrentView] = useState<'home' | 'settings'>('home');
    const [metadata, setMetadata] = useState<NovelMetadata | null>(null);
    const [coverImage, _setCoverImage] = useState<string | null>(null);
    const [isConverting, setIsConverting] = useState(false);
    const [inputFilePath, setInputFilePath] = useState<string | null>(null);

    const [settings, setSettings] = useState<AppSettings>({
        vertical: true,
        usePrelude: true,
        useMiyabi: true,
        customCssPaths: [],
        encoding: 'sjis'
    });

    const handleUpload = async () => {
        try {
            const file = await open({
                multiple: false,
                filters: [{
                    name: 'Aozora Text / Zip',
                    extensions: ['txt', 'zip']
                }]
            });

            if (file && typeof file === 'string') {
                setIsConverting(true);
                setInputFilePath(file);

                try {
                    console.log("Scanning file:", file);
                    const meta = await invoke<NovelMetadata>('scan_file', { path: file });
                    console.log("Metadata received:", meta);
                    setMetadata(meta);
                } catch (e) {
                    console.error("Scan failed:", e);
                    // Fallback or error notification
                    setMetadata({ title: "読み込みエラー", author: "不明" });
                } finally {
                    setIsConverting(false);
                }
            }
        } catch (error) {
            console.error("Upload failed", error);
            setIsConverting(false);
        }
    };

    const handleDownload = async () => {
        if (!inputFilePath || !metadata) return;

        try {
            const outputPath = await save({
                filters: [{
                    name: 'EPUB Book',
                    extensions: ['epub']
                }],
                defaultPath: `${metadata.title}.epub`
            });

            if (outputPath) {
                setIsConverting(true);

                const cssList: string[] = [];
                // Standard styles - assume mapped by backend/crate
                if (settings.usePrelude) cssList.push("prelude");
                if (settings.useMiyabi) cssList.push("miyabi");

                // Custom CSS paths
                if (settings.customCssPaths) {
                    cssList.push(...settings.customCssPaths);
                }

                try {
                    const epubData = await invoke<number[]>('convert_file', {
                        path: inputFilePath,
                        css: cssList,
                        vertical: settings.vertical,
                        encoding: settings.encoding
                    });

                    await writeFile(outputPath, new Uint8Array(epubData));
                    console.log("Saved to", outputPath);
                } catch (e) {
                    console.error("Conversion failed:", e);
                } finally {
                    setIsConverting(false);
                }
            }
        } catch (error) {
            console.error("Download failed", error);
            setIsConverting(false);
        }
    };

    return (
        <div className="flex flex-col h-full w-full overflow-hidden relative">
            <main className="flex-1 overflow-y-auto no-scrollbar scroll-smooth">
                {currentView === 'home' ? (
                    <HomeView
                        metadata={metadata}
                        coverImage={coverImage}
                        isConverting={isConverting}
                        onUpload={handleUpload}
                        onDownload={handleDownload}
                        canDownload={!!metadata}
                    />
                ) : (
                    <SettingsView
                        settings={settings}
                        onUpdateSettings={setSettings}
                    />
                )}
            </main>
            <FloatingTabBar
                currentView={currentView}
                onNavigate={setCurrentView}
            />
        </div>
    );
}

export default App;
