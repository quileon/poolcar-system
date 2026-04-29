<script lang="ts">
	import {
		useActivitiesQuery,
		useCreateActivityMutation,
		useEditActivityMutation

		// useCreateActivityMutation,
		// useEditActivityMutation
	} from "$lib/hooks/use-activity";
	import * as Card from "$lib/components/ui/card/index";
	import * as Dialog from "$lib/components/ui/dialog/index";
	import * as Table from "$lib/components/ui/table/index";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";

	import PencilIcon from "@lucide/svelte/icons/pencil";
	import InfoIcon from "@lucide/svelte/icons/info";
	import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";
	import { DateTime } from "luxon";
	import type { ActivityDetails } from "$lib/bindings/ActivityDetails";

	import Button, { buttonVariants } from "$lib/components/ui/button/button.svelte";

	import Input from "$lib/components/ui/input/input.svelte";
	import { Checkbox } from "$lib/components/ui/checkbox/index";
	import Textarea from "$lib/components/ui/textarea/textarea.svelte";
	import { useContactsQuery } from "$lib/hooks/use-contact";
	import { useCarsQuery } from "$lib/hooks/use-car";
	import { useActivityTypesQuery } from "$lib/hooks/use-activity-type";
	import { useTrackersQuery } from "$lib/hooks/use-tracker";
	import TimerIcon from "@lucide/svelte/icons/timer";

	let selectedActivityId = $state<number | null>(null);

	const activitiesQuery = useActivitiesQuery(
		() => "all",
		() => DateTime.now().minus({ days: 7 }).toFormat("yyyy-LL-dd")
	);
	const createActivityMutation = useCreateActivityMutation({ navigateTo: null });
	const editActivityMutation = useEditActivityMutation(() => selectedActivityId, {
		navigateTo: null
	});

	let createContactId = $state("");
	let createActivityTypeId = $state("");
	let createStartedDateAt = $state("");
	let createStartedTimeAt = $state("");
	let createDescription = $state("");

	let editContactId = $state("");
	let editActivityTypeId = $state("");
	let editStartedDateAt = $state("");
	let editStartedTimeAt = $state("");
	let editDescription = $state("");

	const contactsQuery = useContactsQuery(() => "active");
	const activityTypesQuery = useActivityTypesQuery(() => "active");

	function handleCreateSubmit(e: Event) {
		e.preventDefault();

		if (!createContactId || !createActivityTypeId || !createStartedDateAt || !createStartedTimeAt) {
			return;
		}

		const createStartedDateTime = DateTime.fromISO(`${createStartedDateAt}T${createStartedTimeAt}`)
			.toUTC()
			.toFormat("yyyy-LL-dd'T'HH:mm:ss");

		createActivityMutation.mutate({
			carId: null,
			contactId: Number.parseInt(createContactId, 10),
			activityTypeId: Number.parseInt(createActivityTypeId, 10),
			trackerId: null,
			startedAt: createStartedDateTime,
			finishedAt: null,
			finishedLatitude: null,
			finishedLongitude: null,
			description: createDescription
		});
	}

	function setCreateStartedCurrent() {
		const currentTime = DateTime.now();
		createStartedDateAt = currentTime.toFormat("yyyy-LL-dd");
		createStartedTimeAt = currentTime.toFormat("HH:mm:ss");
	}

	function openEditDialog(activity: ActivityDetails) {
		selectedActivityId = activity.activity_id;
		editContactId = activity.contact_id.toString();
		editActivityTypeId = activity.activity_type_id.toString();

		if (activity.started_at) {
			const startedDateTime = DateTime.fromISO(activity.started_at);
			editStartedDateAt = startedDateTime.toFormat("yyyy-LL-dd");
			editStartedTimeAt = startedDateTime.toFormat("HH:mm:ss");
		}
	}

	function handleEditSubmit(e: Event) {
		e.preventDefault();

		const editStartedDateTime = DateTime.fromISO(`${editStartedDateAt}T${editStartedTimeAt}`)
			.toUTC()
			.toFormat("yyyy-MM-dd'T'HH:mm:ss");

		editActivityMutation.mutate({
			carId: null,
			contactId: Number.parseInt(editContactId, 10),
			activityTypeId: Number.parseInt(editActivityTypeId, 10),
			trackerId: null,
			startedAt: editStartedDateTime,
			finishedAt: null,
			finishedLatitude: null,
			finishedLongitude: null,
			description: editDescription
		});
	}

	function setEditStartedCurrent() {
		const currentTime = DateTime.now();
		editStartedDateAt = currentTime.toISODate();
		editStartedTimeAt = currentTime.toFormat("HH:mm:ss");
	}

	const contactTrigger = $derived(
		contactsQuery.data?.contacts.find(
			(contact) => contact.contact_id.toString() === createContactId
		)?.name ?? "Select Contact"
	);
	const activityTypeTrigger = $derived(
		activityTypesQuery.data?.activity_types.find(
			(activityType) => activityType.activity_type_id.toString() === createActivityTypeId
		)?.name ?? "Select Activity Type"
	);
	const editContactTrigger = $derived(
		contactsQuery.data?.contacts.find((contact) => contact.contact_id.toString() === editContactId)
			?.name ?? "Select Contact"
	);
	const editActivityTypeTrigger = $derived(
		activityTypesQuery.data?.activity_types.find(
			(activityType) => activityType.activity_type_id.toString() === editActivityTypeId
		)?.name ?? "Select Activity Type"
	);
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

		<Dialog.Root>
			<Dialog.Trigger type="button" class={buttonVariants({ variant: "outline" })}>
				Create New Trip
			</Dialog.Trigger>
			<Dialog.Content class="sm:max-w-lg">
				<form onsubmit={handleCreateSubmit}>
					<Dialog.Header>
						<Dialog.Title>Create New Trip</Dialog.Title>
						<Dialog.Description>Create new trip to contact.</Dialog.Description>
					</Dialog.Header>
					<Field.Set class="py-4">
						<Field.Field>
							<Field.Label for="contact_id">Contact</Field.Label>
							<Select.Root type="single" bind:value={createContactId}>
								<Select.Trigger id="contact_id">{contactTrigger}</Select.Trigger>
								<Select.Content>
									{#if contactsQuery.data?.contacts}
										{#each contactsQuery.data?.contacts as contact (contact.contact_id)}
											<Select.Item value={contact.contact_id.toString()}>{contact.name}</Select.Item
											>
										{/each}
									{/if}
								</Select.Content>
							</Select.Root>
							<Field.Description>Enter the destination contact.</Field.Description>
						</Field.Field>
						<Field.Field>
							<Field.Label for="activity_type_id">Activity Type</Field.Label>
							<Select.Root type="single" bind:value={createActivityTypeId}>
								<Select.Trigger id="activity_type_id">{activityTypeTrigger}</Select.Trigger>
								<Select.Content>
									{#if activityTypesQuery.data?.activity_types}
										{#each activityTypesQuery.data?.activity_types as activityType (activityType.activity_type_id)}
											<Select.Item value={activityType.activity_type_id.toString()}
												>{activityType.name}</Select.Item
											>
										{/each}
									{/if}
								</Select.Content>
							</Select.Root>
							<Field.Description>Enter the type of activity.</Field.Description>
						</Field.Field>
						<Field.Field>
							<Field.Label for="started_at">Started At</Field.Label>
							<div class="flex items-center gap-2">
								<Input
									type="date"
									id="started_at"
									bind:value={createStartedDateAt}
									class="flex-6"
								/>
								<Input
									type="time"
									step="1"
									id="started_time_at"
									bind:value={createStartedTimeAt}
									class="flex-5"
								/>
								<Button
									type="button"
									variant="outline"
									size="icon"
									onclick={setCreateStartedCurrent}
									class="flex-1"
								>
									<TimerIcon />
								</Button>
							</div>
						</Field.Field>
						<Field.Field>
							<Field.Label for="description">Description</Field.Label>
							<Textarea placeholder="Additional description." bind:value={createDescription} />
						</Field.Field>
					</Field.Set>
					<Dialog.Footer>
						<Dialog.Close
							type="button"
							class={buttonVariants({ variant: "outline" })}
							disabled={createActivityMutation.isPending}
						>
							Cancel
						</Dialog.Close>
						<Button type="submit" disabled={createActivityMutation.isPending}>Create Trip</Button>
					</Dialog.Footer>
				</form>
			</Dialog.Content>
		</Dialog.Root>

		<div class="flex flex-1 gap-4 overflow-hidden">
			<Table.Root>
				<Table.Caption>A list of activity.</Table.Caption>
				<Table.Header>
					<Table.Row>
						<Table.Head>#</Table.Head>
						<Table.Head>Car Name</Table.Head>
						<Table.Head>Car Police Number</Table.Head>
						<Table.Head>Contact Name</Table.Head>
						<Table.Head>Activity Type</Table.Head>
						<Table.Head>Tracker Name</Table.Head>
						<Table.Head>Started At</Table.Head>
						<Table.Head>Finished At</Table.Head>
						<Table.Head>Finished Latitude</Table.Head>
						<Table.Head>Finished Longitude</Table.Head>
						<Table.Head>Actions</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each activitiesQuery.data.activities as activity (activity.activity_id)}
						<Table.Row class={activity.deleted_at ? "text-red-700" : ""}>
							<Table.Cell>{activity.activity_id}</Table.Cell>
							<Table.Cell>{activity.car_name || "-"}</Table.Cell>
							<Table.Cell>{activity.car_police_number || "-"}</Table.Cell>
							<Table.Cell>{activity.contact_name}</Table.Cell>
							<Table.Cell>{activity.activity_type_name}</Table.Cell>
							<Table.Cell>{activity.tracker_name || "-"}</Table.Cell>
							<Table.Cell
								>{activity.started_at
									? DateTime.fromISO(activity.started_at).toLocaleString(DateTime.DATETIME_MED)
									: "-"}</Table.Cell
							>
							<Table.Cell
								>{activity.finished_at
									? DateTime.fromISO(activity.finished_at).toLocaleString(DateTime.DATETIME_MED)
									: "-"}</Table.Cell
							>
							<Table.Cell>{activity.finished_latitude || "-"}</Table.Cell>
							<Table.Cell>{activity.finished_longitude || "-"}</Table.Cell>
							<Table.Cell>
								<Button size="icon" variant="outline">
									<InfoIcon />
								</Button>
								<Dialog.Root>
									<Dialog.Trigger
										type="button"
										class={buttonVariants({ variant: "outline", size: "icon" })}
										onclick={() => openEditDialog(activity)}
									>
										<PencilIcon />
									</Dialog.Trigger>
									<Dialog.Content class="sm:max-w-lg">
										<form onsubmit={handleEditSubmit}>
											<Dialog.Header>
												<Dialog.Title>Edit Trip</Dialog.Title>
												<Dialog.Description>Update activity details.</Dialog.Description>
											</Dialog.Header>
											<Field.Set class="py-4">
												<Field.Field>
													<Field.Label for="edit_contact_id">Contact</Field.Label>
													<Select.Root type="single" bind:value={editContactId}>
														<Select.Trigger id="edit_contact_id"
															>{editContactTrigger}</Select.Trigger
														>
														<Select.Content>
															{#if contactsQuery.data?.contacts}
																{#each contactsQuery.data?.contacts as contact (contact.contact_id)}
																	<Select.Item value={contact.contact_id.toString()}>
																		{contact.name}
																	</Select.Item>
																{/each}
															{/if}
														</Select.Content>
													</Select.Root>
													<Field.Description>Enter the destination contact.</Field.Description>
												</Field.Field>

												<Field.Field>
													<Field.Label for="edit_activity_type_id">Activity Type</Field.Label>
													<Select.Root type="single" bind:value={editActivityTypeId}>
														<Select.Trigger id="edit_activity_type_id">
															{editActivityTypeTrigger}
														</Select.Trigger>
														<Select.Content>
															{#if activityTypesQuery.data?.activity_types}
																{#each activityTypesQuery.data?.activity_types as activityType (activityType.activity_type_id)}
																	<Select.Item value={activityType.activity_type_id.toString()}>
																		{activityType.name}
																	</Select.Item>
																{/each}
															{/if}
														</Select.Content>
													</Select.Root>
													<Field.Description>Enter the type of activity.</Field.Description>
												</Field.Field>

												<Field.Field>
													<Field.Label for="edit_started_at">Started At</Field.Label>
													<div class="flex items-center gap-2">
														<Input
															type="date"
															id="edit_started_at"
															bind:value={editStartedDateAt}
															class="flex-6"
														/>
														<Input
															type="time"
															step="1"
															id="edit_started_time_at"
															bind:value={editStartedTimeAt}
															class="flex-5"
														/>
														<Button
															type="button"
															variant="outline"
															size="icon"
															onclick={setEditStartedCurrent}
															class="flex-1"
														>
															<TimerIcon />
														</Button>
													</div>
												</Field.Field>

												<Field.Field>
													<Field.Label for="edit_description">Description</Field.Label>
													<Textarea
														placeholder="Additional description."
														bind:value={editDescription}
													/>
												</Field.Field>
											</Field.Set>
											<Dialog.Footer>
												<Dialog.Close
													type="button"
													class={buttonVariants({ variant: "outline" })}
													disabled={editActivityMutation.isPending}
												>
													Cancel
												</Dialog.Close>
												<Button type="submit" disabled={editActivityMutation.isPending}>
													Save Changes
												</Button>
											</Dialog.Footer>
										</form>
									</Dialog.Content>
								</Dialog.Root>
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</div>
	</div>
{/if}
