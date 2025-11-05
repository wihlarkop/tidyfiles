export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
    id: string;
    message: string;
    type: ToastType;
    duration?: number;
}

class ToastStore {
    private toasts = $state<Toast[]>([]);

    get all() {
        return this.toasts;
    }

    show(message: string, type: ToastType = 'info', duration: number = 5000) {
        const id = crypto.randomUUID();
        const toast: Toast = {id, message, type, duration};

        this.toasts = [...this.toasts, toast];

        if (duration > 0) {
            setTimeout(() => {
                this.remove(id);
            }, duration);
        }
    }

    success(message: string, duration?: number) {
        this.show(message, 'success', duration);
    }

    error(message: string, duration?: number) {
        this.show(message, 'error', duration || 7000); // Errors stay longer
    }

    warning(message: string, duration?: number) {
        this.show(message, 'warning', duration);
    }

    info(message: string, duration?: number) {
        this.show(message, 'info', duration);
    }

    remove(id: string) {
        this.toasts = this.toasts.filter(t => t.id !== id);
    }

    clear() {
        this.toasts = [];
    }
}

export const toastStore = new ToastStore();
