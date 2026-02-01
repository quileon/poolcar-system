<script lang="ts">
	import Button from "$lib/components/ui/button/button.svelte";
	import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
	import * as Field from "$lib/components/ui/field/index";
	import * as Card from "$lib/components/ui/card/index";
	import Input from "$lib/components/ui/input/input.svelte";
	import placeholderImage from "$lib/assets/placeholder.svg";

	let usernumber = $state("");
	let password = $state("");
	let rememberMe = $state(false);
</script>

<div class="w-full max-w-sm md:max-w-3xl">
	<Card.Root class="p-0">
		<Card.Content class="grid p-0 md:grid-cols-2">
			<form class="p-6 md:p-8">
				<Field.Group>
					<Field.Set>
						<div class="flex flex-col items-center gap-2 text-center">
							<h1 class="text-2xl font-bold">Log In</h1>
							<p class="text-balance text-muted-foreground">Please insert your credentials</p>
						</div>

						<Field.Group>
							<Field.Field>
								<Field.Label for="usernumber">Identity Number</Field.Label>
								<Input
									id="usernumber"
									bind:value={usernumber}
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
			</form>
			<div class="relative hidden bg-muted md:block">
				<img
					src={placeholderImage}
					alt="Login Placeholder"
					class="dark:brigtness-[0.2] absolute inset-0 h-full object-cover dark:grayscale"
				/>
			</div>
		</Card.Content>
	</Card.Root>
</div>
