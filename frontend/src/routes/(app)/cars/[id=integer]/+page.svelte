<script lang="ts">
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import { useCarQuery, useDeleteCarMutation, useEditCarMutation } from "$lib/hooks/use-car";
	import { useCarTypesQuery } from "$lib/hooks/user-car-type";
	import { useTrackersQuery } from "$lib/hooks/use-tracker";

	const carId = $derived(parseInt(page.params.id!, 10));

	// Queries
	const carQuery = useCarQuery(() => carId);
	const trackersQuery = useTrackersQuery();
	const carTypesQuery = useCarTypesQuery();

	// Mutations
	const editCarMutation = useEditCarMutation(() => carId);
	const deleteCarMutation = useDeleteCarMutation(() => carId);

	// Form state
	let carName = $state("");
	let policeNumber = $state("");
	let carTypeId = $state("");
	let trackerId = $state("");
	let active = $state(true);

	// Sync form with loaded data
	$effect(() => {
		if (carQuery.data) {
			carName = carQuery.data.name;
			policeNumber = carQuery.data.police_number;
			carTypeId = carQuery.data.car_type_id.toString();
			trackerId = carQuery.data.tracker_id?.toString() ?? "";
			active = carQuery.data.active;
		}
	});

	// Derived values for select triggers
	const trackerTrigger = $derived(
		trackersQuery.data?.trackers.find((tracker) => tracker.tracker_id.toString() === trackerId)
			?.name ?? "Select Tracker"
	);
	const carTypeTrigger = $derived(
		carTypesQuery.data?.car_types.find((carType) => carType.car_type_id.toString() === carTypeId)
			?.name ?? "Select Car Type"
	);

	// Event handlers
	function handleSubmit(event: Event) {
		event.preventDefault();
		editCarMutation.mutate({
			carName,
			policeNumber,
			carTypeId: Number.parseInt(carTypeId, 10),
			trackerId: trackerId.length === 0 ? null : Number.parseInt(trackerId, 10),
			active
		});
	}
	function handleDelete() {
		if (confirm(`Are you sure you want to delete "${carName}"?`)) {
			deleteCarMutation.mutate();
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Edit Car</Field.Legend>
				<Field.Description>Car or Poolcar can be lend for activities.</Field.Description>
				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Car Name</Field.Label>
						<Input
							id="name"
							bind:value={carName}
							type="text"
							placeholder="Enter car name"
							disabled={carQuery.isPending}
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
								disabled={carQuery.isPending}
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
											disabled={tracker.car_id !== null && tracker.car_id !== carId}
											>{tracker.name}</Select.Item
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
			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editCarMutation.isPending ||
							carQuery.isPending ||
							deleteCarMutation.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editCarMutation.isPending ||
							carQuery.isPending ||
							deleteCarMutation.isPending}
						href={resolve("/cars")}>Cancel</Button
					>
				</div>
				<Button
					type="button"
					disabled={editCarMutation.isPending || carQuery.isPending || deleteCarMutation.isPending}
					variant="destructive"
					onclick={handleDelete}>Delete</Button
				>
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if carQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{carQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editCarMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editCarMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteCarMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteCarMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
