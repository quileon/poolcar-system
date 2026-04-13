import type L from "leaflet";

type MarkerEntry = {
	marker: L.Marker;
	id: number;
};

type PolylineEntry = {
	polyline: L.Polyline;
	id: string;
};

type LeafletMapOptions = {
	center: [number, number];
	zoom: number;
	onDragStart?: () => void;
	onMapClick?: (lat: number, lng: number) => void;
};

export class LeafletMap {
	#map: L.Map | null = $state(null);
	#L: typeof import("leaflet") | null = null;
	#ready: boolean = $state(false);

	// Uses Map instead of SvelteMap because the markers itself aren't being shown to the page.
	// Inserting and deleting SvelteMap would also reruns `effect`
	// eslint-disable-next-line svelte/prefer-svelte-reactivity
	#trackerMarkers: Map<number, MarkerEntry> = new Map();
	// eslint-disable-next-line svelte/prefer-svelte-reactivity
	#destinationMarkers: Map<number, MarkerEntry> = new Map();
	// eslint-disable-next-line svelte/prefer-svelte-reactivity
	#auditMarkers: Map<string, MarkerEntry> = new Map();
	// eslint-disable-next-line svelte/prefer-svelte-reactivity
	#polylines: Map<string, PolylineEntry> = new Map();

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

