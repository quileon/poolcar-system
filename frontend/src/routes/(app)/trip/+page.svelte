<script lang="ts">
	import "leaflet/dist/leaflet.css";
	import {
		useActivitiesQuery,
		useCreateActivityMutation,
		useEditActivityMutation,
		useDeleteActivityMutation
	} from "$lib/hooks/use-activity";
	import * as Card from "$lib/components/ui/card/index";
	import * as Table from "$lib/components/ui/table/index";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import * as Dialog from "$lib/components/ui/dialog/index";

	import homeMarker from "$lib/assets/home.png";
	import destinationMarker from "$lib/assets/flag.png";
	import TrashIcon from "@lucide/svelte/icons/trash-2";
	import PencilIcon from "@lucide/svelte/icons/pencil";
	import InfoIcon from "@lucide/svelte/icons/info";
	import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";
	import { DateTime } from "luxon";

	import type { ActivityDetails } from "$lib/bindings/ActivityDetails";
	import Button, { buttonVariants } from "$lib/components/ui/button/button.svelte";
	import Input from "$lib/components/ui/input/input.svelte";
	import Textarea from "$lib/components/ui/textarea/textarea.svelte";
	import { useContactsQuery } from "$lib/hooks/use-contact";
	import { useActivityTypesQuery } from "$lib/hooks/use-activity-type";
	import { LeafletMap } from "$lib/hooks/leaflet-map.svelte";
	import TimerIcon from "@lucide/svelte/icons/timer";
	import Badge from "$lib/components/ui/badge/badge.svelte";

	let selectedActivityId = $state<number | null>(null);
	const isEditing = $derived(selectedActivityId !== null);

	const activitiesQuery = useActivitiesQuery(
		() => "active",
		() => DateTime.now().minus({ days: 7 }).toFormat("yyyy-LL-dd")
	);
	const createActivityMutation = useCreateActivityMutation({ navigateTo: null });
	const editActivityMutation = useEditActivityMutation(() => selectedActivityId, {
		navigateTo: null
	});
	let deleteActivityId = $state<number | null>(null);
	const deleteActivityMutation = useDeleteActivityMutation(() => {
		if (deleteActivityId === null) {
			throw new Error("Missing activity id");
		}
		return deleteActivityId;
	});

	let contactId = $state("");
	let activityTypeId = $state("");
	let startedDateAt = $state("");
	let startedTimeAt = $state("");
	let description = $state("");

	let mapElement: HTMLElement;
	const leaflet = new LeafletMap();
	const initialCoordinates: [number, number] = [-6.382310833, 107.1725405];

	function startCreateMode() {
		selectedActivityId = null;
		contactId = "";
		activityTypeId = "";
		startedDateAt = "";
		startedTimeAt = "";
		description = "";

		if (leaflet.ready && leaflet.hasDestinationMarker(0)) {
			leaflet.removeDestinationMarker(0);
			leaflet.map?.setView(initialCoordinates, 12);
		}
	}

	function startEditMode(activity: ActivityDetails) {
		selectedActivityId = activity.activity_id;
		contactId = activity.contact_id.toString();
		activityTypeId = activity.activity_type_id.toString();
		description = activity.description ?? "";
		startedDateAt = "";
		startedTimeAt = "";

		if (activity.started_at) {
			const startedDateTime = DateTime.fromISO(activity.started_at);
			startedDateAt = startedDateTime.toFormat("yyyy-LL-dd");
			startedTimeAt = startedDateTime.toFormat("HH:mm:ss");
		}
	}

	const contactsQuery = useContactsQuery(() => "active");
	const activityTypesQuery = useActivityTypesQuery(() => "active");

	async function handleSubmit(e: Event) {
		e.preventDefault();

		if (!contactId || !activityTypeId || !startedDateAt || !startedTimeAt) {
			return;
		}

		const startedDateTime = DateTime.fromISO(`${startedDateAt}T${startedTimeAt}`)
			.toUTC()
			.toFormat("yyyy-LL-dd'T'HH:mm:ss");

		try {
			if (isEditing) {
				if (selectedActivityId === null) {
					return;
				}
				await editActivityMutation.mutateAsync({
					carId: null,
					contactId: Number.parseInt(contactId, 10),
					activityTypeId: Number.parseInt(activityTypeId, 10),
					trackerId: null,
					startedAt: startedDateTime,
					finishedAt: null,
					finishedLatitude: null,
					finishedLongitude: null,
					description: description
				});
				startCreateMode();
			} else {
				await createActivityMutation.mutateAsync({
					carId: null,
					contactId: Number.parseInt(contactId, 10),
					activityTypeId: Number.parseInt(activityTypeId, 10),
					trackerId: null,
					startedAt: startedDateTime,
					finishedAt: null,
					finishedLatitude: null,
					finishedLongitude: null,
					description: description
				});
				startCreateMode();
			}
		} catch (err) {
			console.error("Failed to submit activity", err);
		}
	}

	async function handleDelete(activity: ActivityDetails) {
		const confirmed = window.confirm(`Delete activity #${activity.activity_id}?`);
		if (!confirmed) {
			return;
		}

		deleteActivityId = activity.activity_id;
		try {
			await deleteActivityMutation.mutateAsync();
		} catch (err) {
			console.error("Failed to delete activity", err);
		} finally {
			deleteActivityId = null;
			startCreateMode();
		}
	}

	function setCurrent() {
		const currentTime = DateTime.now();
		startedDateAt = currentTime.toFormat("yyyy-LL-dd");
		startedTimeAt = currentTime.toFormat("HH:mm:ss");
	}

	const contactTrigger = $derived(
		contactsQuery.data?.contacts.find((contact) => contact.contact_id.toString() === contactId)
			?.name ?? "Select Contact"
	);
	const activityTypeTrigger = $derived(
		activityTypesQuery.data?.activity_types.find(
			(activityType) => activityType.activity_type_id.toString() === activityTypeId
		)?.name ?? "Select Activity Type"
	);

	const selectedContact = $derived(
		contactsQuery.data?.contacts.find((contact) => contact.contact_id.toString() === contactId)
	);

	$effect(() => {
		if (!activitiesQuery.isSuccess) return;
		if (leaflet.ready) return;
		leaflet.init(mapElement, {
			center: initialCoordinates,
			zoom: 12
		});
	});

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

	$effect(() => {
		if (!leaflet.ready) return;
		if (!selectedContact) return;

		const lat = selectedContact.latitude;
		const lng = selectedContact.longitude;
		if (Number.isNaN(lat) || Number.isNaN(lng)) return;

		const icon = leaflet.createIcon({
			iconUrl: destinationMarker,
			iconSize: [15.5, 23],
			iconAnchor: [7.75, 21],
			popupAnchor: [0, -20]
		});
		leaflet.upsertDestinationMarker(0, lat, lng, icon, selectedContact.name);
		leaflet.map?.fitBounds(
			[
				[initialCoordinates[0], initialCoordinates[1]],
				[lat, lng]
			],
			{ padding: [24, 24] }
		);
	});
