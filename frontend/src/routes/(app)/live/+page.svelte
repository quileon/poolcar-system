<script lang="ts">
	import "leaflet/dist/leaflet.css";
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
	import { useMqttPayloadHistoriesQuery } from "$lib/hooks/use-mqtt-payload-history";
	import { useTrackersQuery } from "$lib/hooks/use-tracker";
	import { useActivitiesQuery } from "$lib/hooks/use-activity";
	import homeMarker from "$lib/assets/home.png";
	import destinationMarker from "$lib/assets/flag.png";
	import type { ActivityMarker } from "$lib/bindings/ActivityMarker";

	const initialCoordinates: [number, number] = [-6.382310833, 107.1725405];

	const trackerData = new LiveData<MqttPayloadWithId | ActivityMarker>(`${config.wsBaseUrl}/live`);
	const leaflet = new LeafletMap();
	const sidebar = useSidebar();
	const colors = [
		"fafa6e",
		"c6ec73",
		"96db7c",
		"6ac985",
		"42b58b",
		"2a4a58",
		"225f6d",
		"1da08c",
		"11747d",
		"028a87"
	];

	const trackersQuery = useTrackersQuery();
	const activitiesQuery = useActivitiesQuery(() => "active");
	const mqttPayloadHistoriesQuery = useMqttPayloadHistoriesQuery();

	let mapElement: HTMLElement;

	let selectedTrackerId = $state<number | null>(null);
	let isFollowing = $state<boolean>(false);

	const trackerTrigger = $derived(
		trackersQuery.data?.trackers.find((tracker) => tracker.tracker_id === selectedTrackerId)
			?.name ?? "Select Tracker to View"
	);

	function isTrackerMarker(data: MqttPayloadWithId | ActivityMarker): data is MqttPayloadWithId {
		return "location" in data && "uptime" in data;
	}
	function isDestinationMarker(data: MqttPayloadWithId | ActivityMarker): data is ActivityMarker {
		return "action" in data;
	}

	// Leaflet Initialization
	onMount(() => {
		leaflet.init(mapElement, {
			center: initialCoordinates,
			zoom: 13,
			onDragStart: () => {
				isFollowing = false;
			}
		});

		return () => leaflet.destroy();
	});

	// After Leaflet Initialization - Home Marker
	$effect(() => {
		if (!leaflet.ready) return;

		const homeIcon = leaflet.createIcon({
			iconUrl: homeMarker,
			iconSize: [42, 42],
			iconAnchor: [21, 21],
			popupAnchor: [0, -15]
		});
		leaflet.addStaticMarker(initialCoordinates[0], initialCoordinates[1], homeIcon);
	});

	// After Leaflet Initialization - Focus on Selected Tracker
	$effect(() => {
		if (!leaflet.ready) return;
		if (!selectedTrackerId) return;

		const marker = leaflet.getTrackerMarker(selectedTrackerId);
		if (marker) {
			const markerPosition = marker.getLatLng();
			leaflet.panTo(markerPosition.lat, markerPosition.lng);
		}
	});

	// After Leaflet Initialization - Sidebar Resize Handling
	// Track sidebar state to re-run on change
	$effect(() => {
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		const _sidebarState = sidebar.state;

		if (leaflet.ready) {
			setTimeout(() => leaflet.invalidateSize(), 300);
		}
	});

	// After Leaflet Initialization - Initial Tracker Marker
	$effect(() => {
		if (!leaflet.ready) return;
		if (!trackersQuery.data) return;
		if (!mqttPayloadHistoriesQuery.data) return;

		mqttPayloadHistoriesQuery.data.forEach((mqttPayload) => {
			const id = mqttPayload.id;
			const latitude = mqttPayload.location?.latitude;
			const longitude = mqttPayload.location?.longitude;
			if (!id || !latitude || !longitude) return;

			const trackerDetails = trackersQuery.data!.trackers.find((t) => t.tracker_id === id);
			if (!trackerDetails) return;

			const iconColor = colors[id % colors.length];
			const iconName = trackerDetails.car_type_name === "Truck" ? "truck" : "car";
			const icon = leaflet.createIcon({
				iconUrl: new URL(`/src/lib/assets/${iconName}-${iconColor}.png`, import.meta.url).href,
				iconSize: [31, 46],
				iconAnchor: [15.5, 42],
				popupAnchor: [0, -40]
			});

			leaflet.upsertTrackerMarker(
				id,
				latitude,
				longitude,
				icon,
				trackerDetails.car_name
					? `Tracker ${trackerDetails.name} (${trackerDetails.car_name})`
					: `Tracker ${trackerDetails.name}`
			);
		});
	});

	// After Leaflet Initialization - Activities Marker
	$effect(() => {
		if (!leaflet.ready) return;
		if (!activitiesQuery.data) return;

		const filteredActivities = activitiesQuery.data.activities.filter(
			(activity) =>
				activity.finished_at === null &&
				activity.finished_latitude === null &&
				activity.finished_longitude === null
		);

		filteredActivities.forEach((activity) => {
			const id = activity.activity_id;
			const latitude = activity.contact_latitude;
			const longitude = activity.contact_longitude;
			const contactName = activity.contact_name;

			const icon = leaflet.createIcon({
				iconUrl: destinationMarker,
				iconSize: [31, 46],
				iconAnchor: [15.5, 42],
				popupAnchor: [0, -40]
			});

			leaflet.upsertDestinationMarker(id, latitude, longitude, icon, contactName);
		});
	});

	// After Leaflet Initialization - WebSocket
	$effect(() => {
		if (!leaflet.ready) return;
		if (!trackerData.current) return;
		const currentData = trackerData.current;

		if (isTrackerMarker(currentData)) {
			if (
				!currentData?.location?.latitude ||
				!currentData?.location?.longitude ||
				!currentData?.id
			) {
				return;
			}
			if (!trackersQuery.data) return;

			const id = currentData.id;
			const latitude = currentData.location.latitude;
			const longitude = currentData.location.longitude;
			if (!id || !latitude || !longitude) return;

			const trackerDetails = trackersQuery.data.trackers.find(
				(tracker) => tracker.tracker_id === id
			);
			if (!trackerDetails) return;

			const iconColor = colors[id % colors.length];
			const iconName = trackerDetails.car_type_name === "Truck" ? "truck" : "car";
			const icon = leaflet.createIcon({
				iconUrl: new URL(`/src/lib/assets/${iconName}-${iconColor}.png`, import.meta.url).href,
				iconSize: [31, 46],
				iconAnchor: [15.5, 42],
				popupAnchor: [0, -40]
			});

			const shouldPan = isFollowing && selectedTrackerId === id;

			if (shouldPan) {
				leaflet.upsertTrackerMarkerAndPan(id, latitude, longitude, icon);
			} else {
				leaflet.upsertTrackerMarker(id, latitude, longitude, icon);
			}
		} else if (isDestinationMarker(currentData)) {
			if (currentData.action === "DELETE") {
				leaflet.removeDestinationMarker(currentData.id);
			} else if (currentData.action === "POST") {
				if (!currentData.latitude || !currentData.longitude || !currentData.name) return;
				leaflet.upsertDestinationMarker(
					currentData.id,
					currentData.latitude,
					currentData.longitude,
					leaflet.createIcon({
						iconUrl: destinationMarker,
						iconSize: [31, 46],
						iconAnchor: [15.5, 42],
						popupAnchor: [0, -40]
					}),
					currentData.name
				);
			} else if (currentData.action === "PUT") {
				if (!currentData.latitude || !currentData.longitude || !currentData.name) return;
				leaflet.upsertDestinationMarker(
					currentData.id,
					currentData.latitude,
					currentData.longitude,
					leaflet.createIcon({
						iconUrl: destinationMarker,
						iconSize: [31, 46],
						iconAnchor: [15.5, 42],
						popupAnchor: [0, -40]
					}),
					currentData.name
				);
			}
		}
	});
