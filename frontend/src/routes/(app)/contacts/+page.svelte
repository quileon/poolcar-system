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
	import { useContactsQuery } from "$lib/hooks/use-contact";

	const filters = [
		{ label: "Active", value: "active" },
		{ label: "Deleted", value: "deleted" },
		{ label: "All", value: "all" }
	];
	let filterValue = $state("");
	const filterTrigger = $derived(
		filters.find((filter) => filter.value === filterValue)?.label ?? "Filter by"
	);

	const contactsQuery = useContactsQuery();
</script>

<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Contacts Management</h1>

{#if contactsQuery.isPending}
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

{#if contactsQuery.isError}
	<Alert.Root variant="destructive">
		<AlertCircleIcon />
		<Alert.Title>Error</Alert.Title>
		<Alert.Description>
			<p>{contactsQuery.error.message}</p>
		</Alert.Description>
	</Alert.Root>
{/if}

{#if contactsQuery.isSuccess}
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
					href={`${config.apiBaseUrl}/contacts/export`}
					download="contacts.csv"
					size="default"
					variant="outline">Export</Button
				>
			</ButtonGroup.Root>
			<ButtonGroup.Root>
				<Button href={resolve("/contacts/create")} size="default" variant="outline"
					>Create Contact</Button
				>
			</ButtonGroup.Root>
		</ButtonGroup.Root>
	</div>
	<Table.Root>
		<Table.Caption>A list of contacts.</Table.Caption>
		<Table.Header>
			<Table.Row>
				<Table.Head>#</Table.Head>
				<Table.Head>Name</Table.Head>
				<Table.Head>Latitude</Table.Head>
				<Table.Head>Longitude</Table.Head>
				<Table.Head>Contact Type</Table.Head>
				<Table.Head>Actions</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each contactsQuery.data.contacts as contact (contact.contact_id)}
				<Table.Row class={contact.deleted_at ? "text-red-700" : ""}>
					<Table.Cell>{contact.contact_id}</Table.Cell>
					<Table.Cell>{contact.name}</Table.Cell>
					<Table.Cell>{contact.latitude}</Table.Cell>
					<Table.Cell>{contact.longitude}</Table.Cell>
					<Table.Cell>{contact.contact_type_name}</Table.Cell>
					<Table.Cell>
						<Button href={resolve(`/contacts/${contact.contact_id}`)} size="icon" variant="outline">
							<PencilIcon />
						</Button>
					</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
{/if}