		if (options.onMapClick) {
			this.#map.on("click", (e: L.LeafletMouseEvent) => {
				options.onMapClick?.(e.latlng.lat, e.latlng.lng);
			});
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
	upsertTrackerMarker(
		id: number,
		lat: number,
		lng: number,
		icon?: L.Icon,
		text?: string
	): L.Marker {
		if (!this.#L || !this.#map) throw new Error("LeafletMap not initialized");

		const existing = this.#trackerMarkers.get(id);

		if (existing) {
			const newLatLng: L.LatLngExpression = [lat, lng];
			existing.marker.setLatLng(newLatLng);
			return existing.marker;
		}

		const marker = this.#L.marker([lat, lng], icon ? { icon } : {});
		marker.addTo(this.#map);
		if (text) {
			marker.bindPopup(text);
		}
		this.#trackerMarkers.set(id, { marker, id });
		return marker;
	}

	/**
	 * Updates a tracked marker's position and pans the map to follow it.
	 */
	upsertTrackerMarkerAndPan(id: number, lat: number, lng: number, icon?: L.Icon): L.Marker {
		const marker = this.upsertTrackerMarker(id, lat, lng, icon);

		if (this.#map) {
			this.#map.panTo([lat, lng]);
		}

		return marker;
	}

	/**
	 * Removes a destination marker by ID.
	 */
	removeTrackerMarker(id: number): void {
		const existing = this.#trackerMarkers.get(id);
		if (existing && this.#map) {
			this.#map.removeLayer(existing.marker);
			this.#trackerMarkers.delete(id);
		}
	}

	/**
	 * Returns true if a tracked marker with the given ID exists.
	 */
	hasTrackerMarker(id: number): boolean {
		return this.#trackerMarkers.has(id);
	}

	/**
	 * Gets a tracked marker by ID, or undefined if it doesn't exist.
	 */
	getTrackerMarker(id: number): L.Marker | undefined {
		return this.#trackerMarkers.get(id)?.marker;
	}

	/**
	 * Adds or updates a destination marker by ID.
	 * - If the marker doesn't exist, it's created and added to the map.
	 * - If it exists, its position is updated.
	 *
	 * Returns the marker instance.
	 */
	upsertDestinationMarker(
		id: number,
		lat: number,
		lng: number,
		icon?: L.Icon,
		text?: string
	): L.Marker {
		if (!this.#L || !this.#map) throw new Error("LeafletMap not initialized");

		const existing = this.#destinationMarkers.get(id);

		if (existing) {
			const newLatLng: L.LatLngExpression = [lat, lng];
			existing.marker.setLatLng(newLatLng);
			return existing.marker;
		}

		const marker = this.#L.marker([lat, lng], icon ? { icon } : {});
		marker.addTo(this.#map);
		if (text) {
			marker.bindPopup(text);
		}
		this.#destinationMarkers.set(id, { marker, id });
		return marker;
	}

	/**
	 * Updates a destination marker's position and pans the map to follow it.
	 */
	upsertDestinationMarkerAndPan(id: number, lat: number, lng: number, icon?: L.Icon): L.Marker {
		const marker = this.upsertDestinationMarker(id, lat, lng, icon);

		if (this.#map) {
			this.#map.panTo([lat, lng]);
		}

		return marker;
	}

	/**
	 * Removes a destination marker by ID.
	 */
	removeDestinationMarker(id: number): void {
		const existing = this.#destinationMarkers.get(id);
		if (existing && this.#map) {
			this.#map.removeLayer(existing.marker);
			this.#destinationMarkers.delete(id);
		}
	}

	/**
	 * Returns true if a destination marker with the given ID exists.
	 */
	hasDestinationMarker(id: number): boolean {
		return this.#destinationMarkers.has(id);
	}

	/**
	 * Gets a destination marker by ID, or undefined if it doesn't exist.
	 */
	getDestinationMarker(id: number): L.Marker | undefined {
		return this.#destinationMarkers.get(id)?.marker;
	}

	/**
	 * Adds an audit marker by string ID (e.g., "audit_123" or "tracker_1_record_5").
	 * Useful for displaying individual audit records on the map.
	 */
	addAuditMarker(id: string, lat: number, lng: number, icon?: L.Icon, text?: string): L.Marker {
		if (!this.#L || !this.#map) throw new Error("LeafletMap not initialized");

		const existing = this.#auditMarkers.get(id);
		if (existing) {
			return existing.marker;
		}

		const marker = this.#L.marker([lat, lng], icon ? { icon } : {});
		marker.addTo(this.#map);
		if (text) {
			marker.bindPopup(text);
		}
		this.#auditMarkers.set(id, { marker, id: parseInt(id) });
		return marker;
	}

	/**
	 * Removes an audit marker by ID.
	 */
	removeAuditMarker(id: string): void {
		const existing = this.#auditMarkers.get(id);
		if (existing && this.#map) {
			this.#map.removeLayer(existing.marker);
			this.#auditMarkers.delete(id);
		}
	}

	/**
	 * Clears all audit markers.
	 */
	clearAuditMarkers(): void {
		this.#auditMarkers.forEach((entry) => {
			if (this.#map) {
				this.#map.removeLayer(entry.marker);
			}
		});
		this.#auditMarkers.clear();
	}

	/**
	 * Gets an audit marker by ID, or undefined if it doesn't exist.
	 */
	getAuditMarker(id: string): L.Marker | undefined {
		return this.#auditMarkers.get(id)?.marker;
	}

	/**
	 * Adds a polyline connecting multiple coordinates.
	 * Useful for visualizing routes or audit trails.
	 *
	 * @param id Unique identifier for the polyline
	 * @param coordinates Array of [lat, lng] coordinates
	 * @param options Optional polyline styling options (color, weight, opacity, etc.)
	 */
	addPolyline(
		id: string,
		coordinates: [number, number][],
		options?: L.PolylineOptions
	): L.Polyline {
		if (!this.#L || !this.#map) throw new Error("LeafletMap not initialized");

		const existing = this.#polylines.get(id);
		if (existing) {
			return existing.polyline;
		}

		const defaultOptions: L.PolylineOptions = {
			color: "#3b82f6",
			weight: 3,
			opacity: 0.7,
			...options
		};

		const polyline = this.#L.polyline(coordinates, defaultOptions);
		polyline.addTo(this.#map);
		this.#polylines.set(id, { polyline, id });
		return polyline;
	}

	/**
	 * Updates an existing polyline's coordinates.
	 * If the polyline doesn't exist, it's created.
	 */
	upsertPolyline(
		id: string,
		coordinates: [number, number][],
		options?: L.PolylineOptions
	): L.Polyline {
		const existing = this.#polylines.get(id);
		if (existing) {
			existing.polyline.setLatLngs(coordinates);
			return existing.polyline;
		}
		return this.addPolyline(id, coordinates, options);
	}

	/**
	 * Removes a polyline by ID.
	 */
	removePolyline(id: string): void {
		const existing = this.#polylines.get(id);
		if (existing && this.#map) {
			this.#map.removeLayer(existing.polyline);
			this.#polylines.delete(id);
		}
	}

	/**
	 * Clears all polylines.
	 */
	clearPolylines(): void {
		this.#polylines.forEach((entry) => {
			if (this.#map) {
				this.#map.removeLayer(entry.polyline);
			}
		});
		this.#polylines.clear();
	}

	/**
	 * Clears all audit-related elements (markers and polylines).
	 * Useful when changing the tracker/car filter.
	 */
	clearAuditVisualization(): void {
		this.clearAuditMarkers();
		this.clearPolylines();
	}

	/**
	 * Registers a callback for map click events.
	 * The callback receives the latitude and longitude of the clicked point.
	 */
	registerMapClickHandler(callback: (lat: number, lng: number) => void): void {
		if (!this.#map) throw new Error("LeafletMap not initialized");
		this.#map.on("click", (e: L.LeafletMouseEvent) => {
			callback(e.latlng.lat, e.latlng.lng);
		});
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
		this.#trackerMarkers.clear();
		this.#destinationMarkers.clear();
		this.#auditMarkers.clear();
		this.#polylines.clear();
		if (this.#map) {
			this.#map.remove();
			this.#map = null;
		}
		this.#L = null;
		this.#ready = false;
	}
}
