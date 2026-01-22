<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
	import { createMutation, createQuery, useQueryClient } from "@tanstack/svelte-query";
	import type { GetTrackerResponse } from "$lib/bindings/GetTrackerResponse";
	import type { GetCarTypesResponse } from "$lib/bindings/GetCarTypesResponse";
	import { resolve } from "$app/paths";
	import { goto } from "$app/navigation";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { config } from "$lib/config";

	let carName = $state("");
	let policeNumber = $state("");
	let carTypeId = $state("");
	let trackerId = $state("");
	let active = $state(true);

	const trackersQuery = createQuery<GetTrackerResponse>(() => ({
		queryKey: ["trackers"],
		queryFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/trackers`);
			if (!response.ok) throw new Error("Failed to fetch trackers");
			return response.json();
		}
	}));
	const carTypesQuery = createQuery<GetCarTypesResponse>(() => ({
		queryKey: ["carTypes"],
		queryFn: async () => {
			const response = await fetch(`${config.apiBaseUrl}/cars/types`);
			if (!response.ok) throw new Error("Failed to fetch car types");
			return response.json();
		}
	}));
	const queryClient = useQueryClient();
	const mutation = createMutation(() => ({
		mutationFn: async (data: {
			carName: string;
			policeNumber: string;
			carTypeId: number;
			trackerId: number | null;
			active: boolean;
		}) => {
			const response = await fetch(`${config.apiBaseUrl}/cars`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				body: JSON.stringify({
					name: data.carName,
					police_number: data.policeNumber,
					car_type_id: data.carTypeId,
					tracker_id: data.trackerId,
					active: data.active
				})
			});
			if (!response.ok) throw new Error("Failed to create car");
			return response.json();
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["cars", "car-types", "trackers"] });
			goto(resolve("/cars"));
		}
	}));

	function handleSubmit(e: Event) {
		e.preventDefault();
		mutation.mutate({
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
				<Button type="submit" disabled={mutation.isPending}>Submit</Button>
				<Button variant="outline" type="button" disabled={mutation.isPending} href="/cars"
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

		{#if mutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Creating Car</Alert.Title>
				<Alert.Description>
					<p>{mutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
