<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import { Checkbox } from "$lib/components/ui/checkbox/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import TimerIcon from "@lucide/svelte/icons/timer";
	import { useContactsQuery } from "$lib/hooks/use-contact";
	import { resolve } from "$app/paths";
	import { useCarsQuery } from "$lib/hooks/use-car";
	import { useActivityTypesQuery } from "$lib/hooks/use-activity-type";
	import { useTrackersQuery } from "$lib/hooks/use-tracker";
	import { useCreateActivityMutation } from "$lib/hooks/use-activity";
	import { DateTime } from "luxon";
	import Textarea from "$lib/components/ui/textarea/textarea.svelte";

	let isFinished = $state(false);
	let carId = $state("");
	let contactId = $state("");
	let activityTypeId = $state("");
	let trackerId = $state("");
	let startedDateAt = $state("");
	let startedTimeAt = $state("");
	let finishedDateAt = $state("");
	let finishedTimeAt = $state("");
	let finishedLatitude = $state("");
	let finishedLongitude = $state("");
	let description = $state("");

	const carsQuery = useCarsQuery();
	const contactsQuery = useContactsQuery();
	const activityTypesQuery = useActivityTypesQuery();
	const trackersQuery = useTrackersQuery();

	const createActivityMutation = useCreateActivityMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();

		const startedDateTime = DateTime.fromISO(`${startedDateAt}T${startedTimeAt}`)
			.toUTC()
			.toFormat("yyyy-MM-dd'T'HH:mm:ss.SSS");
		const finishedDateTime = isFinished
			? DateTime.fromISO(`${finishedDateAt}T${finishedTimeAt}`)
					.toUTC()
					.toFormat("yyyy-MM-dd'T'HH:mm:ss.SSS")
			: null;

		createActivityMutation.mutate({
			carId: isFinished ? Number.parseInt(carId, 10) : null,
			contactId: Number.parseInt(contactId, 10),
			activityTypeId: Number.parseInt(activityTypeId, 10),
			trackerId: isFinished ? Number.parseInt(trackerId, 10) : null,
			startedAt: startedDateTime,
			finishedAt: finishedDateTime,
			finishedLatitude: isFinished ? Number.parseFloat(finishedLatitude) : null,
			finishedLongitude: isFinished ? Number.parseFloat(finishedLongitude) : null,
			description: description ? description : null
		});
	}

	function setStartedCurrent() {
		const currentTime = DateTime.now();
		startedDateAt = currentTime.toISODate();
		startedTimeAt = currentTime.toFormat("HH:mm:ss");
	}

	function setFinishedCurrent() {
		const currentTime = DateTime.now();
		finishedDateAt = currentTime.toISODate();
		finishedTimeAt = currentTime.toFormat("HH:mm:ss");
	}

	const carTrigger = $derived(
		carsQuery.data?.cars.find((car) => car.car_id.toString() === carId)?.name ?? "Select Car Type"
	);
	const contactTrigger = $derived(
		contactsQuery.data?.contacts.find((contact) => contact.contact_id.toString() === contactId)
			?.name ?? "Select Contact"
	);
	const activityTypeTrigger = $derived(
		activityTypesQuery.data?.activity_types.find(
			(activityType) => activityType.activity_type_id.toString() === activityTypeId
		)?.name ?? "Select Activity Type"
	);
	const trackerTrigger = $derived(
		trackersQuery.data?.trackers.find((tracker) => tracker.tracker_id.toString() === trackerId)
			?.name ?? "Select Tracker"
	);
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Activity</Field.Legend>
				<Field.Description>Activity is used to save the details of poolcar lend.</Field.Description>

				<Field.Group>
					<!-- Required -->
					<Field.Field>
						<Field.Label for="contact_id">Contact</Field.Label>
						<Select.Root type="single" bind:value={contactId}>
							<Select.Trigger id="contact_id">{contactTrigger}</Select.Trigger>
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
						<Field.Label for="activity_type_id">Activity Type</Field.Label>
						<Select.Root type="single" bind:value={activityTypeId}>
							<Select.Trigger id="activity_type_id">{activityTypeTrigger}</Select.Trigger>
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

					<Field.Field>
						<Field.Label for="started_at">Started At</Field.Label>

						<div class="flex items-center gap-2">
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
								onclick={setStartedCurrent}
								class="flex-1"
							>
								<TimerIcon />
							</Button>
						</div>
					</Field.Field>

					<!-- Optional Finished -->
					<Field.Field>
						<Field.Label for="description">Description</Field.Label>
						<Textarea placeholder="Additional description." bind:value={description}></Textarea>
					</Field.Field>

					<div class="flex items-center gap-2 pt-4 pb-2">
						<Checkbox id="is_finished" bind:checked={isFinished} />
						<Field.Label for="is_finished" class="cursor-pointer font-bold"
							>Activity Finished</Field.Label
						>
					</div>

					<Field.Field>
						<Field.Label for="car_id" class={!isFinished ? "text-muted-foreground" : ""}
							>Car</Field.Label
						>
						<Select.Root type="single" bind:value={carId} disabled={!isFinished}>
							<Select.Trigger id="car_id">{carTrigger}</Select.Trigger>
							<Select.Content>
								{#if carsQuery.data?.cars}
									{#each carsQuery.data.cars as car (car.car_id)}
										<Select.Item value={car.car_id.toString()}>
											{car.name}
										</Select.Item>
									{/each}
								{/if}
							</Select.Content>
						</Select.Root>
						<Field.Description>Enter the car that is used.</Field.Description>
					</Field.Field>

					<Field.Field>
						<Field.Label for="tracker_id" class={!isFinished ? "text-muted-foreground" : ""}
							>Tracker</Field.Label
						>
						<Select.Root type="single" bind:value={trackerId} disabled={!isFinished}>
							<Select.Trigger id="tracker_id">{trackerTrigger}</Select.Trigger>
							<Select.Content>
								{#if trackersQuery.data?.trackers}
									{#each trackersQuery.data.trackers as tracker (tracker.tracker_id)}
										<Select.Item value={tracker.tracker_id.toString()}>
											{tracker.name}
										</Select.Item>
									{/each}
								{/if}
							</Select.Content>
						</Select.Root>
						<Field.Description>Enter the tracker used to complete the activity.</Field.Description>
					</Field.Field>

					<Field.Field>
						<Field.Label for="finished_at" class={!isFinished ? "text-muted-foreground" : ""}
							>Finished At</Field.Label
						>

						<div class="flex items-center gap-2">
							<Input
								type="date"
								id="finished_date_at"
								bind:value={finishedDateAt}
								class="flex-6"
								disabled={!isFinished}
								required={isFinished}
							/>
							<Input
								type="time"
								step="1"
								id="finished_time_at"
								bind:value={finishedTimeAt}
								class="flex-5"
								disabled={!isFinished}
								required={isFinished}
							/>
							<Button
								type="button"
								variant="outline"
								size="icon"
								onclick={setFinishedCurrent}
								class="flex-1"
								disabled={!isFinished}
							>
								<TimerIcon />
							</Button>
						</div>
					</Field.Field>

					<Field.Field>
						<Field.Label for="latitude" class={!isFinished ? "text-muted-foreground" : ""}
							>Finished Coordinates</Field.Label
						>

						<div class="flex gap-4">
							<Input
								id="latitude"
								bind:value={finishedLatitude}
								type="text"
								placeholder="Enter finished latitude"
								disabled={!isFinished}
								required={isFinished}
							/>
							<Input
								id="longitude"
								bind:value={finishedLongitude}
								type="text"
								placeholder="Enter finished longitude"
								disabled={!isFinished}
								required={isFinished}
							/>
						</div>
					</Field.Field>
				</Field.Group>
			</Field.Set>

			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createActivityMutation.isPending}>Submit</Button>
				<Button
					variant="outline"
					type="button"
					disabled={createActivityMutation.isPending}
					href={resolve("/activities")}
					>Cancel
				</Button>
			</Field.Field>
		</Field.Group>
	</form>

	<div class="mt-8 space-y-4">
		{#if carsQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Loading Cars</Alert.Title>
				<Alert.Description>
					<p>{carsQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if contactsQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Loading Contacts</Alert.Title>
				<Alert.Description>
					<p>{contactsQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if activityTypesQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Loading Activity Types</Alert.Title>
				<Alert.Description>
					<p>{activityTypesQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if trackersQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Loading Trackers</Alert.Title>
				<Alert.Description>
					<p>{trackersQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if createActivityMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Creating Activity</Alert.Title>
				<Alert.Description>
					<p>{createActivityMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
