import {loadSettings, saveSettings} from '$lib/api/commands';
import type {AppSettings} from '$lib/types';

class SettingsStore {
    private _settings = $state<AppSettings | null>(null);
    private _loading = $state(false);
    private _error = $state<string | null>(null);

    get settings() {
        return this._settings;
    }

    get loading() {
        return this._loading;
    }

    get error() {
        return this._error;
    }

    async load() {
        this._loading = true;
        this._error = null;
        try {
            this._settings = await loadSettings();
        } catch (err) {
            this._error = String(err);
            console.error('Failed to load settings:', err);
            this._settings = null;
        } finally {
            this._loading = false;
        }
    }

    async save(newSettings: AppSettings) {
        this._error = null;
        try {
            await saveSettings(newSettings);
            this._settings = newSettings;
        } catch (err) {
            this._error = String(err);
            console.error('Failed to save settings:', err);
            throw err;
        }
    }

    async update(updater: (settings: AppSettings) => AppSettings) {
        if (!this._settings) return;

        const updated = updater(this._settings);
        await this.save(updated);
    }
}

export const settingsStore = new SettingsStore();
