<script lang="ts">
	import { createQuery } from "@tanstack/svelte-query";
	import type { CarWithTracker } from "$lib/bindings/CarWithTracker";
	import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import * as Alert from "$lib/components/ui/alert/index";
	import * as Table from "$lib/components/ui/table/index";

	const carsQuery = createQuery<CarWithTracker[]>(() => ({
		queryKey: ["cars"],
		queryFn: async () => {
			const response = await fetch("http://localhost:3000/cars");
			if (!response.ok) throw new Error("Failed to fetch cars");
			return response.json();
		}
	}));
</script>

<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Cars Management</h1>

{#if carsQuery.isPending}
	<Skeleton class="rounded-xl" />
{/if}

{#if carsQuery.isError}
	<Alert.Root variant="destructive">
		<AlertCircleIcon />
		<Alert.Title>Error</Alert.Title>
		<Alert.Description>
			<p>{carsQuery.error.message}</p>
		</Alert.Description>
	</Alert.Root>
{/if}

{#if carsQuery.isSuccess}
	<Table.Root>
		<Table.Caption>A list of cars.</Table.Caption>
		<Table.Header>
			<Table.Row>
				<Table.Head>#</Table.Head>
				<Table.Head>Name</Table.Head>
				<Table.Head>Police Number</Table.Head>
				<Table.Head>Active</Table.Head>
				<Table.Head>Car Type</Table.Head>
				<Table.Head>Tracker</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each carsQuery.data as car (car.car_id)}
				<Table.Row>
					<Table.Cell>{car.car_id}</Table.Cell>
					<Table.Cell>{car.name}</Table.Cell>
					<Table.Cell>{car.police_number}</Table.Cell>
					<Table.Cell>{car.active ? "Yes" : "No"}</Table.Cell>
					<Table.Cell>{car.car_type_name}</Table.Cell>
					<Table.Cell>{car.tracker_name}</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
{/if}