</script>

<h2 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Live Tracking</h2>

<div class="z-100">
	<ButtonGroup.Root>
		<ButtonGroup.Root>
			<Select.Root
				type="single"
				value={selectedTrackerId?.toString()}
				onValueChange={(v) => {
					selectedTrackerId = v ? parseInt(v, 10) : null;
					isFollowing = true;
				}}
			>
				<Select.Trigger class="w-75">{trackerTrigger}</Select.Trigger>
				<Select.Content>
					<Select.Group>
						<Select.Label>Trackers</Select.Label>
						{#each trackersQuery.data?.trackers ?? [] as tracker (tracker.tracker_id)}
							<Select.Item
								value={tracker.tracker_id.toString()}
								disabled={!leaflet.hasTrackerMarker(tracker.tracker_id)}
							>
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
				variant={isFollowing ? "outline" : "default"}
				disabled={!selectedTrackerId}
				onclick={() => {
					isFollowing = true;
					if (selectedTrackerId) {
						const marker = leaflet.getTrackerMarker(selectedTrackerId);
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
					<p>Error WebSocket: {trackerData.error}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if trackersQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>Error getting Trackers: {trackersQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if activitiesQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>Error getting Activities: {activitiesQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if mqttPayloadHistoriesQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>Error getting MQTT Payload History: {mqttPayloadHistoriesQuery.error.message}</p>
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
