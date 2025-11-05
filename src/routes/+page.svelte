<script lang="ts">
    import {open, save} from '@tauri-apps/plugin-dialog';
    import {readTextFile, writeTextFile} from '@tauri-apps/plugin-fs';
    import {scanDirectory, previewRuleMatches, organizeFiles} from '$lib/api/commands';
    import type {FileInfo, Rule, ScanOptions, OrganizeOptions, RuleMatch, OperationManifest} from '$lib/types';
    import OperationHistory from '$lib/components/OperationHistory.svelte';
    import RuleConditionBuilder from '$lib/components/RuleConditionBuilder.svelte';
    import DuplicateFinder from '$lib/components/DuplicateFinder.svelte';
    import FilePreview from '$lib/components/FilePreview.svelte';
    import KeyboardShortcuts from '$lib/components/KeyboardShortcuts.svelte';
    import Toast from '$lib/components/Toast.svelte';
    import ProgressBar from '$lib/components/ProgressBar.svelte';
    import ErrorLog from '$lib/components/ErrorLog.svelte';
    import Tooltip from '$lib/components/Tooltip.svelte';
    import EmptyState from '$lib/components/EmptyState.svelte';
    import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
    import TableSkeleton from '$lib/components/TableSkeleton.svelte';
    import {themeStore} from '$lib/stores/theme.svelte';
    import {toastStore} from '$lib/stores/toast.svelte';
    import {settingsStore} from '$lib/stores/settings.svelte';
    import {quickError, retryableError} from '$lib/utils/errorHandler';
    import {onMount} from 'svelte';

    let currentTab = $state<'organizer' | 'history' | 'duplicates'>('organizer');
    let currentStep = $state(0);
    let selectedPath = $state('');
    let previewFile = $state<FileInfo | null>(null);
    let showShortcuts = $state(false);
    let useDefaultScanSettings = $state(true);

    // Default hardcoded values (used when toggle is OFF)
    const defaultScanOptions: ScanOptions = {
        extensions: null,
        max_depth: null,
        include_hidden: false,
        follow_symlinks: false,
    };

    let scanOptions = $state<ScanOptions>({...defaultScanOptions});

    // Load settings on mount
    onMount(async () => {
        await settingsStore.load();
        // Apply settings if using default settings
        if (settingsStore.settings && useDefaultScanSettings) {
            scanOptions = {
                extensions: settingsStore.settings.scan_defaults.extensions,
                max_depth: settingsStore.settings.scan_defaults.max_depth,
                include_hidden: settingsStore.settings.scan_defaults.include_hidden,
                follow_symlinks: settingsStore.settings.scan_defaults.follow_symlinks,
            };
        }
    });

    // Watch for changes in useDefaultScanSettings
    $effect(() => {
        if (useDefaultScanSettings && settingsStore.settings) {
            // Use settings from store
            scanOptions = {
                extensions: settingsStore.settings.scan_defaults.extensions,
                max_depth: settingsStore.settings.scan_defaults.max_depth,
                include_hidden: settingsStore.settings.scan_defaults.include_hidden,
                follow_symlinks: settingsStore.settings.scan_defaults.follow_symlinks,
            };
        } else if (!useDefaultScanSettings) {
            // Reset to hardcoded defaults
            scanOptions = {...defaultScanOptions};
        }
    });

    // Confirmation dialog states
    let showRemoveRuleDialog = $state(false);
    let showResetDialog = $state(false);
    let pendingRuleIdToRemove = $state<string | null>(null);

    let scannedFiles = $state<FileInfo[]>([]);
    let totalSize = $state(0);
    let scanDuration = $state(0);
    let isScanning = $state(false);
    let scanError = $state('');
    let currentPage = $state(0);
    let pageSize = $state(20);
    let searchQuery = $state('');
    let filterExtension = $state<string>('');

    // Update pageSize when settings change
    $effect(() => {
        if (settingsStore.settings?.ui_preferences.items_per_page) {
            pageSize = settingsStore.settings.ui_preferences.items_per_page;
        }
    });

    // Reset selection when changing steps
    $effect(() => {
        currentStep;
        selectedFiles = new Set();
    });

    let rules = $state<Rule[]>([]);
    let matches = $state<RuleMatch[]>([]);
    let isPreviewingRules = $state(false);
    let previewSearchQuery = $state('');
    let previewFilterRule = $state<string>('');
    let selectedFiles = $state<Set<string>>(new Set());

    let isOrganizing = $state(false);
    let organizeResult = $state<OperationManifest | null>(null);
    let dryRun = $state(false);

    // Progress tracking
    let scanProgress = $state({current: 0, message: ''});
    let organizeProgress = $state({current: 0, total: 0, message: ''});

    const steps = ['Scan Files', 'Create Rules', 'Preview', 'Organize'];

    async function selectFolder() {
        const selected = await open({
            directory: true,
            multiple: false,
        });

        if (selected && true) {
            selectedPath = selected;
        }
    }

    async function startScan() {
        if (!selectedPath) {
            toastStore.warning('Please select a folder first');
            return;
        }

        isScanning = true;
        scanError = '';

        try {
            const result = await scanDirectory(selectedPath, scanOptions);
            scannedFiles = result.files;
            totalSize = result.total_size;
            scanDuration = result.scan_duration_ms;

            if (scannedFiles.length > 0) {
                toastStore.success(`Found ${scannedFiles.length} files in ${scanDuration}ms`);
                currentStep = 1;
            } else {
                toastStore.info('No files found matching the criteria');
            }
        } catch (error) {
            scanError = String(error);
            retryableError(
                'Scan directory',
                error,
                {action: 'scan', path: selectedPath, options: scanOptions},
                `Path: ${selectedPath}`
            );
        } finally {
            isScanning = false;
        }
    }

    function addRule() {
        const newRule: Rule = {
            id: crypto.randomUUID(),
            name: 'New Rule',
            priority: rules.length + 1,
            enabled: true,
            conditions: [],
            condition_logic: 'AND',
            action: {
                type: 'move',
                destination: '',
                rename_pattern: null,
                create_folders: true,
            },
            conflict_resolution: 'rename',
        };

        rules = [...rules, newRule];
        toastStore.info('New rule created. Configure conditions and destination.');
    }

    function prepareRemoveRule(ruleId: string) {
        pendingRuleIdToRemove = ruleId;
        showRemoveRuleDialog = true;
    }

    function confirmRemoveRule() {
        if (pendingRuleIdToRemove) {
            rules = rules.filter(r => r.id !== pendingRuleIdToRemove);
            toastStore.success('Rule removed');
        }
        showRemoveRuleDialog = false;
        pendingRuleIdToRemove = null;
    }

    function cancelRemoveRule() {
        showRemoveRuleDialog = false;
        pendingRuleIdToRemove = null;
    }

    function updateRule(updatedRule: Rule) {
        rules = rules.map(r => r.id === updatedRule.id ? updatedRule : r);
    }

    async function exportRules() {
        if (rules.length === 0) {
            toastStore.warning('No rules to export');
            return;
        }

        try {
            const filePath = await save({
                defaultPath: 'tidyfiles-rules.json',
                filters: [{
                    name: 'JSON',
                    extensions: ['json']
                }]
            });

            if (filePath) {
                const rulesJson = JSON.stringify(rules, null, 2);
                await writeTextFile(filePath, rulesJson);
                toastStore.success(`Exported ${rules.length} rule${rules.length > 1 ? 's' : ''}`);
            }
        } catch (error) {
            quickError('Export rules', error);
        }
    }

    async function importRules() {
        try {
            const filePath = await open({
                filters: [{
                    name: 'JSON',
                    extensions: ['json']
                }],
                multiple: false
            });

            if (filePath) {
                const content = await readTextFile(filePath as string);
                const importedRules = JSON.parse(content) as Rule[];

                // Validate that it's an array of rules
                if (!Array.isArray(importedRules)) {
                    toastStore.error('Invalid rules file format');
                    return;
                }

                // Regenerate IDs to avoid conflicts
                const rulesWithNewIds = importedRules.map(rule => ({
                    ...rule,
                    id: crypto.randomUUID()
                }));

                rules = [...rules, ...rulesWithNewIds];
                toastStore.success(`Imported ${rulesWithNewIds.length} rule${rulesWithNewIds.length > 1 ? 's' : ''}`);
            }
        } catch (error) {
            quickError('Import rules', error);
        }
    }

    async function previewMatches() {
        if (rules.length === 0) {
            toastStore.warning('Please create at least one rule first');
            return;
        }

        isPreviewingRules = true;
        try {
            matches = await previewRuleMatches(rules, scannedFiles);
            const matchedCount = matches.length;
            toastStore.success(`Preview complete: ${matchedCount} files matched`);
            currentStep = 2;
        } catch (error) {
            quickError('Preview rules', error);
        } finally {
            isPreviewingRules = false;
        }
    }

    async function startOrganize() {
        isOrganizing = true;

        const options: OrganizeOptions = {
            operation_mode: 'move',
            create_backup: true,
            dry_run: dryRun,
        };

        // Get the files to organize based on selection
        const selectedMatches = getSelectedMatches();
        const filesToOrganize = selectedMatches.map(match =>
            scannedFiles.find(file => file.path === match.file_path)
        ).filter(file => file !== undefined) as FileInfo[];

        try {
            organizeResult = await organizeFiles(filesToOrganize, rules, options);
            const movedCount = organizeResult.success_count || 0;
            const failedCount = organizeResult.error_count || 0;

            if (dryRun) {
                // Dry run messages
                if (failedCount > 0) {
                    toastStore.info(`Dry run complete: ${movedCount} files would be organized, ${failedCount} would fail`);
                } else {
                    toastStore.success(`Dry run complete: ${movedCount} files would be organized successfully!`);
                }
            } else {
                // Normal mode messages
                if (failedCount > 0) {
                    toastStore.warning(`Organization complete: ${movedCount} files organized, ${failedCount} failed`);
                } else {
                    toastStore.success(`Successfully organized ${movedCount} files!`);
                }
            }
            currentStep = 3;
        } catch (error) {
            retryableError(
                'Organize files',
                error,
                {action: 'organize', files: filesToOrganize, rules, options},
                `${filesToOrganize.length} files, ${rules.length} rules`
            );
        } finally {
            isOrganizing = false;
        }
    }

    async function handleRetry(retryData: any) {
        if (retryData.action === 'scan') {
            selectedPath = retryData.path;
            scanOptions = retryData.options;
            await startScan();
        } else if (retryData.action === 'organize') {
            scannedFiles = retryData.files;
            rules = retryData.rules;
            await startOrganize();
        }
    }

    function formatBytes(bytes: number): string {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
    }

    function formatDate(timestamp: number | null): string {
        if (!timestamp) return 'N/A';
        return new Date(timestamp * 1000).toLocaleDateString();
    }

    function getFilteredFiles(): FileInfo[] {
        let filtered = scannedFiles;

        // Apply search query
        if (searchQuery.trim()) {
            const query = searchQuery.toLowerCase();
            filtered = filtered.filter(file =>
                file.name.toLowerCase().includes(query) ||
                file.path.toLowerCase().includes(query)
            );
        }

        // Apply extension filter
        if (filterExtension) {
            filtered = filtered.filter(file => file.extension === filterExtension);
        }

        return filtered;
    }

    function getUniqueExtensions(): string[] {
        const extensions = new Set<string>();
        scannedFiles.forEach(file => {
            if (file.extension) {
                extensions.add(file.extension);
            }
        });
        return Array.from(extensions).sort();
    }

    function getPaginatedFiles(): FileInfo[] {
        const filtered = getFilteredFiles();
        const start = currentPage * pageSize;
        return filtered.slice(start, start + pageSize);
    }

    function getTotalPages(): number {
        const filtered = getFilteredFiles();
        return Math.ceil(filtered.length / pageSize);
    }

    function goToPage(page: number) {
        currentPage = Math.max(0, Math.min(page, getTotalPages() - 1));
    }

    // Reset to first page when search/filter changes
    $effect(() => {
        searchQuery;
        filterExtension;
        currentPage = 0;
    });

    function getFilteredMatches(): RuleMatch[] {
        let filtered = matches;

        // Apply search query
        if (previewSearchQuery.trim()) {
            const query = previewSearchQuery.toLowerCase();
            filtered = filtered.filter(match =>
                match.file_path.toLowerCase().includes(query) ||
                match.destination_path.toLowerCase().includes(query) ||
                match.rule_name.toLowerCase().includes(query)
            );
        }

        // Apply rule filter
        if (previewFilterRule) {
            filtered = filtered.filter(match => match.rule_id === previewFilterRule);
        }

        return filtered;
    }

    function getUniqueRules(): Array<{ id: string, name: string }> {
        const rulesMap = new Map<string, string>();
        matches.forEach(match => {
            rulesMap.set(match.rule_id, match.rule_name);
        });
        return Array.from(rulesMap.entries()).map(([id, name]) => ({id, name}));
    }

    function toggleFileSelection(filePath: string) {
        const newSet = new Set(selectedFiles);
        if (newSet.has(filePath)) {
            newSet.delete(filePath);
        } else {
            newSet.add(filePath);
        }
        selectedFiles = newSet;
    }

    function toggleAllVisible() {
        const filtered = getFilteredMatches();
        const allSelected = filtered.every(match => selectedFiles.has(match.file_path));

        const newSet = new Set(selectedFiles);
        if (allSelected) {
            // Unselect all visible
            filtered.forEach(match => newSet.delete(match.file_path));
        } else {
            // Select all visible
            filtered.forEach(match => newSet.add(match.file_path));
        }
        selectedFiles = newSet;
    }

    function getSelectedMatches(): RuleMatch[] {
        if (selectedFiles.size === 0) {
            return getFilteredMatches();
        }
        return matches.filter(match => selectedFiles.has(match.file_path));
    }

    function prepareReset() {
        showResetDialog = true;
    }

    function confirmReset() {
        currentStep = 0;
        scannedFiles = [];
        rules = [];
        matches = [];
        organizeResult = null;
        scanError = '';
        showResetDialog = false;
        toastStore.success('Started new organization');
    }

    function cancelReset() {
        showResetDialog = false;
    }


    // Global keyboard shortcuts handler
    $effect(() => {
        function handleKeydown(e: KeyboardEvent) {
            // Close modals with Escape
            if (e.key === 'Escape') {
                if (showShortcuts) {
                    showShortcuts = false;
                    e.preventDefault();
                    return;
                }
                if (previewFile) {
                    previewFile = null;
                    e.preventDefault();
                    return;
                }
            }

            // Keyboard shortcuts help - Ctrl+/
            if (e.ctrlKey && e.key === '/') {
                showShortcuts = !showShortcuts;
                e.preventDefault();
                return;
            }

            // Theme toggle - Ctrl+Shift+T
            if (e.ctrlKey && e.shiftKey && e.key === 'T') {
                themeStore.toggle();
                e.preventDefault();
                return;
            }

            // Tab navigation - Ctrl+1/2/3 (disabled during operations)
            if (e.ctrlKey && !e.shiftKey && !e.altKey && !isScanning && !isOrganizing) {
                if (e.key === '1') {
                    currentTab = 'organizer';
                    e.preventDefault();
                    return;
                }
                if (e.key === '2') {
                    currentTab = 'duplicates';
                    e.preventDefault();
                    return;
                }
                if (e.key === '3') {
                    currentTab = 'history';
                    e.preventDefault();
                    return;
                }
            }

            // Open the folder dialog - Ctrl+O (only in the organizer tab)
            if (e.ctrlKey && e.key === 'o' && currentTab === 'organizer' && currentStep === 0) {
                selectFolder();
                e.preventDefault();
                return;
            }

            // Start scan - Ctrl+Enter (when the folder is selected)
            if (e.ctrlKey && e.key === 'Enter' && currentTab === 'organizer' && currentStep === 0 && selectedPath) {
                startScan();
                e.preventDefault();
                return;
            }
        }

        window.addEventListener('keydown', handleKeydown);

        return () => {
            window.removeEventListener('keydown', handleKeydown);
        };
    });
