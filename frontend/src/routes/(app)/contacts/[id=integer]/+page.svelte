<script lang="ts">
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import {
		useContactQuery,
		useDeleteContactMutation,
		useEditContactMutation,
		useRestoreContactmutation
	} from "$lib/hooks/use-contact";
	import { useContactTypesQuery } from "$lib/hooks/use-contact-type";

	const contactId = $derived(parseInt(page.params.id!, 10));

	// Queries
	const contactQuery = useContactQuery(() => contactId);
	const contactTypesQuery = useContactTypesQuery();

	// Mutations
	const editContactMutation = useEditContactMutation(() => contactId);
	const deleteContactMutation = useDeleteContactMutation(() => contactId);
	const restoreContactMutation = useRestoreContactmutation(() => contactId);

	// Form state
	let name = $state("");
	let latitude = $state("");
	let longitude = $state("");
	let contactTypeId = $state("");

	// Sync form with loaded data
	$effect(() => {
		if (contactQuery.data) {
			name = contactQuery.data.name;
			latitude = contactQuery.data.latitude.toString();
			longitude = contactQuery.data.longitude.toString();
			contactTypeId = contactQuery.data.contact_type_id.toString();
		}
	});

	const contactTypeTrigger = $derived(
		contactTypesQuery.data?.contact_types.find(
			(contactType) => contactType.contact_type_id.toString() === contactTypeId
		)?.name ?? "Select Contact Type"
	);

	function handleSubmit(e: Event) {
		e.preventDefault();
		editContactMutation.mutate({
			name,
			latitude: Number.parseFloat(latitude),
			longitude: Number.parseFloat(longitude),
			contactTypeId: Number.parseInt(contactTypeId, 10)
		});
	}
	function handleDelete() {
		if (confirm(`Are you sure you want to delete "${name}"?`)) {
			deleteContactMutation.mutate();
		}
	}
	function handleRestore() {
		if (confirm(`Are you sure you want to restore "${name}"?`)) {
			restoreContactMutation.mutate();
			console.log(contactTypeId, typeof contactTypeId);
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Edit Car</Field.Legend>
				<Field.Description>Contact is used for identifying car destination.</Field.Description>
				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Contact Name</Field.Label>
						<Input
							id="name"
							bind:value={name}
							type="text"
							placeholder="Enter contact name"
							disabled={contactQuery.isPending}
							required
						/>
					</Field.Field>

					<div class="flex gap-4">
						<Field.Field>
							<Field.Label for="latitude">Latitude</Field.Label>
							<Input
								id="latitude"
								bind:value={latitude}
								type="text"
								placeholder="Enter contact latitude"
								required
							/>
						</Field.Field>
						<Field.Field>
							<Field.Label for="longitude">Longitude</Field.Label>
							<Input
								id="longitude"
								bind:value={longitude}
								type="text"
								placeholder="Enter contact longitude"
								required
							/>
						</Field.Field>
					</div>

					<Field.Field>
						<Field.Label for="contact_type_id">Contact Type</Field.Label>
						<Select.Root type="single" bind:value={contactTypeId}>
							<Select.Trigger id="contact_type_id">{contactTypeTrigger}</Select.Trigger>
							<Select.Content>
								{#if contactTypesQuery.data?.contact_types}
									{#each contactTypesQuery.data.contact_types as contactType (contactType.contact_type_id)}
										<Select.Item value={contactType.contact_type_id.toString()}>
											{contactType.name}
										</Select.Item>
									{/each}
								{/if}
							</Select.Content>
						</Select.Root>
						<Field.Description>Enter the type of contact.</Field.Description>
					</Field.Field>
				</Field.Group>
			</Field.Set>

			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editContactMutation.isPending ||
							deleteContactMutation.isPending ||
							restoreContactMutation.isPending ||
							contactQuery.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editContactMutation.isPending ||
							deleteContactMutation.isPending ||
							restoreContactMutation.isPending ||
							contactQuery.isPending}
						href={resolve("/contacts")}>Cancel</Button
					>
				</div>
				{#if !contactQuery.data?.deleted_at}
					<Button
						type="button"
						disabled={editContactMutation.isPending ||
							deleteContactMutation.isPending ||
							restoreContactMutation.isPending ||
							contactQuery.isPending}
						variant="destructive"
						onclick={handleDelete}>Delete</Button
					>
				{:else}
					<Button
						type="button"
						disabled={editContactMutation.isPending ||
							deleteContactMutation.isPending ||
							restoreContactMutation.isPending ||
							contactQuery.isPending}
						variant="destructive"
						onclick={handleRestore}>Restore</Button
					>
				{/if}
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if contactQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{contactQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editContactMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editContactMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteContactMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteContactMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if restoreContactMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{restoreContactMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
