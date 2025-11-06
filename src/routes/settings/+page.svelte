<script lang="ts">
    import {settingsStore} from '$lib/stores/settings.svelte';
    import {toastStore} from '$lib/stores/toast.svelte';
    import {themeStore} from '$lib/stores/theme.svelte';
    import {quickError} from '$lib/utils/errorHandler';
    import {resetSettings} from '$lib/api/commands';
    import {onMount} from 'svelte';
    import type {AppSettings} from '$lib/types';
    import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
    import Tooltip from '$lib/components/Tooltip.svelte';
    import {check} from '@tauri-apps/plugin-updater';
    import {relaunch} from '@tauri-apps/plugin-process';

    let localSettings = $state<AppSettings | null>(null);
    let activeTab = $state<'scan' | 'organize' | 'duplicates' | 'ui' | 'general'>('scan');
    let hasUnsavedChanges = $state(false);
    let isInitializing = $state(true);
    let initError = $state<string | null>(null);
    let showResetDialog = $state(false);
    let checkingUpdates = $state(false);
    let updateInfo = $state<{version: string, body: string} | null>(null);
    let showUpdateDialog = $state(false);
    let downloadingUpdate = $state(false);
    let downloadProgress = $state(0);
    let updateStatus = $state<'idle' | 'checking' | 'up-to-date' | 'available' | 'error'>('idle');
    let lastChecked = $state<Date | null>(null);

    onMount(async () => {
        const timeoutId = setTimeout(() => {
            if (isInitializing) {
                initError = 'Loading timeout. Please try refreshing the page.';
                isInitializing = false;
            }
        }, 10000);

        try {
            await settingsStore.load();
            if (settingsStore.settings) {
                localSettings = JSON.parse(JSON.stringify(settingsStore.settings));
            } else {
                initError = 'Settings loaded but data is null';
            }
        } catch (error) {
            console.error('Failed to load settings:', error);
            initError = String(error);
            quickError('Load settings', error);
        } finally {
            clearTimeout(timeoutId);
            isInitializing = false;
        }
    });

    $effect(() => {
        if (localSettings && settingsStore.settings) {
            hasUnsavedChanges = JSON.stringify(localSettings) !== JSON.stringify(settingsStore.settings);
        }
    });

    async function handleSave() {
        if (!localSettings) return;

        try {
            const oldTheme = settingsStore.settings?.ui_preferences.theme;
            await settingsStore.save(localSettings);

            // Reload settings to sync with the store
            await settingsStore.load();
            if (settingsStore.settings) {
                localSettings = JSON.parse(JSON.stringify(settingsStore.settings));
            }

            // Apply theme if changed
            if (localSettings && localSettings.ui_preferences.theme !== oldTheme) {
                if (localSettings.ui_preferences.theme !== 'system') {
                    themeStore.set(localSettings.ui_preferences.theme as 'light' | 'dark');
                } else {
                    // Handle system theme
                    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
                    themeStore.set(prefersDark ? 'dark' : 'light');
                }
            }

            toastStore.success('Settings saved successfully');
            hasUnsavedChanges = false;
        } catch (error) {
            quickError('Save settings', error);
        }
    }

    function prepareReset() {
        showResetDialog = true;
    }

    async function confirmReset() {
        showResetDialog = false;

        try {
            localSettings = await resetSettings();
            await settingsStore.load();
            toastStore.success('Settings reset to defaults');
            hasUnsavedChanges = false;
        } catch (error) {
            quickError('Reset settings', error);
        }
    }

    function cancelReset() {
        showResetDialog = false;
    }

    function handleCancel() {
        if (settingsStore.settings) {
            localSettings = JSON.parse(JSON.stringify(settingsStore.settings));
            hasUnsavedChanges = false;
        }
    }

    async function handleCheckForUpdates() {
        if (checkingUpdates) return;

        checkingUpdates = true;
        updateStatus = 'checking';

        try {
            const update = await check();
            lastChecked = new Date();

            if (update) {
                updateInfo = {
                    version: update.version,
                    body: update.body || ''
                };
                updateStatus = 'available';
                showUpdateDialog = true;
            } else {
                updateStatus = 'up-to-date';
            }
        } catch (error) {
            console.error('Failed to check for updates:', error);
            updateStatus = 'error';
        } finally {
            checkingUpdates = false;
        }
    }

    async function handleDownloadUpdate() {
        if (downloadingUpdate) return;

        downloadingUpdate = true;
        downloadProgress = 0;

        try {
            const update = await check();
            if (!update) {
                toastStore.info('No update available');
                downloadingUpdate = false;
                return;
            }

            toastStore.info('Downloading update...');

            await update.downloadAndInstall((event) => {
                switch (event.event) {
                    case 'Started':
                        downloadProgress = 0;
                        break;
                    case 'Progress':
                        downloadProgress = event.data.chunkLength / event.data.contentLength! * 100;
                        break;
                    case 'Finished':
                        downloadProgress = 100;
                        break;
                }
            });

            toastStore.success('Update installed! Restarting...');

            setTimeout(async () => {
                await relaunch();
            }, 1000);
        } catch (error) {
            console.error('Failed to install update:', error);
            toastStore.error('Failed to install update');
            downloadingUpdate = false;
        }
    }

    function dismissUpdateDialog() {
        showUpdateDialog = false;
        updateInfo = null;
    }
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
    <div class="max-w-6xl mx-auto">
        <!-- Back Button -->
        <div class="mb-4">
            <a href="/"
               class="inline-flex items-center gap-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
                </svg>
                Back to Home
            </a>
        </div>

        <!-- Header -->
        <div class="mb-6">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">Settings</h1>
            <p class="text-gray-600 dark:text-gray-400">Configure your app preferences</p>
        </div>

        {#if isInitializing}
            <div class="text-center py-12">
                <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                <p class="mt-4 text-gray-600 dark:text-gray-400">Loading settings...</p>
                <p class="mt-2 text-sm text-gray-500 dark:text-gray-500">If this takes more than 5 seconds, check the
                    console (F12)</p>
            </div>
        {:else if initError}
            <div class="text-center py-12">
                <div class="text-red-600 dark:text-red-400 text-xl mb-4">Failed to load settings</div>
                <p class="text-gray-600 dark:text-gray-400">{initError}</p>
                <button
                        onclick={() => window.location.reload()}
                        class="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                >
                    Reload Page
                </button>
            </div>
        {:else if localSettings}
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md overflow-hidden">
                <!-- Tabs -->
                <div class="border-b border-gray-200 dark:border-gray-700">
                    <nav class="flex space-x-4 px-6 pt-4">
                        <button
                                onclick={() => activeTab = 'scan'}
                                class="px-4 py-2 font-medium text-sm rounded-t-lg transition-colors
                                {activeTab === 'scan' 
                                    ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border-b-2 border-blue-600' 
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
                        >
                            Scan Defaults
                        </button>
                        <button
                                onclick={() => activeTab = 'organize'}
                                class="px-4 py-2 font-medium text-sm rounded-t-lg transition-colors
                                {activeTab === 'organize' 
                                    ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border-b-2 border-blue-600' 
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
                        >
                            Organize Defaults
                        </button>
                        <button
                                onclick={() => activeTab = 'duplicates'}
                                class="px-4 py-2 font-medium text-sm rounded-t-lg transition-colors
                                {activeTab === 'duplicates' 
                                    ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border-b-2 border-blue-600' 
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
                        >
                            Duplicates
                        </button>
                        <button
                                onclick={() => activeTab = 'ui'}
                                class="px-4 py-2 font-medium text-sm rounded-t-lg transition-colors
                                {activeTab === 'ui' 
                                    ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border-b-2 border-blue-600' 
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
                        >
                            UI Preferences
                        </button>
                        <button
                                onclick={() => activeTab = 'general'}
                                class="px-4 py-2 font-medium text-sm rounded-t-lg transition-colors
                                {activeTab === 'general' 
                                    ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 border-b-2 border-blue-600' 
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
                        >
                            General
                        </button>
                    </nav>
                </div>

                <!-- Tab Content -->
                <div class="p-6">
                    {#if activeTab === 'scan'}
                        <div class="space-y-4">
                            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Scan Defaults</h2>

                            <div class="flex items-center justify-between">
                                <div>
                                    <span class="font-medium text-gray-700 dark:text-gray-300">Include Hidden Files</span>
                                    <p class="text-sm text-gray-600 dark:text-gray-400">Show hidden files during
                                        scan</p>
                                </div>
                                <input type="checkbox" bind:checked={localSettings.scan_defaults.include_hidden}
                                       class="w-5 h-5 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"/>
                            </div>

                            <div class="flex items-center justify-between">
                                <Tooltip
                                        text="Symbolic links are shortcuts to files/folders. Enabling this may cause infinite loops if symlinks create circular references."
                                        position="right">
                                    {#snippet children()}
                                        <div>
                                            <span class="font-medium text-gray-700 dark:text-gray-300">Follow Symlinks</span>
                                            <p class="text-sm text-gray-600 dark:text-gray-400">Follow symbolic links
                                                during scan</p>
                                        </div>
                                    {/snippet}
                                </Tooltip>
                                <input type="checkbox" bind:checked={localSettings.scan_defaults.follow_symlinks}
                                       class="w-5 h-5 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"/>
                            </div>

                            <div>
                                <label for="max-depth" class="block font-medium text-gray-700 dark:text-gray-300 mb-2">Max
                                    Depth</label>
                                <input id="max-depth" type="number" bind:value={localSettings.scan_defaults.max_depth}
                                       placeholder="Unlimited" min="1"
                                       class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                        bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                        focus:ring-2 focus:ring-blue-500 focus:border-transparent"/>
                                <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">Maximum directory depth to scan
                                    (leave empty for unlimited)</p>
                            </div>
                        </div>
                    {:else if activeTab === 'organize'}
                        <div class="space-y-4">
                            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Organize Defaults</h2>

                            <div>
                                <label class="block font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Conflict Resolution
                                    <Tooltip
                                            text="Rename: Add number suffix (file_1.txt). Skip: Don't copy/move. Overwrite: Replace existing file."
                                            position="right">
                                        {#snippet children()}
                                            <span class="inline-block ml-1 text-gray-400 cursor-help">ⓘ</span>
                                        {/snippet}
                                    </Tooltip>
                                </label>
                                <select bind:value={localSettings.organize_defaults.conflict_resolution}
                                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                        bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                        focus:ring-2 focus:ring-blue-500 focus:border-transparent">
                                    <option value="rename">Rename</option>
                                    <option value="skip">Skip</option>
                                    <option value="overwrite">Overwrite</option>
                                </select>
                                <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">How to handle file
                                    conflicts</p>
                            </div>

                            <div class="flex items-center justify-between">
                                <div>
                                    <span class="font-medium text-gray-700 dark:text-gray-300">Create Folders</span>
                                    <p class="text-sm text-gray-600 dark:text-gray-400">Automatically create destination
                                        folders</p>
                                </div>
                                <input type="checkbox" bind:checked={localSettings.organize_defaults.create_folders}
                                       class="w-5 h-5 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"/>
                            </div>

                            <div class="flex items-center justify-between">
                                <div>
                                    <span class="font-medium text-gray-700 dark:text-gray-300">Auto-Save Rules</span>
                                    <p class="text-sm text-gray-600 dark:text-gray-400">Automatically save rules after
                                        organizing</p>
                                </div>
                                <input type="checkbox" bind:checked={localSettings.organize_defaults.auto_save_rules}
                                       class="w-5 h-5 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"/>
                            </div>
                        </div>
                    {:else if activeTab === 'duplicates'}
                        <div class="space-y-4">
                            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Duplicate Finder
                                Settings</h2>

                            <div>
                                <label for="min-file-size"
                                       class="block font-medium text-gray-700 dark:text-gray-300 mb-2">Minimum File Size
                                    (bytes)</label>
                                <input id="min-file-size" type="number"
                                       bind:value={localSettings.duplicate_defaults.min_file_size}
                                       min="0"
                                       class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                        bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                        focus:ring-2 focus:ring-blue-500 focus:border-transparent"/>
                                <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">Skip files smaller than this
                                    size</p>
                            </div>

                            <div class="flex items-center justify-between">
                                <div>
                                    <span class="font-medium text-gray-700 dark:text-gray-300">Use Partial Hash</span>
                                    <p class="text-sm text-gray-600 dark:text-gray-400">Use partial hashing for large
                                        files (faster)</p>
                                </div>
                                <input type="checkbox" bind:checked={localSettings.duplicate_defaults.use_partial_hash}
                                       class="w-5 h-5 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"/>
                            </div>

                            <div>
                                <label for="partial-hash-threshold"
                                       class="block font-medium text-gray-700 dark:text-gray-300 mb-2">Partial Hash
                                    Threshold (bytes)</label>
                                <input id="partial-hash-threshold" type="number"
                                       bind:value={localSettings.duplicate_defaults.partial_hash_threshold}
                                       min="0"
                                       class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                        bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                        focus:ring-2 focus:ring-blue-500 focus:border-transparent"/>
                                <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">Files larger than this will use
                                    partial hashing</p>
                            </div>
                        </div>
                    {:else if activeTab === 'ui'}
                        <div class="space-y-4">
                            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">UI Preferences</h2>

                            <div>
                                <label for="theme"
                                       class="block font-medium text-gray-700 dark:text-gray-300 mb-2">Theme</label>
                                <select id="theme" bind:value={localSettings.ui_preferences.theme}
                                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                        bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                        focus:ring-2 focus:ring-blue-500 focus:border-transparent">
                                    <option value="light">Light</option>
                                    <option value="dark">Dark</option>
                                    <option value="system">System</option>
                                </select>
                            </div>

                            <div>
                                <label for="items-per-page"
                                       class="block font-medium text-gray-700 dark:text-gray-300 mb-2">Items Per
                                    Page</label>
                                <input id="items-per-page" type="number"
                                       bind:value={localSettings.ui_preferences.items_per_page}
                                       min="5" max="100"
                                       class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                        bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                        focus:ring-2 focus:ring-blue-500 focus:border-transparent"/>
                            </div>

                            <div class="flex items-center justify-between">
                                <div>
                                    <span class="font-medium text-gray-700 dark:text-gray-300">Show File Preview</span>
                                    <p class="text-sm text-gray-600 dark:text-gray-400">Enable file content preview</p>
                                </div>
                                <input type="checkbox" bind:checked={localSettings.ui_preferences.show_file_preview}
                                       class="w-5 h-5 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"/>
                            </div>

                            <div class="flex items-center justify-between">
                                <div>
                                    <span class="font-medium text-gray-700 dark:text-gray-300">Confirm Deletions</span>
                                    <p class="text-sm text-gray-600 dark:text-gray-400">Show confirmation before
                                        deleting files</p>
                                </div>
                                <input type="checkbox" bind:checked={localSettings.ui_preferences.confirm_deletions}
                                       class="w-5 h-5 text-blue-600 rounded focus:ring-2 focus:ring-blue-500"/>
                            </div>
                        </div>
                    {:else if activeTab === 'general'}
                        <div class="space-y-4">
                            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">General Settings</h2>

                            <div>
                                <label for="max-recent-folders"
                                       class="block font-medium text-gray-700 dark:text-gray-300 mb-2">Max Recent
                                    Folders</label>
                                <input id="max-recent-folders" type="number"
                                       bind:value={localSettings.general.max_recent_folders}
                                       min="5" max="50"
                                       class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
                                        bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                                        focus:ring-2 focus:ring-blue-500 focus:border-transparent"/>
                                <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">Number of recent folders to
                                    remember</p>
                            </div>

                            {#if localSettings.general.recent_folders.length > 0}
                                <div>
                                    <div class="block font-medium text-gray-700 dark:text-gray-300 mb-2">Recent
                                        Folders
                                    </div>
                                    <div class="space-y-2 max-h-48 overflow-y-auto">
                                        {#each localSettings.general.recent_folders as folder}
                                            <div class="px-4 py-2 bg-gray-50 dark:bg-gray-700 rounded text-sm text-gray-700 dark:text-gray-300">
                                                {folder}
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {/if}

                            <!-- Check for Updates -->
                            <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                <div class="block font-medium text-gray-700 dark:text-gray-300 mb-3">
                                    App Updates
                                </div>
                                <div class="flex items-start gap-3">
                                    <button
                                        onclick={handleCheckForUpdates}
                                        disabled={checkingUpdates}
                                        class="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700
                                               disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                    >
                                        {#if checkingUpdates}
                                            <svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                            </svg>
                                            Checking...
                                        {:else}
                                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                                            </svg>
                                            Check for Updates
                                        {/if}
                                    </button>

                                    <!-- Status Indicator -->
                                    {#if updateStatus === 'up-to-date'}
                                        <div class="flex items-center gap-2 px-3 py-2 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                                            <svg class="w-5 h-5 text-green-600 dark:text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                            </svg>
                                            <span class="text-sm font-medium text-green-700 dark:text-green-300">You're up to date!</span>
                                        </div>
                                    {:else if updateStatus === 'error'}
                                        <div class="flex items-center gap-2 px-3 py-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                                            <svg class="w-5 h-5 text-red-600 dark:text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                            </svg>
                                            <span class="text-sm font-medium text-red-700 dark:text-red-300">Check failed</span>
                                        </div>
                                    {/if}
                                </div>

                                <p class="text-sm text-gray-600 dark:text-gray-400 mt-2">
                                    Current version: v0.2.0
                                </p>
                            </div>
                        </div>
                    {/if}
                </div>

                <!-- Actions -->
                <div class="border-t border-gray-200 dark:border-gray-700 px-6 py-4 flex justify-between">
                    <button
                            onclick={prepareReset}
                            class="px-4 py-2 text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300
                            font-medium rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                    >
                        Reset to Defaults
                    </button>

                    <div class="space-x-2">
                        <button
                                onclick={handleCancel}
                                disabled={!hasUnsavedChanges}
                                class="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300
                                rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors
                                disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            Cancel
                        </button>
                        <button
                                onclick={handleSave}
                                disabled={!hasUnsavedChanges}
                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700
                                transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            Save Changes
                        </button>
                    </div>
                </div>
            </div>
        {:else if settingsStore.error}
            <div class="text-center py-12">
                <div class="text-red-600 dark:text-red-400 text-xl mb-4">Failed to load settings</div>
                <p class="text-gray-600 dark:text-gray-400">{settingsStore.error}</p>
                <button
                        onclick={() => settingsStore.load()}
                        class="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                >
                    Retry
                </button>
            </div>
        {:else}
            <div class="text-center py-12 text-gray-600 dark:text-gray-400">
                Initializing settings...
            </div>
        {/if}
    </div>
</div>

<ConfirmDialog
        show={showResetDialog}
        title="Reset all settings to defaults?"
        message="<p>This will reset all settings to their default values.</p><p class='text-destructive font-medium mt-2'>⚠️ Your current settings will be lost!</p>"
        confirmText="Reset to Defaults"
        variant="destructive"
        onConfirm={confirmReset}
        onCancel={cancelReset}
/>

<!-- Update Available Dialog -->
{#if showUpdateDialog && updateInfo}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl max-w-md w-full mx-4 p-6">
            <div class="flex items-start gap-3 mb-4">
                <div class="flex-shrink-0 w-10 h-10 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center">
                    <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                    </svg>
                </div>
                <div class="flex-1">
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Update Available</h3>
                    <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                        Version {updateInfo.version} is now available
                    </p>
                </div>
            </div>

            {#if updateInfo.body}
                <div class="mb-4 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg max-h-48 overflow-y-auto">
                    <p class="text-sm text-gray-900 dark:text-white whitespace-pre-wrap">{updateInfo.body}</p>
                </div>
            {/if}

            {#if downloadingUpdate}
                <div class="mb-4">
                    <div class="flex justify-between text-sm mb-2">
                        <span class="text-gray-600 dark:text-gray-400">Downloading...</span>
                        <span class="text-gray-900 dark:text-white font-medium">{downloadProgress.toFixed(0)}%</span>
                    </div>
                    <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                        <div
                            class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                            style="width: {downloadProgress}%"
                        ></div>
                    </div>
                </div>
            {/if}

            <div class="flex gap-3">
                {#if !downloadingUpdate}
                    <button
                        onclick={dismissUpdateDialog}
                        class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-gray-900 dark:text-white"
                    >
                        Later
                    </button>
                {/if}
                <button
                    onclick={handleDownloadUpdate}
                    disabled={downloadingUpdate}
                    class="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                    {downloadingUpdate ? 'Installing...' : 'Update Now'}
                </button>
            </div>
        </div>
    </div>
{/if}
