<script lang="ts">
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import {
		useContactTypeQuery,
		useDeleteContactTypeMutation,
		useEditContactTypeMutation,
		useRestoreContactTypeMutation
	} from "$lib/hooks/use-contact-type";

	const contactTypeId = $derived(parseInt(page.params.id!, 10));

	// Queries
	const contactTypeQuery = useContactTypeQuery(() => contactTypeId);

	// Mutations
	const editContactTypeMutation = useEditContactTypeMutation(() => contactTypeId);
	const deleteContactTypeMutation = useDeleteContactTypeMutation(() => contactTypeId);
	const restoreContactTypeMutation = useRestoreContactTypeMutation(() => contactTypeId);

	// Form state
	let name = $state("");

	// Sync form with loaded data
	$effect(() => {
		if (contactTypeQuery.data) {
			name = contactTypeQuery.data.name;
		}
	});

	function handleSubmit(event: Event) {
		event.preventDefault();
		editContactTypeMutation.mutate({
			name
		});
	}
	function handleDelete() {
		if (confirm(`Are you sure you want to delete "${name}"?`)) {
			deleteContactTypeMutation.mutate();
		}
	}
	function handleRestore() {
		if (confirm(`Are you sure you want to restore "${name}"?`)) {
			restoreContactTypeMutation.mutate();
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Edit Contact Type</Field.Legend>
				<Field.Description>Contact Type defines the type type of Contact.</Field.Description>
				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Contact Type Name</Field.Label>
						<Input
							id="name"
							bind:value={name}
							type="text"
							placeholder="Enter contact type name"
							disabled={contactTypeQuery.isPending}
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editContactTypeMutation.isPending ||
							contactTypeQuery.isPending ||
							deleteContactTypeMutation.isPending ||
							restoreContactTypeMutation.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editContactTypeMutation.isPending ||
							contactTypeQuery.isPending ||
							deleteContactTypeMutation.isPending ||
							restoreContactTypeMutation.isPending}
						href={resolve("/contact-types")}>Cancel</Button
					>
				</div>
				{#if !contactTypeQuery.data?.deleted_at}
					<Button
						type="button"
						disabled={editContactTypeMutation.isPending ||
							contactTypeQuery.isPending ||
							deleteContactTypeMutation.isPending ||
							restoreContactTypeMutation.isPending}
						variant="destructive"
						onclick={handleDelete}>Delete</Button
					>
				{:else}
					<Button
						type="button"
						disabled={editContactTypeMutation.isPending ||
							contactTypeQuery.isPending ||
							deleteContactTypeMutation.isPending ||
							restoreContactTypeMutation.isPending}
						variant="destructive"
						onclick={handleRestore}>Restore</Button
					>
				{/if}
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if contactTypeQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{contactTypeQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editContactTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editContactTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteContactTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteContactTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if restoreContactTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{restoreContactTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
