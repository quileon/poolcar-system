<script lang="ts">
	import "leaflet/dist/leaflet.css";
	import * as Card from "$lib/components/ui/card/index";
	import * as RadioGroup from "$lib/components/ui/radio-group/index";
	import * as Popover from "$lib/components/ui/popover/index";
	import * as Command from "$lib/components/ui/command/index";
	import Button from "$lib/components/ui/button/button.svelte";
	import Input from "$lib/components/ui/input/input.svelte";
	import { useTrackersQuery } from "$lib/hooks/use-tracker";
	import { useCarsQuery } from "$lib/hooks/use-car";
	import { useAuditQuery } from "$lib/hooks/use-audit";
	import { LeafletMap } from "$lib/hooks/leaflet-map.svelte";
	import { onMount } from "svelte";
	import { DateTime } from "luxon";
	import Label from "$lib/components/ui/label/label.svelte";
	import { buttonVariants } from "$lib/components/ui/button/button.svelte";
	import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
	import TimerIcon from "@lucide/svelte/icons/timer";

	let filterType = $state<"tracker" | "car">("tracker");
	let comboboxValue = $state<number | null>(null);
	let comboboxOpen = $state(false);
	let searchQuery = $state<string>("");

	let selectedDate = $state<string>("");
	let mapElement: HTMLElement;

	const initialCoordinates: [number, number] = [-6.3709188, 106.8220167];
	const trackersQuery = useTrackersQuery(() => "active");
	const carsQuery = useCarsQuery(() => "active");
	const auditQuery = useAuditQuery(
		() => (filterType === "tracker" ? comboboxValue : null),
		() => (filterType === "car" ? comboboxValue : null),
		() => selectedDate || null
	);

	const leaflet = new LeafletMap();

	onMount(() => {
		if (mapElement) {
			leaflet.init(mapElement, {
				center: initialCoordinates,
				zoom: 13
			});
		}

		return () => {
			leaflet.destroy();
		};
	});

	$effect(() => {
		if (!leaflet.ready || !auditQuery.data) return;
		leaflet.clearAuditVisualization();

		const records = auditQuery.data.audit_records;
		const icon = leaflet.createIcon({
			iconUrl: new URL(`/src/lib/assets/car-1da08c.png`, import.meta.url).href,
			iconSize: [15.5, 23],
			iconAnchor: [7.75, 21],
			popupAnchor: [0, -20]
		});

		// Add markers for each audit record
		records.forEach((audit) => {
			const timestamp = DateTime.fromISO(audit.recorded_at).toLocaleString(DateTime.DATETIME_MED);
			leaflet.addAuditMarker(
				audit.recorded_at,
				audit.latitude,
				audit.longitude,
				icon,
				`${timestamp} (${audit.latitude}, ${audit.longitude})`
			);
		});

		// Create a polyline connecting all audit points in chronological order (oldest to newest)
		if (records.length > 1) {
			const coordinates = [...records]
				.reverse()
				.map((audit) => [audit.latitude, audit.longitude] as [number, number]);
			leaflet.addPolyline("audit_trail", coordinates, {
				color: "#3b82f6",
				weight: 3,
				opacity: 0.7
			});
		}
	});

	function zoomToAudit(lat: number, lng: number) {
		if (leaflet.map) {
			leaflet.map.setView([lat, lng], 20);
		}
	}

	function setCurrentDate() {
		selectedDate = DateTime.now().toISODate();
	}

	function refreshCombobox() {
		comboboxValue = null;
		searchQuery = "";
	}
</script>

