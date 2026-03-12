import { createSubscriber } from "svelte/reactivity";

export class LiveData<T> {
	#socket: WebSocket | null = null;
	#url: string;
	#latestData: T | null = null;
	#error: string | null = null;

	#subscribe: () => void;

	constructor(url: string) {
		this.#url = url;

		this.#subscribe = createSubscriber((update) => {
			this.#socket = new WebSocket(this.#url);

			this.#socket.onopen = () => {
				this.#error = null;
				update();
			};

			this.#socket.onerror = () => {
				this.#error = "WebSocket connection failed";
				update();
			};

			this.#socket.onclose = (event) => {
				if (!event.wasClean) {
					this.#error = `WebSocket closed unexpectedly (code: ${event.code})`;
					update();
				}
			};

			this.#socket.onmessage = (event) => {
				try {
					this.#latestData = JSON.parse(event.data) as T;
					update();
				} catch (e) {
					console.error("Failed to parse WS message", e);
				}
			};

			return () => {
				this.#socket?.close();
			};
		});
	}

	get current() {
		this.#subscribe();
		return this.#latestData;
	}

	get isConnected() {
		this.#subscribe();
		return this.#socket?.readyState === WebSocket.OPEN;
	}

	get error() {
		this.#subscribe();
		return this.#error;
	}
}
