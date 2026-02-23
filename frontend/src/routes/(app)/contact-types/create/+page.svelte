<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { useCreateContactTypeMutation } from "$lib/hooks/use-contact-type";
	import { resolve } from "$app/paths";

	let contactName = $state("");

	const createContactTypeMutation = useCreateContactTypeMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		createContactTypeMutation.mutate({
			name: contactName
		});
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create Contact Type</Field.Legend>
				<Field.Description>Contact Type defines the type type of Contact.</Field.Description>
				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Contact Name</Field.Label>
						<Input
							id="name"
							bind:value={contactName}
							type="text"
							placeholder="Enter contact name"
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createContactTypeMutation.isPending}>Submit</Button>
				<Button
					variant="outline"
					type="button"
					disabled={createContactTypeMutation.isPending}
					href={resolve("/contact-types")}>Cancel</Button
				>
			</Field.Field>
		</Field.Group>
	</form>

	<div class="mt-8 space-y-4">
		{#if createContactTypeMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Creating Contact</Alert.Title>
				<Alert.Description>
					<p>{createContactTypeMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
