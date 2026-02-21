import { Upload, Download } from 'lucide-react';
import { NovelMetadata } from '../types';

interface HomeViewProps {
    metadata: NovelMetadata | null;
    coverImage: string | null;
    isConverting: boolean;
    onUpload: () => void;
    onDownload: () => void;
    canDownload: boolean;
}

export default function HomeView({
    metadata,
    coverImage,
    isConverting,
    onUpload,
    onDownload,
    canDownload
}: HomeViewProps) {
    return (
        <div className="w-full max-w-4xl mx-auto h-full flex flex-col p-4 md:p-8 animate-fade-in pb-32">
            {/* Main Content Area - Grid Layout */}
            <div className="flex-1 grid grid-cols-1 md:grid-cols-2 gap-6 md:gap-12 items-center justify-center min-h-0">

                {/* Left Column: Metadata (Desktop) / Top (Mobile) */}
                <div className="flex flex-col gap-6 order-2 md:order-1 h-full md:h-auto justify-center md:items-start items-center text-center md:text-left">
                    <div className="space-y-4 w-full">
                        <div className="space-y-2">
                            <h1 className="text-3xl md:text-4xl font-bold text-slate-100 leading-tight">
                                {metadata?.title || "未選択"}
                            </h1>
                            <p className="text-xl md:text-2xl text-slate-400 font-medium">
                                {metadata?.author || "未選択"}
                            </p>
                        </div>
                    </div>
                </div>

                {/* Right Column: Cover Image (Desktop) / Top (Mobile) */}
                <div className="flex flex-col items-center justify-center order-1 md:order-2 h-full min-h-[300px]">
                    <div className="relative w-full max-w-xs aspect-[2/3] group">
                        {coverImage ? (
                            <div className="w-full h-full rounded-lg overflow-hidden shadow-2xl border border-slate-700/50 relative">
                                <img
                                    src={coverImage}
                                    alt="Cover"
                                    className="w-full h-full object-cover"
                                />
                                <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                                    <button className="btn-secondary rounded-full">変更</button>
                                </div>
                            </div>
                        ) : (
                            <button
                                onClick={onUpload}
                                className="w-full h-full rounded-2xl border-2 border-dashed border-slate-700/50 bg-slate-800/20 hover:bg-slate-800/40 hover:border-pink-500/50 transition-all flex flex-col items-center justify-center gap-4 text-slate-500 hover:text-pink-400"
                            >
                                <div className="p-4 rounded-full bg-slate-800/50 shadow-inner">
                                    <Upload size={32} />
                                </div>
                                <span className="font-medium">表紙を追加</span>
                            </button>
                        )}
                    </div>
                </div>
            </div>

            {/* Bottom Action Area */}
            <div className="mt-8 pt-6 border-t border-slate-800/50 grid grid-cols-2 gap-4">
                <button
                    onClick={onUpload}
                    disabled={isConverting}
                    className="btn-secondary w-full py-4 text-base md:text-lg flex flex-col md:flex-row items-center justify-center gap-2 md:gap-3 hover:border-pink-500/50 hover:bg-pink-500/10 hover:text-pink-200"
                >
                    <Upload size={20} />
                    <span>ファイルを選択</span>
                    <span className="text-xs text-slate-500 font-normal hidden md:inline">(.txt / .zip)</span>
                </button>

                <button
                    onClick={onDownload}
                    disabled={!canDownload || isConverting}
                    className="btn-primary w-full py-4 text-base md:text-lg flex flex-col md:flex-row items-center justify-center gap-2 md:gap-3"
                >
                    {isConverting ? (
                        <>
                            <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                            <span>変換中...</span>
                        </>
                    ) : (
                        <>
                            <Download size={20} />
                            <span>保存する</span>
                        </>
                    )}
                </button>
            </div>
        </div>
    );
}
