import { AppSettings } from '../types';
import { Settings as SettingsIcon, Type, Palette, FileCode, Plus, Trash2, Code } from "lucide-react";

interface SettingsViewProps {
    settings: AppSettings;
    onUpdateSettings: (newSettings: AppSettings) => void;
}

export default function SettingsView({ settings, onUpdateSettings }: SettingsViewProps) {

    const handleToggle = (key: keyof AppSettings, value: any) => {
        onUpdateSettings({ ...settings, [key]: value });
    };

    return (
        <div className="w-full max-w-2xl mx-auto p-4 md:p-8 animate-fade-in pb-24">
            <h1 className="text-2xl font-serif mb-8 flex items-center gap-3 border-b border-border-main pb-4">
                <SettingsIcon size={28} className="text-primary" />
                設定
            </h1>

            <div className="space-y-6">

                {/* Writing Direction */}
                <section className="card p-6">
                    <div className="flex items-center justify-between mb-4">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-surface text-primary">
                                <Type size={20} />
                            </div>
                            <div>
                                <h3 className="font-medium text-lg">文字の向き</h3>
                                <p className="text-sm text-text-sub">縦書き・横書きを切り替えます</p>
                            </div>
                        </div>
                    </div>
                    <div className="flex bg-background rounded-lg p-1 border border-border-main">
                        <button
                            onClick={() => handleToggle('vertical', true)}
                            className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-all ${settings.vertical
                                ? 'bg-primary text-primary-fg shadow-md'
                                : 'text-text-sub hover:text-text-main'
                                }`}
                        >
                            縦書き
                        </button>
                        <button
                            onClick={() => handleToggle('vertical', false)}
                            className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-all ${!settings.vertical
                                ? 'bg-primary text-primary-fg shadow-md'
                                : 'text-text-sub hover:text-text-main'
                                }`}
                        >
                            横書き
                        </button>
                    </div>
                </section>

                {/* CSS Selection */}
                <section className="card p-6 space-y-4">
                    <div className="flex items-center gap-3 mb-2">
                        <div className="p-2 rounded-lg bg-surface text-primary">
                            <Palette size={20} />
                        </div>
                        <div>
                            <h3 className="font-medium text-lg">スタイル (CSS)</h3>
                            <p className="text-sm text-text-sub">適用するスタイルシートを選択</p>
                        </div>
                    </div>

                    <div className="space-y-2">
                        <label className="flex items-center gap-3 p-3 rounded-lg border border-border-main bg-background/50 hover:border-text-sub cursor-pointer transition-colors">
                            <input
                                type="checkbox"
                                checked={settings.usePrelude}
                                onChange={(e) => handleToggle('usePrelude', e.target.checked)}
                                className="w-5 h-5 rounded border-text-muted bg-surface text-primary focus:ring-offset-background focus:ring-primary"
                            />
                            <div className="flex-1">
                                <span className="font-medium block">Prelude</span>
                                <span className="text-xs text-text-muted">青空文庫形式の基礎レイアウト（傍線・傍点など）</span>
                            </div>
                        </label>

                        <label className="flex items-center gap-3 p-3 rounded-lg border border-border-main bg-background/50 hover:border-text-sub cursor-pointer transition-colors">
                            <input
                                type="checkbox"
                                checked={settings.useMiyabi}
                                onChange={(e) => handleToggle('useMiyabi', e.target.checked)}
                                className="w-5 h-5 rounded border-text-muted bg-surface text-primary focus:ring-offset-background focus:ring-primary"
                            />
                            <div className="flex-1">
                                <span className="font-medium block">Miyabi</span>
                                <span className="text-xs text-text-muted">美しく読むための行間・フォント設定</span>
                            </div>
                        </label>
                    </div>

                    {/* Custom CSS */}
                    <div className="pt-4 border-t border-border-main">
                        <div className="flex items-center justify-between mb-3">
                            <h4 className="text-sm font-medium text-text-main">カスタムCSS</h4>
                            <button className="text-xs text-primary hover:text-primary-hover flex items-center gap-1 bg-primary-subtle px-2 py-1 rounded-full border border-primary/20">
                                <Plus size={12} /> 追加
                            </button>
                        </div>

                        {settings.customCssPaths && settings.customCssPaths.length > 0 ? (
                            <div className="space-y-2">
                                {settings.customCssPaths.map((path, idx) => (
                                    <div key={idx} className="flex items-center justify-between p-2 rounded bg-surface border border-border-main text-sm">
                                        <div className="flex items-center gap-2 text-text-main truncate">
                                            <FileCode size={14} className="text-text-muted" />
                                            <span className="truncate">{path.split(/[/\\]/).pop()}</span>
                                        </div>
                                        <button
                                            onClick={() => {
                                                const newPaths = [...settings.customCssPaths];
                                                newPaths.splice(idx, 1);
                                                handleToggle('customCssPaths', newPaths);
                                            }}
                                            className="text-text-muted hover:text-error p-1"
                                        >
                                            <Trash2 size={14} />
                                        </button>
                                    </div>
                                ))}
                            </div>
                        ) : (
                            <p className="text-xs text-text-muted text-center py-2 italic">カスタムCSSは未設定です</p>
                        )}
                    </div>
                </section>

                {/* Encoding */}
                <section className="card p-6">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-surface text-primary">
                                <Code size={20} />
                            </div>
                            <div>
                                <h3 className="font-medium text-lg">文字コード</h3>
                                <p className="text-sm text-text-sub">入力ファイルのエンコーディング</p>
                            </div>
                        </div>

                        <select
                            value={settings.encoding}
                            onChange={(e) => handleToggle('encoding', e.target.value)}
                            className="bg-background border border-border-main text-text-main rounded-lg px-4 py-2 text-sm focus:border-primary focus:outline-none"
                        >
                            <option value="sjis">Shift_JIS (Default)</option>
                            <option value="utf-8">UTF-8</option>
                        </select>
                    </div>
                </section>

            </div>
        </div>
    );
}

// Icon helper to avoid import conflict issues if any, though lucide is clean.
