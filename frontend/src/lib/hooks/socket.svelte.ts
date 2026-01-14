import { createSubscriber } from 'svelte/reactivity';

export class LiveData<T> {
	#socket: WebSocket | null = null;
	#url: string;
	#latestData: T | null = null;

	#subscribe: () => void;

	constructor(url: string) {
		this.#url = url;

		this.#subscribe = createSubscriber((update) => {
			this.#socket = new WebSocket(this.#url);

			this.#socket.onopen = () => update(); // Update status

			this.#socket.onmessage = (event) => {
				try {
					this.#latestData = JSON.parse(event.data);
					update(); // TRIGGER REACTIVITY
				} catch (e) {
					console.error("Failed to parse WS message", e);
				}
			};

			return () => {
				this.#socket?.close();
			};
		});
	}

	// The getter the UI will use
	get current() {
		this.#subscribe(); // <--- CRITICAL: Tells Svelte "I am using this!"
		return this.#latestData;
	}

	get isConnected() {
		this.#subscribe();
		return this.#socket?.readyState === WebSocket.OPEN;
	}
}
