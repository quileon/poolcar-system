<script lang="ts">
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import { useCreateTrackerMutation } from "$lib/hooks/use-mutations";

	let trackerName = $state("");

	const createTrackerMutation = useCreateTrackerMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		createTrackerMutation.mutate({ name: trackerName });
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Tracker</Field.Legend>
				<Field.Description>Tracker will be assigned to each car uniquely.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Tracker Name</Field.Label>
						<Input
							id="name"
							bind:value={trackerName}
							type="text"
							placeholder="Enter tracker name"
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createTrackerMutation.isPending}>Submit</Button>
				<Button
					variant="outline"
					type="button"
					disabled={createTrackerMutation.isPending}
					href="/trackers">Cancel</Button
				>
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if createTrackerMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{createTrackerMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