</script>

<main id="main-content" class="h-screen w-screen flex flex-col bg-background">
    <!-- Header -->
    <header class="border-b border-border px-6 py-4">
        <div class="flex items-center justify-between">
            <div>
                <div class="flex items-center gap-3">
                    <h1 class="text-2xl font-bold text-foreground">TidyFiles</h1>
                    {#if isScanning || isOrganizing}
                        <span class="px-2 py-1 bg-blue-600/20 text-blue-600 text-xs rounded-full animate-pulse">
                            {isScanning ? 'Scanning...' : 'Organizing...'}
                        </span>
                    {/if}
                </div>
                <p class="text-sm text-muted-foreground">Organize your files automatically</p>
            </div>

            <!-- Tabs and Theme Toggle -->
            <div class="flex items-center gap-4">
                <div class="flex gap-2" role="tablist" aria-label="Main navigation">
                    <button
                            onclick={() => currentTab = 'organizer'}
                            disabled={isScanning || isOrganizing}
                            role="tab"
                            aria-selected={currentTab === 'organizer'}
                            aria-label="Organizer tab - Scan and organize files with rules"
                            class="px-4 py-2 rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed {
                currentTab === 'organizer'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted text-muted-foreground hover:bg-secondary'
              }"
                    >
                        Organizer
                    </button>
                    <button
                            onclick={() => currentTab = 'duplicates'}
                            disabled={isScanning || isOrganizing}
                            role="tab"
                            aria-selected={currentTab === 'duplicates'}
                            aria-label="Duplicates tab - Find and remove duplicate files"
                            class="px-4 py-2 rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed {
                currentTab === 'duplicates'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted text-muted-foreground hover:bg-secondary'
              }"
                    >
                        Duplicates
                    </button>
                    <button
                            onclick={() => currentTab = 'history'}
                            disabled={isScanning || isOrganizing}
                            role="tab"
                            aria-selected={currentTab === 'history'}
                            aria-label="History tab - View and manage operation history"
                            class="px-4 py-2 rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed {
                currentTab === 'history'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted text-muted-foreground hover:bg-secondary'
              }"
                    >
                        History
                    </button>
                </div>

                <!-- Settings Button -->
                <a
                        href="/settings"
                        class="p-2 rounded-lg border border-border hover:bg-secondary transition-colors"
                        aria-label="Open settings page"
                        title="Settings"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                    </svg>
                </a>

                <!-- Keyboard Shortcuts Button -->
                <button
                        onclick={() => showShortcuts = true}
                        class="p-2 rounded-lg border border-border hover:bg-secondary transition-colors"
                        aria-label="Show keyboard shortcuts"
                        title="Keyboard shortcuts (Ctrl+/)"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
                    </svg>
                </button>

                <!-- Theme Toggle -->
                <button
                        onclick={() => themeStore.toggle()}
                        class="p-2 rounded-lg border border-border hover:bg-secondary transition-colors"
                        aria-label="Toggle between light and dark theme"
                        title="Toggle theme (Ctrl+Shift+T)"
                >
                    {#if themeStore.current === 'dark'}
                        <!-- Sun Icon -->
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"/>
                        </svg>
                    {:else}
                        <!-- Moon Icon -->
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"/>
                        </svg>
                    {/if}
                </button>
            </div>
        </div>
    </header>

    {#if currentTab === 'organizer'}
        <!-- Steps -->
        <div class="border-b border-border px-6 py-3">
            <div class="flex gap-2">
                {#each steps as step, index}
                    <button
                            class="px-4 py-2 rounded-lg text-sm font-medium transition-colors {
              currentStep === index
                ? 'bg-primary text-primary-foreground'
                : currentStep > index
                  ? 'bg-secondary text-secondary-foreground'
                  : 'bg-muted text-muted-foreground'
            }"
                            onclick={() => currentStep = index}
                            disabled={index > 0 && scannedFiles.length === 0}
                    >
                        {index + 1}. {step}
                    </button>
                {/each}
            </div>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-auto p-6">
            {#if currentStep === 0}
                <!-- Step 1: Scan Files -->
                <div class="max-w-2xl mx-auto space-y-6">
                    <div class="space-y-2">
                        <h2 class="text-xl font-semibold">Select Folder to Scan</h2>
                        <p class="text-sm text-muted-foreground">
                            Choose a folder containing documents you want to organize
                        </p>
                    </div>

                    <div class="flex gap-2">
                        <input
                                type="text"
                                value={selectedPath}
                                readonly
                                placeholder="No folder selected - Click Browse to select"
                                class="flex-1 px-4 py-2 rounded-lg border border-input bg-background text-foreground"
                        />
                        <button
                                onclick={selectFolder}
                                class="px-4 py-2 bg-primary text-primary-foreground rounded-lg font-medium hover:opacity-90"
                        >
                            Browse
                        </button>
                    </div>

                    <!-- Scan Options -->
                    <div class="space-y-4 p-4 border border-border rounded-lg">
                        <div class="flex items-center justify-between">
                            <h3 class="font-medium">Scan Options</h3>
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" bind:checked={useDefaultScanSettings} class="rounded"/>
                                <span class="text-sm text-muted-foreground">Use default settings</span>
                            </label>
                        </div>

                        <Tooltip text="Include files and folders that are hidden by the operating system"
                                 position="right">
                            {#snippet children()}
                                <label class="flex items-center gap-2 cursor-pointer">
                                    <input type="checkbox" bind:checked={scanOptions.include_hidden}
                                           disabled={useDefaultScanSettings} class="rounded"/>
                                    <span class="text-sm"
                                          class:opacity-50={useDefaultScanSettings}>Include hidden files</span>
                                </label>
                            {/snippet}
                        </Tooltip>

                        <Tooltip
                                text="Follow symbolic links (shortcuts). Warning: May cause infinite loops if circular references exist."
                                position="right">
                            {#snippet children()}
                                <label class="flex items-center gap-2 cursor-pointer">
                                    <input type="checkbox" bind:checked={scanOptions.follow_symlinks}
                                           disabled={useDefaultScanSettings} class="rounded"/>
                                    <span class="text-sm" class:opacity-50={useDefaultScanSettings}>Follow symbolic links</span>
                                </label>
                            {/snippet}
                        </Tooltip>
                    </div>

                    {#if scanError}
                        <div class="p-4 bg-destructive/10 border border-destructive text-destructive rounded-lg">
                            {scanError}
                        </div>
                    {/if}

                    <button
                            onclick={startScan}
                            disabled={!selectedPath || isScanning}
                            class="w-full px-4 py-3 bg-primary text-primary-foreground rounded-lg font-medium hover:opacity-90 disabled:opacity-50"
                    >
                        {isScanning ? 'Scanning...' : 'Start Scan'}
                    </button>

                    {#if isScanning}
                        <div class="space-y-4">
                            <ProgressBar progress={-1} message={scanProgress.message || 'Scanning directory...'}/>
                            <TableSkeleton rows={8} columns={6}/>
                        </div>
                    {/if}

                    {#if scannedFiles.length > 0}
                        <div class="space-y-4">
                            <div class="p-4 bg-secondary rounded-lg space-y-2">
                                <div class="font-medium">Scan Complete!</div>
                                <div class="text-sm text-muted-foreground">
                                    Found {scannedFiles.length} files ({formatBytes(totalSize)})
                                </div>
                                <div class="text-sm text-muted-foreground">
                                    Scan took {scanDuration}ms
                                </div>
                            </div>

                            <!-- File List Table -->
                            <div class="border border-border rounded-lg overflow-hidden">
                                <div class="px-4 py-3 bg-muted border-b border-border space-y-3">
                                    <div class="flex items-center justify-between">
                                        <h3 class="font-medium">Scanned Files</h3>
                                        <span class="text-sm text-muted-foreground">
                                            {getFilteredFiles().length} of {scannedFiles.length} files
                                        </span>
                                    </div>

                                    <!-- Search and Filter -->
                                    <div class="flex gap-2">
                                        <div class="flex-1 relative">
                                            <input
                                                    type="text"
                                                    bind:value={searchQuery}
                                                    placeholder="Search by file name or path..."
                                                    class="w-full pl-9 pr-3 py-2 text-sm rounded border border-input bg-background"
                                            />
                                            <svg class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
                                                 fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                      d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                                            </svg>
                                        </div>
                                        <select
                                                bind:value={filterExtension}
                                                class="px-3 py-2 text-sm rounded border border-input bg-background"
                                        >
                                            <option value="">All Extensions</option>
                                            {#each getUniqueExtensions() as ext}
                                                <option value={ext}>.{ext}</option>
                                            {/each}
                                        </select>
                                        {#if searchQuery || filterExtension}
                                            <button
                                                    onclick={() => { searchQuery = ''; filterExtension = ''; }}
                                                    class="px-3 py-2 text-sm border border-border rounded hover:bg-secondary"
                                                    title="Clear filters"
                                            >
                                                Clear
                                            </button>
                                        {/if}
                                    </div>
                                </div>

                                <div class="overflow-auto max-h-96">
                                    <table class="w-full">
                                        <thead class="bg-muted sticky top-0">
                                        <tr>
                                            <th class="px-4 py-2 text-left text-sm font-medium">File Name</th>
                                            <th class="px-4 py-2 text-left text-sm font-medium">Extension</th>
                                            <th class="px-4 py-2 text-left text-sm font-medium">Size</th>
                                            <th class="px-4 py-2 text-left text-sm font-medium">Modified</th>
                                            <th class="px-4 py-2 text-left text-sm font-medium">Path</th>
                                            <th class="px-4 py-2 text-left text-sm font-medium">Actions</th>
                                        </tr>
                                        </thead>
                                        <tbody>
                                        {#each getPaginatedFiles() as file}
                                            <tr class="border-t border-border hover:bg-secondary/50">
                                                <td class="px-4 py-2 text-sm font-medium">{file.name}</td>
                                                <td class="px-4 py-2 text-sm">
                                                    {#if file.extension}
                                                        <span class="px-2 py-0.5 bg-primary/10 text-primary rounded text-xs">
                                                            {file.extension}
                                                        </span>
                                                    {:else}
                                                        <span class="text-muted-foreground text-xs">No ext</span>
                                                    {/if}
                                                </td>
                                                <td class="px-4 py-2 text-sm">{formatBytes(file.size)}</td>
                                                <td class="px-4 py-2 text-sm text-muted-foreground">{formatDate(file.modified)}</td>
                                                <td class="px-4 py-2 font-mono text-xs text-muted-foreground truncate max-w-xs"
                                                    title={file.path}>
                                                    {file.path}
                                                </td>
                                                <td class="px-4 py-2 text-sm">
                                                    <button
                                                            onclick={() => previewFile = file}
                                                            class="px-3 py-1 text-xs border border-border rounded hover:bg-secondary"
                                                            title="Preview file"
                                                    >
                                                        Preview
                                                    </button>
                                                </td>
                                            </tr>
                                        {/each}
                                        </tbody>
                                    </table>
                                </div>

                                <!-- Pagination -->
                                {#if getTotalPages() > 1}
                                    <div class="px-4 py-3 bg-muted border-t border-border flex items-center justify-between">
                                        <div class="text-sm text-muted-foreground">
                                            Showing {currentPage * pageSize + 1}
                                            - {Math.min((currentPage + 1) * pageSize, getFilteredFiles().length)}
                                            of {getFilteredFiles().length} files
                                        </div>
                                        <div class="flex gap-2">
                                            <button
                                                    onclick={() => goToPage(currentPage - 1)}
                                                    disabled={currentPage === 0}
                                                    class="px-3 py-1 border border-border rounded hover:bg-secondary disabled:opacity-50 disabled:cursor-not-allowed text-sm"
                                            >
                                                Previous
                                            </button>
                                            <div class="flex items-center gap-1">
                                                {#each Array.from({length: getTotalPages()}, (_, i) => i) as page}
                                                    {#if page < 3 || page >= getTotalPages() - 3 || Math.abs(page - currentPage) <= 1}
                                                        <button
                                                                onclick={() => goToPage(page)}
                                                                class="px-3 py-1 rounded text-sm transition-colors {
                                                                    currentPage === page
                                                                      ? 'bg-primary text-primary-foreground'
                                                                      : 'hover:bg-secondary'
                                                                  }"
                                                        >
                                                            {page + 1}
                                                        </button>
                                                    {:else if page === 3 || page === getTotalPages() - 4}
                                                        <span class="px-2 text-muted-foreground">...</span>
                                                    {/if}
                                                {/each}
                                            </div>
                                            <button
                                                    onclick={() => goToPage(currentPage + 1)}
                                                    disabled={currentPage >= getTotalPages() - 1}
                                                    class="px-3 py-1 border border-border rounded hover:bg-secondary disabled:opacity-50 disabled:cursor-not-allowed text-sm"
                                            >
                                                Next
                                            </button>
                                        </div>
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {/if}
                </div>

            {:else if currentStep === 1}
                <!-- Step 2: Create Rules - Simplified for now -->
                <div class="max-w-4xl mx-auto space-y-6">
                    <div class="flex items-center justify-between">
                        <div>
                            <h2 class="text-xl font-semibold">Organization Rules</h2>
                            <p class="text-sm text-muted-foreground">
                                Define rules to organize your {scannedFiles.length} files
                            </p>
                        </div>
                        <div class="flex gap-2">
                            <button
                                    onclick={importRules}
                                    class="px-4 py-2 border border-border rounded-lg hover:bg-secondary"
                                    title="Import rules from JSON file"
                            >
                                Import Rules
                            </button>
                            <button
                                    onclick={exportRules}
                                    class="px-4 py-2 border border-border rounded-lg hover:bg-secondary"
                                    title="Export rules to JSON file"
                            >
                                Export Rules
                            </button>
                            <button
                                    onclick={addRule}
                                    class="px-4 py-2 bg-primary text-primary-foreground rounded-lg font-medium hover:opacity-90"
                            >
                                + Add Rule
                            </button>
                        </div>
                    </div>

                    {#if rules.length === 0}
                        <div class="border border-dashed border-border rounded-lg">
                            <EmptyState
                                    icon="📝"
                                    title="No Rules Created"
                                    description="Create rules to automatically organize your files. Rules can match files by name, extension, size, or date, and move them to specific folders."
                                    actionText="Create Your First Rule"
                                    onAction={addRule}
                            />
                        </div>
                    {:else}
                        <div class="space-y-4">
                            {#each rules as rule (rule.id)}
                                <div class="p-4 border border-border rounded-lg space-y-3">
                                    <div class="flex items-center justify-between">
                                        <input
                                                type="text"
                                                bind:value={rule.name}
                                                class="text-lg font-medium bg-transparent border-none focus:outline-none"
                                                placeholder="Rule name"
                                        />
                                        <button
                                                onclick={() => prepareRemoveRule(rule.id)}
                                                class="text-destructive hover:underline text-sm"
                                        >
                                            Remove
                                        </button>
                                    </div>

                                    <div class="grid grid-cols-2 gap-4">
                                        <div>
                                            <label for="rule-action-{rule.id}" class="text-sm text-muted-foreground">Action</label>
                                            <select
                                                    id="rule-action-{rule.id}"
                                                    bind:value={rule.action.type}
                                                    class="w-full px-3 py-2 rounded border border-input bg-background"
                                            >
                                                <option value="move">Move</option>
                                                <option value="copy">Copy</option>
                                            </select>
                                        </div>

                                        <div>
                                            <label for="rule-conflict-{rule.id}" class="text-sm text-muted-foreground">Conflict
                                                Resolution</label>
                                            <select
                                                    id="rule-conflict-{rule.id}"
                                                    bind:value={rule.conflict_resolution}
                                                    class="w-full px-3 py-2 rounded border border-input bg-background"
                                            >
                                                <option value="skip">Skip</option>
                                                <option value="rename">Rename</option>
                                                <option value="overwrite">Overwrite</option>
                                            </select>
                                        </div>
                                    </div>

                                    <div>
                                        <label for="rule-destination-{rule.id}" class="text-sm text-muted-foreground">Destination
                                            Folder</label>
                                        <div class="flex gap-2">
                                            <input
                                                    id="rule-destination-{rule.id}"
                                                    type="text"
                                                    bind:value={rule.action.destination}
                                                    placeholder="e.g., C:/Documents/Organized"
                                                    class="flex-1 px-3 py-2 rounded border border-input bg-background"
                                            />
                                            <button
                                                    onclick={async () => {
                                                        const selected = await open({
                                                            directory: true,
                                                            multiple: false,
                                                        });
                                                        if (selected && typeof selected === 'string') {
                                                            rule.action.destination = selected;
                                                        }
                                                    }}
                                                    type="button"
                                                    class="px-4 py-2 border border-border rounded hover:bg-secondary transition-colors"
                                                    title="Browse for folder"
                                            >
                                                Browse
                                            </button>
                                        </div>
                                        <details class="mt-2 text-xs">
                                            <summary class="cursor-pointer text-primary hover:underline">Available
                                                placeholders
                                            </summary>
                                            <div class="mt-2 p-3 bg-muted rounded space-y-1 text-muted-foreground">
                                                <div><code>{'{filename}'}</code> - File name without extension</div>
                                                <div><code>{'{ext}'}</code> - File extension</div>
                                                <div><code>{'{name}'}</code> - Full file name</div>
                                                <div><code>{'{parent}'}</code> - Parent folder name</div>
                                                <div><code>{'{size}'}</code> - File size (e.g., 1.5MB)</div>
                                                <div class="pt-1 font-semibold">Modified date:</div>
                                                <div><code>{'{year}'}</code>, <code>{'{month}'}</code>,
                                                    <code>{'{day}'}</code> - Modified date parts
                                                </div>
                                                <div><code>{'{modified_date}'}</code> - Full modified date (YYYY-MM-DD)
                                                </div>
                                                <div class="pt-1 font-semibold">Created date:</div>
                                                <div><code>{'{created_year}'}</code>, <code>{'{created_month}'}</code>,
                                                    <code>{'{created_day}'}</code></div>
                                                <div><code>{'{created_date}'}</code> - Full created date (YYYY-MM-DD)
                                                </div>
                                            </div>
                                        </details>
                                    </div>

                                    <div>
                                        <label for="rule-rename-{rule.id}" class="text-sm text-muted-foreground">
                                            Rename Pattern (optional)
                                        </label>
                                        <input
                                                id="rule-rename-{rule.id}"
                                                type="text"
                                                bind:value={rule.action.rename_pattern}
                                                placeholder="e.g., {'{modified_date}'}_document or {'{parent}'}_{'{counter:03d}'}"
                                                class="w-full px-3 py-2 rounded border border-input bg-background"
                                        />
                                        <p class="text-xs text-muted-foreground mt-1">
                                            Same placeholders as above, plus <code>{'{counter:03d}'}</code> for
                                            sequential numbering (001, 002, ...)
                                        </p>
                                        <p class="text-xs text-blue-600 mt-1">
                                            Leave empty to keep original filename
                                        </p>
                                    </div>

                                    <!-- Rule Conditions -->
                                    <div class="pt-3 border-t border-border">
                                        <RuleConditionBuilder
                                                rule={rule}
                                                onUpdate={updateRule}
                                        />
                                    </div>

                                    <label class="flex items-center gap-2">
                                        <input type="checkbox" bind:checked={rule.enabled} class="rounded"/>
                                        <span class="text-sm">Enabled</span>
                                    </label>
                                </div>
                            {/each}
                        </div>
                    {/if}

                    {#if isPreviewingRules}
                        <div class="space-y-4">
                            <div class="space-y-2 p-4 border border-border rounded-lg">
                                <div class="flex items-center justify-between text-sm">
                                    <span class="font-medium">Matching rules...</span>
                                    <span class="text-muted-foreground">Analyzing {scannedFiles.length} files</span>
                                </div>
                                <div class="w-full h-2 bg-secondary rounded-full overflow-hidden">
                                    <div class="h-full bg-primary animate-pulse"></div>
                                </div>
                            </div>
                            <TableSkeleton rows={10} columns={3}/>
                        </div>
                    {/if}

                    <div class="flex gap-2">
                        <button
                                onclick={() => currentStep = 0}
                                class="px-4 py-2 border border-border rounded-lg font-medium hover:bg-secondary"
                        >
                            Back
                        </button>
                        <button
                                onclick={previewMatches}
                                disabled={rules.length === 0 || isPreviewingRules}
                                class="flex-1 px-4 py-2 bg-primary text-primary-foreground rounded-lg font-medium hover:opacity-90 disabled:opacity-50"
                        >
                            {isPreviewingRules ? 'Loading...' : 'Preview Matches'}
                        </button>
                    </div>
                </div>

            {:else if currentStep === 2}
                <!-- Step 3: Preview -->
                <div class="max-w-4xl mx-auto space-y-6">
                    <div>
                        <h2 class="text-xl font-semibold">Preview Changes</h2>
                        <p class="text-sm text-muted-foreground">
                            {matches.length} files will be organized
                        </p>
                    </div>

                    <div class="border border-border rounded-lg overflow-hidden">
                        <!-- Search and Filter -->
                        <div class="px-4 py-3 bg-muted border-b border-border space-y-3">
                            <div class="flex items-center justify-between">
                                <h3 class="font-medium">Matched Files</h3>
                                <span class="text-sm text-muted-foreground">
                                    {getFilteredMatches().length} of {matches.length} files
                                </span>
                            </div>

                            <div class="flex gap-2">
                                <div class="flex-1 relative">
                                    <input
                                            type="text"
                                            bind:value={previewSearchQuery}
                                            placeholder="Search by file, destination, or rule name..."
                                            class="w-full pl-9 pr-3 py-2 text-sm rounded border border-input bg-background"
                                    />
                                    <svg class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
                                         fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                                    </svg>
                                </div>
                                <select
                                        bind:value={previewFilterRule}
                                        class="px-3 py-2 text-sm rounded border border-input bg-background"
                                >
                                    <option value="">All Rules</option>
                                    {#each getUniqueRules() as rule}
                                        <option value={rule.id}>{rule.name}</option>
                                    {/each}
                                </select>
                                {#if previewSearchQuery || previewFilterRule}
                                    <button
                                            onclick={() => { previewSearchQuery = ''; previewFilterRule = ''; }}
                                            class="px-3 py-2 text-sm border border-border rounded hover:bg-secondary"
                                            title="Clear filters"
                                    >
                                        Clear
                                    </button>
                                {/if}
                            </div>
                        </div>

                        <div class="max-h-96 overflow-auto">
                            <table class="w-full">
                                <thead class="bg-muted sticky top-0">
                                <tr>
                                    <th class="px-4 py-2 text-center text-sm font-medium w-12">
                                        <input
                                                type="checkbox"
                                                checked={getFilteredMatches().length > 0 && getFilteredMatches().every(match => selectedFiles.has(match.file_path))}
                                                onchange={toggleAllVisible}
                                                class="rounded cursor-pointer"
                                                title="Select all visible files"
                                        />
                                    </th>
                                    <th class="px-4 py-2 text-left text-sm font-medium">File</th>
                                    <th class="px-4 py-2 text-left text-sm font-medium">Destination</th>
                                    <th class="px-4 py-2 text-left text-sm font-medium">Rule</th>
                                </tr>
                                </thead>
                                <tbody>
                                {#each getFilteredMatches() as match}
                                    <tr class="border-t border-border hover:bg-muted/50 {selectedFiles.has(match.file_path) ? 'bg-blue-50 dark:bg-blue-900/20' : ''}">
                                        <td class="px-4 py-2 text-center">
                                            <input
                                                    type="checkbox"
                                                    checked={selectedFiles.has(match.file_path)}
                                                    onchange={() => toggleFileSelection(match.file_path)}
                                                    class="rounded cursor-pointer"
                                            />
                                        </td>
                                        <td class="px-4 py-2 text-sm font-mono truncate max-w-xs"
                                            title={match.file_path}>{match.file_path}</td>
                                        <td class="px-4 py-2 text-sm font-mono truncate max-w-xs"
                                            title={match.destination_path}>{match.destination_path}</td>
                                        <td class="px-4 py-2 text-sm">{match.rule_name}</td>
                                    </tr>
                                {/each}
                                </tbody>
                            </table>
                        </div>
                    </div>

                    {#if selectedFiles.size > 0}
                        <div class="p-3 bg-primary/10 border border-primary/20 rounded-lg flex items-center justify-between">
                            <span class="text-sm font-medium">{selectedFiles.size}
                                file{selectedFiles.size === 1 ? '' : 's'} selected</span>
                            <button
                                    onclick={() => selectedFiles = new Set()}
                                    class="text-sm text-primary hover:underline"
                            >
                                Clear selection
                            </button>
                        </div>
                    {/if}

                    {#if isOrganizing}
                        <div class="space-y-2 p-4 border border-border rounded-lg">
                            <ProgressBar
                                    progress={organizeProgress.total > 0 ? (organizeProgress.current / organizeProgress.total) * 100 : undefined}
                                    message={organizeProgress.message || `Processing ${getSelectedMatches().length} files...`}
                            />
                        </div>
                    {/if}

                    {#if getFilteredMatches().length === 0}
                        <div class="p-4 bg-muted rounded-lg text-center">
                            <p class="text-muted-foreground">No files match your current filters</p>
                            <button
                                    onclick={() => { previewSearchQuery = ''; previewFilterRule = ''; }}
                                    class="mt-2 text-sm text-primary hover:underline"
                            >
                                Clear filters to see all {matches.length} files
                            </button>
                        </div>
                    {/if}

                    <!-- Dry Run Mode Toggle -->
                    <div class="p-4 border border-border rounded-lg {dryRun ? 'bg-blue-50 dark:bg-blue-900/10 border-blue-200 dark:border-blue-800' : 'bg-muted'}">
                        <div class="flex items-start gap-3">
                            <input
                                    type="checkbox"
                                    bind:checked={dryRun}
                                    id="dry-run-toggle"
                                    class="mt-1 rounded cursor-pointer"
                            />
                            <div class="flex-1">
                                <label for="dry-run-toggle" class="cursor-pointer inline-block">
                                    <span class="font-medium inline-flex items-center gap-2">
                                        Dry Run Mode (Test Mode)
                                        {#if dryRun}
                                            <span class="text-xs px-2 py-0.5 bg-blue-500 text-white rounded">ENABLED</span>
                                        {/if}
                                    </span>
                                </label>
                                <div class="text-sm text-muted-foreground mt-1">
                                    {#if dryRun}
                                        <strong>Files will NOT be moved.</strong> This will only simulate the operation
                                        and show you what would happen.
                                    {:else}
                                        When enabled, no files will actually be moved. Use this to safely test your
                                        organization rules.
                                    {/if}
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="flex gap-2">
                        <button
                                onclick={() => currentStep = 1}
                                class="px-4 py-2 border border-border rounded-lg font-medium hover:bg-secondary"
                        >
                            Back to Rules
                        </button>
                        <button
                                onclick={startOrganize}
                                disabled={isOrganizing || getFilteredMatches().length === 0}
                                class="flex-1 px-4 py-2 rounded-lg font-medium hover:opacity-90 disabled:opacity-50 {dryRun ? 'bg-blue-600 text-white' : 'bg-primary text-primary-foreground'}"
                        >
                            {#if isOrganizing}
                                {dryRun ? 'Testing...' : 'Organizing...'}
                            {:else}
                                {dryRun ? 'Test Organization' : 'Start Organization'} ({getSelectedMatches().length}
                                file{getSelectedMatches().length === 1 ? '' : 's'})
                            {/if}
                        </button>
                    </div>
                </div>

            {:else if currentStep === 3}
                <!-- Step 4: Results -->
                <div class="max-w-2xl mx-auto space-y-6">
                    {#if dryRun}
                        <div class="p-4 bg-blue-50 dark:bg-blue-900/20 border-2 border-blue-500 rounded-lg">
                            <div class="flex items-center gap-3">
                                <div class="text-2xl">ℹ️</div>
                                <div>
                                    <h3 class="font-semibold text-blue-900 dark:text-blue-100">Dry Run Mode - No Files
                                        Were Moved</h3>
                                    <p class="text-sm text-blue-700 dark:text-blue-300">This was a test run. The results
                                        below show what would have happened if you ran this for real.</p>
                                </div>
                            </div>
                        </div>
                    {/if}

                    <div class="text-center space-y-2">
                        <div class="text-4xl">✓</div>
                        <h2 class="text-2xl font-semibold">{dryRun ? 'Dry Run Complete!' : 'Organization Complete!'}</h2>
                    </div>

                    {#if organizeResult}
                        <div class="grid grid-cols-3 gap-4">
                            <div class="p-4 bg-secondary rounded-lg text-center">
                                <div class="text-2xl font-bold">{organizeResult.total_files}</div>
                                <div class="text-sm text-muted-foreground">Total Files</div>
                            </div>
                            <div class="p-4 bg-secondary rounded-lg text-center">
                                <div class="text-2xl font-bold text-green-600">{organizeResult.success_count}</div>
                                <div class="text-sm text-muted-foreground">{dryRun ? 'Would Succeed' : 'Success'}</div>
                            </div>
                            <div class="p-4 bg-secondary rounded-lg text-center">
                                <div class="text-2xl font-bold text-red-600">{organizeResult.error_count}</div>
                                <div class="text-sm text-muted-foreground">{dryRun ? 'Would Fail' : 'Errors'}</div>
                            </div>
                        </div>

                        {#if organizeResult.errors.length > 0}
                            <div class="space-y-2">
                                <h3 class="font-medium">Errors</h3>
                                <div class="space-y-2">
                                    {#each organizeResult.errors as error}
                                        <div class="p-3 bg-destructive/10 border border-destructive text-destructive rounded text-sm">
                                            <div class="font-medium">{error.file}</div>
                                            <div>{error.error}</div>
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        {/if}
                    {/if}

                    <button
                            onclick={prepareReset}
                            class="w-full px-4 py-3 bg-primary text-primary-foreground rounded-lg font-medium hover:opacity-90"
                    >
                        Start New Organization
                    </button>
                </div>
            {/if}
        </div>
    {:else if currentTab === 'duplicates'}
        <!-- Duplicates Tab -->
        <DuplicateFinder/>
    {:else}
        <!-- History Tab -->
        <OperationHistory/>
    {/if}
</main>

<!-- File Preview Modal -->
<FilePreview file={previewFile} onClose={() => previewFile = null}/>

<!-- Keyboard Shortcuts Modal -->
<KeyboardShortcuts isOpen={showShortcuts} onClose={() => showShortcuts = false}/>

<!-- Toast Notifications -->
<!-- Confirmation Dialogs -->
<ConfirmDialog
        show={showRemoveRuleDialog}
        title="Remove this rule?"
        message="<p>This rule will be permanently deleted.</p><p class='mt-2'>This action cannot be undone.</p>"
        confirmText="Remove Rule"
        variant="destructive"
        onConfirm={confirmRemoveRule}
        onCancel={cancelRemoveRule}
/>

<ConfirmDialog
        show={showResetDialog}
        title="Start new organization?"
        message="<p>This will clear all current work including:</p><ul class='list-disc list-inside mt-2 space-y-1'><li>Scanned files</li><li>Created rules</li><li>Preview matches</li><li>Organization results</li></ul><p class='text-destructive font-medium mt-3'>⚠️ Any unsaved progress will be lost!</p>"
        confirmText="Start New"
        variant="warning"
        onConfirm={confirmReset}
        onCancel={cancelReset}
/>

<Toast/>

<!-- Error Log -->
<ErrorLog onRetry={handleRetry}/>