</script>

{#if activitiesQuery.isLoading}
	<div class="flex h-full w-full flex-col gap-4">
		<div class="flex flex-row gap-4">
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<Skeleton class="h-7 w-full flex-11" />
				<Skeleton class="h-8 w-full flex-1" />
			</Card.Root>
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<Skeleton class="h-7 w-full flex-11" />
				<Skeleton class="h-8 w-full flex-1" />
			</Card.Root>
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<Skeleton class="h-7 w-full flex-11" />
				<Skeleton class="h-8 w-full flex-1" />
			</Card.Root>
		</div>
		<div class="flex flex-1 gap-4 overflow-hidden">
			<Skeleton class="h-full w-full rounded-xl" />
		</div>
	</div>
{/if}

{#if activitiesQuery.isSuccess}
	<div class="flex h-full w-full flex-col gap-4">
		<div class="flex flex-row gap-4">
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<p class="text-xl">Total Activities this Week</p>
				<p class="text-2xl font-bold">{activitiesQuery.data.activity_count}</p>
			</Card.Root>
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<p class="text-xl">Total Pending Trip this Week</p>
				<p class="text-2xl font-bold">
					{activitiesQuery.data.activities.filter((activity) => activity.finished_at === null)
						.length}
				</p>
			</Card.Root>
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<p class="text-xl">Total Finished Trip this Week</p>
				<p class="text-2xl font-bold">
					{activitiesQuery.data.activities.filter((activity) => activity.finished_at !== null)
						.length}
				</p>
			</Card.Root>
		</div>

		<div class="flex gap-4">
			<!-- Create/Edit -->
			<Card.Root class="w-full max-w-md p-4">
				<form class="w-full" onsubmit={handleSubmit}>
					<Field.Set class="w-full">
						<Field.Legend>{isEditing ? "Edit Trip" : "Create New Trip"}</Field.Legend>
						<Field.Description
							>{isEditing
								? "Update activity details."
								: "Create new trip to contact."}</Field.Description
						>
						<!-- Contact -->
						<Field.Field>
							<Field.Label for="contact_id">Contact</Field.Label>
							<Select.Root type="single" bind:value={contactId}>
								<Select.Trigger id="contact_id" class="relative z-10 w-full">
									{contactTrigger}
								</Select.Trigger>
								<Select.Content>
									{#if contactsQuery.data?.contacts}
										{#each contactsQuery.data.contacts as contact (contact.contact_id)}
											<Select.Item value={contact.contact_id.toString()}>
												{contact.name}
											</Select.Item>
										{/each}
									{/if}
								</Select.Content>
							</Select.Root>
							<Field.Description>Enter the destiantion contact.</Field.Description>
							<!-- Contact Map -->
							<div class="relative z-0 mt-3 h-52">
								<div
									bind:this={mapElement}
									class="h-full w-full overflow-hidden rounded-lg border border-border"
								></div>
							</div>
						</Field.Field>
						<!-- Activity Type -->
						<Field.Field>
							<Field.Label for="activity_type_id">Activity Type</Field.Label>
							<Select.Root type="single" bind:value={activityTypeId}>
								<Select.Trigger id="contact_id" class="w-full">{activityTypeTrigger}</Select.Trigger
								>
								<Select.Content>
									{#if activityTypesQuery.data?.activity_types}
										{#each activityTypesQuery.data.activity_types as activityType (activityType.activity_type_id)}
											<Select.Item value={activityType.activity_type_id.toString()}>
												{activityType.name}
											</Select.Item>
										{/each}
									{/if}
								</Select.Content>
							</Select.Root>
							<Field.Description>Enter the type of activity.</Field.Description>
						</Field.Field>
						<!-- Started At -->
						<Field.Field>
							<Field.Label for="started_at">Started At</Field.Label>
							<div class="flex w-full items-center gap-2">
								<Input type="date" id="started_at" bind:value={startedDateAt} class="flex-6" />
								<Input
									type="time"
									step="1"
									id="started_time_at"
									bind:value={startedTimeAt}
									class="flex-5"
								/>
								<Button
									type="button"
									variant="outline"
									size="icon"
									onclick={setCurrent}
									class="flex-1"
								>
									<TimerIcon />
								</Button>
							</div>
						</Field.Field>
						<!-- Description -->
						<Field.Field>
							<Field.Label for="description">Description</Field.Label>
							<Textarea id="description" bind:value={description} class="w-full" />
						</Field.Field>
						<!-- Submit -->
						<div class="flex items-center justify-end gap-2">
							{#if isEditing}
								<Button type="button" variant="outline" onclick={startCreateMode}>Cancel</Button>
							{/if}
							<Button
								type="submit"
								disabled={createActivityMutation.isPending || editActivityMutation.isPending}
							>
								{isEditing ? "Save Changes" : "Create Trip"}
							</Button>
						</div>
					</Field.Set>
				</form>
			</Card.Root>

			<!-- Table -->
			<div class="flex min-w-0 flex-1">
				<Table.Root>
					<Table.Caption>A list of activity.</Table.Caption>
					<Table.Header>
						<Table.Row>
							<Table.Head>#</Table.Head>
							<Table.Head>Contact Name</Table.Head>
							<Table.Head>Activity Type</Table.Head>
							<Table.Head>Started At</Table.Head>
							<Table.Head>Status</Table.Head>
							<Table.Head>Actions</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each activitiesQuery.data.activities as activity (activity.activity_id)}
							<Table.Row class={activity.deleted_at ? "text-red-700" : ""}>
								<Table.Cell>{activity.activity_id}</Table.Cell>
								<Table.Cell>{activity.contact_name}</Table.Cell>
								<Table.Cell>{activity.activity_type_name}</Table.Cell>
								<Table.Cell
									>{activity.started_at
										? DateTime.fromISO(activity.started_at).toLocaleString(DateTime.DATETIME_MED)
										: "-"}</Table.Cell
								>
								<Table.Cell>
									{#if activity.finished_at}
										<Badge variant="default">Finished</Badge>
									{:else}
										<Badge variant="outline">Pending</Badge>
									{/if}
								</Table.Cell>
								<Table.Cell>
									<Dialog.Root>
										<Dialog.Trigger class={buttonVariants({ variant: "outline", size: "icon" })}>
											<InfoIcon />
										</Dialog.Trigger>
										<Dialog.Content class="sm:max-w-md">
											<Dialog.Header>
												<Dialog.Title>Trip Details</Dialog.Title>
												<Dialog.Description>Activity information summary.</Dialog.Description>
											</Dialog.Header>
											<div class="mt-4 space-y-3">
												<div class="flex items-center justify-between">
													<span class="text-sm text-muted-foreground">Activity ID</span>
													<span class="text-sm font-medium">{activity.activity_id}</span>
												</div>
												<div class="flex items-center justify-between">
													<span class="text-sm text-muted-foreground">Contact</span>
													<span class="text-sm font-medium">{activity.contact_name}</span>
												</div>
												<div class="flex items-center justify-between">
													<span class="text-sm text-muted-foreground">Activity Type</span>
													<span class="text-sm font-medium">{activity.activity_type_name}</span>
												</div>
												<div class="flex items-center justify-between">
													<span class="text-sm text-muted-foreground">Tracker</span>
													<span class="text-sm font-medium">{activity.tracker_name || "-"}</span>
												</div>
												<div class="flex items-center justify-between">
													<span class="text-sm text-muted-foreground">Started At</span>
													<span class="text-sm font-medium">
														{activity.started_at
															? DateTime.fromISO(activity.started_at).toLocaleString(
																	DateTime.DATETIME_MED
																)
															: "-"}
													</span>
												</div>
												<div class="flex items-center justify-between">
													<span class="text-sm text-muted-foreground">Finished At</span>
													<span class="text-sm font-medium">
														{activity.finished_at
															? DateTime.fromISO(activity.finished_at).toLocaleString(
																	DateTime.DATETIME_MED
																)
															: "-"}
													</span>
												</div>
												<div class="flex items-center justify-between">
													<span class="text-sm text-muted-foreground">Finished Lat</span>
													<span class="text-sm font-medium"
														>{activity.finished_latitude ?? "-"}</span
													>
												</div>
												<div class="flex items-center justify-between">
													<span class="text-sm text-muted-foreground">Finished Lng</span>
													<span class="text-sm font-medium"
														>{activity.finished_longitude ?? "-"}</span
													>
												</div>
												<div class="space-y-1">
													<span class="text-sm text-muted-foreground">Description</span>
													<p class="text-sm">{activity.description || "-"}</p>
												</div>
											</div>
										</Dialog.Content>
									</Dialog.Root>
									<Button size="icon" variant="outline" onclick={() => startEditMode(activity)}>
										<PencilIcon />
									</Button>
									<Button
										size="icon"
										variant="destructive"
										onclick={() => handleDelete(activity)}
										disabled={deleteActivityMutation.isPending}
									>
										<TrashIcon />
									</Button>
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			</div>
		</div>
	</div>
{/if}
