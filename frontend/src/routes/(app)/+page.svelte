<script lang="ts">
	import "leaflet/dist/leaflet.css";
	import * as Card from "$lib/components/ui/card/index";
	import * as Chart from "$lib/components/ui/chart/index";
	import * as Table from "$lib/components/ui/table/index";
	import { config } from "$lib/config";
	import { LeafletMap } from "$lib/hooks/leaflet-map.svelte";
	import { LiveData } from "$lib/hooks/socket.svelte";
	import { onMount } from "svelte";
	import { useActivitiesQuery } from "$lib/hooks/use-activity";
	import { useMqttPayloadHistoriesQuery } from "$lib/hooks/use-mqtt-payload-history";
	import { useSidebar } from "$lib/components/ui/sidebar";
	import { useTrackersQuery } from "$lib/hooks/use-tracker";
	import homeMarker from "$lib/assets/home.png";
	import destinationMarker from "$lib/assets/flag.png";
	import type { WebSocketMessage } from "$lib/bindings/WebSocketMessage";
	import type { MqttPayloadWithId } from "$lib/bindings/MqttPayloadWithId";
	import { AreaChart } from "layerchart";
	import { LatencyChart } from "$lib/hooks/chart.svelte";
	import type { MqttPayloadWithTrackerCar } from "$lib/bindings/MqttPayloadWithTrackerCar";
	import type { UpdateActivity } from "$lib/bindings/UpdateActivity";
	import type { DeleteActivity } from "$lib/bindings/DeleteActivity";
	import type { Distances } from "$lib/bindings/Distances";
	import { SvelteMap } from "svelte/reactivity";
	import { scaleTime } from "d3-scale";

	const trackersQuery = useTrackersQuery(() => "active");
	const activitiesQuery = useActivitiesQuery(() => "active");
	const mqttPayloadHistoriesQuery = useMqttPayloadHistoriesQuery();

	let mapElement: HTMLElement;
	let distancesMap = new SvelteMap<number, Distances>();
	const initialCoordinates: [number, number] = [-6.382310833, 107.1725405];
	const wsData = new LiveData<WebSocketMessage>(`${config.wsBaseUrl}/live`);
	const latencyChart = new LatencyChart();
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

	const chartConfig = $derived.by(() => {
		const config: Chart.ChartConfig = {
			time: {
				label: "Time"
			}
		};
		if (trackersQuery.data) {
			trackersQuery.data.trackers.forEach((tracker) => {
				const color = colors[tracker.tracker_id % colors.length];
				config[tracker.tracker_id.toString()] = {
					label: tracker.name,
					color: `#${color}`
				};
			});
		}
		return config;
	});

	const chartSeries = $derived.by(() => {
		if (!trackersQuery.data) return [];
		return trackersQuery.data.trackers.map((tracker) => {
			return {
				key: tracker.tracker_id.toString(),
				label: tracker.name,
				color: chartConfig[tracker.tracker_id.toString()].color
			};
		});
	});

	function isTrackerMarker(message: WebSocketMessage): boolean {
		return message.message_type === "tracker_location";
	}
	function isUpdateActivity(message: WebSocketMessage): boolean {
		return message.message_type === "update_destination";
	}
	function isDeleteActivity(message: WebSocketMessage): boolean {
		return message.message_type === "remove_destination";
	}
	function isDistances(message: WebSocketMessage): boolean {
		return message.message_type === "distances";
	}
	function isAudit(message: WebSocketMessage): boolean {
		return message.message_type === "audit";
	}

	// Leaflet & WebSocket Initialization
	onMount(() => {
		leaflet.init(mapElement, {
			center: initialCoordinates,
			zoom: 12
		});

		wsData.connect();

		const unsubscribeWS = wsData.onMessage((message) => {
			console.log(message);
			if (!leaflet.ready) return;

			if (isTrackerMarker(message)) {
				const currentData = message.data as MqttPayloadWithId;
				if (!currentData.location.latitude || !currentData.location.longitude || !currentData.id)
					return;
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

				leaflet.upsertTrackerMarker(
					id,
					latitude,
					longitude,
					icon,
					trackerDetails.car_name
						? `Tracker ${trackerDetails.name} (${trackerDetails.car_name})`
						: `Tracker ${trackerDetails.name}`
				);
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
				distancesMap.delete(deleteData.activity_id);
			} else if (isDistances(message)) {
				const distances = message.data as Record<string, Distances>;
				Object.entries(distances).forEach(([activity_id, distance]) => {
					distancesMap.set(Number(activity_id), distance);
				});
			} else if (isAudit(message)) {
				const auditData = message.data as Record<string, MqttPayloadWithTrackerCar | null>;
				latencyChart.addAuditData(auditData);
			}
		});

		return () => {
			leaflet.destroy();
			unsubscribeWS();
			wsData.disconnect();
		};
	});

	// After Leaflet Initialization - Sidebar Resize Handling
	$effect(() => {
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		const _sidebarState = sidebar.state;

		if (leaflet.ready) {
			setTimeout(() => leaflet.invalidateSize(), 300);
		}
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

	// After Leaflet Initialization - Destination Marker
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

	// After Leaflet Initialization - Tracker Marker
	$effect(() => {
		if (!leaflet.ready) return;
		if (!trackersQuery.data) return;
		if (!mqttPayloadHistoriesQuery.data) return;

		mqttPayloadHistoriesQuery.data.forEach((mqttPayload) => {
			const id = mqttPayload.id;
			const latitude = mqttPayload.location.latitude;
			const longitude = mqttPayload.location.longitude;
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
</script>

<div class="flex h-full w-full flex-col gap-4">
	<div class="flex flex-8 gap-4">
		<!-- Details -->
		<div class="flex flex-3 flex-col gap-4">
			<Card.Root class="flex-1"></Card.Root>
			<Card.Root class="flex-1"></Card.Root>
			<!-- Chart -->
			<Card.Root class="flex-2 p-4">
				<Chart.Container config={chartConfig} class="h-full w-full">
					<AreaChart
						data={latencyChart.data}
						xScale={scaleTime()}
						x="time"
						axis="x"
						legend
						series={chartSeries}
						props={{
							xAxis: {
								format: (d: Date) =>
									d.toLocaleTimeString(undefined, {
										hour12: false,
										hour: "2-digit",
										minute: "2-digit",
										second: "2-digit"
									})
							},
							yAxis: {
								format: (d) => `${d} ms`
							}
						}}
					>
						{#snippet tooltip()}
							<Chart.Tooltip labelFormatter={(d: Date) => d.toLocaleTimeString()} />
						{/snippet}
					</AreaChart>
				</Chart.Container>
			</Card.Root>
		</div>
		<!-- Map -->
		<Card.Root class="flex-9 gap-0 p-0">
			<div bind:this={mapElement} class="h-full w-full overflow-hidden rounded-xl"></div>
		</Card.Root>
	</div>
	<div class="flex flex-4 gap-4">
		<!-- Car Status -->
		<Card.Root class="flex-4 gap-0 p-0">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Car</Table.Head>
						<Table.Head>Police Number</Table.Head>
						<Table.Head>Gas Level</Table.Head>
						<Table.Head>Kilometer</Table.Head>
					</Table.Row>
				</Table.Header>
			</Table.Root>
		</Card.Root>
		<!-- Destination -->
		<Card.Root class="flex-8 gap-0 p-0">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Destination</Table.Head>
						<Table.Head>Type</Table.Head>
						<Table.Head>Starts At</Table.Head>
						<Table.Head>Nearest Car</Table.Head>
						<Table.Head>Progress</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each activitiesQuery.data?.activities as activity (activity.activity_id)}
						<Table.Row>
							<Table.Cell>{activity.contact_name}</Table.Cell>
							<Table.Cell>{activity.activity_type_name}</Table.Cell>
							<Table.Cell>{activity.started_at}</Table.Cell>
							<Table.Cell>
								{#if activity.finished_at}
									{activity.car_name || "N/A"}
								{:else}
									{distancesMap.get(activity.activity_id)
										? `${distancesMap.get(activity.activity_id)?.car_name} - ${distancesMap.get(activity.activity_id)?.car_police_number}`
										: "No Car Assigned"}
								{/if}
							</Table.Cell>
							<Table.Cell>
								{#if activity.finished_at}
									Finished
								{:else}
									{distancesMap.get(activity.activity_id)?.distance.toFixed(2) ?? "N/A"} km
								{/if}
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</Card.Root>
	</div>
</div>
