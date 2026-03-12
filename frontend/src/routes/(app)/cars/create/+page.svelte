<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { useTrackersQuery } from "$lib/hooks/use-tracker";
	import { useCarTypesQuery } from "$lib/hooks/use-car-type";
	import { useCreateCarMutation } from "$lib/hooks/use-car";

	let carName = $state("");
	let policeNumber = $state("");
	let carTypeId = $state("");
	let trackerId = $state("");
	let active = $state(true);

	const trackersQuery = useTrackersQuery(() => "active");
	const carTypesQuery = useCarTypesQuery(() => "active");
	const createCarMutation = useCreateCarMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		createCarMutation.mutate({
			carName,
			policeNumber,
			carTypeId: Number.parseInt(carTypeId),
			trackerId: trackerId.length === 0 ? null : Number.parseInt(trackerId),
			active
		});
	}

	const trackerTrigger = $derived(
		trackersQuery.data?.trackers.find((tracker) => tracker.tracker_id.toString() === trackerId)
			?.name ?? "Select Tracker"
	);
	const carTypeTrigger = $derived(
		carTypesQuery.data?.car_types.find((carType) => carType.car_type_id.toString() === carTypeId)
			?.name ?? "Select Car Type"
	);
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Car</Field.Legend>
				<Field.Description>Car or Poolcar can be lend for activities.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Car Name</Field.Label>
						<Input
							id="name"
							bind:value={carName}
							type="text"
							placeholder="Enter car name"
							required
						/>
					</Field.Field>

					<div class="flex gap-4">
						<Field.Field>
							<Field.Label for="police_number">Police Number</Field.Label>
							<Input
								id="police_number"
								bind:value={policeNumber}
								type="text"
								placeholder="Enter police number"
								required
							/>
						</Field.Field>

						<Field.Field>
							<Field.Label for="car_type_id">Car Type</Field.Label>
							<Select.Root type="single" bind:value={carTypeId} required>
								<Select.Trigger id="car_type_id">{carTypeTrigger}</Select.Trigger>
								<Select.Content>
									{#if carTypesQuery.data?.car_types}
										{#each carTypesQuery.data.car_types as carType (carType.car_type_id)}
											<Select.Item value={carType.car_type_id.toString()}
												>{carType.name}</Select.Item
											>
										{/each}
									{/if}
								</Select.Content>
							</Select.Root>
						</Field.Field>
					</div>

					<Field.Field>
						<Field.Label for="tracker_id">Tracker (Optional)</Field.Label>
						<Select.Root type="single" bind:value={trackerId}>
							<Select.Trigger id="tracker_id">{trackerTrigger}</Select.Trigger>
							<Select.Content>
								<Select.Item value="">None</Select.Item>
								{#if trackersQuery.data?.trackers}
									{#each trackersQuery.data.trackers as tracker (tracker.tracker_id)}
										<Select.Item
											value={tracker.tracker_id.toString()}
											disabled={tracker.car_id !== null}>{tracker.name}</Select.Item
										>
									{/each}
								{/if}
							</Select.Content>
						</Select.Root>
						<Field.Description
							>Enter the tracker that will be used to track the car (optional).</Field.Description
						>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Separator />
			<Field.Set>
				<Field.Group>
					<Field.Field orientation="horizontal">
						<Checkbox id="active" bind:checked={active} />
						<Field.Content>
							<Field.Label for="active">Active</Field.Label>
							<Field.Description>Set the car status to active.</Field.Description>
						</Field.Content>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createCarMutation.isPending}>Submit</Button>
				<Button variant="outline" type="button" disabled={createCarMutation.isPending} href="/cars"
					>Cancel</Button
				>
			</Field.Field>
		</Field.Group>
	</form>

	<div class="mt-8 space-y-4">
		{#if trackersQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Loading Trackers</Alert.Title>
				<Alert.Description>
					<p>{trackersQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if carTypesQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Loading Car Types</Alert.Title>
				<Alert.Description>
					<p>{carTypesQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if createCarMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Creating Car</Alert.Title>
				<Alert.Description>
					<p>{createCarMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
