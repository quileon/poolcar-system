<script lang="ts">
	import "leaflet/dist/leaflet.css";
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import {
		useContactQuery,
		useDeleteContactMutation,
		useEditContactMutation,
		useRestoreContactmutation
	} from "$lib/hooks/use-contact";
	import { useContactTypesQuery } from "$lib/hooks/use-contact-type";
	import { LeafletMap } from "$lib/hooks/leaflet-map.svelte";
	import { useSidebar } from "$lib/components/ui/sidebar";
	import { useSearchPlacesQuery } from "$lib/hooks/use-google-map";
	import { onMount } from "svelte";
	import homeMarker from "$lib/assets/home.png";
	import destinationMarker from "$lib/assets/flag.png";

	const contactId = $derived(parseInt(page.params.id!, 10));

	// Queries
	const contactQuery = useContactQuery(() => contactId);
	const contactTypesQuery = useContactTypesQuery(() => "active");

	// Mutations
	const editContactMutation = useEditContactMutation(() => contactId);
	const deleteContactMutation = useDeleteContactMutation(() => contactId);
	const restoreContactMutation = useRestoreContactmutation(() => contactId);

	// Form state
	let name = $state("");
	let latitude = $state("");
	let longitude = $state("");
	let contactTypeId = $state("");

	let searchQuery = $state("");
	let debouncedQuery = $state("");
	let mapElement: HTMLElement;
	const leaflet = new LeafletMap();
	const sidebar = useSidebar();
	const initialCoordinates: [number, number] = [-6.3709188, 106.8220167];

	// Sync form with loaded data
	$effect(() => {
		if (contactQuery.data) {
			name = contactQuery.data.name;
			latitude = contactQuery.data.latitude.toString();
			longitude = contactQuery.data.longitude.toString();
			contactTypeId = contactQuery.data.contact_type_id.toString();
		}
	});

	const contactTypeTrigger = $derived(
		contactTypesQuery.data?.contact_types.find(
			(contactType) => contactType.contact_type_id.toString() === contactTypeId
		)?.name ?? "Select Contact Type"
	);

	function handleSubmit(e: Event) {
		e.preventDefault();
		editContactMutation.mutate({
			name,
			latitude: Number.parseFloat(latitude),
			longitude: Number.parseFloat(longitude),
			contactTypeId: Number.parseInt(contactTypeId, 10)
		});
	}
	function handleDelete() {
		if (confirm(`Are you sure you want to delete "${name}"?`)) {
			deleteContactMutation.mutate();
		}
	}
	function handleRestore() {
		if (confirm(`Are you sure you want to restore "${name}"?`)) {
			restoreContactMutation.mutate();
		}
	}

	onMount(() => {
		leaflet.init(mapElement, {
			center: initialCoordinates,
			zoom: 13,
			onMapClick: (lat, lng) => {
				latitude = lat.toString();
				longitude = lng.toString();
			}
		});
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

	// After Leaflet Initialization - Contact Location Marker
	$effect(() => {
		if (!leaflet.ready) return;
		if (!latitude || !longitude) return;

		const lat = Number.parseFloat(latitude);
		const lng = Number.parseFloat(longitude);
		if (Number.isNaN(lat) || Number.isNaN(lng)) return;

		const icon = leaflet.createIcon({
			iconUrl: destinationMarker,
			iconSize: [15.5, 23],
			iconAnchor: [7.75, 21],
			popupAnchor: [0, -20]
		});
		leaflet.upsertDestinationMarker(0, lat, lng, icon, name || "Contact");
		leaflet.panTo(lat, lng);
	});

	// Debouncing Google Maps Search
	$effect(() => {
		if (!leaflet.ready) return;

		const query = searchQuery;
		const timer = setTimeout(() => {
			debouncedQuery = query;
		}, 1000);
		return () => clearTimeout(timer);
	});

	// After Leaflet Initialization - Sidebar Resize Handling
	$effect(() => {
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		const _sidebarState = sidebar.state;

		if (leaflet.ready) {
			setTimeout(() => leaflet.invalidateSize(), 300);
		}
	});

	const searchPlacesQuery = useSearchPlacesQuery(() => debouncedQuery);
</script>

<div class="mx-auto w-full max-w-md px-4">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Edit Contact</Field.Legend>
				<Field.Description>Contact is used for identifying car destination.</Field.Description>
				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Contact Name</Field.Label>
						<Input
							id="name"
							bind:value={name}
							type="text"
							placeholder="Enter contact name"
							disabled={contactQuery.isPending}
							required
						/>
					</Field.Field>

					<div class="flex gap-4">
						<Field.Field>
							<Field.Label for="latitude">Latitude</Field.Label>
							<Input
								id="latitude"
								bind:value={latitude}
								type="text"
								placeholder="Enter contact latitude"
								required
							/>
						</Field.Field>
						<Field.Field>
							<Field.Label for="longitude">Longitude</Field.Label>
							<Input
								id="longitude"
								bind:value={longitude}
								type="text"
								placeholder="Enter contact longitude"
								required
							/>
						</Field.Field>
					</div>

					<Field.Field>
						<Field.Label for="contact_type_id">Contact Type</Field.Label>
						<Select.Root type="single" bind:value={contactTypeId}>
							<Select.Trigger id="contact_type_id">{contactTypeTrigger}</Select.Trigger>
							<Select.Content>
								{#if contactTypesQuery.data?.contact_types}
									{#each contactTypesQuery.data.contact_types as contactType (contactType.contact_type_id)}
										<Select.Item value={contactType.contact_type_id.toString()}>
											{contactType.name}
										</Select.Item>
									{/each}
								{/if}
							</Select.Content>
						</Select.Root>
						<Field.Description>Enter the type of contact.</Field.Description>
					</Field.Field>
				</Field.Group>
			</Field.Set>

			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editContactMutation.isPending ||
							deleteContactMutation.isPending ||
							restoreContactMutation.isPending ||
							contactQuery.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editContactMutation.isPending ||
							deleteContactMutation.isPending ||
							restoreContactMutation.isPending ||
							contactQuery.isPending}
						href={resolve("/contacts")}>Cancel</Button
					>
				</div>
				{#if !contactQuery.data?.deleted_at}
					<Button
						type="button"
						disabled={editContactMutation.isPending ||
							deleteContactMutation.isPending ||
							restoreContactMutation.isPending ||
							contactQuery.isPending}
						variant="destructive"
						onclick={handleDelete}>Delete</Button
					>
				{:else}
					<Button
						type="button"
						disabled={editContactMutation.isPending ||
							deleteContactMutation.isPending ||
							restoreContactMutation.isPending ||
							contactQuery.isPending}
						variant="destructive"
						onclick={handleRestore}>Restore</Button
					>
				{/if}
			</Field.Field>

			<Field.Separator />

			<div class="h-52">
				<div
					bind:this={mapElement}
					class="h-full w-full overflow-hidden rounded-lg border border-border"
				></div>
			</div>

			<Field.Set>
				<Field.Field>
					<Field.Label for="search">Search on Google Map</Field.Label>
					<Input
						id="search"
						bind:value={searchQuery}
						disabled={editContactMutation.isPending}
						type="text"
						placeholder="Enter Keyword"
					/>
				</Field.Field>

				{#if searchPlacesQuery.data?.places && searchPlacesQuery.data.places.length > 0}
					<div class="rounded-lg border border-border bg-card shadow-sm">
						<div class="max-h-64 divide-y divide-border overflow-y-auto">
							{#each searchPlacesQuery.data.places as place (place.id)}
								<button
									type="button"
									class="w-full px-4 py-3 text-left transition-colors hover:bg-muted focus-visible:bg-muted focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
									disabled={editContactMutation.isPending}
									onclick={() => {
										name = place.display_name.text;
										latitude = place.location.latitude.toString();
										longitude = place.location.longitude.toString();
										debouncedQuery = "";
									}}
								>
									<div class="text-sm leading-tight font-medium">{place.display_name.text}</div>
									<div class="mt-1 text-xs text-muted-foreground">
										{place.formatted_address}
									</div>
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</Field.Set>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if contactQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{contactQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editContactMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editContactMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteContactMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteContactMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if restoreContactMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{restoreContactMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>

<style>
	:global(.leaflet-container) {
		cursor: pointer !important;
	}

	:global(.leaflet-grab) {
		cursor: pointer !important;
	}

	:global(.leaflet-grabbing) {
		cursor: pointer !important;
	}
</style>
