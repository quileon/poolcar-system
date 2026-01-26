<script lang="ts">
	import { resolve } from "$app/paths";
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { useTrackerQuery } from "$lib/hooks/use-query";
	import { useDeleteTrackerMutation, useEditTrackerMutation } from "$lib/hooks/use-mutations";

	const trackerId = $derived(parseInt(page.params.id!, 10));
	const trackerQuery = useTrackerQuery(() => trackerId);
	let trackerName = $state("");
	$effect(() => {
		if (trackerQuery.data) {
			trackerName = trackerQuery.data.name;
		}
	});
	const editTrackerMutation = useEditTrackerMutation(() => trackerId);
	const deleteTrackerMutation = useDeleteTrackerMutation(() => trackerId);

	function handleSubmit(event: Event) {
		event.preventDefault();
		editTrackerMutation.mutate({ name: trackerName });
	}

	function handleDelete() {
		if (confirm(`Are you sure you want to delete "${trackerName}"?`)) {
			deleteTrackerMutation.mutate();
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Modify Tracker</Field.Legend>
				<Field.Description>Tracker will be assigned to each car uniquely.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Tracker Name</Field.Label>
						<Input
							id="name"
							bind:value={trackerName}
							name="name"
							type="text"
							placeholder="Enter tracker name"
							disabled={trackerQuery.isPending}
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editTrackerMutation.isPending ||
							trackerQuery.isPending ||
							deleteTrackerMutation.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editTrackerMutation.isPending ||
							trackerQuery.isPending ||
							deleteTrackerMutation.isPending}
						href={resolve("/trackers")}>Cancel</Button
					>
				</div>
				<Button
					type="button"
					disabled={editTrackerMutation.isPending ||
						trackerQuery.isPending ||
						deleteTrackerMutation.isPending}
					variant="destructive"
					onclick={handleDelete}>Delete</Button
				>
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if trackerQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{trackerQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editTrackerMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editTrackerMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteTrackerMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteTrackerMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
