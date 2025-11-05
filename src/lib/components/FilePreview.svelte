<script lang="ts">
    import {convertFileSrc} from '@tauri-apps/api/core';
    import {openPath, revealItemInDir} from '@tauri-apps/plugin-opener';
    import type {FileInfo} from '$lib/types';
    import {readFileContent} from '$lib/api/commands';

    interface Props {
        file: FileInfo | null;
        onClose: () => void;
    }

    let {file, onClose}: Props = $props();
    let textContent = $state<string | null>(null);
    let textError = $state<string | null>(null);
    let isLoadingText = $state(false);
    let imageError = $state(false);

    function getFileType(file: FileInfo): 'image' | 'video' | 'audio' | 'text' | 'code' | 'markdown' | 'pdf' | 'csv' | 'unknown' {
        const ext = file.extension?.toLowerCase();
        if (!ext) return 'unknown';

        const imageExts = ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp', 'svg', 'ico'];
        const videoExts = ['mp4', 'webm', 'ogg', 'mov', 'avi', 'mkv'];
        const audioExts = ['mp3', 'wav', 'ogg', 'flac', 'm4a', 'aac'];
        const codeExts = ['js', 'ts', 'jsx', 'tsx', 'py', 'java', 'cpp', 'c', 'h', 'cs', 'go', 'rs', 'php', 'rb', 'swift'];
        const textExts = ['txt', 'log', 'ini', 'cfg', 'conf'];
        const markupExts = ['html', 'xml', 'svg'];
        const dataExts = ['json', 'yml', 'yaml', 'toml'];

        if (imageExts.includes(ext)) return 'image';
        if (videoExts.includes(ext)) return 'video';
        if (audioExts.includes(ext)) return 'audio';
        if (ext === 'md' || ext === 'markdown') return 'markdown';
        if (ext === 'csv') return 'csv';
        if (codeExts.includes(ext)) return 'code';
        if (textExts.includes(ext) || markupExts.includes(ext) || dataExts.includes(ext)) return 'text';
        if (ext === 'pdf') return 'pdf';
        return 'unknown';
    }

    function getFileIcon(type: string): string {
        switch (type) {
            case 'image': return '🖼️';
            case 'video': return '🎥';
            case 'audio': return '🎵';
            case 'code': return '💻';
            case 'markdown': return '📝';
            case 'csv': return '📊';
            case 'pdf': return '📄';
            case 'text': return '📃';
            default: return '📎';
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
        return new Date(timestamp * 1000).toLocaleString();
    }

    async function loadTextContent(file: FileInfo) {
        textContent = null;
        textError = null;
        isLoadingText = true;

        try {
             // 512KB limit
            textContent = await readFileContent(file.path, 512 * 1024);
        } catch (error) {
            textError = error as string;
        } finally {
            isLoadingText = false;
        }
    }

    // Load text content when a file changes and is a text-based file
    $effect(() => {
        if (file) {
            imageError = false;
            const fileType = getFileType(file);
            if (fileType === 'text' || fileType === 'code' || fileType === 'markdown' || fileType === 'csv') {
                loadTextContent(file);
            } else {
                textContent = null;
                textError = null;
                isLoadingText = false;
            }
        }
    });
</script>

{#if file}
    <div
            class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4"
            onclick={onClose}
            onkeydown={(e) => e.key === 'Escape' && onClose()}
            role="button"
            tabindex="-1"
    >
        <div
                class="bg-background border border-border rounded-lg shadow-2xl max-w-4xl w-full max-h-[90vh] flex flex-col"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
                role="dialog"
                aria-modal="true"
                tabindex="0"
        >
            <!-- Header -->
            <div class="p-4 border-b border-border flex items-start justify-between">
                <div class="flex-1 min-w-0 flex items-start gap-3">
                    <div class="text-3xl flex-shrink-0">{getFileIcon(getFileType(file))}</div>
                    <div class="flex-1 min-w-0">
                        <h3 class="text-lg font-semibold truncate">{file.name}</h3>
                        <div class="flex gap-4 mt-1 text-sm text-muted-foreground">
                            <span>{formatBytes(file.size)}</span>
                            {#if file.extension}
                                <span class="uppercase">{file.extension}</span>
                            {/if}
                            <span>Modified: {formatDate(file.modified)}</span>
                        </div>
                        <div class="text-xs text-muted-foreground mt-1 font-mono truncate" title={file.path}>
                            {file.path}
                        </div>
                    </div>
                </div>
                <button
                        onclick={onClose}
                        class="ml-4 p-2 hover:bg-secondary rounded transition-colors flex-shrink-0"
                        aria-label="Close preview"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M6 18L18 6M6 6l12 12"/>
                    </svg>
                </button>
            </div>

            <!-- Content -->
            <div class="flex-1 overflow-auto p-6 bg-muted/30">
                {#if getFileType(file) === 'image'}
                    <div class="flex flex-col items-center justify-center h-full">
                        {#if imageError}
                            <div class="text-center">
                                <svg class="w-16 h-16 text-muted-foreground mb-4 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"/>
                                </svg>
                                <p class="text-lg font-medium mb-2">Failed to load image</p>
                                <p class="text-sm text-muted-foreground mb-4">The image file could not be displayed.</p>
                                <button
                                        onclick={() => openPath(file.path)}
                                        class="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90"
                                >
                                    Open in External App
                                </button>
                            </div>
                        {:else}
                            <img
                                    src={convertFileSrc(file.path)}
                                    alt={file.name}
                                    class="max-w-full max-h-full object-contain rounded"
                                    onerror={() => imageError = true}
                            />
                        {/if}
                    </div>
                {:else if getFileType(file) === 'video'}
                    <div class="flex items-center justify-center h-full">
                        <!-- svelte-ignore a11y_media_has_caption -->
                        <video
                                src={convertFileSrc(file.path)}
                                controls
                                class="max-w-full max-h-full rounded shadow-lg"
                        >
                            Your browser does not support the video tag.
                        </video>
                    </div>
                {:else if getFileType(file) === 'audio'}
                    <div class="flex flex-col items-center justify-center h-full">
                        <div class="text-6xl mb-4">🎵</div>
                        <div class="text-lg font-semibold mb-4">{file.name}</div>
                        <audio
                                src={convertFileSrc(file.path)}
                                controls
                                class="w-full max-w-md"
                        >
                            Your browser does not support the audio tag.
                        </audio>
                    </div>
                {:else if getFileType(file) === 'code'}
                    <div class="bg-background p-4 rounded border border-border font-mono text-sm h-full overflow-auto">
                        {#if isLoadingText}
                            <div class="flex items-center justify-center h-full text-muted-foreground">
                                <div class="text-center">
                                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-2"></div>
                                    <div>Loading code...</div>
                                </div>
                            </div>
                        {:else if textError}
                            <div class="text-destructive">
                                <div class="font-semibold mb-2">Error loading file:</div>
                                <div>{textError}</div>
                            </div>
                        {:else if textContent !== null}
                            <pre class="whitespace-pre-wrap break-words language-{file.extension}"><code>{textContent}</code></pre>
                        {/if}
                    </div>
                {:else if getFileType(file) === 'markdown'}
                    <div class="bg-background p-6 rounded border border-border prose prose-sm dark:prose-invert max-w-none h-full overflow-auto">
                        {#if isLoadingText}
                            <div class="flex items-center justify-center h-full text-muted-foreground">
                                <div class="text-center">
                                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-2"></div>
                                    <div>Loading markdown...</div>
                                </div>
                            </div>
                        {:else if textError}
                            <div class="text-destructive">
                                <div class="font-semibold mb-2">Error loading file:</div>
                                <div>{textError}</div>
                            </div>
                        {:else if textContent !== null}
                            <pre class="whitespace-pre-wrap break-words font-mono text-sm">{textContent}</pre>
                        {/if}
                    </div>
                {:else if getFileType(file) === 'csv'}
                    <div class="bg-background p-4 rounded border border-border h-full overflow-auto">
                        {#if isLoadingText}
                            <div class="flex items-center justify-center h-full text-muted-foreground">
                                <div class="text-center">
                                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-2"></div>
                                    <div>Loading CSV...</div>
                                </div>
                            </div>
                        {:else if textError}
                            <div class="text-destructive">
                                <div class="font-semibold mb-2">Error loading file:</div>
                                <div>{textError}</div>
                            </div>
                        {:else if textContent !== null}
                            <pre class="whitespace-pre font-mono text-xs">{textContent}</pre>
                        {/if}
                    </div>
                {:else if getFileType(file) === 'pdf'}
                    <div class="flex flex-col items-center justify-center h-full text-center">
                        <svg class="w-16 h-16 text-muted-foreground mb-4" fill="none" stroke="currentColor"
                             viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"/>
                        </svg>
                        <p class="text-lg font-medium mb-2">PDF Document</p>
                        <p class="text-sm text-muted-foreground">
                            PDF preview is not available. Click below to open in external viewer.
                        </p>
                        <button
                                onclick={() => openPath(file.path)}
                                class="mt-4 px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90"
                        >
                            Open PDF
                        </button>
                    </div>
                {:else if getFileType(file) === 'text'}
                    <div class="bg-background p-4 rounded border border-border font-mono text-sm h-full overflow-auto">
                        {#if isLoadingText}
                            <div class="flex items-center justify-center h-full text-muted-foreground">
                                <div class="text-center">
                                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-2"></div>
                                    <div>Loading file content...</div>
                                </div>
                            </div>
                        {:else if textError}
                            <div class="text-destructive">
                                <div class="font-semibold mb-2">Error loading file:</div>
                                <div>{textError}</div>
                            </div>
                        {:else if textContent !== null}
                            <pre class="whitespace-pre-wrap break-words">{textContent}</pre>
                        {/if}
                    </div>
                {:else}
                    <div class="flex flex-col items-center justify-center h-full text-center">
                        <svg class="w-16 h-16 text-muted-foreground mb-4" fill="none" stroke="currentColor"
                             viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                        </svg>
                        <p class="text-lg font-medium mb-2">Preview Not Available</p>
                        <p class="text-sm text-muted-foreground">
                            This file type cannot be previewed in the app.
                        </p>
                        <button
                                onclick={() => openPath(file.path)}
                                class="mt-4 px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90"
                        >
                            Open in External App
                        </button>
                    </div>
                {/if}
            </div>

            <!-- Footer -->
            <div class="p-4 border-t border-border flex justify-between items-center">
                <div class="text-sm text-muted-foreground">
                    {#if file.is_hidden}
                        <span class="px-2 py-1 bg-yellow-600/20 text-yellow-600 rounded text-xs">Hidden File</span>
                    {/if}
                </div>
                <div class="flex gap-2">
                    <button
                            onclick={() => revealItemInDir(file.path)}
                            class="px-4 py-2 border border-border rounded hover:bg-secondary text-sm"
                    >
                        Open Folder
                    </button>
                    <button
                            onclick={onClose}
                            class="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 text-sm"
                    >
                        Close
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}
