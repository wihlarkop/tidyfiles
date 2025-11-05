import { toastStore } from '$lib/stores/toast.svelte';
import { errorLogStore } from '$lib/stores/errorLog.svelte';

interface ErrorHandlerOptions {
    operation: string;
    error: unknown;
    details?: string;
    retryable?: boolean;
    retryData?: any;
    customMessage?: string;
    skipToast?: boolean;
    skipLog?: boolean;
}

/**
 * Centralized error handler that provides contextual error messages,
 * toast notifications, and error logging.
 */
export function handleError(options: ErrorHandlerOptions): string {
    const {
        operation,
        error,
        details,
        retryable = false,
        retryData,
        customMessage,
        skipToast = false,
        skipLog = false
    } = options;

    const errorMsg = String(error);
    const contextualMessage = customMessage || generateContextualMessage(operation, errorMsg);

    // Show toast notification
    if (!skipToast) {
        toastStore.error(contextualMessage);
    }

    // Log to the error store
    if (!skipLog) {
        errorLogStore.log(operation, errorMsg, details, retryable, retryData);
    }

    return contextualMessage;
}

/**
 * Generates a contextual error message based on the operation and error type.
 */
function generateContextualMessage(operation: string, error: string): string {
    const errorLower = error.toLowerCase();

    // Permission errors
    if (errorLower.includes('permission') || errorLower.includes('access denied')) {
        return `${operation} failed: Permission denied. Try running as administrator or check file permissions.`;
    }

    // File/folder not found
    if (errorLower.includes('not found') || errorLower.includes('does not exist')) {
        return `${operation} failed: File or folder not found. It may have been moved or deleted.`;
    }

    // File in use / locked
    if (errorLower.includes('in use') || errorLower.includes('locked')) {
        return `${operation} failed: File is in use. Close any programs using this file and try again.`;
    }

    // Disk space
    if (errorLower.includes('space') || errorLower.includes('disk full')) {
        return `${operation} failed: Insufficient disk space. Free up space and try again.`;
    }

    // Timeout
    if (errorLower.includes('timeout') || errorLower.includes('timed out')) {
        return `${operation} failed: Operation timed out. Try again or check system resources.`;
    }

    // Network errors
    if (errorLower.includes('network') || errorLower.includes('connection')) {
        return `${operation} failed: Network error. Check your connection and try again.`;
    }

    // Invalid input
    if (errorLower.includes('invalid') || errorLower.includes('malformed')) {
        return `${operation} failed: Invalid input. Check the data format and try again.`;
    }

    // Database/corruption errors
    if (errorLower.includes('corrupt') || errorLower.includes('database')) {
        return `${operation} failed: Database error. The data may be corrupted or inaccessible.`;
    }

    // Generic fallback
    return `${operation} failed: ${error}`;
}

/**
 * Quick helper for simple error handling (most common use case).
 */
export function quickError(operation: string, error: unknown): string {
    return handleError({ operation, error });
}

/**
 * Error handler with retry support.
 */
export function retryableError(
    operation: string,
    error: unknown,
    retryData?: any,
    details?: string
): string {
    return handleError({
        operation,
        error,
        details,
        retryable: true,
        retryData
    });
}
