<script lang="ts">
	import { page } from "$app/state";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import * as Alert from "$lib/components/ui/alert/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import { useUserRolesQuery } from "$lib/hooks/use-user-role";
	import {
		useDeleteUserMutation,
		useEditUserMutations,
		useRestoreUserMutation,
		useUserQuery
	} from "$lib/hooks/use-user";

	const userId = $derived(parseInt(page.params.id!, 10));

	// Queries
	const userRolesQuery = useUserRolesQuery(() => "active");
	const userQuery = useUserQuery(() => userId);

	// Mutations
	const editUserMutation = useEditUserMutations(() => userId);
	const deleteUserMutation = useDeleteUserMutation(() => userId);
	const restoreUserMutation = useRestoreUserMutation(() => userId);

	// Form state
	let username = $state("");
	let password = $state("");
	let email = $state("");
	let fullName = $state("");
	let userRoleId = $state("");
	let changePassword = $state(false);

	// Sync form with loaded data
	$effect(() => {
		if (userQuery.data) {
			username = userQuery.data.username;
			email = userQuery.data.email;
			fullName = userQuery.data.full_name;
			userRoleId = userQuery.data.user_role_id.toString();
		}
	});

	const userRoleTrigger = $derived(
		userRolesQuery.data?.user_roles.find(
			(user_role) => user_role.user_role_id.toString() === userRoleId
		)?.name ?? "Select User Role"
	);

	function handleSubmit(e: Event) {
		e.preventDefault();
		editUserMutation.mutate({
			username,
			password,
			email,
			fullName,
			userRoleId: Number.parseInt(userRoleId, 10)
		});
	}

	function handleDelete() {
		if (confirm(`Are you sure you want to delete "${username}"?`)) {
			deleteUserMutation.mutate();
		}
	}

	function handleRestore() {
		if (confirm(`Are you sure you want to restore "${username}"?`)) {
			restoreUserMutation.mutate();
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Edit User</Field.Legend>
				<Field.Description>User is used to access this website.</Field.Description>
				<Field.Group>
					<Field.Field>
						<Field.Label for="username">Username</Field.Label>
						<Input
							id="username"
							bind:value={username}
							type="text"
							placeholder="Enter username"
							disabled={userQuery.isPending}
							required
						/>
					</Field.Field>

					<Field.Field>
						<Field.Label for="email">Email</Field.Label>
						<Input
							id="email"
							bind:value={email}
							type="email"
							placeholder="Enter email"
							disabled={userQuery.isPending}
							required
						/>
					</Field.Field>

					<Field.Field>
						<Field.Label for="fullName">Full Name</Field.Label>
						<Input
							id="fullName"
							bind:value={fullName}
							type="text"
							placeholder="Enter full name"
							required
						/>
					</Field.Field>

					<Field.Field>
						<Field.Label for="userRoleId">User Role</Field.Label>
						<Select.Root type="single" bind:value={userRoleId} required>
							<Select.Trigger id="userRoleId" disabled={userQuery.isPending}
								>{userRoleTrigger}</Select.Trigger
							>
							<Select.Content>
								{#if userRolesQuery.data?.user_roles}
									{#each userRolesQuery.data.user_roles as userRole (userRole.user_role_id)}
										<Select.Item value={userRole.user_role_id.toString()}>
											{userRole.name}
										</Select.Item>
									{/each}
								{/if}
							</Select.Content>
						</Select.Root>
					</Field.Field>
				</Field.Group>
			</Field.Set>
			<Field.Separator />
			<Field.Set>
				<Field.Field orientation="horizontal">
					<Checkbox
						id="changePassword"
						bind:checked={changePassword}
						disabled={userQuery.isPending}
					/>
					<Field.Content>
						<Field.Label for="changePassword">Change Password</Field.Label>
						<Field.Description>Change current active password.</Field.Description>
					</Field.Content>
				</Field.Field>

				<Field.Field>
					<Field.Label for="password">Password</Field.Label>
					<Input
						id="password"
						type="password"
						bind:value={password}
						disabled={userQuery.isPending || !changePassword}
					/>
				</Field.Field>
			</Field.Set>
			<Field.Field orientation="horizontal" class="flex justify-between">
				<div class="flex gap-3">
					<Button
						type="submit"
						disabled={editUserMutation.isPending ||
							userQuery.isPending ||
							deleteUserMutation.isPending ||
							restoreUserMutation.isPending}>Submit</Button
					>
					<Button
						variant="outline"
						type="button"
						disabled={editUserMutation.isPending ||
							userQuery.isPending ||
							deleteUserMutation.isPending ||
							restoreUserMutation.isPending}
						href={resolve("/users")}>Cancel</Button
					>
				</div>
				{#if !userQuery.data?.deleted_at}
					<Button
						type="button"
						disabled={editUserMutation.isPending ||
							userQuery.isPending ||
							deleteUserMutation.isPending ||
							restoreUserMutation.isPending}
						variant="destructive"
						onclick={handleDelete}>Delete</Button
					>
				{:else}
					<Button
						type="button"
						disabled={editUserMutation.isPending ||
							userQuery.isPending ||
							deleteUserMutation.isPending ||
							restoreUserMutation.isPending}
						variant="destructive"
						onclick={handleRestore}>Restore</Button
					>
				{/if}
			</Field.Field>
		</Field.Group>
	</form>
	<div class="mt-8">
		{#if userQuery.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{userQuery.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if editUserMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{editUserMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if deleteUserMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{deleteUserMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}

		{#if restoreUserMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error</Alert.Title>
				<Alert.Description>
					<p>{restoreUserMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
