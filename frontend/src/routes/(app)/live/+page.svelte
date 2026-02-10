<script lang="ts">
	import "leaflet/dist/leaflet.css";
	import chroma from "chroma-js";
	import { LiveData } from "$lib/hooks/socket.svelte";
	import { LeafletMap } from "$lib/hooks/leaflet-map.svelte";
	import type { MqttPayloadWithId } from "$lib/bindings/MqttPayloadWithId";
	import { config } from "$lib/config";
	import { onMount } from "svelte";
	import * as Select from "$lib/components/ui/select/index";
	import { useSidebar } from "$lib/components/ui/sidebar/context.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import * as Alert from "$lib/components/ui/alert/index";
	import FocusIcon from "@lucide/svelte/icons/focus";
	import * as ButtonGroup from "$lib/components/ui/button-group/index";
	import Button from "$lib/components/ui/button/button.svelte";
	import { useTrackersQuery } from "$lib/hooks/use-reference-queries";

	const GEOAPIFY_API_KEY = "e0f80f7132454023b038a039b4d8c962";
	const initialCoordinates: [number, number] = [-6.382310833, 107.1725405];

	const trackerData = new LiveData<MqttPayloadWithId>(`${config.wsBaseUrl}/live`);
	const leaflet = new LeafletMap();
	const sidebar = useSidebar();
	const colors = chroma.scale(["#fafa6e", "#2a4a58"]).mode("lch").colors(10);

	const trackersQuery = useTrackersQuery();

	let mapElement: HTMLElement;
	let focusId = $state<string | undefined>(undefined);
	let focusMap = $state<boolean>(false);

	const focusParsedId = $derived(focusId ? parseInt(focusId, 10) : null);

	const trackerTrigger = $derived(
		trackersQuery.data?.trackers.find((tracker) => tracker.tracker_id.toString() === focusId)
			?.name ?? "Select Tracker to View"
	);

	// Auto-enable focus when a tracker is selected
	$effect(() => {
		if (focusId !== undefined) {
			focusMap = true;
		}
	});

	// Leaflet Initialization
	onMount(() => {
		leaflet.init(mapElement, {
			center: initialCoordinates,
			zoom: 13,
			onDragStart: () => {
				focusMap = false;
			}
		});

		return () => leaflet.destroy();
	});

	// After Leaflet Initialization - Home Marker
	$effect(() => {
		if (!leaflet.ready) return;

		const homeIcon = leaflet.createIcon({
			iconUrl: `https://api.geoapify.com/v2/icon/?type=circle&color=%230083ff&size=36&icon=home&iconType=awesome&contentSize=15&scaleFactor=2&apiKey=${GEOAPIFY_API_KEY}`,
			iconSize: [42, 42],
			iconAnchor: [21, 21],
			popupAnchor: [0, -15]
		});
		leaflet.addStaticMarker(initialCoordinates[0], initialCoordinates[1], homeIcon);
	});

	// After Leaflet Initialization - Sidebar Resize Handling
	$effect(() => {
		// Track sidebar state to re-run on change
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		const _sidebarState = sidebar.state;

		if (leaflet.ready) {
			setTimeout(() => leaflet.invalidateSize(), 300);
		}
	});

	// After Leaflet Initialization - WebSocket
	$effect(() => {
		if (!leaflet.ready) return;

		const currentData = trackerData.current;
		if (!currentData?.location?.latitude || !currentData?.location?.longitude || !currentData?.id) {
			return;
		}
		if (!trackersQuery.data) return;

		const id = currentData.id;
		const latitude = currentData.location.latitude;
		const longitude = currentData.location.longitude;
		if (!id || !latitude || !longitude) return;

		const trackerDetails = trackersQuery.data.trackers.find((t) => t.tracker_id === id);
		if (!trackerDetails) return;

		const iconColor = colors[id % colors.length]?.replace("#", "%23");
		const iconName = trackerDetails.car_type_name === "Truck" ? "truck" : "car";
		const icon = leaflet.createIcon({
			iconUrl: `https://api.geoapify.com/v2/icon/?type=material&color=${iconColor}&size=42&icon=${iconName}&iconType=awesome&contentSize=15&scaleFactor=2&apiKey=${GEOAPIFY_API_KEY}`,
			iconSize: [31, 46],
			iconAnchor: [15.5, 42],
			popupAnchor: [0, -40]
		});

		const shouldPan = focusMap && focusParsedId === id;

		if (shouldPan) {
			leaflet.upsertMarkerAndPan(id, latitude, longitude, icon);
		} else {
			leaflet.upsertMarker(id, latitude, longitude, icon);
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
						{#each trackersQuery.data?.trackers ?? [] as tracker (tracker.tracker_id)}
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
				disabled={!focusParsedId}
				onclick={() => {
					focusMap = true;
					if (focusParsedId) {
						const marker = leaflet.getMarker(focusParsedId);
						if (marker) {
							const pos = marker.getLatLng();
							leaflet.panTo(pos.lat, pos.lng);
						}
					}
				}}
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
