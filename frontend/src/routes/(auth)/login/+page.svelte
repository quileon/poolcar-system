<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import * as Card from "$lib/components/ui/card/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import logo from "$lib/assets/logo.png";
	import { useLoginMutation } from "$lib/hooks/use-login";
	import * as Alert from "$lib/components/ui/alert/index";
	import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";

	let username = $state("");
	let password = $state("");
	let rememberMe = $state(false);

	const loginMutation = useLoginMutation();

	function handleSubmit(e: Event) {
		e.preventDefault();
		loginMutation.mutate({
			username,
			password
		});
	}
</script>

<div class="w-full max-w-sm md:max-w-3xl">
	<Card.Root class="p-0">
		<Card.Content class="grid p-0 md:grid-cols-2">
			<form class="p-6 md:p-8" onsubmit={handleSubmit}>
				<Field.Group>
					<Field.Set>
						<div class="flex flex-col items-center gap-2 text-center">
							<h1 class="text-2xl font-bold">Log In</h1>
							<p class="text-balance text-muted-foreground">Please insert your credentials</p>
						</div>

						<Field.Group>
							<Field.Field>
								<Field.Label for="username">Identity Number</Field.Label>
								<Input
									id="username"
									bind:value={username}
									type="text"
									placeholder="Insert your identity number"
									required
								/>
							</Field.Field>
							<Field.Field>
								<Field.Label for="password">Password</Field.Label>
								<Input
									id="password"
									bind:value={password}
									type="password"
									placeholder="Insert your password"
									required
								/>
							</Field.Field>
						</Field.Group>
					</Field.Set>
					<Field.Separator />
					<Field.Set>
						<Field.Group>
							<Field.Field orientation="horizontal">
								<Checkbox id="remember_me" bind:checked={rememberMe} />
								<Field.Content>
									<Field.Label for="remember_me">Remember Me</Field.Label>
									<Field.Description>Remember my login for 7 days.</Field.Description>
								</Field.Content>
							</Field.Field>
						</Field.Group>
					</Field.Set>
					<Field.Field>
						<Button type="submit">Login</Button>
					</Field.Field>
				</Field.Group>

				{#if loginMutation.isError}
					<div class="mt-8 space-y-4">
						<Alert.Root variant="destructive">
							<AlertCircleIcon />
							<Alert.Title>Error Login</Alert.Title>
							<Alert.Description>
								<p>{loginMutation.error.message}</p>
							</Alert.Description>
						</Alert.Root>
					</div>
				{/if}
			</form>
			<div class="relative hidden bg-neutral-50 dark:bg-neutral-950 md:flex items-center justify-center p-8 border-s border-border">
				<img
					src={logo}
					alt="Poolcar Logo"
					class="max-h-64 max-w-xs object-contain"
				/>
			</div>
		</Card.Content>
	</Card.Root>
</div>
