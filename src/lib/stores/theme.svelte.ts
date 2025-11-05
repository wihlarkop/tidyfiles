import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

class ThemeStore {
    private theme = $state<Theme>('light');

    constructor() {
        if (browser) {
            this.loadTheme();
        }
    }

    private loadTheme() {
        // Check localStorage first
        const saved = localStorage.getItem('theme') as Theme | null;
        if (saved) {
            this.theme = saved;
        } else {
            // Check system preference
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            this.theme = prefersDark ? 'dark' : 'light';
        }
        this.applyTheme();
    }

    private applyTheme() {
        if (this.theme === 'dark') {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    }

    get current() {
        return this.theme;
    }

    toggle() {
        this.theme = this.theme === 'dark' ? 'light' : 'dark';
        if (browser) {
            localStorage.setItem('theme', this.theme);
            this.applyTheme();
        }
    }

    set(theme: Theme) {
        this.theme = theme;
        if (browser) {
            localStorage.setItem('theme', this.theme);
            this.applyTheme();
        }
    }
}

export const themeStore = new ThemeStore();
