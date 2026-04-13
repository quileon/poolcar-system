<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import { useCarsQuery } from "$lib/hooks/use-car";
	import { useCreateCarStatusMutation } from "$lib/hooks/use-car-status";

	let carId = $state("");
	let statusType = $state("");
	let gasLevel = $state("");
	let kilometres = $state("");

	const carsQuery = useCarsQuery(() => "active");
	const createCarStatusQuery = useCreateCarStatusMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (statusType !== "DEPARTURE" && statusType !== "RETURN") return;

		createCarStatusQuery.mutate({
			carId: Number.parseInt(carId, 10),
			statusType: statusType,
			gasLevel: Number.parseFloat(gasLevel),
			kilometres: Number.parseFloat(kilometres)
		});
	}

	const carsTrigger = $derived.by(() => {
		const car = carsQuery.data?.cars.find((c) => c.car_id.toString() === carId);
		return car ? `${car.police_number} (${car.name})` : "Select Car";
	});

	const statusTypeTrigger = $derived.by(() => {
		if (statusType === "DEPARTURE") return "Departure";
		if (statusType === "RETURN") return "Return";
		return "Select Status";
	});
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
							<Select.Trigger id="carId">{carsTrigger}</Select.Trigger>
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
							<Select.Trigger id="statusType">{statusTypeTrigger}</Select.Trigger>
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
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createCarStatusQuery.isPending}>Submit</Button>
				<Button
					variant="outline"
					type="button"
					disabled={createCarStatusQuery.isPending}
					href={resolve("/car-status")}>Cancel</Button
				>
			</Field.Field>
		</Field.Group>
	</form>

	<div class="mt-8 space-y-4">
		{#if createCarStatusQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Creating Car Status</Alert.Title>
				<Alert.Description>
					<p>{createCarStatusQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
