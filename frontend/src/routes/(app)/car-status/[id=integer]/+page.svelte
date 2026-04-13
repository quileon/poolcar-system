<script lang="ts">
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import { useCarsQuery } from "$lib/hooks/use-car";
	import {
		useCarStatusQuery,
		useDeleteCarStatusMutation,
		useEditCarStatusMutation,
		useRestoreCarStatusMutation
	} from "$lib/hooks/use-car-status";

	const carStatusId = $derived(parseInt(page.params.id!, 10));

	// Queries
	const carsQuery = useCarsQuery(() => "active");
	const carStatusQuery = useCarStatusQuery(() => carStatusId);

	// Mutations
	const editCarStatusMutation = useEditCarStatusMutation(() => carStatusId);
	const deleteCarStatusMutation = useDeleteCarStatusMutation(() => carStatusId);
	const restoreCarStatusMutation = useRestoreCarStatusMutation(() => carStatusId);

	// Form state
	let carId = $state("");
	let statusType = $state("");
	let gasLevel = $state("");
	let kilometres = $state("");

	// Sync form with loaded data
	$effect(() => {
		if (carStatusQuery.data) {
			carId = carStatusQuery.data.car_id.toString();
			statusType = carStatusQuery.data.status_type;
			gasLevel = carStatusQuery.data.gas_level.toString();
			kilometres = carStatusQuery.data.kilometres.toString();
		}
	});

	const carsTrigger = $derived.by(() => {
		const car = carsQuery.data?.cars.find((c) => c.car_id.toString() === carId);
		return car ? `${car.police_number} (${car.name})` : "Select Car";
	});

	const statusTypeTrigger = $derived.by(() => {
		if (statusType === "DEPARTURE") return "Departure";
		if (statusType === "RETURN") return "Return";
		return "Select Status";
	});

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (statusType !== "DEPARTURE" && statusType !== "RETURN") return;

		editCarStatusMutation.mutate({
			carId: Number.parseInt(carId, 10),
			statusType: statusType,
			gasLevel: Number.parseFloat(gasLevel),
			kilometres: Number.parseFloat(kilometres)
		});
	}

	function handleDelete() {
		if (confirm(`Are you sure you want to delete data from "${carStatusQuery.data?.car_name}"?`)) {
			deleteCarStatusMutation.mutate();
		}
	}

	function handleRestore() {
		if (confirm(`Are you sure you want to restore data from "${carStatusQuery.data?.car_name}"?`)) {
			restoreCarStatusMutation.mutate();
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Car Status</Field.Legend>
				<Field.Description
					>Car Status tells about the car before and after activities</Field.Description
				>
				<Field.Group>
					<Field.Field>
						<Field.Label for="carId">Car</Field.Label>
						<Select.Root type="single" bind:value={carId}>
							<Select.Trigger id="carId" disabled={carStatusQuery.isPending}
								>{carsTrigger}</Select.Trigger
							>
							<Select.Content>
								{#if carsQuery.data?.cars}
									{#each carsQuery.data.cars as car (car.car_id)}
										<Select.Item value={car.car_id.toString()}>
											{`${car.police_number} (${car.name})`}
										</Select.Item>
									{/each}
								{/if}
							</Select.Content>
						</Select.Root>
					</Field.Field>

					<Field.Field>
						<Field.Label for="statusType">Status Type</Field.Label>
						<Select.Root type="single" bind:value={statusType}>
							<Select.Trigger id="statusType" disabled={carStatusQuery.isPending}
								>{statusTypeTrigger}</Select.Trigger
							>
							<Select.Content>
								<Select.Item value="DEPARTURE">Departure</Select.Item>
								<Select.Item value="RETURN">Return</Select.Item>
							</Select.Content>
						</Select.Root>
					</Field.Field>

					<Field.Field>
						<Field.Label for="gasLevel">Gas Level</Field.Label>
						<Input
							id="gasLevel"
							bind:value={gasLevel}
							type="number"
							step="any"
							placeholder="Enter gas level"
							disabled={carStatusQuery.isPending}
							required
						/>
					</Field.Field>

					<Field.Field>
						<Field.Label for="kilometres">Kilometres</Field.Label>
						<Input
							id="kilometres"
							bind:value={kilometres}
							type="number"
							step="any"
							placeholder="Enter kilometres"
							disabled={carStatusQuery.isPending}
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editCarStatusMutation.isPending ||
							carStatusQuery.isPending ||
							deleteCarStatusMutation.isPending ||
							restoreCarStatusMutation.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editCarStatusMutation.isPending ||
							carStatusQuery.isPending ||
							deleteCarStatusMutation.isPending ||
							restoreCarStatusMutation.isPending}
						href={resolve("/users")}>Cancel</Button
					>
				</div>
				{#if !carStatusQuery.data?.deleted_at}
					<Button
						type="button"
						disabled={editCarStatusMutation.isPending ||
							carStatusQuery.isPending ||
							deleteCarStatusMutation.isPending ||
							restoreCarStatusMutation.isPending}
						variant="destructive"
						onclick={handleDelete}>Delete</Button
					>
				{:else}
					<Button
						type="button"
						disabled={editCarStatusMutation.isPending ||
							carStatusQuery.isPending ||
							deleteCarStatusMutation.isPending ||
							restoreCarStatusMutation.isPending}
						variant="destructive"
						onclick={handleRestore}>Restore</Button
					>
				{/if}
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if carStatusQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{carStatusQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editCarStatusMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editCarStatusMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteCarStatusMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteCarStatusMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if restoreCarStatusMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{restoreCarStatusMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