<div class="flex h-full w-full flex-row gap-4">
	<Card.Root class="flex-2 p-0">
		<div bind:this={mapElement} class="h-full w-full rounded-xl"></div>
	</Card.Root>
	<div class="flex flex-1 flex-col gap-4">
		<!-- Select Car or Tracker to Audit -->
		<Card.Root class="flex flex-1 flex-col gap-4 p-4">
			<div>
				<h3 class="text-md mb-3 font-medium">Filter by</h3>
				<RadioGroup.Root bind:value={filterType} onValueChange={refreshCombobox}>
					<div class="flex items-center space-x-2">
						<RadioGroup.Item value="tracker" id="tracker" />
						<Label for="tracker">Tracker ID</Label>
					</div>
					<div class="flex items-center space-x-2">
						<RadioGroup.Item value="car" id="car" />
						<Label for="car">Car ID</Label>
					</div>
				</RadioGroup.Root>
			</div>
			<Popover.Root bind:open={comboboxOpen}>
				<Popover.Trigger class={`${buttonVariants({ variant: "outline" })} w-fit`}>
					{#if comboboxValue !== null}
						{#if filterType === "tracker"}
							#{comboboxValue}
							{trackersQuery.data?.trackers.find((t) => t.tracker_id === comboboxValue)?.name}
						{:else}
							#{comboboxValue}
							{carsQuery.data?.cars.find((c) => c.car_id === comboboxValue)?.name}
							({carsQuery.data?.cars.find((c) => c.car_id === comboboxValue)?.police_number})
						{/if}
					{:else}
						{filterType === "tracker" ? "Select Tracker ID" : "Select Car ID"}
					{/if}
				</Popover.Trigger>
				<Popover.Content align="start" class="w-75">
					<Command.Root>
						<Command.Input bind:value={searchQuery} placeholder="Filter name" />
						<Command.List>
							{#if filterType === "tracker" && trackersQuery.data}
								{@const filtered = trackersQuery.data.trackers.filter(
									(tracker) =>
										tracker.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
										tracker.tracker_id.toString().includes(searchQuery)
								)}
								{#if filtered.length === 0}
									<Command.Empty>No trackers found</Command.Empty>
								{:else}
									<Command.Group>
										{#each filtered as tracker (tracker.tracker_id)}
											<Command.Item
												value={tracker.tracker_id.toString()}
												onSelect={() => {
													comboboxValue = tracker.tracker_id;
													comboboxOpen = false;
													searchQuery = "";
												}}
											>
												#{tracker.tracker_id}
												{tracker.name}
											</Command.Item>
										{/each}
									</Command.Group>
								{/if}
							{:else if filterType === "car" && carsQuery.data}
								{@const filtered = carsQuery.data.cars.filter(
									(car) =>
										car.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
										car.police_number.toLowerCase().includes(searchQuery.toLowerCase()) ||
										car.car_id.toString().includes(searchQuery)
								)}
								{#if filtered.length === 0}
									<Command.Empty>No cars found</Command.Empty>
								{:else}
									<Command.Group>
										{#each filtered as car (car.car_id)}
											<Command.Item
												value={car.car_id.toString()}
												onSelect={() => {
													comboboxValue = car.car_id;
													comboboxOpen = false;
													searchQuery = "";
												}}
											>
												#{car.car_id}
												{car.name} ({car.police_number})
											</Command.Item>
										{/each}
									</Command.Group>
								{/if}
							{:else}
								<Command.Empty>No results found</Command.Empty>
							{/if}
						</Command.List>
					</Command.Root>
				</Popover.Content>
			</Popover.Root>
			<!-- Date Filter -->
			<Label for="audit_date">Date</Label>
			<div class="flex items-center gap-2">
				<Input type="date" id="audit_date" bind:value={selectedDate} class="flex-1" />
				<Button
					type="button"
					variant="outline"
					size="icon"
					onclick={setCurrentDate}
					title="Set to today's date"
				>
					<TimerIcon />
				</Button>
			</div>
		</Card.Root>
		<!-- Audit Data -->
		<Card.Root class="flex flex-3 flex-col gap-3 p-4">
			<h4 class="text-medium font-medium">Audit Data</h4>
			<ScrollArea class="h-140 w-full">
				{#if auditQuery.data}
					<div class="space-y-2 pr-4">
						{#each auditQuery.data.audit_records as audit (audit)}
							<div
								class="cursor-pointer rounded border bg-slate-50 p-3 text-sm transition-colors hover:bg-slate-100"
								onclick={() => zoomToAudit(audit.latitude, audit.longitude)}
								role="button"
								tabindex="0"
								onkeydown={(e) => {
									if (e.key === "Enter" || e.key === " ") {
										zoomToAudit(audit.latitude, audit.longitude);
									}
								}}
							>
								<div class="font-medium text-gray-900">
									{DateTime.fromISO(audit.recorded_at).toLocaleString(DateTime.DATETIME_MED)}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</ScrollArea>
		</Card.Root>
	</div>
</div>
