interface ErrorEntry {
    id: string;
    timestamp: number;
    operation: string;
    error: string;
    details?: string;
    recovery?: string;
    retryable: boolean;
    retryData?: any;
}

class ErrorLogStore {
    errors = $state<ErrorEntry[]>([]);
    maxErrors = 50;

    log(operation: string, error: string, details?: string, retryable: boolean = false, retryData?: any) {
        const recovery = this.getRecoverySuggestion(error);

        const entry: ErrorEntry = {
            id: crypto.randomUUID(),
            timestamp: Date.now(),
            operation,
            error,
            details,
            recovery,
            retryable,
            retryData
        };

        this.errors = [entry, ...this.errors].slice(0, this.maxErrors);

        // Log to console for debugging
        console.error(`[${operation}]`, error, details || '');
    }

    clear(id?: string) {
        if (id) {
            this.errors = this.errors.filter(e => e.id !== id);
        } else {
            this.errors = [];
        }
    }

    clearAll() {
        this.errors = [];
    }

    getRecoverySuggestion(error: string): string {
        const errorLower = error.toLowerCase();

        if (errorLower.includes('permission') || errorLower.includes('access denied')) {
            return 'Check file permissions. Try running as administrator or change file ownership.';
        }

        if (errorLower.includes('not found') || errorLower.includes('does not exist')) {
            return 'Verify the file or folder path exists. It may have been moved or deleted.';
        }

        if (errorLower.includes('locked') || errorLower.includes('in use')) {
            return 'Close any programs that might be using this file and try again.';
        }

        if (errorLower.includes('space') || errorLower.includes('disk full')) {
            return 'Free up disk space by deleting unnecessary files or moving files to another drive.';
        }

        if (errorLower.includes('timeout') || errorLower.includes('timed out')) {
            return 'The operation took too long. Try again or check your system resources.';
        }

        if (errorLower.includes('network') || errorLower.includes('connection')) {
            return 'Check your network connection and try again.';
        }

        if (errorLower.includes('invalid') || errorLower.includes('malformed')) {
            return 'Check the input format and try again with valid data.';
        }

        return 'Try the operation again. If the problem persists, restart the application.';
    }
}

export const errorLogStore = new ErrorLogStore();
