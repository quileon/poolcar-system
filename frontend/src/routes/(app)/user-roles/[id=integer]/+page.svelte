<script lang="ts">
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import {
		useDeleteUserRoleMutation,
		useEditUserRoleMutation,
		useRestoreUserRoleMutation,
		useUserRoleQuery
	} from "$lib/hooks/use-user-role";
	import { resolve } from "$app/paths";

	const userRoleId = $derived(parseInt(page.params.id!, 10));

	// Queries
	const userRoleQuery = useUserRoleQuery(() => userRoleId);

	// Mutations
	const editUserRoleMutation = useEditUserRoleMutation(() => userRoleId);
	const deleteUserRoleMutation = useDeleteUserRoleMutation(() => userRoleId);
	const restoreUserRoleMutation = useRestoreUserRoleMutation(() => userRoleId);

	// Form state
	let name = $state("");

	// Sync form with loaded data
	$effect(() => {
		if (userRoleQuery.data) {
			name = userRoleQuery.data.name;
		}
	});

	function handleSubmit(event: Event) {
		event.preventDefault();
		editUserRoleMutation.mutate({ name });
	}
	function handleDelete() {
		if (confirm(`Are you sure you want to delete "${name}"?`)) {
			deleteUserRoleMutation.mutate();
		}
	}
	function handleRestore() {
		if (confirm(`Are you sure you want to restore "${name}"?`)) {
			restoreUserRoleMutation.mutate();
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Modify User Role</Field.Legend>
				<Field.Description
					>User Role is used to separate access and improve security.</Field.Description
				>

				<Field.Group>
					<Field.Field>
						<Field.Label for="name">Role Name</Field.Label>
						<Input
							id="name"
							bind:value={name}
							name="name"
							type="text"
							placeholder="Enter role name"
							disabled={userRoleQuery.isPending}
							required
						/>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editUserRoleMutation.isPending ||
							userRoleQuery.isPending ||
							deleteUserRoleMutation.isPending ||
							restoreUserRoleMutation.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editUserRoleMutation.isPending ||
							userRoleQuery.isPending ||
							deleteUserRoleMutation.isPending ||
							restoreUserRoleMutation.isPending}
						href={resolve("/user-roles")}>Cancel</Button
					>
				</div>
				{#if !userRoleQuery.data?.deleted_at}
					<Button
						type="button"
						disabled={editUserRoleMutation.isPending ||
							userRoleQuery.isPending ||
							deleteUserRoleMutation.isPending ||
							restoreUserRoleMutation.isPending}
						variant="destructive"
						onclick={handleDelete}>Delete</Button
					>
				{:else}
					<Button
						type="button"
						disabled={editUserRoleMutation.isPending ||
							userRoleQuery.isPending ||
							deleteUserRoleMutation.isPending ||
							restoreUserRoleMutation.isPending}
						variant="destructive"
						onclick={handleRestore}>Restore</Button
					>
				{/if}
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if userRoleQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{userRoleQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editUserRoleMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editUserRoleMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteUserRoleMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteUserRoleMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if restoreUserRoleMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{restoreUserRoleMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
