import type L from "leaflet";
import { SvelteMap } from "svelte/reactivity";

type MarkerEntry = {
	marker: L.Marker;
	id: number;
};

type LeafletMapOptions = {
	center: [number, number];
	zoom: number;
	onDragStart?: () => void;
};

export class LeafletMap {
	#map: L.Map | null = $state(null);
	#L: typeof import("leaflet") | null = null;
	#markers: Map<number, MarkerEntry> = new SvelteMap();
	#ready: boolean = $state(false);

	get ready() {
		return this.#ready;
	}

	get map() {
		return this.#map;
	}

	/**
	 * Dynamically imports Leaflet, creates the map, adds the tile layer,
	 * and marks the instance as ready. Call this inside `onMount`.
	 */
	async init(element: HTMLElement, options: LeafletMapOptions): Promise<void> {
		const module = await import("leaflet");
		const L = module.default;
		this.#L = L;

		this.#map = L.map(element, { preferCanvas: true }).setView(options.center, options.zoom);

		L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
			attribution:
				'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
		}).addTo(this.#map);

		if (options.onDragStart) {
			this.#map.on("dragstart", options.onDragStart);
		}

		// Give the map a tick to settle in the DOM before marking ready
		setTimeout(() => {
			this.#map?.invalidateSize();
		}, 100);

		this.#ready = true;
	}

	/**
	 * Creates an icon using the loaded Leaflet module.
	 * Only call after `ready` is true.
	 */
	createIcon(options: L.IconOptions): L.Icon {
		if (!this.#L) throw new Error("LeafletMap not initialized");
		return this.#L.icon(options);
	}

	/**
	 * Adds a standalone marker (e.g. a home marker) that is not tracked by ID.
	 */
	addStaticMarker(lat: number, lng: number, icon?: L.Icon): L.Marker {
		if (!this.#L || !this.#map) throw new Error("LeafletMap not initialized");
		const marker = this.#L.marker([lat, lng], icon ? { icon } : {});
		marker.addTo(this.#map);
		return marker;
	}

	/**
	 * Adds or updates a tracked marker by ID.
	 * - If the marker doesn't exist, it's created and added to the map.
	 * - If it exists, its position is updated.
	 *
	 * Returns the marker instance.
	 */
	upsertMarker(id: number, lat: number, lng: number, icon?: L.Icon): L.Marker {
		if (!this.#L || !this.#map) throw new Error("LeafletMap not initialized");

		const existing = this.#markers.get(id);

		if (existing) {
			const newLatLng: L.LatLngExpression = [lat, lng];
			existing.marker.setLatLng(newLatLng);
			return existing.marker;
		}

		const marker = this.#L.marker([lat, lng], icon ? { icon } : {});
		marker.addTo(this.#map);
		this.#markers.set(id, { marker, id });
		return marker;
	}

	/**
	 * Updates a tracked marker's position and pans the map to follow it.
	 */
	upsertMarkerAndPan(id: number, lat: number, lng: number, icon?: L.Icon): L.Marker {
		const marker = this.upsertMarker(id, lat, lng, icon);

		if (this.#map) {
			this.#map.panTo([lat, lng]);
		}

		return marker;
	}

	/**
	 * Returns true if a tracked marker with the given ID exists.
	 */
	hasMarker(id: number): boolean {
		return this.#markers.has(id);
	}

	/**
	 * Gets a tracked marker by ID, or undefined if it doesn't exist.
	 */
	getMarker(id: number): L.Marker | undefined {
		return this.#markers.get(id)?.marker;
	}

	/**
	 * Pans the map to the given coordinates.
	 */
	panTo(lat: number, lng: number): void {
		this.#map?.panTo([lat, lng]);
	}

	/**
	 * Invalidates the map size (e.g. after a container resize).
	 */
	invalidateSize(): void {
		this.#map?.invalidateSize();
	}

	/**
	 * Removes the map and cleans up all resources. Call in your
	 * `onMount` cleanup or `onDestroy`.
	 */
	destroy(): void {
		this.#markers.clear();
		if (this.#map) {
			this.#map.remove();
			this.#map = null;
		}
		this.#L = null;
		this.#ready = false;
	}
}
