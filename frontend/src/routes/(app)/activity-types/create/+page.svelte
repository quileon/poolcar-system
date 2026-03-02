<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import { useCreateActivityTypeMutation } from "$lib/hooks/use-activity-type";

	let name = $state("");

	const createActivityTypeMutation = useCreateActivityTypeMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		createActivityTypeMutation.mutate({
			name
		});
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
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createActivityTypeMutation.isPending}>Submit</Button>
				<Button
					variant="outline"
					type="button"
					disabled={createActivityTypeMutation.isPending}
					href={resolve("/activity-types")}>Cancel</Button
				>
			</Field.Field>
		</Field.Group>
	</form>

	<div class="mt-8 space-y-4">
		{#if createActivityTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Creating Activity Type</Alert.Title>
				<Alert.Description>
					<p>{createActivityTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
