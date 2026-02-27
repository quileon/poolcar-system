<script lang="ts">
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import {
		useActivityTypeQuery,
		useDeleteActivityTypeMutation,
		useEditActivityTypeMutation,
		useRestoreActivityTypeMutation
	} from "$lib/hooks/use-activity-type";

	const activityTypeId = $derived(parseInt(page.params.id!, 10));

	// Queries
	const activityTypeQuery = useActivityTypeQuery(() => activityTypeId);

	// Mutations
	const editActivityTypeMutation = useEditActivityTypeMutation(() => activityTypeId);
	const deleteActivityTypeMutation = useDeleteActivityTypeMutation(() => activityTypeId);
	const restoreActivityTypeMutation = useRestoreActivityTypeMutation(() => activityTypeId);

	// Form state
	let name = $state("");

	// Sync form with loaded data
	$effect(() => {
		if (activityTypeQuery.data) {
			name = activityTypeQuery.data.name;
		}
	});

	function handleSubmit(event: Event) {
		event.preventDefault();
		editActivityTypeMutation.mutate({
			name
		});
	}
	function handleDelete() {
		if (confirm(`Are you sure you want to delete "${name}"?`)) {
			deleteActivityTypeMutation.mutate();
		}
	}
	function handleRestore() {
		if (confirm(`Are you sure you want to restore "${name}"?`)) {
			restoreActivityTypeMutation.mutate();
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Activity Type</Field.Legend>
				<Field.Description>Activity Type categorize the lend activity.</Field.Description>
				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Activity Type Name</Field.Label>
						<Input
							id="name"
							bind:value={name}
							type="text"
							placeholder="Enter activity type name"
							disabled={activityTypeQuery.isPending}
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editActivityTypeMutation.isPending ||
							activityTypeQuery.isPending ||
							deleteActivityTypeMutation.isPending ||
							restoreActivityTypeMutation.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editActivityTypeMutation.isPending ||
							activityTypeQuery.isPending ||
							deleteActivityTypeMutation.isPending ||
							restoreActivityTypeMutation.isPending}
						href={resolve("/activity-types")}>Cancel</Button
					>
				</div>
				{#if !activityTypeQuery.data?.deleted_at}
					<Button
						type="button"
						disabled={editActivityTypeMutation.isPending ||
							activityTypeQuery.isPending ||
							deleteActivityTypeMutation.isPending ||
							restoreActivityTypeMutation.isPending}
						variant="destructive"
						onclick={handleDelete}>Delete</Button
					>
				{:else}
					<Button
						type="button"
						disabled={editActivityTypeMutation.isPending ||
							activityTypeQuery.isPending ||
							deleteActivityTypeMutation.isPending ||
							restoreActivityTypeMutation.isPending}
						variant="destructive"
						onclick={handleRestore}>Restore</Button
					>
				{/if}
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if activityTypeQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{activityTypeQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editActivityTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editActivityTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteActivityTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteActivityTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if restoreActivityTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{restoreActivityTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
