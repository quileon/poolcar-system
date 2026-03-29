export class LiveData<T> {
	#socket: WebSocket | null = null;
	#url: string;
	// eslint-disable-next-line svelte/prefer-svelte-reactivity
	#callbacks: Set<(data: T) => void> = new Set();

	isConnected = $state(false);
	error = $state<string | null>(null);

	constructor(url: string) {
		this.#url = url;
	}

	connect() {
		if (this.#socket) return;
		this.#socket = new WebSocket(this.#url);

		this.#socket.onopen = () => {
			this.error = null;
			this.isConnected = true;
		};

		this.#socket.onerror = () => {
			this.error = "WebSocket connection failed";
		};

		this.#socket.onclose = (event) => {
			this.isConnected = false;
			if (!event.wasClean) {
				this.error = `WebSocket closed unexpectedly (code: ${event.code})`;
			}
			this.#socket = null;
		};

		this.#socket.onmessage = (event) => {
			try {
				const data = JSON.parse(event.data) as T;
				this.#callbacks.forEach((cb) => cb(data));
			} catch (e) {
				console.error("Failed to parse WS message", e);
			}
		};
	}

	disconnect() {
		this.#socket?.close();
		this.#socket = null;
	}

	onMessage(callback: (data: T) => void) {
		this.#callbacks.add(callback);
		return () => this.#callbacks.delete(callback);
	}
}
