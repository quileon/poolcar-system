<script lang="ts">
	import {
		useActivitiesQuery
		// useCreateActivityMutation,
		// useEditActivityMutation
	} from "$lib/hooks/use-activity";
	import * as Card from "$lib/components/ui/card/index";
	import * as Table from "$lib/components/ui/table/index";
	import PencilIcon from "@lucide/svelte/icons/pencil";
	import InfoIcon from "@lucide/svelte/icons/info";
	import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";
	import { DateTime } from "luxon";
	import { resolve } from "$app/paths";
	import Button from "$lib/components/ui/button/button.svelte";

	// let selectedActivityId = $state<number | null>(null);

	const activitiesQuery = useActivitiesQuery(
		() => "active",
		() => DateTime.now().minus({ days: 7 }).toFormat("yyyy-MM-dd")
	);
	// const createActivityMutation = useCreateActivityMutation();
	// const editActivityMutation = useEditActivityMutation(() => selectedActivityId);
</script>

{#if activitiesQuery.isLoading}
	<div class="flex h-full w-full flex-col gap-4">
		<div class="flex flex-row gap-4">
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<Skeleton class="h-7 w-full flex-11" />
				<Skeleton class="h-8 w-full flex-1" />
			</Card.Root>
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<Skeleton class="h-7 w-full flex-11" />
				<Skeleton class="h-8 w-full flex-1" />
			</Card.Root>
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<Skeleton class="h-7 w-full flex-11" />
				<Skeleton class="h-8 w-full flex-1" />
			</Card.Root>
		</div>
		<div class="flex flex-1 gap-4 overflow-hidden">
			<Skeleton class="h-full w-full rounded-xl" />
		</div>
	</div>
{/if}

{#if activitiesQuery.isSuccess}
	<div class="flex h-full w-full flex-col gap-4">
		<div class="flex flex-row gap-4">
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<p class="text-xl">Total Activities this Week</p>
				<p class="text-2xl font-bold">{activitiesQuery.data.activity_count}</p>
			</Card.Root>
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<p class="text-xl">Total Pending Trip this Week</p>
				<p class="text-2xl font-bold">
					{activitiesQuery.data.activities.filter((activity) => activity.finished_at === null)
						.length}
				</p>
			</Card.Root>
			<Card.Root class="flex flex-1 flex-row items-center justify-between p-8">
				<p class="text-xl">Total Finished Trip this Week</p>
				<p class="text-2xl font-bold">
					{activitiesQuery.data.activities.filter((activity) => activity.finished_at !== null)
						.length}
				</p>
			</Card.Root>
		</div>
		<div class="flex flex-1 gap-4 overflow-hidden">
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
								<Button size="icon" variant="outline">
									<InfoIcon />
								</Button>
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
		</div>
	</div>
{/if}
