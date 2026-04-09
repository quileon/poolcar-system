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
	import { useActivitiesQuery } from "$lib/hooks/use-activity";
	import { page } from "$app/state";
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

	$effect(() => {
		const status = page.url.searchParams.get("status");
		filterValue = status ?? "";
	});
	const activitiesQuery = useActivitiesQuery(() => filterValue || null);
</script>

<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
	Activities Management
</h1>

{#if activitiesQuery.isPending}
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

{#if activitiesQuery.isError}
	<Alert.Root variant="destructive">
		<AlertCircleIcon />
		<Alert.Title>Error</Alert.Title>
		<Alert.Description>
			<p>{activitiesQuery.error.message}</p>
		</Alert.Description>
	</Alert.Root>
{/if}

{#if activitiesQuery.isSuccess}
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
					href={`${config.apiBaseUrl}/activities/export`}
					download="activities.csv"
					size="default"
					variant="outline">Export</Button
				>
			</ButtonGroup.Root>
			<ButtonGroup.Root>
				<Button href={resolve("/activities/create")} size="default" variant="outline"
					>Create Activity</Button
				>
			</ButtonGroup.Root>
		</ButtonGroup.Root>
	</div>
	<Table.Root>
		<Table.Caption>A list of activity.</Table.Caption>
		<Table.Header>
			<Table.Row>
				<Table.Head>#</Table.Head>
				<Table.Head>Car Name</Table.Head>
				<Table.Head>Car Police Number</Table.Head>
				<Table.Head>Contact Name</Table.Head>
				<Table.Head>Activity Type</Table.Head>
				<Table.Head>Tracker Name</Table.Head>
				<Table.Head>Started At</Table.Head>
				<Table.Head>Finished At</Table.Head>
				<Table.Head>Finished Latitude</Table.Head>
				<Table.Head>Finished Longitude</Table.Head>
				<Table.Head>Actions</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each activitiesQuery.data.activities as activity (activity.activity_id)}
				<Table.Row class={activity.deleted_at ? "text-red-700" : ""}>
					<Table.Cell>{activity.activity_id}</Table.Cell>
					<Table.Cell>{activity.car_name || "-"}</Table.Cell>
					<Table.Cell>{activity.car_police_number || "-"}</Table.Cell>
					<Table.Cell>{activity.contact_name}</Table.Cell>
					<Table.Cell>{activity.activity_type_name}</Table.Cell>
					<Table.Cell>{activity.tracker_name || "-"}</Table.Cell>
					<Table.Cell
						>{activity.started_at
							? DateTime.fromISO(activity.started_at).toLocaleString(DateTime.DATETIME_MED)
							: "-"}</Table.Cell
					>
					<Table.Cell
						>{activity.finished_at
							? DateTime.fromISO(activity.finished_at).toLocaleString(DateTime.DATETIME_MED)
							: "-"}</Table.Cell
					>
					<Table.Cell>{activity.finished_latitude || "-"}</Table.Cell>
					<Table.Cell>{activity.finished_longitude || "-"}</Table.Cell>
					<Table.Cell>
						<Button
							href={resolve(`/activities/${activity.activity_id}`)}
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
