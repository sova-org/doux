let errors = $state<{ id: number; message: string }[]>([]);
let nextId = 0;

export function pushError(message: string) {
	const id = nextId++;
	errors.push({ id, message });
	setTimeout(() => dismiss(id), 5000);
}

export function dismiss(id: number) {
	const idx = errors.findIndex((e) => e.id === id);
	if (idx !== -1) errors.splice(idx, 1);
}

export function getErrors() {
	return errors;
}
