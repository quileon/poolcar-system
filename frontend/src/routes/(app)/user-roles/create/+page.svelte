<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import { useCreateUserRoleMutation } from "$lib/hooks/use-user-role";

	let name = $state("");

	const createUserRoleMutation = useCreateUserRoleMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		createUserRoleMutation.mutate({
			name
		});
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create User Role</Field.Legend>
				<Field.Description
					>User Role is used to separate access and improve security.</Field.Description
				>
				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Role Name</Field.Label>
						<Input id="name" bind:value={name} type="text" placeholder="Enter name" required />
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createUserRoleMutation.isPending}>Submit</Button>
				<Button
					variant="outline"
					type="button"
					disabled={createUserRoleMutation.isPending}
					href={resolve("/user-roles")}>Cancel</Button
				>
			</Field.Field>
		</Field.Group>
	</form>

	<div class="mt-8 space-y-4">
		{#if createUserRoleMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Creating User Role</Alert.Title>
				<Alert.Description>
					<p>{createUserRoleMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
