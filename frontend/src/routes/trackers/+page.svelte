<script lang="ts">
	import { createQuery } from "@tanstack/svelte-query";
	import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import * as Alert from "$lib/components/ui/alert/index";
	import * as Table from "$lib/components/ui/table/index";
	import * as Select from "$lib/components/ui/select/index";
	import * as ButtonGroup from "$lib/components/ui/button-group/index";
	import Button from "$lib/components/ui/button/button.svelte";
	import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
	import PencilIcon from "@lucide/svelte/icons/pencil";
	import type { TrackerWithDetails } from "$lib/bindings/TrackerWithDetails";

	const filters = [
		{ label: "Active", value: "active" },
		{ label: "Deleted", value: "deleted" },
		{ label: "All", value: "all" }
	];
	let filterValue = $state("");
	const filterTrigger = $derived(
		filters.find((filter) => filter.value === filterValue)?.label ?? "Filter by"
	);

	const trackersQuery = createQuery<TrackerWithDetails[]>(() => ({
		queryKey: ["trackers"],
		queryFn: async () => {
			const response = await fetch("http://localhost:3000/trackers");
			if (!response.ok) throw new Error("Failed to fetch trackers");
			return response.json();
		}
	}));
</script>

<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Tracker Management</h1>

{#if trackersQuery.isPending}
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

{#if trackersQuery.isError}
	<Alert.Root variant="destructive">
		<AlertCircleIcon />
		<Alert.Title>Error</Alert.Title>
		<Alert.Description>
			<p>{trackersQuery.error.message}</p>
		</Alert.Description>
	</Alert.Root>
{/if}

{#if trackersQuery.isSuccess}
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
				<Button href="/trackers/create" size="default" variant="outline">Create Tracker</Button>
			</ButtonGroup.Root>
		</ButtonGroup.Root>
	</div>
	<Table.Root>
		<Table.Caption>A list of trackers.</Table.Caption>
		<Table.Header>
			<Table.Row>
				<Table.Head>#</Table.Head>
				<Table.Head>Name</Table.Head>
				<Table.Head>Assigned Car</Table.Head>
				<Table.Head>Actions</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each trackersQuery.data as tracker (tracker.tracker_id)}
				<Table.Row>
					<Table.Cell>{tracker.tracker_id}</Table.Cell>
					<Table.Cell>{tracker.name}</Table.Cell>
					<Table.Cell>{tracker.car_name}</Table.Cell>
					<Table.Cell>
						<Button href={`/trackers/${tracker.tracker_id}`} size="icon" variant="outline">
							<PencilIcon />
						</Button>
					</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
{/if}
