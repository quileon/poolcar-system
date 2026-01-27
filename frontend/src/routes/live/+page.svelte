<script lang="ts">
	import type L from "leaflet";
	import "leaflet/dist/leaflet.css";
	import chroma from "chroma-js";
	import { LiveData } from "$lib/hooks/socket.svelte";
	import type { TrackerPayloadWithId } from "$lib/bindings/TrackerPayloadWithId";
	import { config } from "$lib/config";
	import { getCarIcon, getTruckIcon, panToMarker, updateMarker } from "$lib/utils/map";
	import { onMount } from "svelte";
	import * as Select from "$lib/components/ui/select/index";
	import { useSidebar } from "$lib/components/ui/sidebar/context.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import * as Alert from "$lib/components/ui/alert/index";
	import FocusIcon from "@lucide/svelte/icons/focus";
	import * as ButtonGroup from "$lib/components/ui/button-group/index";
	import Button from "$lib/components/ui/button/button.svelte";
	import { useTrackersQuery } from "$lib/hooks/use-reference-queries";

	const initialCoordinates = { lat: -6.382310833, lng: 107.1725405 };
	const trackerData = new LiveData<TrackerPayloadWithId>(`${config.wsBaseUrl}/live`);
	const sidebar = useSidebar();
	const colors = chroma.scale(["#fafa6e", "#2a4a58"]).mode("lch").colors(10);
	const trackersQuery = useTrackersQuery();
	let mapElement: HTMLElement;
	let map: L.Map;
	let L_module: typeof import("leaflet");
	let trackerMarkerList: { [id: number]: L.Marker } = {};
	let focusId = $state<string | undefined>(undefined);
	let focusMap = $state<boolean>(false);
	const focus = $derived({
		map: focusMap,
		id: focusId ? parseInt(focusId, 10) : null
	});

	// Set focusMap to true when focusId changes
	$effect(() => {
		if (focusId !== undefined) {
			focusMap = true;
		}
	});

	// Pan to marker when focusMap becomes true
	$effect(() => {
		if (focusMap && focus.id && map && trackerMarkerList[focus.id]) {
			const marker = trackerMarkerList[focus.id];
			const position = marker.getLatLng();
			map.panTo(position);
		}
	});

	function enableFocus() {
		focusMap = true;
	}

	const trackerTrigger = $derived(
		trackersQuery.data?.trackers.find((tracker) => tracker.tracker_id.toString() === focusId)
			?.name ?? "Select Tracker to View"
	);

	// Map initialization - runs once on mount
	onMount(() => {
		import("leaflet").then((module) => {
			const L = module.default;
			L_module = L;

			const homeIcon = L.icon({
				iconUrl:
					"https://api.geoapify.com/v2/icon/?type=circle&color=%230083ff&size=36&icon=home&iconType=awesome&contentSize=15&scaleFactor=2&apiKey=e0f80f7132454023b038a039b4d8c962",
				iconSize: [42, 42],
				iconAnchor: [21, 21],
				popupAnchor: [0, -15]
			});

			// Map Initialization
			map = L.map(mapElement, { preferCanvas: true }).setView(initialCoordinates, 13);

			L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
				attribution:
					'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
			}).addTo(map);
			L.marker(initialCoordinates, { icon: homeIcon }).addTo(map);

			setTimeout(() => {
				map.invalidateSize();
			}, 100);

			map.on("dragstart", () => {
				focusMap = false;
			});
		});

		// Cleanup on unmount
		return () => {
			if (map) {
				map.remove();
			}
		};
	});

	// Invalidate map size when sidebar state changes
	$effect(() => {
		// Track sidebar state to make this reactive
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		const sidebarState = sidebar.state;

		if (map) {
			setTimeout(() => {
				map.invalidateSize();
			}, 300);
		}
	});

	// WebSocket
	$effect(() => {
		// Clone `trackerData.current`
		const currentData = { ...trackerData.current };

		if (!currentData || !map || !L_module) {
			console.warn("Exiting early: missing deps");
			return;
		}
		if (!currentData.location) {
			console.warn("Exiting early: no location");
			return;
		}
		if (
			!currentData.location.longitude ||
			!currentData.location.latitude ||
			!currentData.id ||
			!trackersQuery.data
		) {
			console.warn("Exiting early: missing location data or query", {
				lng: currentData.location.longitude,
				lat: currentData.location.latitude,
				id: currentData.id,
				queryData: !!trackersQuery.data
			});
			return;
		}

		const tracker = trackerMarkerList[currentData.id];
		const trackerDetails = trackersQuery.data.trackers.find((t) => t.tracker_id === currentData.id);

		if (!trackerDetails) {
			console.log("Exiting early: no tracker details");
			return;
		}

		const icon =
			trackerDetails.car_type_name === "Passenger"
				? getCarIcon(L_module, colors, currentData.id)
				: trackerDetails.car_type_name === "Truck"
					? getTruckIcon(L_module, colors, currentData.id)
					: getCarIcon(L_module, colors, currentData.id); // Default to car icon

		if (tracker) {
			// Update existing marker
			if (focus.map && focus.id === currentData.id) {
				panToMarker(map, tracker, currentData.location.latitude, currentData.location.longitude);
			} else {
				updateMarker(tracker, currentData.location.latitude, currentData.location.longitude);
			}
		} else {
			const marker = L_module.marker(
				[currentData.location.latitude, currentData.location.longitude],
				{ icon }
			);
			trackerMarkerList[currentData.id] = marker;
			marker.addTo(map);
		}
	});
</script>

<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Live Tracking</h1>

<div class="z-100">
	<ButtonGroup.Root>
		<ButtonGroup.Root>
			<Select.Root type="single" bind:value={focusId}>
				<Select.Trigger class="w-75">{trackerTrigger}</Select.Trigger>
				<Select.Content>
					<Select.Group>
						<Select.Label>Trackers</Select.Label>
						{#each trackersQuery.data?.trackers as tracker (tracker.tracker_id)}
							<Select.Item value={tracker.tracker_id.toString()}>
								{tracker.name}
								{#if tracker.car_type_name}({tracker.car_name}){/if}
							</Select.Item>
						{/each}
					</Select.Group>
				</Select.Content>
			</Select.Root>
		</ButtonGroup.Root>
		<ButtonGroup.Root>
			<Button
				aria-label="Focus"
				size="icon"
				variant={focusMap ? "default" : "outline"}
				disabled={!focus.id}
				onclick={enableFocus}
			>
				<FocusIcon />
			</Button>
		</ButtonGroup.Root>
	</ButtonGroup.Root>
</div>

{#if trackerData.error || trackersQuery.isError}
	<div class="space-y-4">
		{#if trackerData.error}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{trackerData.error}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if trackersQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{trackersQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
{/if}

<div bind:this={mapElement} class="map h-[calc(100vh-10rem)] w-full"></div>

<style>
	:global(.leaflet-container) {
		height: 100%;
		width: 100%;
		z-index: 0;
	}
</style>
