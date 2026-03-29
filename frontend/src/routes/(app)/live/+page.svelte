<script lang="ts">
	import "leaflet/dist/leaflet.css";
	import { LiveData } from "$lib/hooks/socket.svelte";
	import { LeafletMap } from "$lib/hooks/leaflet-map.svelte";
	import type { WebSocketMessage } from "$lib/bindings/WebSocketMessage";
	import type { MqttPayloadWithId } from "$lib/bindings/MqttPayloadWithId";
	import type { UpdateActivity } from "$lib/bindings/UpdateActivity";
	import type { DeleteActivity } from "$lib/bindings/DeleteActivity";
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

	const initialCoordinates: [number, number] = [-6.382310833, 107.1725405];

	const wsData = new LiveData<WebSocketMessage>(`${config.wsBaseUrl}/live`);
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

	const trackersQuery = useTrackersQuery(() => "active");
	const activitiesQuery = useActivitiesQuery(() => "active");
	const mqttPayloadHistoriesQuery = useMqttPayloadHistoriesQuery();

	let mapElement: HTMLElement;

	let selectedTrackerId = $state<number | null>(null);
	let isFollowing = $state<boolean>(false);

	const trackerTrigger = $derived(
		trackersQuery.data?.trackers.find((tracker) => tracker.tracker_id === selectedTrackerId)
			?.name ?? "Select Tracker to View"
	);

	function isTrackerMarker(message: WebSocketMessage): boolean {
		return message.message_type === "tracker_location";
	}
	function isUpdateActivity(message: WebSocketMessage): boolean {
		return message.message_type === "update_destination";
	}
	function isDeleteActivity(message: WebSocketMessage): boolean {
		return message.message_type === "remove_destination";
	}

	// Leaflet & WebSocket Initialization
	onMount(() => {
		leaflet.init(mapElement, {
			center: initialCoordinates,
			zoom: 13,
			onDragStart: () => {
				isFollowing = false;
			}
		});

		wsData.connect();

		const unsubscribeWS = wsData.onMessage((message) => {
			if (!leaflet.ready) return;

			if (isTrackerMarker(message)) {
				const currentData = message.data as MqttPayloadWithId;
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
					iconSize: [15.5, 23],
					iconAnchor: [7.75, 21],
					popupAnchor: [0, -20]
				});

				const shouldPan = isFollowing && selectedTrackerId === id;

				if (shouldPan) {
					leaflet.upsertTrackerMarkerAndPan(id, latitude, longitude, icon);
				} else {
					leaflet.upsertTrackerMarker(id, latitude, longitude, icon);
				}
			} else if (isUpdateActivity(message)) {
				const activity = message.data as UpdateActivity;
				if (!activity.contact_latitude || !activity.contact_longitude || !activity.contact_name)
					return;
				leaflet.upsertDestinationMarker(
					activity.activity_id,
					activity.contact_latitude,
					activity.contact_longitude,
					leaflet.createIcon({
						iconUrl: destinationMarker,
						iconSize: [15.5, 23],
						iconAnchor: [7.75, 21],
						popupAnchor: [0, -20]
					}),
					activity.contact_name
				);
			} else if (isDeleteActivity(message)) {
				const deleteData = message.data as DeleteActivity;
				leaflet.removeDestinationMarker(deleteData.activity_id);
			}
		});

		return () => {
			leaflet.destroy();
			unsubscribeWS();
			wsData.disconnect();
		};
	});

	// After Leaflet Initialization - Home Marker
	$effect(() => {
		if (!leaflet.ready) return;

		const homeIcon = leaflet.createIcon({
			iconUrl: homeMarker,
			iconSize: [21, 21],
			iconAnchor: [10.5, 10.5],
			popupAnchor: [0, -7.5]
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
				iconSize: [15.5, 23],
				iconAnchor: [7.75, 21],
				popupAnchor: [0, -20]
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
				iconSize: [15.5, 23],
				iconAnchor: [7.75, 21],
				popupAnchor: [0, -20]
			});

			leaflet.upsertDestinationMarker(id, latitude, longitude, icon, contactName);
		});
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

{#if wsData.error || trackersQuery.isError}
	<div class="space-y-4">
		{#if wsData.error}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>Error WebSocket: {wsData.error}</p>
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
