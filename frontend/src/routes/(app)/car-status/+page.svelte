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
	import { config } from "$lib/config";
	import { resolve } from "$app/paths";
	import { page } from "$app/state";
	import { useCarStatusesQuery } from "$lib/hooks/use-car-status";
	import { DateTime } from "luxon";

	const filters = [
		{ label: "Active", value: "active" },
		{ label: "Deleted", value: "deleted" },
		{ label: "All", value: "all" }
	];
	let filterValue = $state("");
	const filterTrigger = $derived(
		filters.find((filter) => filter.value === filterValue)?.label ?? "Filter by"
	);

	const carStatusesQuery = useCarStatusesQuery(() => filterValue || null);

	$effect(() => {
		const status = page.url.searchParams.get("status");
		filterValue = status ?? "";
	});
</script>

<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
	Car Status Management
</h1>

{#if carStatusesQuery.isPending}
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

{#if carStatusesQuery.isError}
	<Alert.Root variant="destructive">
		<AlertCircleIcon />
		<Alert.Title>Error</Alert.Title>
		<Alert.Description>
			<p>{carStatusesQuery.error.message}</p>
		</Alert.Description>
	</Alert.Root>
{/if}

{#if carStatusesQuery.isSuccess}
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
					href={`${config.apiBaseUrl}/cars/status/export`}
					download="car-status.csv"
					size="default"
					variant="outline">Export</Button
				>
			</ButtonGroup.Root>
			<ButtonGroup.Root>
				<Button href={resolve("/car-status/create")} size="default" variant="outline"
					>Create Car Status</Button
				>
			</ButtonGroup.Root>
		</ButtonGroup.Root>
	</div>
	<Table.Root>
		<Table.Caption>A list of car statuses.</Table.Caption>
		<Table.Header>
			<Table.Row>
				<Table.Head>#</Table.Head>
				<Table.Head>Car Name</Table.Head>
				<Table.Head>Car Police Number</Table.Head>
				<Table.Head>Status Type</Table.Head>
				<Table.Head>Gas Level</Table.Head>
				<Table.Head>Kilometres</Table.Head>
				<Table.Head>Recorded At</Table.Head>
				<Table.Head>Actions</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each carStatusesQuery.data.car_statuses as car_status (car_status.car_status_id)}
				<Table.Row class={car_status.deleted_at ? "text-red-700" : ""}>
					<Table.Cell>{car_status.car_status_id}</Table.Cell>
					<Table.Cell>{car_status.car_name}</Table.Cell>
					<Table.Cell>{car_status.car_police_number || "-"}</Table.Cell>
					<Table.Cell
						>{car_status.status_type.charAt(0).toUpperCase() +
							car_status.status_type.slice(1).toLowerCase() || "-"}</Table.Cell
					>
					<Table.Cell>{car_status.gas_level || "-"}</Table.Cell>
					<Table.Cell>{car_status.kilometres}</Table.Cell>
					<Table.Cell
						>{DateTime.fromISO(car_status.recorded_at).toLocaleString(
							DateTime.DATETIME_MED
						)}</Table.Cell
					>
					<Table.Cell>
						<Button
							href={resolve(`/car-status/${car_status.car_status_id}`)}
							size="icon"
							variant="outline"
						>
							<PencilIcon />
						</Button>
					</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
{/if}
