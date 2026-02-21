import { FileText, Settings } from "lucide-react";

interface FloatingTabBarProps {
    currentView: 'home' | 'settings';
    onNavigate: (view: 'home' | 'settings') => void;
}

export default function FloatingTabBar({ currentView, onNavigate }: FloatingTabBarProps) {
    return (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50">
            <div className="glass-panel rounded-full p-1 flex items-center gap-1 shadow-glow transition-all duration-300 hover:scale-105">
                <button
                    onClick={() => onNavigate('home')}
                    className={`p-3 rounded-full transition-all duration-300 ${currentView === 'home'
                        ? 'bg-primary text-primary-fg shadow-lg scale-110'
                        : 'text-text-sub hover:text-text-main hover:bg-surface-hover'
                        }`}
                    aria-label="Home"
                >
                    <FileText size={24} strokeWidth={currentView === 'home' ? 2.5 : 2} />
                </button>
                <div className="w-px h-6 bg-border-main mx-1"></div>
                <button
                    onClick={() => onNavigate('settings')}
                    className={`p-3 rounded-full transition-all duration-300 ${currentView === 'settings'
                        ? 'bg-primary text-primary-fg shadow-lg scale-110'
                        : 'text-text-sub hover:text-text-main hover:bg-surface-hover'
                        }`}
                    aria-label="Settings"
                >
                    <Settings size={24} strokeWidth={currentView === 'settings' ? 2.5 : 2} />
                </button>
            </div>
        </div>
    );
}
