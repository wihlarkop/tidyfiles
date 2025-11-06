<script lang="ts">
    import { onMount } from 'svelte';
    import { check } from '@tauri-apps/plugin-updater';
    import { relaunch } from '@tauri-apps/plugin-process';
    import { toastStore } from '$lib/stores/toast.svelte';

    let checking = $state(false);
    let downloading = $state(false);
    let updateAvailable = $state(false);
    let updateVersion = $state('');
    let updateBody = $state('');
    let downloadProgress = $state(0);
    let showUpdateDialog = $state(false);

    async function checkForUpdates(silent = false) {
        if (checking) return;

        checking = true;
        try {
            const update = await check();

            if (update) {
                updateAvailable = true;
                updateVersion = update.version;
                updateBody = update.body || '';
                showUpdateDialog = true;

                if (!silent) {
                    toastStore.info(`Update available: v${update.version}`);
                }
            } else {
                if (!silent) {
                    toastStore.success('You are using the latest version');
                }
            }
        } catch (error) {
            console.error('Failed to check for updates:', error);
            if (!silent) {
                toastStore.error('Failed to check for updates');
            }
        } finally {
            checking = false;
        }
    }

    async function downloadAndInstall() {
        if (downloading) return;

        downloading = true;
        downloadProgress = 0;

        try {
            const update = await check();
            if (!update) {
                toastStore.info('No update available');
                downloading = false;
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

            // Wait a moment before restarting
            setTimeout(async () => {
                await relaunch();
            }, 1000);
        } catch (error) {
            console.error('Failed to download and install update:', error);
            toastStore.error('Failed to install update');
            downloading = false;
        }
    }

    function dismissUpdate() {
        showUpdateDialog = false;
        updateAvailable = false;
    }

    // Check for updates on the mount (silently)
    onMount(() => {
        // Check for updates 2 seconds after the app starts
        setTimeout(() => {
            checkForUpdates(true);
        }, 2000);
    });

    // Export function to allow manual check
    export function manualCheck() {
        checkForUpdates(false);
    }
</script>

{#if showUpdateDialog}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-background border border-border rounded-lg shadow-xl max-w-md w-full mx-4 p-6">
            <div class="flex items-start gap-3 mb-4">
                <div class="flex-shrink-0 w-10 h-10 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center">
                    <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                    </svg>
                </div>
                <div class="flex-1">
                    <h3 class="text-lg font-semibold text-foreground">Update Available</h3>
                    <p class="text-sm text-muted-foreground mt-1">
                        Version {updateVersion} is now available
                    </p>
                </div>
            </div>

            {#if updateBody}
                <div class="mb-4 p-3 bg-muted rounded-lg max-h-48 overflow-y-auto">
                    <p class="text-sm text-foreground whitespace-pre-wrap">{updateBody}</p>
                </div>
            {/if}

            {#if downloading}
                <div class="mb-4">
                    <div class="flex justify-between text-sm mb-2">
                        <span class="text-muted-foreground">Downloading...</span>
                        <span class="text-foreground font-medium">{downloadProgress.toFixed(0)}%</span>
                    </div>
                    <div class="w-full bg-muted rounded-full h-2">
                        <div
                            class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                            style="width: {downloadProgress}%"
                        ></div>
                    </div>
                </div>
            {/if}

            <div class="flex gap-3">
                {#if !downloading}
                    <button
                        onclick={dismissUpdate}
                        class="flex-1 px-4 py-2 border border-border rounded-lg hover:bg-muted transition-colors text-foreground"
                    >
                        Later
                    </button>
                {/if}
                <button
                    onclick={downloadAndInstall}
                    disabled={downloading}
                    class="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                    {downloading ? 'Installing...' : 'Update Now'}
                </button>
            </div>
        </div>
    </div>
{/if}

<!-- Hidden - component is used for its side effects -->
<div style="display: none;"></div>
