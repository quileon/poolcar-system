<script lang="ts">
	import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import * as Alert from "$lib/components/ui/alert/index";
	import * as Table from "$lib/components/ui/table/index";
	import * as Select from "$lib/components/ui/select/index";
	import * as ButtonGroup from "$lib/components/ui/button-group/index";
	import Button from "$lib/components/ui/button/button.svelte";
	import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
	import PencilIcon from "@lucide/svelte/icons/pencil";
	import { useCarsQuery } from "$lib/hooks/use-car";
	import { config } from "$lib/config";
	import { resolve } from "$app/paths";
	import { page } from "$app/state";

	const filters = [
		{ label: "Active", value: "active" },
		{ label: "Deleted", value: "deleted" },
		{ label: "All", value: "all" }
	];
	let filterValue = $state("");
	const filterTrigger = $derived(
		filters.find((filter) => filter.value === filterValue)?.label ?? "Filter by"
	);

	$effect(() => {
		const status = page.url.searchParams.get("status");
		filterValue = status ?? "";
	});
	const carsQuery = useCarsQuery(() => filterValue || null);
</script>

<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Cars Management</h1>

{#if carsQuery.isPending}
	<div class="flex flex-col gap-4">
		<Skeleton class="h-12" />
		<div class="flex flex-col space-y-1">
			<Skeleton class="h-6 w-full rounded-xl" />
			<Skeleton class="h-6 w-full rounded-xl" />
			<Skeleton class="h-6 w-full rounded-xl" />
			<Skeleton class="h-6 w-full rounded-xl" />
		</div>
	</div>
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
	<div class="flex justify-end">
		<ButtonGroup.Root>
			<ButtonGroup.Root>
				<Select.Root type="single" name="filter" bind:value={filterValue}>
					<Select.Trigger class="w-45">
						{filterTrigger}
					</Select.Trigger>
					<Select.Content>
						<Select.Group>
							<Select.Label>Filters</Select.Label>
							{#each filters as filter (filter.value)}
								<Select.Item value={filter.value} label={filter.label} />
							{/each}
						</Select.Group>
					</Select.Content>
				</Select.Root>
				<Button variant="outline" aria-label="Send" size="icon">
					<ArrowRightIcon />
				</Button>
			</ButtonGroup.Root>
			<ButtonGroup.Root>
				<Button
					href={`${config.apiBaseUrl}/cars/export`}
					download="cars.csv"
					size="default"
					variant="outline">Export</Button
				>
			</ButtonGroup.Root>
			<ButtonGroup.Root>
				<Button href={resolve("/cars/create")} size="default" variant="outline">Create Car</Button>
			</ButtonGroup.Root>
		</ButtonGroup.Root>
	</div>
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
				<Table.Head>Actions</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each carsQuery.data.cars as car (car.car_id)}
				<Table.Row class={car.deleted_at ? "text-red-700" : ""}>
					<Table.Cell>{car.car_id}</Table.Cell>
					<Table.Cell>{car.name}</Table.Cell>
					<Table.Cell>{car.police_number}</Table.Cell>
					<Table.Cell>{car.active ? "Yes" : "No"}</Table.Cell>
					<Table.Cell>{car.car_type_name}</Table.Cell>
					<Table.Cell>{car.tracker_name}</Table.Cell>
					<Table.Cell>
						<Button href={resolve(`/cars/${car.car_id}`)} size="icon" variant="outline">
							<PencilIcon />
						</Button>
					</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
{/if}
