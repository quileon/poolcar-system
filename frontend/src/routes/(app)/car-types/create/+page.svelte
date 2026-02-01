<script lang="ts">
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import { useCreateCarTypeMutation } from "$lib/hooks/use-mutations";

	let carTypeName = $state("");

	const createCarTypeMutation = useCreateCarTypeMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		createCarTypeMutation.mutate({ name: carTypeName });
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Car Type</Field.Legend>
				<Field.Description>Car Type will be used to categorize cars.</Field.Description>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Car Type Name</Field.Label>
						<Input
							id="name"
							bind:value={carTypeName}
							name="name"
							type="text"
							placeholder="Enter car type name"
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createCarTypeMutation.isPending}>Submit</Button>
				<Button
					variant="outline"
					type="button"
					disabled={createCarTypeMutation.isPending}
					href="/car-types">Cancel</Button
				>
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if createCarTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{createCarTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
