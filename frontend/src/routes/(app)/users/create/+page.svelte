<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import * as Select from "$lib/components/ui/select/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
	import { resolve } from "$app/paths";
	import { useCreateUserMutation } from "$lib/hooks/use-user";
	import { useUserRolesQuery } from "$lib/hooks/use-user-role";

	let username = $state("");
	let password = $state("");
	let email = $state("");
	let fullName = $state("");
	let userRoleId = $state("");

	const userRolesQuery = useUserRolesQuery(() => "active");
	const createUserMutation = useCreateUserMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		createUserMutation.mutate({
			username,
			password,
			email,
			fullName,
			userRoleId: Number.parseInt(userRoleId, 10)
		});
	}

	const userRoleTrigger = $derived(
		userRolesQuery.data?.user_roles.find(
			(user_role) => user_role.user_role_id.toString() === userRoleId
		)?.name ?? "Select User Role"
	);
</script>

<div class="mx-auto w-full max-w-md">
	<form onsubmit={handleSubmit}>
		<Field.Group>
			<Field.Set>
				<Field.Legend>Create User</Field.Legend>
				<Field.Description>User is used to access this website.</Field.Description>
				<Field.Group>
					<Field.Field>
						<Field.Label for="username">Username</Field.Label>
						<Input
							id="username"
							bind:value={username}
							type="text"
							placeholder="Enter username"
							required
						/>
					</Field.Field>
					<Field.Field>
						<Field.Label for="password">Password</Field.Label>
						<Input
							id="password"
							bind:value={password}
							type="password"
							placeholder="Enter password"
							required
						/>
					</Field.Field>
					<Field.Field>
						<Field.Label for="email">Email</Field.Label>
						<Input id="email" bind:value={email} type="email" placeholder="Enter email" required />
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
						<Select.Root type="single" bind:value={userRoleId}>
							<Select.Trigger id="userRoleId">{userRoleTrigger}</Select.Trigger>
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
			<Field.Field orientation="horizontal">
				<Button type="submit" disabled={createUserMutation.isPending}>Submit</Button>
				<Button
					variant="outline"
					type="button"
					disabled={createUserMutation.isPending}
					href={resolve("/users")}>Cancel</Button
				>
			</Field.Field>
		</Field.Group>
	</form>

	<div class="mt-8 space-y-4">
		{#if createUserMutation.isError}
			<Alert.Root variant="destructive">
				<AlertCircleIcon />
				<Alert.Title>Error Creating User</Alert.Title>
				<Alert.Description>
					<p>{createUserMutation.error.message}</p>
				</Alert.Description>
			</Alert.Root>
		{/if}
	</div>
</div>
